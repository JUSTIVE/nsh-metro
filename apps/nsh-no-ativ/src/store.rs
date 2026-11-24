//! The app's one store: the facility ledger plus the few things the phone
//! adds to it (the expiry warning window, live telemetry). Plain Rust, no
//! GPUI: the app wraps it in an entity and re-renders on every change, so a
//! harvest, a new incident or a sensor tick shows up everywhere at once,
//! live tiles included.

use std::fmt;

use crate::data::{
    Assay, Construct, DECK_TODAY, DeckBlock, FacilityData, Incident, IncidentArea,
    IncidentSeverity, IncidentStatus, Instrument, LotState, ReagentLot, Run, RunPhase, Sample,
    Sensor, SensorKind, Shipment, ShipmentStage, Strain, StrainStatus, TimelineEntry, sensor_state,
    to_fixed,
};

/// OD600 at which a culture tops out.
pub const PEAK_OD: f64 = 15.0;
/// Who files incidents from this phone.
pub const OPERATOR: &str = "Ben Park";
/// The expiry warning window the slider moves between, in days.
pub const EXPIRY_WINDOW: std::ops::RangeInclusive<i64> = 7..=90;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreError {
    EmptyTitle,
    UnknownRun,
    NotHarvestable,
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            StoreError::EmptyTitle => {
                "Give the incident a title so the shift lead knows what happened."
            }
            StoreError::UnknownRun => "That run is no longer on the ledger.",
            StoreError::NotHarvestable => {
                "Only a culture in log or stationary phase can be harvested."
            }
        })
    }
}

/// What the new-incident form submits.
#[derive(Clone, Debug)]
pub struct IncidentDraft {
    pub title: String,
    pub area: IncidentArea,
    pub severity: IncidentSeverity,
    pub run_id: Option<String>,
    pub quarantine: bool,
    pub page_lead: bool,
}

pub struct Store {
    pub data: FacilityData,
    /// Shelf days at or below which a lot counts as expiring.
    pub expiry_window: i64,
    /// Each sensor's setpoint: ticks wander around it, not around the middle
    /// of the range, so a sensor that was drifting keeps drifting.
    baselines: Vec<f64>,
    rng: u64,
    next_incident: i64,
}

impl Store {
    pub fn new(data: FacilityData) -> Self {
        let baselines = data.sensors.iter().map(|s| s.value).collect();
        Self {
            data,
            expiry_window: 30,
            baselines,
            rng: 0x9E37_79B9_7F4A_7C15,
            next_incident: 2401,
        }
    }

    /* lookups ------------------------------------------------------------ */

    pub fn run(&self, id: &str) -> Option<&Run> {
        self.data.runs.iter().find(|r| r.id == id)
    }
    pub fn strain(&self, id: &str) -> Option<&Strain> {
        self.data.strains.iter().find(|s| s.id == id)
    }
    pub fn incident(&self, id: &str) -> Option<&Incident> {
        self.data.incidents.iter().find(|i| i.id == id)
    }
    pub fn instrument(&self, id: &str) -> Option<&Instrument> {
        self.data.instruments.iter().find(|i| i.id == id)
    }
    pub fn lot(&self, id: &str) -> Option<&ReagentLot> {
        self.data.lots.iter().find(|l| l.id == id)
    }
    pub fn construct(&self, id: &str) -> Option<&Construct> {
        self.data.constructs.iter().find(|c| c.id == id)
    }
    pub fn assays_for<'a>(&'a self, run_id: &'a str) -> impl Iterator<Item = &'a Assay> {
        self.data.assays.iter().filter(move |a| a.run_id == run_id)
    }
    pub fn runs_of<'a>(&'a self, strain_id: &'a str) -> impl Iterator<Item = &'a Run> {
        self.data
            .runs
            .iter()
            .filter(move |r| r.strain_id == strain_id)
    }
    pub fn samples_of<'a>(&'a self, strain_id: &'a str) -> impl Iterator<Item = &'a Sample> {
        self.data
            .samples
            .iter()
            .filter(move |s| s.strain_id == strain_id)
    }
    pub fn children_of<'a>(&'a self, strain_id: &'a str) -> impl Iterator<Item = &'a Strain> {
        self.data
            .strains
            .iter()
            .filter(move |s| s.parent_id.as_deref() == Some(strain_id))
    }

    /* live-tile readings ------------------------------------------------- */

