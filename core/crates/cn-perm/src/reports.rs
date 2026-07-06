use std::collections::BTreeSet;

use cn_model::PersonId;
use cn_store::{GroupState, QuarantineEntry, QuarantineReason, StoreReport, SubjectRef};

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
            .filter(|finding| subject_index.subject_is_visible(finding.subject))
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
    if subjects.subject_is_visible(entry.subject) {
        return Some(entry.clone());
    }
    viewer
        .is_some_and(|person| person == entry.responsible_human)
        .then(|| redact_own_quarantine(entry))
}

fn redact_own_quarantine(entry: &QuarantineEntry) -> QuarantineEntry {
    QuarantineEntry {
        op_id: entry.op_id,
        responsible_human: entry.responsible_human,
        subject: entry.subject,
        reason: QuarantineReason::NotFound,
        findings: Vec::new(),
    }
}

struct SubjectIndex {
    entities: BTreeSet<cn_model::EntityId>,
    edges: BTreeSet<cn_model::EdgeId>,
    stories: BTreeSet<cn_model::StoryId>,
    group: Option<cn_model::GroupId>,
}

impl SubjectIndex {
    fn new(state: &GroupState, viewer: &ViewerContext) -> Self {
        let projection = project(state, viewer, 0);
        Self {
            entities: projection.entities.iter().map(|entity| entity.id).collect(),
            edges: projection.edges.iter().map(|edge| edge.id).collect(),
            stories: projection.stories.iter().map(|story| story.id).collect(),
            group: state.group.as_ref().map(|group| group.id),
        }
    }

    fn subject_is_visible(&self, subject: Option<SubjectRef>) -> bool {
        match subject {
            Some(SubjectRef::Entity(id)) => self.entities.contains(&id),
            Some(SubjectRef::Edge(id)) => self.edges.contains(&id),
            Some(SubjectRef::Story(id)) => self.stories.contains(&id),
            Some(SubjectRef::Group(id)) => self.group == Some(id),
            Some(SubjectRef::Membership(_) | SubjectRef::TrustGrant(_)) | None => false,
        }
    }
}
