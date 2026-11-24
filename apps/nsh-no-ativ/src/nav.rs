//! The page stack: what can be on it, and what interrupts it.

use nsh_metro::Icon;

use crate::data::{IncidentArea, IncidentSeverity};

/// One entry of the back stack. Per-page view state (the selected pivot
/// item, the panorama section) lives on the entry, so going back restores it.
#[derive(Clone, Debug, PartialEq)]
pub enum Page {
    Start,
    Runs { pivot: usize },
    Run { id: String },
    Telemetry { zone: usize },
    Strains,
    Strain { id: String },
    Incidents { pivot: usize },
    Incident { id: String },
    /// The new-incident form, optionally pre-linked to a run.
    NewIncident { run: Option<String> },
    Reagents { pivot: usize },
    Shipments { pivot: usize },
    Freezer,
    Deck { pivot: usize },
    People { pivot: usize },
    Settings,
}

impl Page {
    /// The selected pivot item or panorama section, for pages that have one.
    pub fn selected_mut(&mut self) -> Option<&mut usize> {
        match self {
            Page::Runs { pivot }
            | Page::Incidents { pivot }
            | Page::Reagents { pivot }
            | Page::Shipments { pivot }
            | Page::Deck { pivot }
            | Page::People { pivot } => Some(pivot),
            Page::Telemetry { zone } => Some(zone),
            _ => None,
        }
    }
}

/// What the page host shows: the page, and a serial that bumps on every
/// push and pop, so even the same page navigated to again turns in afresh.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub serial: usize,
    pub page: Page,
}

pub const AREAS: [IncidentArea; 6] = [
    IncidentArea::Bioreactor,
    IncidentArea::Freezer,
    IncidentArea::Media,
    IncidentArea::Sequencing,
    IncidentArea::Safety,
    IncidentArea::Facility,
];
pub const SEVERITIES: [IncidentSeverity; 3] =
    [IncidentSeverity::Minor, IncidentSeverity::Major, IncidentSeverity::Critical];

/// A message box over the page.
#[derive(Clone, Debug, PartialEq)]
pub enum Dialog {
    Harvest { run_id: String },
    Notice { title: String, message: String },
}

/// The toast in the top-right corner.
#[derive(Clone, Debug, PartialEq)]
pub struct ActiveToast {
    /// What the toast is about. A new toast of the same kind updates this
    /// one in place (it doesn't swivel in again).
    pub kind: &'static str,
    pub title: String,
    pub message: String,
    pub icon: Icon,
    pub serial: usize,
}