    pub fn live_runs(&self) -> impl Iterator<Item = &Run> {
        self.data.runs.iter().filter(|r| is_live(r.phase))
    }
    /// Incidents not yet closed.
    pub fn open_incidents(&self) -> impl Iterator<Item = &Incident> {
        self.data
            .incidents
            .iter()
            .filter(|i| i.status != IncidentStatus::Closed)
    }
    pub fn sensors_in(&self, state: crate::data::SensorState) -> usize {
        self.data
            .sensors
            .iter()
            .filter(|s| s.state == state)
            .count()
    }
    /// The sensor furthest outside (or nearest the edge of) its range.
    pub fn worst_sensor(&self) -> Option<&Sensor> {
        self.data
            .sensors
            .iter()
            .max_by(|a, b| excursion(a).total_cmp(&excursion(b)))
    }
    pub fn active_strains(&self) -> usize {
        self.data
            .strains
            .iter()
            .filter(|s| s.status != StrainStatus::Archived)
            .count()
    }
    /// (filled slots, total slots) across every rack.
    pub fn freezer_occupancy(&self) -> (usize, usize) {
        self.data
            .racks
            .iter()
            .flat_map(|r| &r.boxes)
            .fold((0, 0), |(filled, total), b| {
                (
                    filled + b.slots.iter().filter(|s| s.is_some()).count(),
                    total + b.slots.len(),
                )
            })
    }
    /// The undelivered shipment that lands first.
    pub fn next_shipment(&self) -> Option<&Shipment> {
        self.data
            .shipments
            .iter()
            .filter(|s| s.stage != ShipmentStage::Delivered)
            .min_by_key(|s| s.eta_days)
    }
    pub fn lot_state(&self, lot: &ReagentLot) -> LotState {
        if lot.shelf_days < 0 {
            LotState::Expired
        } else if lot.shelf_days <= self.expiry_window {
            LotState::Expiring
        } else if lot.qty < lot.capacity * 0.15 {
            LotState::Low
        } else {
            LotState::Ok
        }
    }
    pub fn lots_in(&self, state: LotState) -> impl Iterator<Item = &ReagentLot> {
        self.data
            .lots
            .iter()
            .filter(move |l| self.lot_state(l) == state)
    }
    /// Deck blocks that cover today.
    pub fn deck_today(&self) -> impl Iterator<Item = &DeckBlock> {
        self.data
            .deck
            .iter()
            .filter(|b| b.day <= DECK_TODAY && DECK_TODAY < b.day + b.days)
    }
    pub fn active_members(&self) -> usize {
        self.data
            .members
            .iter()
            .filter(|m| m.left.is_none())
            .count()
    }

    /* mutations ---------------------------------------------------------- */

    pub fn set_expiry_window(&mut self, days: i64) {
        self.expiry_window = days.clamp(*EXPIRY_WINDOW.start(), *EXPIRY_WINDOW.end());
    }

    /// Ends a live culture: it leaves the vessel at its current OD and titer.
    pub fn harvest(&mut self, run_id: &str) -> Result<&Run, StoreError> {
        let run = self
            .data
            .runs
            .iter_mut()
            .find(|r| r.id == run_id)
            .ok_or(StoreError::UnknownRun)?;
        if !matches!(run.phase, RunPhase::Log | RunPhase::Stationary) {
            return Err(StoreError::NotHarvestable);
        }
        let minutes = (run.od600 * 160.0) as i64 + 360;
        run.phase = RunPhase::Harvested;
        run.duration = format!("{}h {:02}m", minutes / 60, minutes % 60);
        Ok(run)
    }

    /// Validates and files a new deviation at the top of the ledger. Returns its code.
    pub fn file_incident(&mut self, draft: IncidentDraft) -> Result<String, StoreError> {
        let title = draft.title.trim();
        if title.is_empty() {
            return Err(StoreError::EmptyTitle);
        }
        let instrument_id = match &draft.run_id {
            Some(id) => self
                .run(id)
                .ok_or(StoreError::UnknownRun)?
                .instrument_id
                .clone(),
            None => None,
        };
        let code = format!("DEV-{}", self.next_incident);
        let mut timeline = vec![TimelineEntry {
            at: "today".into(),
            text: format!("Raised by {OPERATOR} — {title}"),
        }];
        if draft.quarantine {
            timeline.push(TimelineEntry {
                at: "today".into(),
                text: "Area quarantined".into(),
            });
        }
        if draft.page_lead {
            timeline.push(TimelineEntry {
                at: "today".into(),
                text: "Floor lead paged".into(),
            });
        }
        self.data.incidents.insert(
            0,
            Incident {
                id: format!("inc-{}", self.next_incident),
                code: code.clone(),
                title: title.to_string(),
                area: draft.area,
                severity: draft.severity,
                status: IncidentStatus::Open,
                opened: "today".into(),
                owner: OPERATOR.into(),
                run_id: draft.run_id,
                instrument_id,
                lot_id: None,
                age: 0,
                timeline,
            },
        );
        self.next_incident += 1;
        Ok(code)
    }

