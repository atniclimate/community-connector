use std::collections::BTreeSet;

use cn_perm::Projection;
use cn_store::{AuthzDenial, OpKind, Operation, SubjectRef, SubmitOutcome};

pub(crate) fn redact_outcomes(
    outcomes: Vec<SubmitOutcome>,
    ops: &[Operation],
    projection: &Projection,
) -> Vec<SubmitOutcome> {
    outcomes
        .into_iter()
        .map(|outcome| redact_outcome(outcome, ops, projection))
        .collect()
}

fn redact_outcome(
    outcome: SubmitOutcome,
    ops: &[Operation],
    projection: &Projection,
) -> SubmitOutcome {
    match outcome {
        SubmitOutcome::Denied { op, denial } => {
            if target_visible(ops, op, projection) {
                SubmitOutcome::Denied { op, denial }
            } else {
                not_found_outcome(op)
            }
        }
        SubmitOutcome::Quarantined(op) => {
            if target_visible(ops, op, projection) {
                SubmitOutcome::Quarantined(op)
            } else {
                not_found_outcome(op)
            }
        }
        SubmitOutcome::Applied(op) => SubmitOutcome::Applied(op),
    }
}

fn target_visible(ops: &[Operation], op_id: cn_model::OpId, projection: &Projection) -> bool {
    match ops
        .iter()
        .find(|op| op.op_id == op_id)
        .and_then(target_subject)
    {
        Some(subject) => projected_subjects(projection).contains(&subject),
        None => true,
    }
}

fn target_subject(op: &Operation) -> Option<SubjectRef> {
    match &op.kind {
        OpKind::EntityLifecycleSet { entity, .. }
        | OpKind::AttributeSet { entity, .. }
        | OpKind::AttributeRemove { entity, .. } => Some(SubjectRef::Entity(*entity)),
        OpKind::VisibilitySet { target, .. } => match target {
            cn_store::VisibilityTarget::EntityPresence(id)
            | cn_store::VisibilityTarget::Attribute(id, _) => Some(SubjectRef::Entity(*id)),
            cn_store::VisibilityTarget::Edge(id) => Some(SubjectRef::Edge(*id)),
            cn_store::VisibilityTarget::Story(id) => Some(SubjectRef::Story(*id)),
        },
        OpKind::TierSet { target, .. } => match target {
            cn_store::TierTarget::Entity(id) | cn_store::TierTarget::Attribute(id, _) => {
                Some(SubjectRef::Entity(*id))
            }
            cn_store::TierTarget::Edge(id) => Some(SubjectRef::Edge(*id)),
            cn_store::TierTarget::Story(id) => Some(SubjectRef::Story(*id)),
        },
        OpKind::EdgeWeightSet { edge, .. } | OpKind::EdgeLifecycleSet { edge, .. } => {
            Some(SubjectRef::Edge(*edge))
        }
        OpKind::MembershipLifecycleSet { membership, .. } => {
            Some(SubjectRef::Membership(*membership))
        }
        OpKind::TrustGrantRevoke { grant, .. } => Some(SubjectRef::TrustGrant(*grant)),
        OpKind::CustodyAppend { target, .. } => match target {
            cn_store::CustodyTarget::Entity(id) | cn_store::CustodyTarget::Attribute(id, _) => {
                Some(SubjectRef::Entity(*id))
            }
            cn_store::CustodyTarget::Edge(id) => Some(SubjectRef::Edge(*id)),
            cn_store::CustodyTarget::Story(id) => Some(SubjectRef::Story(*id)),
            cn_store::CustodyTarget::Group => Some(SubjectRef::Group(op.group_id)),
        },
        OpKind::StoryUpdate { story, .. } | OpKind::StoryLifecycleSet { story, .. } => {
            Some(SubjectRef::Story(*story))
        }
        OpKind::GroupCreate { .. }
        | OpKind::EntityCreate { .. }
        | OpKind::EdgeCreate { .. }
        | OpKind::MembershipAdd { .. }
        | OpKind::TrustGrantCreate { .. }
        | OpKind::StoryCreate { .. } => None,
    }
}

fn projected_subjects(projection: &Projection) -> BTreeSet<SubjectRef> {
    let mut subjects = BTreeSet::new();
    subjects.insert(SubjectRef::Group(projection.group_id));
    subjects.extend(
        projection
            .entities
            .iter()
            .map(|entity| SubjectRef::Entity(entity.id)),
    );
    subjects.extend(
        projection
            .edges
            .iter()
            .map(|edge| SubjectRef::Edge(edge.id)),
    );
    subjects.extend(
        projection
            .stories
            .iter()
            .map(|story| SubjectRef::Story(story.id)),
    );
    subjects
}

fn not_found_outcome(op: cn_model::OpId) -> SubmitOutcome {
    SubmitOutcome::Denied {
        op,
        denial: AuthzDenial {
            code: "not_found".to_string(),
            message: "not found".to_string(),
        },
    }
}
