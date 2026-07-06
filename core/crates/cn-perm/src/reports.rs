use cn_model::PersonId;
use cn_store::{GroupState, QuarantineEntry, QuarantineReason, StoreFinding, StoreReport};

use crate::projection::project;
use crate::viewer::{ViewerContext, is_governance};

/// Filters a store report for one viewer (ADR-003 A-B1 and round-2 no-counts
/// amendment; ADR-003 A-B4).
pub fn redact_report(
    state: &GroupState,
    viewer: &ViewerContext,
    report: &StoreReport,
) -> StoreReport {
    if governance_viewer(state, viewer) {
        return report.clone();
    }
    let subject_index = SubjectIndex::new(state, viewer);
    StoreReport {
        quarantined: report
            .quarantined
            .iter()
            .filter_map(|entry| redact_quarantine(viewer_person(viewer), entry, &subject_index))
            .collect(),
        warnings: report
            .warnings
            .iter()
            .filter(|finding| subject_index.finding_is_visible(finding))
            .cloned()
            .collect(),
        applied: 0,
        deduped: 0,
    }
}

fn governance_viewer(state: &GroupState, viewer: &ViewerContext) -> bool {
    viewer_person(viewer).is_some_and(|person| is_governance(state, person))
}

fn viewer_person(viewer: &ViewerContext) -> Option<PersonId> {
    match viewer {
        ViewerContext::Anonymous => None,
        ViewerContext::Person { person } => Some(*person),
    }
}

fn redact_quarantine(
    viewer: Option<PersonId>,
    entry: &QuarantineEntry,
    subjects: &SubjectIndex,
) -> Option<QuarantineEntry> {
    if viewer.is_some_and(|person| person == entry.responsible_human) {
        return Some(redact_own_quarantine(entry));
    }
    subjects.quarantine_is_visible(entry).then(|| entry.clone())
}

fn redact_own_quarantine(entry: &QuarantineEntry) -> QuarantineEntry {
    QuarantineEntry {
        op_id: entry.op_id,
        responsible_human: entry.responsible_human,
        reason: match entry.reason {
            QuarantineReason::MissingTarget => QuarantineReason::NotFound,
            reason => reason,
        },
        findings: Vec::new(),
    }
}

struct SubjectIndex {
    visible_needles: Vec<String>,
}

impl SubjectIndex {
    fn new(state: &GroupState, viewer: &ViewerContext) -> Self {
        let projection = project(state, viewer, 0);
        let mut visible_needles = Vec::new();
        for entity in projection.entities {
            visible_needles.push(entity.id.to_string());
            visible_needles.push(entity.kind.to_string());
            visible_needles.extend(entity.attributes.keys().map(ToString::to_string));
        }
        for edge in projection.edges {
            visible_needles.push(edge.id.to_string());
            visible_needles.push(edge.kind.to_string());
            visible_needles.extend(edge.attributes.keys().map(ToString::to_string));
        }
        for story in projection.stories {
            visible_needles.push(story.id.to_string());
        }
        Self { visible_needles }
    }

    fn finding_is_visible(&self, finding: &StoreFinding) -> bool {
        self.text_is_visible(&finding.code) || self.text_is_visible(&finding.message)
    }

    fn quarantine_is_visible(&self, entry: &QuarantineEntry) -> bool {
        entry.findings.iter().any(|finding| {
            self.text_is_visible(&finding.path) || self.text_is_visible(&finding.message)
        })
    }

    fn text_is_visible(&self, text: &str) -> bool {
        self.visible_needles
            .iter()
            .any(|needle| !needle.is_empty() && text.contains(needle))
    }
}
