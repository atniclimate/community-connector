use std::collections::BTreeSet;

use cn_model::EntityId;
use cn_perm::{ProjectedStory, Projection, ViewerContext};

use crate::dto::ExportOptions;
use crate::session::GroupSession;

pub(crate) fn export_projection(
    session: &GroupSession,
    viewer: &ViewerContext,
    options: &ExportOptions,
) -> Projection {
    let mut projection = cn_perm::export_projection(&session.state, viewer, session.revision);
    apply_kind_filter(&mut projection, options.kinds.as_deref());
    retain_referenced_content(&mut projection);
    projection
}

fn apply_kind_filter(projection: &mut Projection, kinds: Option<&[cn_model::KindId]>) {
    let Some(kinds) = kinds else {
        return;
    };
    let allowed: BTreeSet<_> = kinds.iter().collect();
    projection
        .entities
        .retain(|entity| allowed.contains(&entity.kind));
}

fn retain_referenced_content(projection: &mut Projection) {
    let ids: BTreeSet<_> = projection.entities.iter().map(|entity| entity.id).collect();
    projection
        .edges
        .retain(|edge| ids.contains(&edge.from) && ids.contains(&edge.to));
    projection.stories = projection
        .stories
        .drain(..)
        .filter_map(|story| retain_story_refs(story, &ids))
        .collect();
}

fn retain_story_refs(
    mut story: ProjectedStory,
    ids: &BTreeSet<EntityId>,
) -> Option<ProjectedStory> {
    story.steps.retain(|step| ids.contains(&step.entity));
    Some(story)
}