    /// Moves an incident one step along open → investigating → capa → closed.
    pub fn advance_incident(&mut self, id: &str) -> Option<IncidentStatus> {
        let incident = self.data.incidents.iter_mut().find(|i| i.id == id)?;
        let (next, note) = match incident.status {
            IncidentStatus::Open => (
                IncidentStatus::Investigating,
                "Root cause analysis opened (5-why)",
            ),
            IncidentStatus::Investigating => (IncidentStatus::Capa, "CAPA filed"),
            IncidentStatus::Capa => (IncidentStatus::Closed, "Closed · verified"),
            IncidentStatus::Closed => return None,
        };
        incident.status = next;
        incident.timeline.push(TimelineEntry {
            at: "today".into(),
            text: format!("{note} by {OPERATOR}"),
        });
        Some(next)
    }

    /// One telemetry interval: every sensor wanders around its setpoint and
    /// every live culture grows a little, moving phase when it should.
    pub fn tick(&mut self) {
        for i in 0..self.data.sensors.len() {
            let noise = self.next_unit() - 0.5;
            let sensor = &mut self.data.sensors[i];
            let span = sensor.high - sensor.low;
            let pull = (self.baselines[i] - sensor.value) * 0.15;
            let (lo, hi) = plausible(sensor);
            let value = (sensor.value + pull + noise * span * 0.06).clamp(lo, hi);
            sensor.value = round(value, decimals(sensor.kind));
            sensor.state = sensor_state(sensor.value, sensor.low, sensor.high);
        }
        for i in 0..self.data.runs.len() {
            if !matches!(self.data.runs[i].phase, RunPhase::Lag | RunPhase::Log) {
                continue;
            }
            let growth = self.next_unit();
            let run = &mut self.data.runs[i];
            if run.phase == RunPhase::Lag {
                run.od600 = round(run.od600 + 0.01 + growth * 0.03, 2);
                if run.od600 >= 0.8 {
                    run.phase = RunPhase::Log;
                }
            } else {
                run.od600 = round((run.od600 + 0.02 + growth * 0.08).min(PEAK_OD), 2);
                if run.od600 >= PEAK_OD * 0.9 {
                    run.phase = RunPhase::Stationary;
                }
            }
        }
    }

