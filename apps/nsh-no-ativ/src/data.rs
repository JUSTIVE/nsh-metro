//! no ativ — the bio-foundry domain.
//!
//! Copied from the morphing-sheet GPUI port of `apps/no-ativ/src/app/data.ts`
//! and trimmed to what the phone app shows. The generators reproduce the
//! TypeScript output byte for byte: every JS numeric quirk the original leans
//! on (`| 0` wrapping, `>>` on int32, `Math.round` ties toward +inf,
//! `toFixed` rounding the exact binary value, UTC calendar math) is emulated
//! by the small helpers at the bottom of this file.
//!
//! Numeric field types: fields that only ever hold integers are `i64`;
//! anything that is ever fractional (or is a physical measurement) is `f64`.

use std::collections::HashMap;

/* ------------------------------------------------------------------------- */
/* String-literal unions                                                     */
/* ------------------------------------------------------------------------- */

macro_rules! str_enum {
    ($(#[$meta:meta])* $name:ident { $($variant:ident = $lit:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            /// The exact TypeScript string literal.
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $lit),+
                }
            }
        }
    };
}

str_enum!(Chassis {
    EColiK12 = "E. coli K-12",
    SCerevisiae = "S. cerevisiae",
    BSubtilis = "B. subtilis",
    PPastoris = "P. pastoris",
});

str_enum!(StrainStatus {
    Culturing = "culturing",
    Sequencing = "sequencing",
    Validated = "validated",
    Archived = "archived",
});

str_enum!(ConstructStatus {
    Design = "design",
    Synthesis = "synthesis",
    Verified = "verified",
});

str_enum!(RunPhase {
    Lag = "lag",
    Log = "log",
    Stationary = "stationary",
    Harvested = "harvested",
    Failed = "failed",
});

str_enum!(
    /// `Assay['kind']`.
    AssayKind {
        Hplc = "HPLC",
        LcMs = "LC-MS",
        PlateReader = "plate reader",
    }
);

str_enum!(
    /// `Assay['verdict']`.
    Verdict {
        Pass = "pass",
        Flag = "flag",
    }
);

str_enum!(
    /// `Sample['form']`.
    SampleForm {
        Glycerol = "glycerol",
        Plasmid = "plasmid",
        Pellet = "pellet",
    }
);

str_enum!(
    /// `Instrument['state']`: 'ready' | 'running' | 'cip' (clean-in-place) | 'down'.
    InstrumentState {
        Ready = "ready",
        Running = "running",
        Cip = "cip",
        Down = "down",
    }
);

str_enum!(
    /// `DeckBlock['kind']`.
    DeckKind {
        Run = "run",
        Cip = "cip",
        Maintenance = "maintenance",
        Booked = "booked",
    }
);

str_enum!(IncidentSeverity {
    Minor = "minor",
    Major = "major",
    Critical = "critical",
});

str_enum!(IncidentStatus {
    Open = "open",
    Investigating = "investigating",
    Capa = "capa",
    Closed = "closed",
});

str_enum!(
    /// `Incident['area']`.
    IncidentArea {
        Bioreactor = "bioreactor",
        Freezer = "freezer",
        Media = "media",
        Sequencing = "sequencing",
        Safety = "safety",
        Facility = "facility",
    }
);

str_enum!(LotState {
    Ok = "ok",
    Low = "low",
    Expiring = "expiring",
    Expired = "expired",
});

str_enum!(SensorKind {
    Temp = "temp",
    Co2 = "co2",
    Rh = "rh",
    Do = "do",
    Ph = "ph",
    Pressure = "pressure",
});

str_enum!(
    /// `Sensor['state']`.
    SensorState {
        Ok = "ok",
        Warn = "warn",
        Alarm = "alarm",
    }
);

str_enum!(ShipmentStage {
    Packed = "packed",
    PickedUp = "picked up",
    InTransit = "in transit",
    Customs = "customs",
    Delivered = "delivered",
});

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RoleId {
    Pi,
    Scientist,
    Technician,
    Observer,
}

str_enum!(
    /// `FacilityData['biosafety']`.
    Biosafety {
        Bsl1 = "BSL-1",
    }
);

/* ------------------------------------------------------------------------- */
/* Records                                                                   */
/* ------------------------------------------------------------------------- */

