//! ADR-002 fold implementation.
//!
//! I5 note: this module is intentionally above 500 lines because the fold
//! rules, field clocks, quarantine fixpoint, and target mutation paths are
//! reviewed as one deterministic state transition unit.

use std::collections::{BTreeMap, BTreeSet};

use cn_model::*;
use serde::{Deserialize, Serialize};

use crate::op::{CustodyTarget, OpKind, Operation, SortKey, TierTarget, VisibilityTarget};

/// In-memory state reconstructed from the operation log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GroupState {
    pub group: Option<Group>,
    pub template: Option<cn_schema::GroupTemplate>,
    pub entities: BTreeMap<EntityId, Entity>,
    pub edges: BTreeMap<EdgeId, Edge>,
    pub memberships: BTreeMap<MembershipId, Membership>,
    pub trust_grants: BTreeMap<TrustGrantId, TrustGrant>,
    pub stories: BTreeMap<StoryId, Story>,
    field_clocks: BTreeMap<(ObjectKey, FieldKey), SortKey>,
    seen: BTreeSet<OpId>,
    quarantine: Vec<Operation>,
}

/// Field-level LWW granularity from ADR-002 round 2.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FieldKey {
    Lifecycle,
    PresenceVisibility,
    Visibility,
    Tier,
    Owner,
    Weight,
    Attribute(AttrId),
    AttributeVisibility(AttrId),
    AttributeTier(AttrId),
    StoryTitle,
    StorySteps,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
enum ObjectKey {
    Group(GroupId),
    Entity(EntityId),
    Edge(EdgeId),
    Membership(MembershipId),
    TrustGrant(TrustGrantId),
    Story(StoryId),
}

/// Machine-readable fold report.
#[derive(Debug, Clone, PartialEq, Serialize, Default)]
pub struct StoreReport {
    pub quarantined: Vec<QuarantineEntry>,
    pub warnings: Vec<StoreFinding>,
    pub applied: usize,
    pub deduped: usize,
}

/// Quarantined operation with validation findings when relevant.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuarantineEntry {
    pub op_id: OpId,
    pub responsible_human: PersonId,
    pub reason: QuarantineReason,
    pub findings: Vec<cn_schema::Finding>,
}

/// Quarantine reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum QuarantineReason {
    NotFound,
    MissingTarget,
    DuplicateCreate,
    TemplateVersionUnknown,
    FailedValidation,
    RevokedBeforeGranted,
    WrongGroup,
}

/// Visible non-quarantining store finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoreFinding {
    pub code: String,
    pub message: String,
}

enum ApplyResult {
    Applied,
    Deduped,
    Quarantined(QuarantineReason, Vec<cn_schema::Finding>),
}

/// Folds operations in ADR-002 D5 canonical order.
pub fn fold(ops: impl IntoIterator<Item = Operation>) -> (GroupState, StoreReport) {
    let mut sorted: Vec<_> = ops.into_iter().collect();
    sorted.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
    let mut state = GroupState::default();
    let mut report = StoreReport::default();
    for op in sorted {
        state.apply(op, &mut report);
    }
    state.flush_quarantine(&mut report);
    (state, report)
}