    /// xorshift64*: deterministic, so the tests can reason about it.
    fn next_unit(&mut self) -> f64 {
        self.rng ^= self.rng >> 12;
        self.rng ^= self.rng << 25;
        self.rng ^= self.rng >> 27;
        (self.rng.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }
}

pub fn is_live(phase: RunPhase) -> bool {
    matches!(phase, RunPhase::Lag | RunPhase::Log | RunPhase::Stationary)
}

/// Growth toward [`PEAK_OD`], 0–1.
pub fn growth(run: &Run) -> f32 {
    (run.od600 / PEAK_OD).clamp(0., 1.) as f32
}

/// Stage progress, 0–1: packed is the first of five steps.
pub fn stage_progress(stage: ShipmentStage) -> f32 {
    let step = match stage {
        ShipmentStage::Packed => 1,
        ShipmentStage::PickedUp => 2,
        ShipmentStage::InTransit => 3,
        ShipmentStage::Customs => 4,
        ShipmentStage::Delivered => 5,
    };
    step as f32 / 5.
}

/// A reading formatted the way its kind is quoted on the floor.
pub fn reading(sensor: &Sensor) -> String {
    to_fixed(sensor.value, decimals(sensor.kind))
}

/// How far a reading sits from the centre of its range, in half-ranges
/// (1 = at a limit, >1 = outside).
fn excursion(sensor: &Sensor) -> f64 {
    let mid = (sensor.low + sensor.high) / 2.;
    (sensor.value - mid).abs() / ((sensor.high - sensor.low) / 2.)
}

/// No real probe reads more than a fifth of its range past either limit.
pub fn plausible(sensor: &Sensor) -> (f64, f64) {
    let span = sensor.high - sensor.low;
    (sensor.low - span * 0.2, sensor.high + span * 0.2)
}

fn decimals(kind: SensorKind) -> usize {
    match kind {
        SensorKind::Ph => 2,
        SensorKind::Co2 => 0,
        _ => 1,
    }
}

fn round(x: f64, digits: usize) -> f64 {
    let f = 10f64.powi(digits as i32);
    (x * f).round() / f
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{SensorState, seed};

    fn store() -> Store {
        Store::new(seed())
    }

    fn draft(title: &str) -> IncidentDraft {
        IncidentDraft {
            title: title.into(),
            area: IncidentArea::Bioreactor,
            severity: IncidentSeverity::Major,
            run_id: None,
            quarantine: true,
            page_lead: false,
        }
    }

    #[test]
    fn harvest_ends_a_live_run() {
        let mut s = store();
        let live = s.live_runs().count();
        assert_eq!(s.run("r1").unwrap().phase, RunPhase::Log);
        let run = s.harvest("r1").unwrap();
        assert_eq!(run.phase, RunPhase::Harvested);
        assert_ne!(run.duration, "—");
        assert_eq!(s.live_runs().count(), live - 1);
        assert_eq!(s.harvest("r1").unwrap_err(), StoreError::NotHarvestable);
        assert_eq!(
            s.harvest("r2").unwrap_err(),
            StoreError::NotHarvestable,
            "lag cultures are not ready"
        );
        assert_eq!(s.harvest("nope").unwrap_err(), StoreError::UnknownRun);
    }

    #[test]
    fn new_incident_is_validated_and_filed_on_top() {
        let mut s = store();
        let (total, open) = (s.data.incidents.len(), s.open_incidents().count());
        assert_eq!(s.file_incident(draft("   ")), Err(StoreError::EmptyTitle));
        assert_eq!(
            s.file_incident(IncidentDraft {
                run_id: Some("ghost".into()),
                ..draft("foam")
            }),
            Err(StoreError::UnknownRun)
        );
        assert_eq!(
            s.data.incidents.len(),
            total,
            "rejected drafts leave the ledger alone"
        );

        let code = s
            .file_incident(IncidentDraft {
                run_id: Some("r1".into()),
                ..draft("  foam over the top ")
            })
            .unwrap();
        assert_eq!(code, "DEV-2401");
        let filed = &s.data.incidents[0];
        assert_eq!(
            (filed.title.as_str(), filed.status),
            ("foam over the top", IncidentStatus::Open)
        );
        assert_eq!(
            filed.instrument_id,
            s.run("r1").unwrap().instrument_id,
            "inherits the run's machine"
        );
        assert_eq!(filed.timeline.len(), 2, "raised + quarantined");
        assert_eq!(s.open_incidents().count(), open + 1);
        assert_eq!(s.file_incident(draft("second")).unwrap(), "DEV-2402");
    }

    #[test]
    fn incidents_advance_to_closed_and_stop() {
        let mut s = store();
        s.file_incident(draft("x")).unwrap();
        let id = s.data.incidents[0].id.clone();
        let steps: Vec<_> = std::iter::from_fn(|| s.advance_incident(&id)).collect();
        assert_eq!(
            steps,
            [
                IncidentStatus::Investigating,
                IncidentStatus::Capa,
                IncidentStatus::Closed
            ]
        );
    }

    #[test]
    fn ticks_keep_sensors_plausible_and_states_honest() {
        let mut s = store();
        for _ in 0..2000 {
            s.tick();
            for sensor in &s.data.sensors {
                let (lo, hi) = plausible(sensor);
                assert!(
                    (lo..=hi).contains(&sensor.value),
                    "{} = {}",
                    sensor.code,
                    sensor.value
                );
                assert_eq!(
                    sensor.state,
                    sensor_state(sensor.value, sensor.low, sensor.high)
                );
            }
        }
        assert!(s.data.runs.iter().all(|r| r.od600 <= PEAK_OD));
        assert!(
            s.sensors_in(SensorState::Ok) > 0,
            "most of the floor stays nominal"
        );
    }

    #[test]
    fn ticks_grow_cultures_through_their_phases() {
        let mut s = store();
        let lag = s.run("r2").unwrap().od600;
        for _ in 0..400 {
            s.tick();
        }
        let r2 = s.run("r2").unwrap();
        assert!(r2.od600 > lag);
        assert_eq!(r2.phase, RunPhase::Stationary, "lag → log → stationary");
        assert_eq!(
            s.run("r3").unwrap().phase,
            RunPhase::Harvested,
            "finished runs don't move"
        );
    }

    #[test]
    fn expiry_window_moves_lots_between_states() {
        let mut s = store();
        s.set_expiry_window(7);
        let narrow = s.lots_in(LotState::Expiring).count();
        s.set_expiry_window(1000);
        assert_eq!(s.expiry_window, 90, "clamped to the slider range");
        assert!(s.lots_in(LotState::Expiring).count() >= narrow);
        assert!(
            s.data
                .lots
                .iter()
                .filter(|l| l.shelf_days < 0)
                .all(|l| s.lot_state(l) == LotState::Expired)
        );
    }

    #[test]
    fn tiles_have_something_to_say() {
        let s = store();
        assert!(s.live_runs().count() > 0);
        assert!(s.open_incidents().count() > 0);
        assert!(s.next_shipment().is_some());
        assert!(s.deck_today().count() > 0);
        let (filled, total) = s.freezer_occupancy();
        assert!(filled > 0 && filled < total);
    }
}