#[derive(Clone, Debug, PartialEq)]
pub struct Strain {
    pub id: String,
    pub code: String,
    pub name: String,
    pub chassis: Chassis,
    pub status: StrainStatus,
    pub generation: i64,
    /// 0–100, growth-normalized product fitness.
    pub fitness: i64,
    pub owner: String,
    pub updated: String,
    pub construct_ids: Vec<String>,
    /// Parent strain this one was passaged from — the lineage graph (#84).
    pub parent_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Construct {
    pub id: String,
    pub code: String,
    pub name: String,
    pub bp: i64,
    pub gc: f64,
    pub status: ConstructStatus,
    pub parts: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub id: String,
    pub code: String,
    pub strain_id: String,
    pub vessel: String,
    /// The machine it actually ran on, and what it drew from the kitchen (#88).
    pub instrument_id: Option<String>,
    /// Empty when the TS field is absent.
    pub lot_ids: Vec<String>,
    /// Litres charged, for the batch record's material balance.
    pub charge_l: Option<f64>,
    pub phase: RunPhase,
    pub od600: f64,
    pub titer: String,
    pub started: String,
    pub duration: String,
    pub operator: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Assay {
    pub id: String,
    pub run_id: String,
    pub kind: AssayKind,
    pub metric: String,
    pub value: String,
    pub verdict: Verdict,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    pub id: String,
    pub code: String,
    pub strain_id: String,
    pub form: SampleForm,
    pub frozen_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FreezerBox {
    pub id: String,
    pub label: String,
    /// 9 slots; null = empty. (Generated boxes carry [`BOX_SLOTS`].)
    pub slots: Vec<Option<String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FreezerRack {
    pub id: String,
    pub name: String,
    pub temp: String,
    pub boxes: Vec<FreezerBox>,
}

/* #84 — sections a foundry has that a workspace app never would. */

/// Physical hardware on the floor. The vessel string on a Run names a *kind*;
/// an instrument is one machine, with its own duty cycle.
#[derive(Clone, Debug, PartialEq)]
pub struct Instrument {
    pub id: String,
    pub code: String,
    pub kind: String,
    /// 'ready' | 'running' | 'cip' (clean-in-place) | 'down'
    pub state: InstrumentState,
    pub since_hours: i64,
    pub cycles: i64,
}

/// One booking on the deck: which machine, which days, what for.
#[derive(Clone, Debug, PartialEq)]
pub struct DeckBlock {
    pub id: String,
    pub instrument_id: String,
    pub run_id: Option<String>,
    pub strain_id: Option<String>,
    /// Column index into the 21-day window, 0 = three weeks ago.
    pub day: i64,
    pub days: i64,
    pub kind: DeckKind,
    pub label: String,
    pub operator: String,
}

/// One entry of `Incident['timeline']` (an inline `{ at, text }` in the TS).
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineEntry {
    pub at: String,
    pub text: String,
}

/// A deviation, the way a regulated lab logs one.
#[derive(Clone, Debug, PartialEq)]
pub struct Incident {
    pub id: String,
    pub code: String,
    pub title: String,
    pub area: IncidentArea,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    pub opened: String,
    pub owner: String,
    pub run_id: Option<String>,
    pub instrument_id: Option<String>,
    pub lot_id: Option<String>,
    /// Days the deviation has been open (or took to close).
    pub age: i64,
    pub timeline: Vec<TimelineEntry>,
}

/// Media kitchen inventory: every reagent arrives as a lot and expires.
#[derive(Clone, Debug, PartialEq)]
pub struct ReagentLot {
    pub id: String,
    pub code: String,
    pub name: String,
    pub vendor: String,
    pub grade: String,
    pub received: String,
    pub expires: String,
    /// Negative once expired.
    pub shelf_days: i64,
    pub shelf_total: i64,
    pub qty: f64,
    pub capacity: f64,
    pub unit: String,
    pub location: String,
    pub reserved_for: Option<String>,
}

/// Facility telemetry: the floor's own vital signs, not a run's.
#[derive(Clone, Debug, PartialEq)]
pub struct Sensor {
    pub id: String,
    pub code: String,
    pub zone: String,
    pub kind: SensorKind,
    pub value: f64,
    pub unit: String,
    pub low: f64,
    pub high: f64,
    pub state: SensorState,
}

/// Cryo shipments leaving the floor, with the chain of custody attached.
#[derive(Clone, Debug, PartialEq)]
pub struct Shipment {
    pub id: String,
    pub code: String,
    pub destination: String,
    pub courier: String,
    pub vials: i64,
    /// Always a whole number in the seed data, but it is a mass, so `f64`.
    pub dry_ice_kg: f64,
    pub stage: ShipmentStage,
    pub eta: String,
    /// Days until arrival; negative once delivered.
    pub eta_days: i64,
    pub excursions: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: RoleId,
    pub joined: String,
    /// Set for people who have left — seven years of hiring means alumni (#83).
    pub left: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Role {
    pub id: RoleId,
    pub name: &'static str,
    pub description: &'static str,
}

pub const ROLES: [Role; 4] = [
    Role {
        id: RoleId::Pi,
        name: "PI",
        description: "Runs the facility. Signs off on biosafety.",
    },
    Role {
        id: RoleId::Scientist,
        name: "Scientist",
        description: "Designs constructs, starts runs, reads assays.",
    },
    Role {
        id: RoleId::Technician,
        name: "Technician",
        description: "Operates vessels and the freezer.",
    },
    Role {
        id: RoleId::Observer,
        name: "Observer",
        description: "Read-only access to runs and results.",
    },
];

pub const VESSELS: [&str; 4] = [
    "BR-250 · 250 mL",
    "BR-2K · 2 L",
    "FLASK · 500 mL baffled",
    "PLATE · 96-well",
];
pub const CHASSIS_OPTIONS: [Chassis; 4] = [
    Chassis::EColiK12,
    Chassis::SCerevisiae,
    Chassis::BSubtilis,
    Chassis::PPastoris,
];

#[derive(Clone, Debug, PartialEq)]
pub struct FacilityData {
    pub strains: Vec<Strain>,
    pub constructs: Vec<Construct>,
    pub runs: Vec<Run>,
    pub assays: Vec<Assay>,
    pub samples: Vec<Sample>,
    pub racks: Vec<FreezerRack>,
    pub instruments: Vec<Instrument>,
    pub deck: Vec<DeckBlock>,
    pub incidents: Vec<Incident>,
    pub lots: Vec<ReagentLot>,
    pub sensors: Vec<Sensor>,
    pub shipments: Vec<Shipment>,
    pub members: Vec<Member>,
    pub biosafety: Biosafety,
    /// When this floor started logging runs (#83).
    pub opened: String,
    /// Years of operating history, one decimal.
    pub years: f64,
}

/* ------------------------------------------------------------------------- */
/* Hand-authored records                                                     */
/* ------------------------------------------------------------------------- */

fn s(v: &str) -> String {
    v.to_string()
}

fn strs(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

fn member(id: &str, name: &str, email: &str, role: RoleId, joined: &str) -> Member {
    Member {
        id: s(id),
        name: s(name),
        email: s(email),
        role,
        joined: s(joined),
        left: None,
    }
}

fn construct(
    id: &str,
    code: &str,
    name: &str,
    bp: i64,
    gc: f64,
    status: ConstructStatus,
    parts: &[&str],
) -> Construct {
    Construct {
        id: s(id),
        code: s(code),
        name: s(name),
        bp,
        gc,
        status,
        parts: strs(parts),
    }
}

#[allow(clippy::too_many_arguments)]
fn strain(
    id: &str,
    code: &str,
    name: &str,
    chassis: Chassis,
    status: StrainStatus,
    generation: i64,
    fitness: i64,
    owner: &str,
    updated: &str,
    construct_ids: &[&str],
) -> Strain {
    Strain {
        id: s(id),
        code: s(code),
        name: s(name),
        chassis,
        status,
        generation,
        fitness,
        owner: s(owner),
        updated: s(updated),
        construct_ids: strs(construct_ids),
        parent_id: None,
    }
}

#[allow(clippy::too_many_arguments)]
fn run(
    id: &str,
    code: &str,
    strain_id: &str,
    vessel: &str,
    phase: RunPhase,
    od600: f64,
    titer: &str,
    started: &str,
    duration: &str,
    operator: &str,
) -> Run {
    Run {
        id: s(id),
        code: s(code),
        strain_id: s(strain_id),
        vessel: s(vessel),
        instrument_id: None,
        lot_ids: Vec::new(),
        charge_l: None,
        phase,
        od600,
        titer: s(titer),
        started: s(started),
        duration: s(duration),
        operator: s(operator),
    }
}

fn assay(
    id: &str,
    run_id: &str,
    kind: AssayKind,
    metric: &str,
    value: &str,
    verdict: Verdict,
) -> Assay {
    Assay {
        id: s(id),
        run_id: s(run_id),
        kind,
        metric: s(metric),
        value: s(value),
        verdict,
    }
}

fn sample(id: &str, code: &str, strain_id: &str, form: SampleForm, frozen_at: &str) -> Sample {
    Sample {
        id: s(id),
        code: s(code),
        strain_id: s(strain_id),
        form,
        frozen_at: s(frozen_at),
    }
}

fn seoul_members() -> Vec<Member> {
    use RoleId::*;
    vec![
        member("m1", "Ben Park", "ben@noativ.bio", Pi, "Mar 2019"),
        member("m2", "Yuna Seo", "yuna@noativ.bio", Scientist, "Aug 2019"),
        member(
            "m3",
            "Dmitri Volkov",
            "dmitri@noativ.bio",
            Scientist,
            "Feb 2020",
        ),
        member("m4", "Hana Cho", "hana@noativ.bio", Technician, "Jun 2021"),
        member(
            "m5",
            "Marco Reyes",
            "marco@noativ.bio",
            Technician,
            "Mar 2022",
        ),
        member("m6", "Iris Tan", "iris@noativ.bio", Scientist, "Nov 2022"),
        member("m7", "Noor Haddad", "noor@noativ.bio", Observer, "Sep 2023"),
        member(
            "m8",
            "Felix Braun",
            "felix@noativ.bio",
            Scientist,
            "May 2024",
        ),
    ]
}

fn seoul_constructs() -> Vec<Construct> {
    use ConstructStatus::*;
    vec![
        construct(
            "c1",
            "pNV-380",
            "limonene pathway v3",
            9421,
            52.4,
            Verified,
            &["pTrc", "GPPS", "LS", "T7te"],
        ),
        construct(
            "c2",
            "pNV-379",
            "violacein operon",
            7286,
            58.1,
            Verified,
            &["pBAD", "vioABCDE", "rrnB"],
        ),
        construct(
            "c3",
            "pNV-378",
            "PHB accumulation",
            5804,
            55.9,
            Synthesis,
            &["pLac", "phaCAB", "T1"],
        ),
        construct(
            "c4",
            "pNV-377",
            "heme sensor v2",
            4212,
            49.7,
            Design,
            &["pTet", "hrtR", "sfGFP"],
        ),
        construct(
            "c5",
            "pNV-376",
            "squalene boost",
            11212,
            54.2,
            Verified,
            &["pGal", "ERG9", "ERG1", "tADH1"],
        ),
        construct(
            "c6",
            "pNV-375",
            "nitrogenase cluster",
            23480,
            61.3,
            Synthesis,
            &["pNif", "nifHDK", "nifENB"],
        ),
        construct(
            "c7",
            "pNV-374",
            "chitinase secretion",
            6120,
            46.8,
            Verified,
            &["pAOX1", "α-MF", "chiA"],
        ),
        construct(
            "c8",
            "pNV-373",
            "indigo dye module",
            5310,
            51.5,
            Design,
            &["pT7", "fmo", "trpBA"],
        ),
        construct(
            "c9",
            "pNV-372",
            "osmotic stress shield",
            4890,
            50.2,
            Verified,
            &["pOsmY", "otsAB", "T7te"],
        ),
        construct(
            "c10",
            "pNV-371",
            "caffeine degrader",
            8130,
            57.6,
            Synthesis,
            &["pCsp", "ndmABCD", "rrnB"],
        ),
    ]
}

fn seoul_strains() -> Vec<Strain> {
    use Chassis::*;
    use StrainStatus::*;
    vec![
        strain(
            "s1",
            "NV-648",
            "limonene HP",
            EColiK12,
            Culturing,
            14,
            87,
            "Yuna Seo",
            "18 minutes ago",
            &["c1", "c9"],
        ),
        strain(
            "s2",
            "NV-647",
            "violacein prod",
            EColiK12,
            Validated,
            9,
            74,
            "Dmitri Volkov",
            "2 hours ago",
            &["c2"],
        ),
        strain(
            "s3",
            "NV-646",
            "PHB packrat",
            BSubtilis,
            Sequencing,
            4,
            61,
            "Iris Tan",
            "yesterday",
            &["c3"],
        ),
        strain(
            "s4",
            "NV-645",
            "heme reporter",
            EColiK12,
            Culturing,
            2,
            55,
            "Felix Braun",
            "41 minutes ago",
            &["c4"],
        ),
        strain(
            "s5",
            "NV-644",
            "squalene max",
            SCerevisiae,
            Validated,
            21,
            92,
            "Yuna Seo",
            "3 days ago",
            &["c5"],
        ),
        strain(
            "s6",
            "NV-643",
            "nif tester",
            EColiK12,
            Sequencing,
            1,
            38,
            "Dmitri Volkov",
            "5 hours ago",
            &["c6"],
        ),
        strain(
            "s7",
            "NV-642",
            "chitin cutter",
            PPastoris,
            Validated,
            17,
            81,
            "Iris Tan",
            "last week",
            &["c7"],
        ),
        strain(
            "s8",
            "NV-641",
            "indigo sketch",
            EColiK12,
            Archived,
            3,
            22,
            "Felix Braun",
            "2 weeks ago",
            &["c8"],
        ),
    ]
}

fn seoul_runs() -> Vec<Run> {
    use RunPhase::*;
    vec![
        run(
            "r1",
            "R-4712",
            "s1",
            "BR-2K · 2 L",
            Log,
            4.8,
            "1.9 g/L",
            "18 minutes ago",
            "—",
            "Hana Cho",
        ),
        run(
            "r2",
            "R-4711",
            "s4",
            "PLATE · 96-well",
            Lag,
            0.3,
            "—",
            "41 minutes ago",
            "—",
            "Felix Braun",
        ),
        run(
            "r3",
            "R-4710",
            "s2",
            "BR-250 · 250 mL",
            Harvested,
            9.1,
            "3.4 g/L",
            "yesterday",
            "21h 12m",
            "Hana Cho",
        ),
        run(
            "r4",
            "R-4709",
            "s5",
            "BR-2K · 2 L",
            Harvested,
            11.6,
            "6.2 g/L",
            "2 days ago",
            "38h 03m",
            "Marco Reyes",
        ),
        run(
            "r5",
            "R-4708",
            "s6",
            "FLASK · 500 mL baffled",
            Failed,
            1.1,
            "0",
            "3 days ago",
            "9h 44m",
            "Marco Reyes",
        ),
        run(
            "r6",
            "R-4707",
            "s1",
            "BR-250 · 250 mL",
            Harvested,
            8.4,
            "1.6 g/L",
            "4 days ago",
            "19h 51m",
            "Hana Cho",
        ),
        run(
            "r7",
            "R-4706",
            "s7",
            "BR-2K · 2 L",
            Harvested,
            14.2,
            "8.8 g/L",
            "last week",
            "44h 20m",
            "Marco Reyes",
        ),
        run(
            "r8",
            "R-4705",
            "s3",
            "FLASK · 500 mL baffled",
            Stationary,
            6.7,
            "0.8 g/L",
            "yesterday",
            "—",
            "Hana Cho",
        ),
        run(
            "r9",
            "R-4704",
            "s2",
            "BR-250 · 250 mL",
            Harvested,
            8.8,
            "3.1 g/L",
            "last week",
            "20h 40m",
            "Felix Braun",
        ),
        run(
            "r10",
            "R-4703",
            "s5",
            "BR-2K · 2 L",
            Harvested,
            10.9,
            "5.7 g/L",
            "2 weeks ago",
            "36h 55m",
            "Hana Cho",
        ),
    ]
}

fn seoul_assays() -> Vec<Assay> {
    use AssayKind::*;
    use Verdict::*;
    vec![
        assay("a1", "r3", Hplc, "violacein titer", "3.4 g/L", Pass),
        assay("a2", "r3", PlateReader, "OD600 endpoint", "9.1", Pass),
        assay("a3", "r4", LcMs, "squalene purity", "96.2 %", Pass),
        assay("a4", "r4", Hplc, "squalene titer", "6.2 g/L", Pass),
        assay("a5", "r5", PlateReader, "OD600 endpoint", "1.1", Flag),
        assay("a6", "r6", Hplc, "limonene titer", "1.6 g/L", Pass),
        assay("a7", "r7", LcMs, "chitinase activity", "412 U/mL", Pass),
        assay("a8", "r9", Hplc, "violacein titer", "3.1 g/L", Pass),
    ]
}

fn seoul_samples() -> Vec<Sample> {
    use SampleForm::*;
    vec![
        sample("g1", "GLY-2201", "s1", Glycerol, "2 weeks ago"),
        sample("g2", "GLY-2202", "s1", Glycerol, "2 weeks ago"),
        sample("g3", "GLY-2188", "s2", Glycerol, "Mar 2026"),
        sample("g4", "PLM-0441", "s2", Plasmid, "Mar 2026"),
        sample("g5", "GLY-2154", "s5", Glycerol, "Feb 2026"),
        sample("g6", "PEL-0092", "s5", Pellet, "Feb 2026"),
        sample("g7", "GLY-2210", "s7", Glycerol, "May 2026"),
        sample("g8", "PLM-0450", "s4", Plasmid, "Jul 2026"),
        sample("g9", "GLY-2216", "s3", Glycerol, "Jul 2026"),
    ]
}

/* ---------------------------------------------------------------------------
 * #81 / #83 — the volume of a foundry that has actually been running.
 *
 * no ativ opened its Seoul floor in March 2019 and has been logging runs ever
 * since. That history is the point: seven years of runs, strains that were
 * archived three years ago, freezer racks whose oldest boxes are solid and
 * whose newest are half empty, and a roster with alumni on it. The
 * hand-authored records above stay at the top of every list — they carry the
 * domain's voice and every flow's happy path — and the generators below extend
 * each collection backwards through time.
 *
 * Everything is hashed from stable ids against a fixed epoch: no Math.random,
 * no Date.now, so the board is byte-identical on every load and screenshot.
 * ------------------------------------------------------------------------- */

const DAY: i64 = 86_400_000;

/// The board's "today" (`Date.UTC(2026, 7, 26)`, in ms). Fixed on purpose —
/// see the note above.
pub const BOARD_EPOCH: i64 = days_from_civil(2026, 8, 26) * DAY;

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Recent work reads as relative time; older work gets a date, the way a lab
/// notebook does.
fn stamp(days_ago: i64) -> String {
    if days_ago <= 0 {
        return s("today");
    }
    if days_ago == 1 {
        return s("yesterday");
    }
    if days_ago < 7 {
        return format!("{days_ago} days ago");
    }
    if days_ago < 21 {
        return format!("{} weeks ago", js_round(days_ago as f64 / 7.0));
    }
    let (y, m, d) = utc_date(days_ago);
    format!("{} {} {}", d, MONTHS[(m - 1) as usize], y)
}

/// The forward-looking twin of [`stamp`].
fn due(days_ahead: i64) -> String {
    match days_ahead {
        d if d <= 0 => s("today"),
        1 => s("tomorrow"),
        d => format!("in {d} days"),
    }
}

fn month_stamp(days_ago: i64) -> String {
    let (y, m, _) = utc_date(days_ago);
    format!("{} {}", MONTHS[(m - 1) as usize], y)
}

fn pick<T: Copy>(items: &[T], n: i64) -> T {
    items[idx(n, items.len())]
}

const PRODUCT_STEMS: [&str; 34] = [
    "limonene",
    "violacein",
    "PHB",
    "squalene",
    "chitinase",
    "indigo",
    "caffeine",
    "heme",
    "lycopene",
    "muconate",
    "itaconate",
    "farnesene",
    "carotene",
    "lactate",
    "succinate",
    "astaxanthin",
    "resveratrol",
    "valencene",
    "patchoulol",
    "shikimate",
    "putrescine",
    "butanediol",
    "glycolate",
    "terpineol",
    "nootkatone",
    "bisabolene",
    "santalene",
    "cadaverine",
    "malonate",
    "thebaine",
    "ectoine",
    "spidroin",
    "collagen",
    "insulin",
];
const STRAIN_SUFFIX: [&str; 12] = [
    "HP", "v2", "v3", "v4", "lite", "max", "delta", "sprint", "core", "alt", "X", "omega",
];
const PROMOTERS: [&str; 10] = [
    "pTrc", "pBAD", "pLac", "pTet", "pT7", "pGal", "pAOX1", "pGAP", "pOsmY", "pCsp",
];
const TERMINATORS: [&str; 5] = ["T7te", "rrnB", "T1", "tADH1", "tCYC1"];
const CDS: [&str; 14] = [
    "GPPS", "LS", "vioABCDE", "phaCAB", "ERG9", "chiA", "fmo", "ndmABCD", "crtEBI", "aroG", "ilvC",
    "mvaS", "idi", "tHMG1",
];
const STRAIN_STATES: [StrainStatus; 5] = [
    StrainStatus::Culturing,
    StrainStatus::Sequencing,
    StrainStatus::Validated,
    StrainStatus::Validated,
    StrainStatus::Archived,
];
const CONSTRUCT_STATES: [ConstructStatus; 4] = [
    ConstructStatus::Design,
    ConstructStatus::Synthesis,
    ConstructStatus::Verified,
    ConstructStatus::Verified,
];
const RUN_PHASES: [RunPhase; 5] = [
    RunPhase::Harvested,
    RunPhase::Harvested,
    RunPhase::Harvested,
    RunPhase::Harvested,
    RunPhase::Failed,
];
const ASSAY_KINDS: [AssayKind; 3] = [AssayKind::Hplc, AssayKind::LcMs, AssayKind::PlateReader];
const SAMPLE_FORMS: [SampleForm; 4] = [
    SampleForm::Glycerol,
    SampleForm::Glycerol,
    SampleForm::Plasmid,
    SampleForm::Pellet,
];
const PEOPLE: [&str; 48] = [
    "Yuna Seo",
    "Dmitri Volkov",
    "Hana Cho",
    "Marco Reyes",
    "Iris Tan",
    "Noor Haddad",
    "Felix Braun",
    "Anya Petrov",
    "Kwame Adjei",
    "Sofia Marchetti",
    "Jonas Lind",
    "Rhea Kapoor",
    "Tomas Keller",
    "Lea Fischer",
    "Ada Okonkwo",
    "Sven Halvorsen",
    "Mei Lin",
    "Priya Nair",
    "Owen Blackwood",
    "Camille Dubois",
    "Rafael Costa",
    "Ingrid Vogel",
    "Sana Rahman",
    "Elias Novak",
    "Hyejin Bae",
    "Tobias Sandberg",
    "Nadia Farouk",
    "Emil Kovac",
    "Rosa Alvarez",
    "Jae-won Lim",
    "Greta Simon",
    "Idris Bello",
    "Lucia Ferrari",
    "Aoife Byrne",
    "Nikhil Rao",
    "Vera Klimova",
    "Omar Nasser",
    "Sigrid Amundsen",
    "Bao Tran",
    "Clara Winter",
    "Yusuf Demir",
    "Marta Sosa",
    "Kenji Arai",
    "Helena Roth",
    "Diego Puentes",
    "Anika Berg",
    "Tariq Aziz",
    "Freya Lindqvist",
];

/// Codes count downward from the newest record, like a real ledger.
fn seq_code(prefix: &str, top: i64, index: i64, width: usize) -> String {
    format!(
        "{}{}",
        prefix,
        pad_start(&(top - index).max(0).to_string(), width)
    )
}

/// Work is not evenly spread over a company's life: the first years are thin,
/// the last two are dense. This maps record index → days ago with that curve,
/// so scrolling back through the ledger feels like scrolling back through time.
fn age_curve(index: usize, count: usize, span_days: f64) -> i64 {
    let t = if count <= 1 {
        0.0
    } else {
        index as f64 / (count - 1) as f64
    };
    js_round(span_days * t.powf(1.6))
}

fn gen_constructs(ns: &str, count: i64, top_number: i64, span_days: i64) -> Vec<Construct> {
    let count = count.max(0) as usize;
    (0..count)
        .map(|i| {
            let h = hash_text(&format!("{ns}-construct-{i}"));
            let stem = pick(&PRODUCT_STEMS, h);
            let kind = pick(
                &[
                    "module", "operon", "cassette", "pathway", "reporter", "library",
                ],
                sr(h, 3),
            );
            Construct {
                id: format!("{ns}-c{}", i + 100),
                code: format!(
                    "pNV-{}",
                    pad_start(&(top_number - i as i64).max(1).to_string(), 3)
                ),
                name: format!("{stem} {kind} v{}", 1 + h % 5),
                bp: 3200 + (h % 210) * 100,
                gc: to_fixed_num(43.0 + (h % 220) as f64 / 10.0, 1),
                // Old constructs are settled; only recent ones are still in design.
                status: if age_curve(i, count, span_days as f64) > 240 {
                    ConstructStatus::Verified
                } else {
                    pick(&CONSTRUCT_STATES, sr(h, 5))
                },
                parts: vec![
                    s(pick(&PROMOTERS, h)),
                    s(pick(&CDS, sr(h, 2))),
                    s(pick(&CDS, sr(h, 6))),
                    s(pick(&TERMINATORS, sr(h, 4))),
                ],
            }
        })
        .collect()
}

fn gen_strains(
    ns: &str,
    count: i64,
    top_code: i64,
    span_days: i64,
    constructs: &[Construct],
) -> Vec<Strain> {
    let count = count.max(0) as usize;
    (0..count)
        .map(|i| {
            let h = hash_text(&format!("{ns}-strain-{i}"));
            let days_ago = age_curve(i, count, span_days as f64);
            let primary = &constructs[idx(h, constructs.len())];
            let second = &constructs[idx(sr(h, 4), constructs.len())];
            Strain {
                id: format!("{ns}-s{}", i + 100),
                code: format!(
                    "NV-{}",
                    pad_start(&(top_code - i as i64).max(1).to_string(), 3)
                ),
                name: format!(
                    "{} {}",
                    pick(&PRODUCT_STEMS, sr(h, 1)),
                    pick(&STRAIN_SUFFIX, sr(h, 7))
                ),
                chassis: pick(&CHASSIS_OPTIONS, sr(h, 3)),
                // A strain older than a year is retired unless it earned its keep.
                status: if days_ago > 420 {
                    if h % 5 == 0 {
                        StrainStatus::Validated
                    } else {
                        StrainStatus::Archived
                    }
                } else {
                    pick(&STRAIN_STATES, sr(h, 5))
                },
                generation: 1 + h % 26,
                fitness: 18 + h % 80,
                owner: s(pick(&PEOPLE, sr(h, 2))),
                updated: stamp(days_ago),
                construct_ids: if h % 3 == 0 {
                    vec![primary.id.clone(), second.id.clone()]
                } else {
                    vec![primary.id.clone()]
                },
                parent_id: None,
            }
        })
        .collect()
}

fn gen_runs(ns: &str, count: i64, top_code: i64, span_days: i64, strains: &[Strain]) -> Vec<Run> {
    let count = count.max(0) as usize;
    (0..count)
        .map(|i| {
            let h = hash_text(&format!("{ns}-run-{i}"));
            let days_ago = age_curve(i, count, span_days as f64);
            let strain = &strains[idx(h, strains.len())];
            // A floor runs a handful of vessels at once, no matter how long its
            // ledger is: only the newest few rows are still on the deck.
            let phase = if i < 5 {
                pick(&[RunPhase::Log, RunPhase::Lag, RunPhase::Stationary], h)
            } else {
                pick(&RUN_PHASES, sr(h, 3))
            };
            let od = if phase == RunPhase::Lag {
                0.2 + (h % 8) as f64 / 10.0
            } else {
                2.0 + (h % 130) as f64 / 10.0
            };
            let finished = phase == RunPhase::Harvested || phase == RunPhase::Failed;
            Run {
                id: format!("{ns}-r{}", i + 100),
                code: seq_code("R-", top_code, i as i64, 4),
                strain_id: strain.id.clone(),
                vessel: s(pick(&VESSELS, sr(h, 2))),
                instrument_id: None,
                lot_ids: Vec::new(),
                charge_l: None,
                phase,
                od600: to_fixed_num(od, 1),
                titer: match phase {
                    RunPhase::Failed => s("0"),
                    RunPhase::Lag => s("—"),
                    _ => format!("{} g/L", to_fixed(0.2 + (h % 90) as f64 / 10.0, 1)),
                },
                started: stamp(days_ago),
                duration: if finished {
                    format!("{}h {}m", 8 + h % 40, pad_start(&(h % 60).to_string(), 2))
                } else {
                    s("—")
                },
                operator: s(pick(&PEOPLE, sr(h, 6))),
            }
        })
        .collect()
}

/// Completed runs carry readouts; failures carry the flag that killed them.
fn gen_assays(ns: &str, runs: &[Run]) -> Vec<Assay> {
    let mut out: Vec<Assay> = Vec::new();
    for run in runs {
        if run.phase == RunPhase::Lag || run.phase == RunPhase::Log {
            continue;
        }
        let h = hash_text(&format!("{ns}-assay-{}", run.id));
        let count = if run.phase == RunPhase::Failed {
            1
        } else {
            1 + h % 2
        };
        for k in 0..count {
            let kind = pick(&ASSAY_KINDS, h + k);
            let metric = if kind == AssayKind::PlateReader {
                s("OD600 endpoint")
            } else {
                format!(
                    "{} {}",
                    pick(&PRODUCT_STEMS, h + k * 3),
                    pick(&["titer", "purity", "activity"], h + k)
                )
            };
            let value = match kind {
                AssayKind::PlateReader => to_fixed(run.od600, 1),
                AssayKind::LcMs => format!("{} %", to_fixed(70.0 + (h % 290) as f64 / 10.0, 1)),
                AssayKind::Hplc => run.titer.clone(),
            };
            let verdict = if run.phase == RunPhase::Failed || sr(h, (k + 2) as u32) % 9 == 0 {
                Verdict::Flag
            } else {
                Verdict::Pass
            };
            out.push(Assay {
                id: format!("{ns}-a{}", out.len() + 100),
                run_id: run.id.clone(),
                kind,
                metric,
                value,
                verdict,
            });
        }
    }
    out
}

fn gen_samples(
    ns: &str,
    count: i64,
    top_code: i64,
    span_days: i64,
    strains: &[Strain],
) -> Vec<Sample> {
    let count = count.max(0) as usize;
    (0..count)
        .map(|i| {
            let h = hash_text(&format!("{ns}-sample-{i}"));
            let form = pick(&SAMPLE_FORMS, h);
            let prefix = match form {
                SampleForm::Glycerol => "GLY-",
                SampleForm::Plasmid => "PLM-",
                SampleForm::Pellet => "PEL-",
            };
            Sample {
                id: format!("{ns}-g{}", i + 100),
                code: seq_code(prefix, top_code, i as i64, 4),
                strain_id: strains[idx(h, strains.len())].id.clone(),
                form,
                frozen_at: month_stamp(age_curve(i, count, span_days as f64)),
            }
        })
        .collect()
}

/// Real −80 °C boxes are 9×9. Racks fill front-to-back over the years, so the
/// oldest rack is solid and the newest is mostly spare — the shape a freezer
/// audit actually finds.
pub const BOX_SLOTS: usize = 81;

fn gen_racks(
    ns: &str,
    rack_count: usize,
    boxes_per_rack: usize,
    samples: &[Sample],
) -> Vec<FreezerRack> {
    // Oldest samples went in first, so fill the racks from the back of the list.
    let queue: Vec<&Sample> = samples.iter().rev().collect();
    let mut cursor = 0usize;
    (0..rack_count)
        .map(|rack_index| {
            let letter = char::from(65 + rack_index as u8);
            let boxes = (0..boxes_per_rack)
                .map(|box_index| {
                    let h = hash_text(&format!("{ns}-box-{rack_index}-{box_index}"));
                    let fill_rate = f64::max(
                        0.04,
                        0.97 - (rack_index as f64 / (rack_count as f64 - 1.0).max(1.0)) * 0.9
                            - (box_index % 4) as f64 * 0.03,
                    );
                    let slots = (0..BOX_SLOTS)
                        .map(|slot| {
                            let taken = ((h + slot as i64 * 37) % 100) as f64 / 100.0 < fill_rate
                                && cursor < queue.len();
                            if !taken {
                                return None;
                            }
                            let sample = queue[cursor];
                            cursor += 1;
                            Some(sample.id.clone())
                        })
                        .collect();
                    let id = format!("{letter}{}", box_index + 1);
                    FreezerBox {
                        id: id.clone(),
                        label: id,
                        slots,
                    }
                })
                .collect();
            FreezerRack {
                id: format!("rack-{}", letter.to_ascii_lowercase()),
                name: format!("Rack {letter}"),
                temp: s(if rack_index == rack_count - 1 {
                    "−20 °C"
                } else {
                    "−80 °C"
                }),
                boxes,
            }
        })
        .collect()
}

/// Seven years of hiring means a roster *and* an alumni list.
fn gen_members(ns: &str, count: usize, span_days: i64, existing: Vec<Member>) -> Vec<Member> {
    let pool: Vec<&str> = PEOPLE
        .iter()
        .copied()
        .filter(|name| !existing.iter().any(|m| m.name == *name))
        .collect();
    let extra_count = count.saturating_sub(existing.len());
    let extra: Vec<Member> = (0..extra_count)
        .map(|i| {
            let h = hash_text(&format!("{ns}-member-{i}"));
            let joined_days_ago =
                js_round(span_days as f64 * (0.08 + ((h % 90) as f64 / 100.0) * 0.9));
            let name = pool[i % pool.len()];
            let handle = name.split(' ').next().unwrap_or("").to_lowercase();
            // Turnover rises with tenure: the earliest hires are mostly alumni now.
            let left = if joined_days_ago > 900 && h % 3 != 0 {
                Some(month_stamp(js_round(joined_days_ago as f64 * 0.35)))
            } else {
                None
            };
            Member {
                id: format!("{ns}-m{}", i + 100),
                name: s(name),
                email: format!(
                    "{handle}{}@noativ.bio",
                    if i >= pool.len() {
                        i.to_string()
                    } else {
                        String::new()
                    }
                ),
                role: pick(
                    &[
                        RoleId::Scientist,
                        RoleId::Scientist,
                        RoleId::Technician,
                        RoleId::Observer,
                    ],
                    h,
                ),
                joined: month_stamp(joined_days_ago),
                left,
            }
        })
        .collect();
    let mut out = existing;
    out.extend(extra);
    out
}

/* ---------------------------------------------------------------------------
 * #84 / #85 — the sections a foundry has that a workspace app never would.
 * Same rules as the rest of the ledger: hashed from stable ids, no randomness.
 * ------------------------------------------------------------------------- */

/// (kind, vessel, count)
const INSTRUMENT_KINDS: [(&str, &str, usize); 4] = [
    ("BR-2K", "BR-2K · 2 L", 4),
    ("BR-250", "BR-250 · 250 mL", 6),
    ("FLASK", "FLASK · 500 mL baffled", 5),
    ("PLATE", "PLATE · 96-well", 3),
];

const INSTRUMENT_STATES: [InstrumentState; 6] = [
    InstrumentState::Ready,
    InstrumentState::Running,
    InstrumentState::Ready,
    InstrumentState::Cip,
    InstrumentState::Ready,
    InstrumentState::Down,
];

fn gen_instruments(ns: &str) -> Vec<Instrument> {
    let mut out: Vec<Instrument> = Vec::new();
    for (kind, vessel, count) in INSTRUMENT_KINDS {
        for i in 0..count {
            let h = hash_text(&format!("{ns}-inst-{kind}-{i}"));
            let state = if out.len() < 3 {
                InstrumentState::Running
            } else {
                pick(&INSTRUMENT_STATES, h)
            };
            out.push(Instrument {
                id: format!("{ns}-i-{kind}-{i}"),
                code: format!("{kind}-{}", pad_start(&(i + 1).to_string(), 2)),
                kind: s(vessel),
                state,
                since_hours: 1 + h % 72,
                cycles: 120 + h % 1400,
            });
        }
    }
    out
}

/// Three weeks of deck: last week done, this week live, next week booked.
pub const DECK_DAYS: i64 = 21;
pub const DECK_TODAY: i64 = 13;

fn gen_deck(
    ns: &str,
    instruments: &[Instrument],
    runs: &[Run],
    strains: &[Strain],
) -> Vec<DeckBlock> {
    let mut out: Vec<DeckBlock> = Vec::new();
    let strain_index: HashMap<&str, &Strain> = {
        // `Array.prototype.find` returns the first match, so keep the first.
        let mut m = HashMap::new();
        for st in strains {
            m.entry(st.id.as_str()).or_insert(st);
        }
        m
    };
    for (lane, instrument) in instruments.iter().enumerate() {
        let mut cursor = hash_text(&format!("{ns}-deck-start-{lane}")) % 3;
        let mut slot = 0;
        while cursor < DECK_DAYS {
            let h = hash_text(&format!("{ns}-deck-{lane}-{slot}"));
            let kind = if h % 11 == 0 {
                DeckKind::Maintenance
            } else if h % 5 == 0 {
                DeckKind::Cip
            } else if cursor > DECK_TODAY && h % 3 == 0 {
                DeckKind::Booked
            } else {
                DeckKind::Run
            };
            let days = if kind == DeckKind::Run { 1 + h % 4 } else { 1 };
            let run = if kind == DeckKind::Run {
                Some(&runs[idx(sr(h, 3), runs.len())])
            } else {
                None
            };
            let strain = run.and_then(|r| strain_index.get(r.strain_id.as_str()).copied());
            out.push(DeckBlock {
                id: format!("{ns}-d{lane}-{slot}"),
                instrument_id: instrument.id.clone(),
                run_id: run.map(|r| r.id.clone()),
                strain_id: strain.map(|st| st.id.clone()),
                day: cursor,
                days: days.min(DECK_DAYS - cursor),
                kind,
                label: match kind {
                    DeckKind::Run => format!(
                        "{} · {}",
                        run.map(|r| r.code.as_str()).unwrap_or(""),
                        strain.map(|st| st.code.as_str()).unwrap_or("")
                    ),
                    DeckKind::Cip => s("CIP"),
                    DeckKind::Maintenance => s("PM"),
                    DeckKind::Booked => s("booked"),
                },
                operator: s(pick(&PEOPLE, sr(h, 5))),
            });
            // A machine is never back-to-back: there is always turnaround.
            cursor += days + 1 + h % 2;
            slot += 1;
        }
    }
    out
}

const INCIDENT_AREAS: [IncidentArea; 6] = [
    IncidentArea::Bioreactor,
    IncidentArea::Freezer,
    IncidentArea::Media,
    IncidentArea::Sequencing,
    IncidentArea::Safety,
    IncidentArea::Facility,
];

fn incident_titles(area: IncidentArea) -> [&'static str; 4] {
    match area {
        IncidentArea::Bioreactor => [
            "foam breakthrough during feed",
            "DO probe drift beyond spec",
            "agitator seal weep",
            "off-gas CO₂ sensor flatline",
        ],
        IncidentArea::Freezer => [
            "−80 door ajar alarm",
            "rack B compressor short-cycling",
            "inventory mismatch on audit",
            "backup CO₂ bank low",
        ],
        IncidentArea::Media => [
            "LB lot failed sterility hold",
            "antifoam dosed twice",
            "pH drift in prepared buffer",
            "autoclave cycle aborted",
        ],
        IncidentArea::Sequencing => [
            "low Q30 on run flowcell",
            "index hopping suspected",
            "library prep bead loss",
            "basecall pipeline stalled",
        ],
        IncidentArea::Safety => [
            "sharps container overfilled",
            "BSC airflow below setpoint",
            "spill kit expired",
            "eyewash flush overdue",
        ],
        IncidentArea::Facility => [
            "DI water loop pressure drop",
            "UPS transferred to battery",
            "HVAC filter pressure high",
            "chilled water excursion",
        ],
    }
}

const INCIDENT_SEVERITIES: [IncidentSeverity; 6] = [
    IncidentSeverity::Minor,
    IncidentSeverity::Minor,
    IncidentSeverity::Minor,
    IncidentSeverity::Major,
    IncidentSeverity::Major,
    IncidentSeverity::Critical,
];

fn gen_incidents(ns: &str, count: usize, span_days: i64, runs: &[Run]) -> Vec<Incident> {
    (0..count)
        .map(|i| {
            let h = hash_text(&format!("{ns}-incident-{i}"));
            let days_ago = age_curve(i, count, span_days as f64);
            let area = pick(&INCIDENT_AREAS, h);
            let severity = pick(&INCIDENT_SEVERITIES, sr(h, 3));
            // Old deviations are closed; only the recent tail is still moving.
            let status = if days_ago > 45 {
                IncidentStatus::Closed
            } else if days_ago > 14 {
                pick(&[IncidentStatus::Closed, IncidentStatus::Capa], sr(h, 5))
            } else if days_ago > 4 {
                pick(
                    &[IncidentStatus::Investigating, IncidentStatus::Capa],
                    sr(h, 6),
                )
            } else {
                IncidentStatus::Open
            };
            let run = if h % 3 == 0 {
                Some(&runs[idx(sr(h, 4), runs.len())])
            } else {
                None
            };
            let age = if status == IncidentStatus::Closed {
                2 + h % 21
            } else {
                days_ago.clamp(1, 30)
            };
            let title = pick(&incident_titles(area), sr(h, 2));
            let mut timeline = vec![
                TimelineEntry {
                    at: stamp(days_ago),
                    text: format!("Raised by {} — {}", pick(&PEOPLE, sr(h, 6)), title),
                },
                TimelineEntry {
                    at: stamp((days_ago - 1).max(0)),
                    text: format!(
                        "Area quarantined, {}",
                        if severity == IncidentSeverity::Critical {
                            "floor lead paged"
                        } else {
                            "shift lead notified"
                        }
                    ),
                },
            ];
            if status != IncidentStatus::Open {
                timeline.push(TimelineEntry {
                    at: stamp((days_ago - 2).max(0)),
                    text: s("Root cause analysis opened (5-why)"),
                });
            }
            if status == IncidentStatus::Capa || status == IncidentStatus::Closed {
                timeline.push(TimelineEntry {
                    at: stamp((days_ago - 4).max(0)),
                    text: format!(
                        "CAPA filed — {}",
                        pick(
                            &[
                                "SOP revised",
                                "probe replaced",
                                "training re-run",
                                "PM interval halved"
                            ],
                            sr(h, 7)
                        )
                    ),
                });
            }
            if status == IncidentStatus::Closed {
                timeline.push(TimelineEntry {
                    at: stamp((days_ago - age).max(0)),
                    text: format!("Closed after {age} days · verified by {}", pick(&PEOPLE, h)),
                });
            }
            Incident {
                id: format!("{ns}-inc{}", i + 100),
                code: format!(
                    "DEV-{}",
                    pad_start(&(2400 - i as i64).max(1).to_string(), 4)
                ),
                title: s(title),
                area,
                severity,
                status,
                opened: stamp(days_ago),
                owner: s(pick(&PEOPLE, sr(h, 1))),
                run_id: run.map(|r| r.id.clone()),
                instrument_id: None,
                lot_id: None,
                age,
                timeline,
            }
        })
        .collect()
}

/// (name, unit, capacity, grade)
const REAGENTS: [(&str, &str, f64, &str); 18] = [
    ("LB Miller broth", "kg", 25.0, "USP"),
    ("Terrific broth", "kg", 25.0, "USP"),
    ("Glucose monohydrate", "kg", 50.0, "food"),
    ("Glycerol 99.5%", "L", 20.0, "ACS"),
    ("IPTG", "g", 100.0, "mol bio"),
    ("L-arabinose", "g", 250.0, "mol bio"),
    ("Kanamycin sulfate", "g", 50.0, "cell culture"),
    ("Carbenicillin", "g", 25.0, "cell culture"),
    ("Antifoam 204", "L", 4.0, "tech"),
    ("Trace metals stock", "L", 10.0, "in-house"),
    ("Yeast extract", "kg", 25.0, "USP"),
    ("Ammonium sulfate", "kg", 25.0, "ACS"),
    ("Sodium phosphate dibasic", "kg", 10.0, "ACS"),
    ("Magnesium sulfate", "kg", 10.0, "ACS"),
    ("Q5 polymerase", "rxn", 500.0, "mol bio"),
    ("Gibson master mix", "rxn", 200.0, "mol bio"),
    ("Plasmid midiprep kit", "preps", 100.0, "mol bio"),
    ("Agarose LE", "kg", 5.0, "mol bio"),
];
const VENDORS: [&str; 6] = [
    "Kyoto Bio",
    "Meridian Labs",
    "Halcyon Reagents",
    "Nordwerk",
    "Sable Chem",
    "Tessellate",
];
const SHELVES: [&str; 6] = ["MK-A", "MK-B", "MK-C", "COLD-1", "COLD-2", "FLAM-1"];

fn gen_lots(ns: &str, count: usize) -> Vec<ReagentLot> {
    (0..count)
        .map(|i| {
            let h = hash_text(&format!("{ns}-lot-{i}"));
            let (name, unit, capacity, grade) = pick(&REAGENTS, h);
            let shelf_total = 180 + (h % 5) * 180;
            // Newest lots first; the tail of the list is old stock and expired stock.
            let shelf_days =
                js_round((shelf_total - age_curve(i, count, shelf_total as f64 * 1.25)) as f64);
            let qty = to_fixed_num(capacity * ((sr(h, 3) % 100) as f64 / 100.0).powf(1.4), 2);
            ReagentLot {
                id: format!("{ns}-lot{}", i + 100),
                code: format!(
                    "LOT-{}",
                    pad_start(&(7400 - i as i64 * 3).max(100).to_string(), 4)
                ),
                name: s(name),
                vendor: s(pick(&VENDORS, sr(h, 2))),
                grade: s(grade),
                received: month_stamp((shelf_total - shelf_days).max(0)),
                expires: if shelf_days >= 0 {
                    due(shelf_days)
                } else {
                    format!("expired {}", stamp(-shelf_days))
                },
                shelf_days,
                shelf_total,
                qty,
                capacity,
                unit: s(unit),
                location: format!("{}·{}", pick(&SHELVES, sr(h, 4)), 1 + h % 9),
                reserved_for: if h % 13 == 0 {
                    Some(format!("R-{}", pad_start(&(4700 - h % 40).to_string(), 4)))
                } else {
                    None
                },
            }
        })
        .collect()
}

/// (kind, unit, low, high, nominal)
const SENSOR_SPECS: [(SensorKind, &str, f64, f64, f64); 6] = [
    (SensorKind::Temp, "°C", 18.0, 24.0, 21.0),
    (SensorKind::Co2, "ppm", 400.0, 1200.0, 640.0),
    (SensorKind::Rh, "%", 35.0, 60.0, 46.0),
    (SensorKind::Do, "%", 20.0, 100.0, 63.0),
    (SensorKind::Ph, "pH", 6.6, 7.4, 7.0),
    (SensorKind::Pressure, "Pa", 5.0, 25.0, 14.0),
];
const ZONES: [&str; 7] = [
    "Fermentation hall",
    "Media kitchen",
    "Cryo room",
    "Molecular lab",
    "Prep bay",
    "Airlock",
    "Utility loft",
];

/// Out of range is an alarm; the outer 12 % of the range on either side warns.
pub fn sensor_state(value: f64, low: f64, high: f64) -> SensorState {
    let margin = (high - low) * 0.12;
    if value < low || value > high {
        SensorState::Alarm
    } else if value < low + margin || value > high - margin {
        SensorState::Warn
    } else {
        SensorState::Ok
    }
}

fn gen_sensors(ns: &str) -> Vec<Sensor> {
    let mut out: Vec<Sensor> = Vec::new();
    for (z, zone) in ZONES.iter().enumerate() {
        for (k, &(kind, unit, low, high, nominal)) in SENSOR_SPECS.iter().enumerate() {
            let h = hash_text(&format!("{ns}-sensor-{z}-{k}"));
            let swing = (high - low) * (((h % 100) as f64 / 100.0) - 0.5) * 0.9;
            let value = to_fixed_num(nominal + swing, if kind == SensorKind::Ph { 2 } else { 0 });
            let state = sensor_state(value, low, high);
            let word = zone.split(' ').next().unwrap_or("");
            let head: String = word.chars().take(4).collect();
            out.push(Sensor {
                id: format!("{ns}-sn-{z}-{k}"),
                code: format!(
                    "{}-{}-{}",
                    head.to_uppercase(),
                    kind.as_str().to_uppercase(),
                    z + 1
                ),
                zone: s(zone),
                kind,
                value,
                unit: s(unit),
                low,
                high,
                state,
            });
        }
    }
    out
}

const DESTINATIONS: [&str; 9] = [
    "Basel Wetlab",
    "Meridian Port QC",
    "Kyoto partner lab",
    "Cambridge collaborator",
    "São Paulo pilot plant",
    "Reykjavík biobank",
    "Lagos field site",
    "Toronto CRO",
    "Singapore scale-up",
];
const COURIERS: [&str; 4] = [
    "CryoFreight",
    "Nordwerk Logistics",
    "PolarLine",
    "Meridian Express",
];
const STAGES: [ShipmentStage; 5] = [
    ShipmentStage::Packed,
    ShipmentStage::PickedUp,
    ShipmentStage::InTransit,
    ShipmentStage::Customs,
    ShipmentStage::Delivered,
];

fn gen_shipments(ns: &str, count: usize, span_days: i64) -> Vec<Shipment> {
    (0..count)
        .map(|i| {
            let h = hash_text(&format!("{ns}-ship-{i}"));
            let days_ago = age_curve(i, count, span_days as f64);
            let stage = if days_ago > 12 {
                ShipmentStage::Delivered
            } else {
                STAGES[(days_ago.div_euclid(3)).min(4) as usize]
            };
            let vials = 4 + h % 40;
            Shipment {
                id: format!("{ns}-shp{}", i + 100),
                code: format!(
                    "SHP-{}",
                    pad_start(&(1800 - i as i64).max(1).to_string(), 4)
                ),
                destination: s(pick(&DESTINATIONS, h)),
                courier: s(pick(&COURIERS, sr(h, 3))),
                vials,
                dry_ice_kg: (3 + h % 18) as f64,
                stage,
                eta_days: if stage == ShipmentStage::Delivered {
                    -days_ago
                } else {
                    1 + h % 5
                },
                eta: if stage == ShipmentStage::Delivered {
                    format!("delivered {}", stamp(days_ago))
                } else {
                    due(1 + h % 5)
                },
                excursions: if h % 17 == 0 { 1 + h % 2 } else { 0 },
            }
        })
        .collect()
}

/// Descent: every strain but the founders was passaged from an older one. The
/// list runs newest → oldest, so a parent always sits at a higher index.
fn assign_lineage(ns: &str, strains: &[Strain]) -> Vec<Strain> {
    strains
        .iter()
        .enumerate()
        .map(|(i, strain)| {
            let h = hash_text(&format!("{ns}-lineage-{}", strain.id));
            // Parents cluster on purpose: a strain that worked gets passaged several
            // times, so candidates snap to every fourth index and the tree branches
            // instead of running as one long chain.
            let i = i as i64;
            let reach = i + 2 + h % 14;
            let snapped = reach - reach % 4;
            let parent_index = if snapped > i { snapped } else { reach };
            let orphan = h % 11 == 0 || parent_index >= strains.len() as i64;
            Strain {
                parent_id: if orphan {
                    None
                } else {
                    Some(strains[parent_index as usize].id.clone())
                },
                ..strain.clone()
            }
        })
        .collect()
}

/// #88 — the ledger stops being parallel lists and becomes a graph: a run knows
/// the machine it occupied, and the lots it drew; a
/// deviation knows what failed. Every drill in the app follows these edges.
fn link_runs(
    ns: &str,
    runs: Vec<Run>,
    instruments: &[Instrument],
    lots: &[ReagentLot],
) -> Vec<Run> {
    let mut by_kind: HashMap<&str, Vec<&Instrument>> = HashMap::new();
    for instrument in instruments {
        by_kind
            .entry(instrument.kind.as_str())
            .or_default()
            .push(instrument);
    }
    let all: Vec<&Instrument> = instruments.iter().collect();
    runs.into_iter()
        .map(|mut run| {
            let h = hash_text(&format!("{ns}-link-{}", run.id));
            let pool = by_kind.get(run.vessel.as_str()).unwrap_or(&all);
            let instrument = if !pool.is_empty() {
                Some(pool[idx(h, pool.len())])
            } else {
                None
            };
            // Media, carbon source, an inducer and an antibiotic: the four things
            // every charge draws.
            let lot_ids = if !lots.is_empty() {
                (0..3 + h % 2)
                    .map(|k| lots[idx(h + k * 29, lots.len())].id.clone())
                    .collect()
            } else {
                Vec::new()
            };
            let charge_l = if run.vessel.starts_with("BR-2K") {
                1.6 + (h % 5) as f64 / 10.0
            } else if run.vessel.starts_with("BR-250") {
                0.18 + (h % 4) as f64 / 100.0
            } else if run.vessel.starts_with("FLASK") {
                0.1 + (h % 3) as f64 / 50.0
            } else {
                0.02
            };
            run.instrument_id = instrument.map(|x| x.id.clone());
            run.lot_ids = lot_ids;
            run.charge_l = Some(charge_l);
            run
        })
        .collect()
}

fn link_incidents(
    ns: &str,
    incidents: Vec<Incident>,
    instruments: &[Instrument],
    lots: &[ReagentLot],
) -> Vec<Incident> {
    incidents
        .into_iter()
        .map(|mut incident| {
            let h = hash_text(&format!("{ns}-inclink-{}", incident.id));
            let machine_area = incident.area == IncidentArea::Bioreactor
                || incident.area == IncidentArea::Facility;
            let lot_area = incident.area == IncidentArea::Media;
            incident.instrument_id = if machine_area && !instruments.is_empty() {
                Some(instruments[idx(h, instruments.len())].id.clone())
            } else {
                None
            };
            incident.lot_id = if lot_area && !lots.is_empty() {
                Some(lots[idx(h, lots.len())].id.clone())
            } else {
                None
            };
            incident
        })
        .collect()
}

/// The Seoul floor at handheld scale. The hand-authored records lead every
/// list and carry the domain's voice; the generators extend each collection
/// back through seven years of history, trimmed to what a phone keeps on device.
pub fn seed() -> FacilityData {
    const NS: &str = "sf";
    const SPAN: i64 = 2725;
    let extra = |target: usize, have: usize| target as i64 - have as i64;

    let mut constructs = seoul_constructs();
    constructs.extend(gen_constructs(NS, extra(60, constructs.len()), 370, SPAN));

    let mut strains = seoul_strains();
    strains.extend(gen_strains(
        NS,
        extra(96, strains.len()),
        640,
        SPAN,
        &constructs,
    ));

    let mut runs = seoul_runs();
    let authored_runs = runs.len();
    runs.extend(gen_runs(NS, extra(160, runs.len()), 4702, SPAN, &strains));

    let mut assays = seoul_assays();
    assays.extend(gen_assays(NS, &runs[authored_runs..]));

    let mut samples = seoul_samples();
    samples.extend(gen_samples(
        NS,
        extra(1400, samples.len()),
        9000,
        SPAN,
        &strains,
    ));
    let racks = gen_racks(NS, 4, 6, &samples);

    let strains = assign_lineage(NS, &strains);
    let instruments = gen_instruments(NS);
    let deck = gen_deck(NS, &instruments, &runs, &strains);
    let lots = gen_lots(NS, 48);
    let incidents = link_incidents(NS, gen_incidents(NS, 40, SPAN, &runs), &instruments, &lots);
    let runs = link_runs(NS, runs, &instruments, &lots);

    FacilityData {
        strains,
        constructs,
        runs,
        assays,
        samples,
        racks,
        instruments,
        deck,
        incidents,
        lots,
        sensors: gen_sensors(NS),
        shipments: gen_shipments(NS, 14, SPAN),
        members: gen_members(NS, 24, SPAN, seoul_members()),
        biosafety: Biosafety::Bsl1,
        opened: month_stamp(SPAN),
        years: js_round((SPAN as f64 / 365.0) * 10.0) as f64 / 10.0,
    }
}

/// Deterministic fake bioreactor log.
pub fn run_log(run: &Run) -> Vec<String> {
    let mut out = vec![
        format!("$ noativ run attach {}", run.code),
        format!("vessel {} · operator {}", run.vessel, run.operator),
        format!("inoculated {} · OD600 0.05", run.started),
    ];
    let od = to_fixed(run.od600, 1);
    let tail: Vec<String> = match run.phase {
        RunPhase::Lag => vec![
            s("phase: LAG — adaptation, DO 98 %"),
            format!("OD600 {od} and holding"),
        ],
        RunPhase::Log => vec![
            s("phase: LOG — µ 0.42 h⁻¹, DO 41 %"),
            s("feed pump 2.1 mL/min"),
            format!("OD600 {od} ↑ · titer {}", run.titer),
        ],
        RunPhase::Stationary => {
            vec![
                s("phase: STATIONARY — µ ~0, DO 78 %"),
                format!("OD600 {od} plateau · titer {}", run.titer),
            ]
        }
        RunPhase::Harvested => vec![
            s("phase: LOG — peak µ 0.51 h⁻¹"),
            format!("phase: STATIONARY at OD600 {od}"),
            format!(
                "harvested after {} · final titer {}",
                run.duration, run.titer
            ),
            s("CIP cycle complete"),
        ],
        RunPhase::Failed => vec![
            s("phase: LOG — µ 0.18 h⁻¹ (low)"),
            s("ALERT foam sensor tripped"),
            s("ALERT off-gas CO₂ collapsed — suspected contamination"),
            format!("run aborted after {}", run.duration),
        ],
    };
    out.extend(tail);
    out
}

/// Deterministic pseudo-measurements — no randomness at runtime.
///
/// `(h * 31 + code) | 0` over UTF-16 code units, then `Math.abs`. The result is
/// in `0..=2^31` (2^31 only for `abs(i32::MIN)`), hence `i64`.
fn hash_text(text: &str) -> i64 {
    let mut h: i32 = 0;
    for unit in text.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(unit as i32);
    }
    (h as i64).abs()
}

/// 24-point OD600 curve shaped by the run's phase.
pub fn od_series(run: &Run) -> Vec<f64> {
    let seed = (hash_text(&run.id) % 13) as f64;
    let wiggle = |i: f64| 0.05 * (i * 1.7 + seed).sin();
    let od = run.od600;
    let logistic = |t: f64| od / (1.0 + (-9.0 * (t - 0.55)).exp());
    (0..24)
        .map(|i| {
            let fi = i as f64;
            let t = fi / 23.0;
            match run.phase {
                RunPhase::Lag => f64::max(0.05 + 0.3 * t + wiggle(fi) * 0.3, 0.02),
                RunPhase::Log => f64::max(logistic(t * 0.7) + wiggle(fi), 0.02),
                RunPhase::Stationary | RunPhase::Harvested => {
                    f64::max(logistic(t) + wiggle(fi), 0.02)
                }
                RunPhase::Failed => f64::max(
                    if t < 0.5 {
                        logistic(t)
                    } else {
                        logistic(0.5) * (1.0 - (t - 0.5) * 1.4) + wiggle(fi)
                    },
                    0.02,
                ),
            }
        })
        .collect()
}

/* ------------------------------------------------------------------------- */
/* JS semantics helpers                                                      */
/* ------------------------------------------------------------------------- */

/// JS `h >> n`: ToInt32 then arithmetic shift.
fn sr(h: i64, n: u32) -> i64 {
    ((h as u32 as i32) >> (n & 31)) as i64
}

/// `n % len` as an index. JS would yield `undefined` for a negative `n`, which
/// cannot happen for real hashes; wrap instead of panicking.
fn idx(n: i64, len: usize) -> usize {
    n.rem_euclid(len as i64) as usize
}

/// `Math.round`: nearest integer, ties toward +∞.
fn js_round(x: f64) -> i64 {
    let f = x.floor();
    (if x - f >= 0.5 { f + 1.0 } else { f }) as i64
}

/// `String.prototype.padStart(width, '0')`.
fn pad_start(v: &str, width: usize) -> String {
    let len = v.encode_utf16().count();
    if len >= width {
        v.to_string()
    } else {
        format!("{}{}", "0".repeat(width - len), v)
    }
}

/// `Number.prototype.toFixed(digits)`: rounds the *exact* binary value, ties
/// to the larger magnitude (unlike Rust's `{:.N}`, which ties to even).
pub fn to_fixed(x: f64, digits: usize) -> String {
    if x.is_nan() {
        return s("NaN");
    }
    if x.abs() >= 1e21 {
        return js_num(x);
    }
    let neg = x < 0.0;
    let a = x.abs();
    // Enough fractional digits to print `a` exactly (no rounding by `format!`).
    let bits = a.to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i64;
    let lsb = if exp == 0 { -1074 } else { exp - 1075 };
    let exact = (-lsb).max(0) as usize;
    let text = format!("{:.*}", exact.max(digits + 1), a);
    let (int_part, frac_part) = text.split_once('.').unwrap_or((&text, ""));
    let mut digits_vec: Vec<u8> = int_part
        .bytes()
        .chain(frac_part.bytes().take(digits))
        .collect();
    let round_up = frac_part.as_bytes().get(digits).is_some_and(|&d| d >= b'5');
    if round_up {
        let mut i = digits_vec.len();
        loop {
            if i == 0 {
                digits_vec.insert(0, b'1');
                break;
            }
            i -= 1;
            if digits_vec[i] == b'9' {
                digits_vec[i] = b'0';
            } else {
                digits_vec[i] += 1;
                break;
            }
        }
    }
    let int_len = digits_vec.len() - digits;
    let mut out = String::new();
    if neg {
        out.push('-');
    }
    out.push_str(std::str::from_utf8(&digits_vec[..int_len]).unwrap());
    if digits > 0 {
        out.push('.');
        out.push_str(std::str::from_utf8(&digits_vec[int_len..]).unwrap());
    }
    out
}

/// `Number(x.toFixed(digits))`.
fn to_fixed_num(x: f64, digits: usize) -> f64 {
    to_fixed(x, digits).parse().unwrap()
}

/// JS `Number.prototype.toString()` / template-literal formatting of a number.
pub fn js_num(x: f64) -> String {
    if x.is_nan() {
        return s("NaN");
    }
    if x == 0.0 {
        return s("0");
    }
    if x.is_infinite() {
        return s(if x > 0.0 { "Infinity" } else { "-Infinity" });
    }
    let sign = if x < 0.0 { "-" } else { "" };
    // Shortest round-trip digits, e.g. "1.2345e3".
    let sci = format!("{:e}", x.abs());
    let (mantissa, exp) = sci.split_once('e').unwrap();
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let k = digits.len() as i64;
    let n = exp.parse::<i64>().unwrap() + 1;
    let body = if k <= n && n <= 21 {
        format!("{digits}{}", "0".repeat((n - k) as usize))
    } else if 0 < n && n <= 21 {
        format!("{}.{}", &digits[..n as usize], &digits[n as usize..])
    } else if -6 < n && n <= 0 {
        format!("0.{}{digits}", "0".repeat((-n) as usize))
    } else {
        let e = n - 1;
        let e_str = if e >= 0 {
            format!("+{e}")
        } else {
            e.to_string()
        };
        if k == 1 {
            format!("{digits}e{e_str}")
        } else {
            format!("{}.{}e{e_str}", &digits[..1], &digits[1..])
        }
    };
    format!("{sign}{body}")
}

/// Days since 1970-01-01 for a proleptic Gregorian date (month 1-based).
const fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// (year, month 1-based, day) for a days-since-epoch count.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// UTC calendar date of `new Date(EPOCH - daysAgo * DAY)`.
fn utc_date(days_ago: i64) -> (i64, i64, i64) {
    civil_from_days((BOARD_EPOCH - days_ago * DAY).div_euclid(DAY))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_matches_date_utc() {
        assert_eq!(BOARD_EPOCH, 1_787_702_400_000);
        assert_eq!(utc_date(0), (2026, 8, 26));
    }

    #[test]
    fn to_fixed_ties_away() {
        assert_eq!(to_fixed(0.25, 1), "0.3");
        assert_eq!(to_fixed(2.5, 0), "3");
        assert_eq!(to_fixed(1.005, 2), "1.00");
        assert_eq!(to_fixed(0.0, 2), "0.00");
        assert_eq!(to_fixed(9.96, 1), "10.0");
    }

    #[test]
    fn js_num_formats() {
        assert_eq!(js_num(2.0), "2");
        assert_eq!(js_num(1.5), "1.5");
        assert_eq!(js_num(0.1 + 0.2), "0.30000000000000004");
        assert_eq!(js_num(1e21), "1e+21");
        assert_eq!(js_num(1e-7), "1e-7");
        assert_eq!(js_num(0.000001), "0.000001");
    }
}