impl GroupState {
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn snapshot_watermark(&self) -> Option<(OpId, SortKey)> {
        self.field_clocks.values().max().map(|sort_key| {
            let sort_key = sort_key.clone();
            (sort_key.op_id, sort_key)
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn from_snapshot_parts(
        group: Option<Group>,
        template: Option<cn_schema::GroupTemplate>,
        entities: BTreeMap<EntityId, Entity>,
        edges: BTreeMap<EdgeId, Edge>,
        memberships: BTreeMap<MembershipId, Membership>,
        trust_grants: BTreeMap<TrustGrantId, TrustGrant>,
        stories: BTreeMap<StoryId, Story>,
    ) -> Self {
        Self {
            group,
            template,
            entities,
            edges,
            memberships,
            trust_grants,
            stories,
            field_clocks: BTreeMap::new(),
            seen: BTreeSet::new(),
            quarantine: Vec::new(),
        }
    }

    /// Applies one operation, then re-examines missing-target quarantine to fixpoint.
    pub fn apply(&mut self, op: Operation, report: &mut StoreReport) {
        match self.try_apply(op.clone()) {
            ApplyResult::Applied => {
                report.applied += 1;
                self.admit_fixpoint(report);
            }
            ApplyResult::Deduped => report.deduped += 1,
            ApplyResult::Quarantined(QuarantineReason::MissingTarget, _) => {
                self.quarantine.push(op);
            }
            ApplyResult::Quarantined(reason, findings) => {
                report.quarantined.push(QuarantineEntry {
                    op_id: op.op_id,
                    responsible_human: op.responsible_human,
                    reason,
                    findings,
                });
            }
        }
    }

    fn admit_fixpoint(&mut self, report: &mut StoreReport) {
        loop {
            self.quarantine.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
            let pending = std::mem::take(&mut self.quarantine);
            let mut admitted = 0;
            for op in pending {
                match self.try_apply(op.clone()) {
                    ApplyResult::Applied => {
                        admitted += 1;
                        report.applied += 1;
                    }
                    ApplyResult::Deduped => report.deduped += 1,
                    ApplyResult::Quarantined(QuarantineReason::MissingTarget, _) => {
                        self.quarantine.push(op);
                    }
                    ApplyResult::Quarantined(reason, findings) => {
                        report.quarantined.push(QuarantineEntry {
                            op_id: op.op_id,
                            responsible_human: op.responsible_human,
                            reason,
                            findings,
                        });
                    }
                }
            }
            if admitted == 0 {
                break;
            }
        }
    }

    fn flush_quarantine(&mut self, report: &mut StoreReport) {
        self.quarantine.sort_by(|a, b| a.sort_key.cmp(&b.sort_key));
        for op in std::mem::take(&mut self.quarantine) {
            report.quarantined.push(QuarantineEntry {
                op_id: op.op_id,
                responsible_human: op.responsible_human,
                reason: QuarantineReason::MissingTarget,
                findings: Vec::new(),
            });
        }
    }

    /// ADR-002 D4 deduplicates by operation id before any mutation.
    fn try_apply(&mut self, op: Operation) -> ApplyResult {
        if self.seen.contains(&op.op_id) {
            return ApplyResult::Deduped;
        }
        let result = self.apply_new(&op);
        if matches!(result, ApplyResult::Applied) {
            self.seen.insert(op.op_id);
        }
        result
    }

    fn apply_new(&mut self, op: &Operation) -> ApplyResult {
        if self.group_mismatch(op) {
            return ApplyResult::Quarantined(QuarantineReason::WrongGroup, Vec::new());
        }
        match &op.kind {
            OpKind::GroupCreate {
                group,
                template_json,
            } => self.apply_group_create(op, group.clone(), template_json),
            _ if self.group.is_none() => {
                ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new())
            }
            _ if !self.template_known(op) => {
                ApplyResult::Quarantined(QuarantineReason::TemplateVersionUnknown, Vec::new())
            }
            OpKind::EntityCreate { entity } => self.apply_entity_create(op, entity.clone()),
            OpKind::EntityLifecycleSet { entity, lifecycle } => {
                self.set_entity_lifecycle(op, *entity, *lifecycle)
            }
            OpKind::AttributeSet {
                entity,
                attr,
                instance,
            } => self.set_attribute(op, *entity, attr.clone(), instance.clone()),
            OpKind::AttributeRemove { entity, attr } => self.remove_attribute(op, *entity, attr),
            OpKind::VisibilitySet { target, visibility } => {
                self.set_visibility(op, target, *visibility)
            }
            OpKind::TierSet { target, tier } => self.set_tier(op, target, *tier),
            OpKind::EdgeCreate { edge } => self.apply_edge_create(op, edge.clone()),
            OpKind::EdgeWeightSet { edge, weight } => self.set_edge_weight(op, *edge, *weight),
            OpKind::EdgeLifecycleSet { edge, lifecycle } => {
                self.set_edge_lifecycle(op, *edge, *lifecycle)
            }
            OpKind::MembershipAdd { membership } => {
                self.apply_membership_add(op, membership.clone())
            }
            OpKind::MembershipLifecycleSet {
                membership,
                lifecycle,
            } => self.set_membership_lifecycle(op, *membership, *lifecycle),
            OpKind::TrustGrantCreate { grant } => self.apply_trust_grant(op, grant.clone()),
            OpKind::TrustGrantRevoke { grant, at } => self.revoke_trust_grant(op, *grant, *at),
            OpKind::CustodyAppend { target, event } => self.append_custody(target, event.clone()),
            OpKind::StoryCreate { story } => self.apply_story_create(op, story.clone()),
            OpKind::StoryUpdate {
                story,
                title,
                steps,
            } => self.update_story(op, *story, title.clone(), steps.clone()),
            OpKind::StoryLifecycleSet { story, lifecycle } => {
                self.set_story_lifecycle(op, *story, *lifecycle)
            }
        }
    }

    fn group_mismatch(&self, op: &Operation) -> bool {
        self.group
            .as_ref()
            .is_some_and(|group| group.id != op.group_id)
    }

    fn template_known(&self, op: &Operation) -> bool {
        self.group
            .as_ref()
            .is_some_and(|group| group.template_version == op.template_version)
    }

    fn apply_group_create(&mut self, op: &Operation, group: Group, json: &str) -> ApplyResult {
        if self.group.is_some() {
            return ApplyResult::Quarantined(QuarantineReason::DuplicateCreate, Vec::new());
        }
        if group.id != op.group_id || group.template_version != op.template_version {
            return ApplyResult::Quarantined(QuarantineReason::TemplateVersionUnknown, Vec::new());
        }
        let Ok((template, validation)) = cn_schema::parse_template(json) else {
            return ApplyResult::Quarantined(QuarantineReason::FailedValidation, Vec::new());
        };
        if !validation.errors.is_empty() {
            return ApplyResult::Quarantined(QuarantineReason::FailedValidation, validation.errors);
        }
        self.group = Some(group.clone());
        self.template = Some(template);
        self.record(ObjectKey::Group(group.id), FieldKey::Tier, &op.sort_key);
        ApplyResult::Applied
    }

    fn apply_entity_create(&mut self, op: &Operation, entity: Entity) -> ApplyResult {
        if self.entities.contains_key(&entity.id) {
            return ApplyResult::Quarantined(QuarantineReason::DuplicateCreate, Vec::new());
        }
        if !self.valid_group(entity.group_id) {
            return ApplyResult::Quarantined(QuarantineReason::WrongGroup, Vec::new());
        }
        if let Some(findings) = self.entity_errors(&entity) {
            return ApplyResult::Quarantined(QuarantineReason::FailedValidation, findings);
        }
        let key = ObjectKey::Entity(entity.id);
        self.record_entity_create(&key, &entity, &op.sort_key);
        self.entities.insert(entity.id, entity);
        ApplyResult::Applied
    }

    fn apply_edge_create(&mut self, op: &Operation, edge: Edge) -> ApplyResult {
        if self.edges.contains_key(&edge.id) {
            return ApplyResult::Quarantined(QuarantineReason::DuplicateCreate, Vec::new());
        }
        if !self.entities.contains_key(&edge.from) || !self.entities.contains_key(&edge.to) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if let Some(findings) = self.edge_errors(&edge) {
            return ApplyResult::Quarantined(QuarantineReason::FailedValidation, findings);
        }
        let key = ObjectKey::Edge(edge.id);
        self.record_edge_create(&key, &edge, &op.sort_key);
        self.edges.insert(edge.id, edge);
        ApplyResult::Applied
    }

    fn apply_membership_add(&mut self, op: &Operation, membership: Membership) -> ApplyResult {
        if self.memberships.contains_key(&membership.id) {
            return ApplyResult::Quarantined(QuarantineReason::DuplicateCreate, Vec::new());
        }
        if !self.valid_group(membership.group_id) {
            return ApplyResult::Quarantined(QuarantineReason::WrongGroup, Vec::new());
        }
        self.record(
            ObjectKey::Membership(membership.id),
            FieldKey::Lifecycle,
            &op.sort_key,
        );
        self.memberships.insert(membership.id, membership);
        ApplyResult::Applied
    }

    fn apply_trust_grant(&mut self, op: &Operation, grant: TrustGrant) -> ApplyResult {
        if self.trust_grants.contains_key(&grant.id) {
            return ApplyResult::Quarantined(QuarantineReason::DuplicateCreate, Vec::new());
        }
        self.record(
            ObjectKey::TrustGrant(grant.id),
            FieldKey::Lifecycle,
            &op.sort_key,
        );
        self.trust_grants.insert(grant.id, grant);
        ApplyResult::Applied
    }

    fn apply_story_create(&mut self, op: &Operation, story: Story) -> ApplyResult {
        if self.stories.contains_key(&story.id) {
            return ApplyResult::Quarantined(QuarantineReason::DuplicateCreate, Vec::new());
        }
        if !self.valid_group(story.group_id) {
            return ApplyResult::Quarantined(QuarantineReason::WrongGroup, Vec::new());
        }
        if !story
            .steps
            .iter()
            .all(|step| self.entities.contains_key(&step.entity))
        {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        let key = ObjectKey::Story(story.id);
        self.record(key.clone(), FieldKey::StoryTitle, &op.sort_key);
        self.record(key.clone(), FieldKey::StorySteps, &op.sort_key);
        self.record(key.clone(), FieldKey::Visibility, &op.sort_key);
        self.record(key.clone(), FieldKey::Lifecycle, &op.sort_key);
        self.record(key, FieldKey::Tier, &op.sort_key);
        self.stories.insert(story.id, story);
        ApplyResult::Applied
    }

    /// ADR-002 round-2 records a field clock for lifecycle writes.
    fn set_entity_lifecycle(
        &mut self,
        op: &Operation,
        entity: EntityId,
        lifecycle: Lifecycle,
    ) -> ApplyResult {
        if !self.entities.contains_key(&entity) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if self.lww(ObjectKey::Entity(entity), FieldKey::Lifecycle, &op.sort_key)
            && let Some(record) = self.entities.get_mut(&entity)
        {
            record.lifecycle = lifecycle;
        }
        ApplyResult::Applied
    }

    /// ADR-002 round-2 stores attribute value and provenance as one LWW field.
    fn set_attribute(
        &mut self,
        op: &Operation,
        entity: EntityId,
        attr: AttrId,
        instance: AttributeInstance,
    ) -> ApplyResult {
        let Some(record) = self.entities.get(&entity) else {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        };
        let mut candidate = record.clone();
        candidate.attributes.insert(attr.clone(), instance.clone());
        if let Some(findings) = self.entity_errors(&candidate) {
            return ApplyResult::Quarantined(QuarantineReason::FailedValidation, findings);
        }
        if self.lww(
            ObjectKey::Entity(entity),
            FieldKey::Attribute(attr.clone()),
            &op.sort_key,
        ) && let Some(record) = self.entities.get_mut(&entity)
        {
            record.attributes.insert(attr, instance);
        }
        ApplyResult::Applied
    }

    /// ADR-002 round-2 treats attribute removal as a field write.
    fn remove_attribute(&mut self, op: &Operation, entity: EntityId, attr: &AttrId) -> ApplyResult {
        let Some(record) = self.entities.get(&entity) else {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        };
        let mut candidate = record.clone();
        candidate.attributes.remove(attr);
        if let Some(findings) = self.entity_errors(&candidate) {
            return ApplyResult::Quarantined(QuarantineReason::FailedValidation, findings);
        }
        if self.lww(
            ObjectKey::Entity(entity),
            FieldKey::Attribute(attr.clone()),
            &op.sort_key,
        ) && let Some(record) = self.entities.get_mut(&entity)
        {
            record.attributes.remove(attr);
        }
        ApplyResult::Applied
    }

    /// ADR-002 round-2 records field clocks for visibility writes.
    fn set_visibility(
        &mut self,
        op: &Operation,
        target: &VisibilityTarget,
        visibility: Circle,
    ) -> ApplyResult {
        match target {
            VisibilityTarget::EntityPresence(id) => self.set_entity_presence(op, *id, visibility),
            VisibilityTarget::Attribute(id, attr) => {
                self.set_attr_visibility(op, *id, attr, visibility)
            }
            VisibilityTarget::Edge(id) => self.set_edge_visibility(op, *id, visibility),
            VisibilityTarget::Story(id) => self.set_story_visibility(op, *id, visibility),
        }
    }

    /// ADR-002 round-2 records field clocks for tier writes.
    fn set_tier(
        &mut self,
        op: &Operation,
        target: &TierTarget,
        tier: SensitivityTier,
    ) -> ApplyResult {
        match target {
            TierTarget::Entity(id) => self.set_entity_tier(op, *id, tier),
            TierTarget::Attribute(id, attr) => self.set_attr_tier(op, *id, attr, tier),
            TierTarget::Edge(id) => self.set_edge_tier(op, *id, tier),
            TierTarget::Story(id) => self.set_story_tier(op, *id, tier),
        }
    }

    /// ADR-002 round-2 records edge weight as an LWW field.
    fn set_edge_weight(
        &mut self,
        op: &Operation,
        edge: EdgeId,
        weight: Option<f64>,
    ) -> ApplyResult {
        let Some(record) = self.edges.get(&edge) else {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        };
        let mut candidate = record.clone();
        if candidate.set_weight(weight).is_err() {
            return ApplyResult::Quarantined(QuarantineReason::FailedValidation, Vec::new());
        }
        if let Some(findings) = self.edge_errors(&candidate) {
            return ApplyResult::Quarantined(QuarantineReason::FailedValidation, findings);
        }
        if self.lww(ObjectKey::Edge(edge), FieldKey::Weight, &op.sort_key)
            && let Some(record) = self.edges.get_mut(&edge)
        {
            record.weight = weight;
        }
        ApplyResult::Applied
    }

    /// ADR-002 round-2 records edge lifecycle as an LWW field.
    fn set_edge_lifecycle(
        &mut self,
        op: &Operation,
        edge: EdgeId,
        lifecycle: Lifecycle,
    ) -> ApplyResult {
        if !self.edges.contains_key(&edge) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if self.lww(ObjectKey::Edge(edge), FieldKey::Lifecycle, &op.sort_key)
            && let Some(record) = self.edges.get_mut(&edge)
        {
            record.lifecycle = lifecycle;
        }
        ApplyResult::Applied
    }

    fn set_membership_lifecycle(
        &mut self,
        op: &Operation,
        membership: MembershipId,
        lifecycle: Lifecycle,
    ) -> ApplyResult {
        if !self.memberships.contains_key(&membership) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if self.lww(
            ObjectKey::Membership(membership),
            FieldKey::Lifecycle,
            &op.sort_key,
        ) && let Some(record) = self.memberships.get_mut(&membership)
        {
            record.lifecycle = lifecycle;
        }
        ApplyResult::Applied
    }

    fn revoke_trust_grant(
        &mut self,
        op: &Operation,
        grant: TrustGrantId,
        at: Timestamp,
    ) -> ApplyResult {
        let Some(existing) = self.trust_grants.get(&grant) else {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        };
        if existing.revoked_at.is_some() {
            return ApplyResult::Applied;
        }
        let mut candidate = existing.clone();
        match candidate.revoke(at) {
            Ok(())
                if self.lww(
                    ObjectKey::TrustGrant(grant),
                    FieldKey::Lifecycle,
                    &op.sort_key,
                ) =>
            {
                self.trust_grants.insert(grant, candidate);
                ApplyResult::Applied
            }
            Ok(()) => ApplyResult::Applied,
            Err(ModelError::AlreadyRevoked) => ApplyResult::Applied,
            Err(ModelError::RevokedBeforeGranted) => {
                ApplyResult::Quarantined(QuarantineReason::RevokedBeforeGranted, Vec::new())
            }
            Err(_) => ApplyResult::Quarantined(QuarantineReason::FailedValidation, Vec::new()),
        }
    }

    /// ADR-002 A-B4 allows custody appends on archived targets and misses only unknown targets.
    fn append_custody(&mut self, target: &CustodyTarget, event: CustodyEvent) -> ApplyResult {
        match target {
            CustodyTarget::Entity(id) => append_to(self.entities.get_mut(id), event),
            CustodyTarget::Attribute(id, attr) => {
                let Some(entity) = self.entities.get_mut(id) else {
                    return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
                };
                append_to(entity.attributes.get_mut(attr), event)
            }
            CustodyTarget::Edge(id) => append_to(self.edges.get_mut(id), event),
            CustodyTarget::Story(id) => append_to(self.stories.get_mut(id), event),
            CustodyTarget::Group => append_to(self.group.as_mut(), event),
        }
    }

    fn update_story(
        &mut self,
        op: &Operation,
        story: StoryId,
        title: Option<String>,
        steps: Option<Vec<StoryStep>>,
    ) -> ApplyResult {
        let Some(existing) = self.stories.get(&story) else {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        };
        if let Some(steps) = &steps
            && !steps
                .iter()
                .all(|step| self.entities.contains_key(&step.entity))
        {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        let mut candidate = existing.clone();
        if let Some(title) = title {
            candidate.title = title;
            if self.lww(ObjectKey::Story(story), FieldKey::StoryTitle, &op.sort_key)
                && let Some(record) = self.stories.get_mut(&story)
            {
                record.title = candidate.title.clone();
            }
        }
        if let Some(steps) = steps
            && self.lww(ObjectKey::Story(story), FieldKey::StorySteps, &op.sort_key)
            && let Some(record) = self.stories.get_mut(&story)
        {
            record.steps = steps;
        }
        ApplyResult::Applied
    }

    fn set_story_lifecycle(
        &mut self,
        op: &Operation,
        story: StoryId,
        lifecycle: Lifecycle,
    ) -> ApplyResult {
        if !self.stories.contains_key(&story) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if self.lww(ObjectKey::Story(story), FieldKey::Lifecycle, &op.sort_key)
            && let Some(record) = self.stories.get_mut(&story)
        {
            record.lifecycle = lifecycle;
        }
        ApplyResult::Applied
    }

    fn set_entity_presence(
        &mut self,
        op: &Operation,
        id: EntityId,
        visibility: Circle,
    ) -> ApplyResult {
        if !self.entities.contains_key(&id) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if self.lww(
            ObjectKey::Entity(id),
            FieldKey::PresenceVisibility,
            &op.sort_key,
        ) && let Some(record) = self.entities.get_mut(&id)
        {
            record.presence_visibility = visibility;
        }
        ApplyResult::Applied
    }

    fn set_attr_visibility(
        &mut self,
        op: &Operation,
        id: EntityId,
        attr: &AttrId,
        visibility: Circle,
    ) -> ApplyResult {
        if !self
            .entities
            .get(&id)
            .is_some_and(|entity| entity.attributes.contains_key(attr))
        {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if self.lww(
            ObjectKey::Entity(id),
            FieldKey::AttributeVisibility(attr.clone()),
            &op.sort_key,
        ) && let Some(instance) = self
            .entities
            .get_mut(&id)
            .and_then(|entity| entity.attributes.get_mut(attr))
        {
            instance.visibility = visibility;
        }
        ApplyResult::Applied
    }

    fn set_edge_visibility(
        &mut self,
        op: &Operation,
        id: EdgeId,
        visibility: Circle,
    ) -> ApplyResult {
        if !self.edges.contains_key(&id) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if self.lww(ObjectKey::Edge(id), FieldKey::Visibility, &op.sort_key)
            && let Some(record) = self.edges.get_mut(&id)
        {
            record.visibility = visibility;
        }
        ApplyResult::Applied
    }

    fn set_story_visibility(
        &mut self,
        op: &Operation,
        id: StoryId,
        visibility: Circle,
    ) -> ApplyResult {
        if !self.stories.contains_key(&id) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if self.lww(ObjectKey::Story(id), FieldKey::Visibility, &op.sort_key)
            && let Some(record) = self.stories.get_mut(&id)
        {
            record.visibility = visibility;
        }
        ApplyResult::Applied
    }

    fn set_entity_tier(
        &mut self,
        op: &Operation,
        id: EntityId,
        tier: SensitivityTier,
    ) -> ApplyResult {
        if !self.entities.contains_key(&id) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if self.lww(ObjectKey::Entity(id), FieldKey::Tier, &op.sort_key)
            && let Some(record) = self.entities.get_mut(&id)
        {
            record.tier = tier;
        }
        ApplyResult::Applied
    }

    fn set_attr_tier(
        &mut self,
        op: &Operation,
        id: EntityId,
        attr: &AttrId,
        tier: SensitivityTier,
    ) -> ApplyResult {
        let Some(entity) = self.entities.get(&id) else {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        };
        let entity_tier = entity.tier;
        if !entity.attributes.contains_key(attr) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        };
        if self.lww(
            ObjectKey::Entity(id),
            FieldKey::AttributeTier(attr.clone()),
            &op.sort_key,
        ) {
            let Some(instance) = self
                .entities
                .get_mut(&id)
                .and_then(|entity| entity.attributes.get_mut(attr))
            else {
                return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
            };
            if instance.tighten_tier(entity_tier, tier).is_err() {
                return ApplyResult::Quarantined(QuarantineReason::FailedValidation, Vec::new());
            }
        }
        ApplyResult::Applied
    }

    fn set_edge_tier(&mut self, op: &Operation, id: EdgeId, tier: SensitivityTier) -> ApplyResult {
        if !self.edges.contains_key(&id) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if self.lww(ObjectKey::Edge(id), FieldKey::Tier, &op.sort_key)
            && let Some(record) = self.edges.get_mut(&id)
        {
            record.tier = tier;
        }
        ApplyResult::Applied
    }

    fn set_story_tier(
        &mut self,
        op: &Operation,
        id: StoryId,
        tier: SensitivityTier,
    ) -> ApplyResult {
        if !self.stories.contains_key(&id) {
            return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
        }
        if self.lww(ObjectKey::Story(id), FieldKey::Tier, &op.sort_key)
            && let Some(record) = self.stories.get_mut(&id)
        {
            record.tier = tier;
        }
        ApplyResult::Applied
    }

    fn valid_group(&self, id: GroupId) -> bool {
        self.group.as_ref().is_some_and(|group| group.id == id)
    }

    fn entity_errors(&self, entity: &Entity) -> Option<Vec<cn_schema::Finding>> {
        let template = self.template.as_ref()?;
        let report = cn_schema::validate_entity(template, entity);
        (!report.errors.is_empty()).then_some(report.errors)
    }

    fn edge_errors(&self, edge: &Edge) -> Option<Vec<cn_schema::Finding>> {
        let template = self.template.as_ref()?;
        let from_kind = &self.entities.get(&edge.from)?.kind;
        let to_kind = &self.entities.get(&edge.to)?.kind;
        let report = cn_schema::validate_edge(template, edge, from_kind, to_kind);
        (!report.errors.is_empty()).then_some(report.errors)
    }

    fn record_entity_create(&mut self, key: &ObjectKey, entity: &Entity, sort_key: &SortKey) {
        self.record(key.clone(), FieldKey::Lifecycle, sort_key);
        self.record(key.clone(), FieldKey::PresenceVisibility, sort_key);
        self.record(key.clone(), FieldKey::Tier, sort_key);
        self.record(key.clone(), FieldKey::Owner, sort_key);
        for attr in entity.attributes.keys() {
            self.record(key.clone(), FieldKey::Attribute(attr.clone()), sort_key);
            self.record(
                key.clone(),
                FieldKey::AttributeVisibility(attr.clone()),
                sort_key,
            );
            self.record(key.clone(), FieldKey::AttributeTier(attr.clone()), sort_key);
        }
    }

    fn record_edge_create(&mut self, key: &ObjectKey, edge: &Edge, sort_key: &SortKey) {
        self.record(key.clone(), FieldKey::Lifecycle, sort_key);
        self.record(key.clone(), FieldKey::Visibility, sort_key);
        self.record(key.clone(), FieldKey::Tier, sort_key);
        self.record(key.clone(), FieldKey::Weight, sort_key);
        for attr in edge.attributes.keys() {
            self.record(key.clone(), FieldKey::Attribute(attr.clone()), sort_key);
            self.record(
                key.clone(),
                FieldKey::AttributeVisibility(attr.clone()),
                sort_key,
            );
            self.record(key.clone(), FieldKey::AttributeTier(attr.clone()), sort_key);
        }
    }

    fn lww(&mut self, object: ObjectKey, field: FieldKey, sort_key: &SortKey) -> bool {
        match self.field_clocks.get(&(object.clone(), field.clone())) {
            Some(recorded) if sort_key <= recorded => false,
            Some(_) | None => {
                self.record(object, field, sort_key);
                true
            }
        }
    }

    fn record(&mut self, object: ObjectKey, field: FieldKey, sort_key: &SortKey) {
        self.field_clocks.insert((object, field), sort_key.clone());
    }
}

trait HasProvenance {
    fn provenance_mut(&mut self) -> &mut ProvenanceEnvelope;
}

impl HasProvenance for Group {
    fn provenance_mut(&mut self) -> &mut ProvenanceEnvelope {
        &mut self.provenance
    }
}

impl HasProvenance for Entity {
    fn provenance_mut(&mut self) -> &mut ProvenanceEnvelope {
        &mut self.provenance
    }
}

impl HasProvenance for Edge {
    fn provenance_mut(&mut self) -> &mut ProvenanceEnvelope {
        &mut self.provenance
    }
}

impl HasProvenance for Story {
    fn provenance_mut(&mut self) -> &mut ProvenanceEnvelope {
        &mut self.provenance
    }
}

impl HasProvenance for AttributeInstance {
    fn provenance_mut(&mut self) -> &mut ProvenanceEnvelope {
        &mut self.provenance
    }
}

fn append_to<T: HasProvenance>(target: Option<&mut T>, event: CustodyEvent) -> ApplyResult {
    let Some(target) = target else {
        return ApplyResult::Quarantined(QuarantineReason::MissingTarget, Vec::new());
    };
    if !target
        .provenance_mut()
        .custody()
        .iter()
        .any(|existing| existing.id == event.id)
    {
        target.provenance_mut().append_custody(event);
    }
    ApplyResult::Applied
}
