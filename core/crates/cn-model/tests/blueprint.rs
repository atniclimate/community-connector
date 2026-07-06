use std::str::FromStr;

use cn_model::*;
use semver::Version;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn ts(ms: i64) -> Timestamp {
    Timestamp(ms)
}

fn person(n: i64) -> PersonId {
    PersonId::from_uuid(Uuid::from_u128(n as u128))
}

fn envelope() -> ProvenanceEnvelope {
    ProvenanceEnvelope::new(
        Origin::SelfReported,
        ActorRef::Human(person(1)),
        person(1),
        ts(10),
    )
    .expect("valid envelope")
}

fn round_trip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let json = serde_json::to_string(value).expect("serialize");
    let decoded: T = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(*value, decoded);
}

#[test]
fn circle_ord_matches_audience_size() {
    assert!(Circle::Private < Circle::Trusted);
    assert!(Circle::Trusted < Circle::Group);
    assert!(Circle::Group < Circle::Network);
    assert!(Circle::Network < Circle::Public);
    assert_eq!(
        std::cmp::min(Circle::Trusted, Circle::Public),
        Circle::Trusted
    );
}

#[test]
fn tier_ceiling_and_restrictiveness() {
    assert_eq!(SensitivityTier::T0.ceiling(), Circle::Public);
    assert_eq!(SensitivityTier::T1.ceiling(), Circle::Network);
    assert_eq!(SensitivityTier::T2.ceiling(), Circle::Group);
    assert_eq!(SensitivityTier::T3.ceiling(), Circle::Private);
    assert_eq!(
        SensitivityTier::most_restrictive(SensitivityTier::T1, SensitivityTier::T3),
        SensitivityTier::T3
    );
    assert_eq!(
        SensitivityTier::most_restrictive(SensitivityTier::T3, SensitivityTier::T1),
        SensitivityTier::T3
    );
}

#[test]
fn provenance_accountability_shapes() {
    let human = person(1);
    let other = person(2);
    let agent = ActorRef::Agent {
        agent_id: "agent-alpha".to_string(),
    };

    assert!(
        ProvenanceEnvelope::new(Origin::Authored, ActorRef::Human(human), human, ts(10)).is_ok()
    );
    assert!(ProvenanceEnvelope::new(Origin::Authored, agent, human, ts(10)).is_ok());
    assert_eq!(
        ProvenanceEnvelope::new(Origin::Authored, ActorRef::Human(human), other, ts(10)),
        Err(ModelError::ResponsibleHumanMismatch)
    );
}

#[test]
fn custody_append_preserves_order() {
    let mut provenance = envelope();
    let first = CustodyEvent {
        id: CustodyEventId::new(ts(1)),
        action: CustodyAction::Created,
        at: ts(1),
        actor: ActorRef::Human(person(1)),
        note: Some("first".to_string()),
    };
    let second = CustodyEvent {
        id: CustodyEventId::new(ts(2)),
        action: CustodyAction::Corrected,
        at: ts(2),
        actor: ActorRef::Human(person(1)),
        note: Some("second".to_string()),
    };
    provenance.append_custody(first.clone());
    provenance.append_custody(second.clone());
    assert_eq!(provenance.custody(), &[first, second]);
}

#[test]
fn attribute_tier_tightening_only() {
    let mut attr = AttributeInstance::new(
        AttributeValue::Text("ok".to_string()),
        Circle::Private,
        envelope(),
    );
    assert_eq!(
        attr.effective_tier(SensitivityTier::T1),
        SensitivityTier::T1
    );
    attr.tighten_tier(SensitivityTier::T1, SensitivityTier::T2)
        .expect("tighten");
    assert_eq!(
        attr.effective_tier(SensitivityTier::T1),
        SensitivityTier::T2
    );
    assert_eq!(
        attr.tighten_tier(SensitivityTier::T1, SensitivityTier::T2),
        Err(ModelError::TierLoosenAttempt {
            current: SensitivityTier::T2,
            attempted: SensitivityTier::T2
        })
    );
    assert_eq!(
        attr.tighten_tier(SensitivityTier::T1, SensitivityTier::T0),
        Err(ModelError::TierLoosenAttempt {
            current: SensitivityTier::T2,
            attempted: SensitivityTier::T0
        })
    );
}

#[test]
fn value_validation_rules() {
    assert!(IsoDate::new("2026-02-28").is_ok());
    assert!(IsoDate::new("2026-02-30").is_err());
    assert!(IsoDate::new("2026-13-01").is_err());
    assert!(IsoDate::new("junk").is_err());
    assert!(IsoDate::new("2026-02-28T00:00:00Z").is_err());

    assert!(GeoValue::point(45.0, 90.0).is_ok());
    assert!(GeoValue::point(91.0, 90.0).is_err());
    assert!(GeoValue::point(45.0, 181.0).is_err());

    let email = LinkValue::new("person@example.test", Some(LinkFormat::Email)).expect("email");
    assert_eq!(email.url(), "mailto:person@example.test");
    assert!(LinkValue::new("not a url", Some(LinkFormat::Url)).is_err());

    assert!(KindId::new("person_kind").is_ok());
    assert!(KindId::new("Person").is_err());
    assert!(AttrId::new("bad slug").is_err());
    assert!(TemplateId::new("1bad").is_err());
}

#[test]
fn serde_round_trip_ids_and_enums() {
    let at = ts(1_783_456_789_000);
    round_trip(&at);
    round_trip(&EntityId::new(at));
    round_trip(&EdgeId::new(at));
    round_trip(&GroupId::new(at));
    round_trip(&PersonId::new(at));
    round_trip(&OpId::new(at));
    round_trip(&StoryId::new(at));
    round_trip(&CustodyEventId::new(at));
    round_trip(&MembershipId::new(at));
    round_trip(&TrustGrantId::new(at));
    round_trip(&KindId::new("entity").expect("kind"));
    round_trip(&AttrId::new("name").expect("attr"));
    round_trip(&TemplateId::new("base_template").expect("template"));
    round_trip(&Circle::Network);
    round_trip(&SensitivityTier::T2);
    round_trip(&Origin::Ingested {
        source: "synthetic".to_string(),
    });
    round_trip(&Origin::Derived {
        inputs: vec![OpId::new(at)],
    });
    round_trip(&ActorRef::Agent {
        agent_id: "agent-alpha".to_string(),
    });
    round_trip(&CustodyAction::Migrated);
    round_trip(&LinkFormat::Email);
    round_trip(&Lifecycle::Archived);
    round_trip(&GroupRole::Governance);
    round_trip(&TrustScope::Group(GroupId::new(at)));
}

#[test]
fn serde_round_trip_provenance() {
    let at = ts(1_783_456_789_000);
    let mut provenance = envelope();
    let custody = CustodyEvent {
        id: CustodyEventId::new(at),
        action: CustodyAction::Imported,
        at,
        actor: ActorRef::Human(person(1)),
        note: None,
    };
    provenance.append_custody(custody.clone());
    round_trip(&custody);
    round_trip(&provenance);
}

#[test]
fn serde_round_trip_attributes() {
    let provenance = envelope();
    round_trip(&AttributeValue::Text("synthetic".to_string()));
    round_trip(&AttributeValue::Number(4.0));
    round_trip(&AttributeValue::Enum("member".to_string()));
    round_trip(&AttributeValue::Tags(["a".to_string()].into()));
    round_trip(&AttributeValue::Date(
        IsoDate::new("2026-02-28").expect("date"),
    ));
    round_trip(&AttributeValue::Geo(
        GeoValue::point(1.0, 2.0).expect("geo"),
    ));
    round_trip(&AttributeValue::Geo(GeoValue::Region {
        name: "region".to_string(),
    }));
    round_trip(&AttributeValue::Link(
        LinkValue::new("https://example.test", Some(LinkFormat::Url)).expect("url"),
    ));
    round_trip(&AttributeValue::Media(
        MediaRefId::new("media-1").expect("media"),
    ));
    let attr = AttributeInstance::new(
        AttributeValue::Text("v".to_string()),
        Circle::Trusted,
        provenance,
    );
    round_trip(&attr);
}

#[test]
fn serde_round_trip_aggregates() {
    let at = ts(1_783_456_789_000);
    let provenance = envelope();
    let group_id = GroupId::new(at);
    let entity = Entity::new(
        EntityId::new(at),
        group_id,
        KindId::new("person").expect("kind"),
        Circle::Group,
        provenance.clone(),
        SensitivityTier::T2,
    );
    round_trip(&entity);
    let edge = Edge::new(
        EdgeId::new(at),
        group_id,
        KindId::new("knows").expect("kind"),
        EntityId::new(at),
        EntityId::new(at),
        false,
        Circle::Trusted,
        provenance.clone(),
        SensitivityTier::T1,
    );
    round_trip(&edge);
    round_trip(
        &Group::new(
            group_id,
            "Synthetic Group",
            TemplateId::new("base").expect("template"),
            Version::new(1, 2, 3),
            provenance.clone(),
            SensitivityTier::T2,
        )
        .expect("group"),
    );
    round_trip(&GroupRole::Governance);
    round_trip(&Membership::new(
        MembershipId::new(at),
        group_id,
        person(1),
        GroupRole::Member,
        provenance.clone(),
    ));
    round_trip(&TrustGrant::new(
        TrustGrantId::new(at),
        person(1),
        person(2),
        TrustScope::All,
        at,
        provenance,
    ));
    round_trip(&Story::new(
        StoryId::new(at),
        group_id,
        "Synthetic Story",
        vec![StoryStep {
            entity: EntityId::new(at),
            narration: "Synthetic step".to_string(),
        }],
        Circle::Group,
        envelope(),
        SensitivityTier::T1,
    ));
}

#[test]
fn serde_rejects_bad_envelope() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct EnvelopeShape {
        origin: Origin,
        recorded_by: ActorRef,
        responsible_human: PersonId,
        recorded_at: Timestamp,
        custody: Vec<CustodyEvent>,
        schema_version: Version,
    }
    let at = ts(1_783_456_789_000);
    let bad = EnvelopeShape {
        origin: Origin::Authored,
        recorded_by: ActorRef::Human(person(1)),
        responsible_human: person(2),
        recorded_at: at,
        custody: Vec::new(),
        schema_version: model_schema_version(),
    };
    let json = serde_json::to_string(&bad).expect("serialize bad shape");
    assert!(serde_json::from_str::<ProvenanceEnvelope>(&json).is_err());
}

#[test]
fn edge_weight_rejects_non_finite() {
    let mut edge = Edge::new(
        EdgeId::new(ts(1)),
        GroupId::new(ts(1)),
        KindId::new("rel").expect("kind"),
        EntityId::new(ts(1)),
        EntityId::new(ts(2)),
        true,
        Circle::Private,
        envelope(),
        SensitivityTier::T0,
    );
    assert!(edge.set_weight(Some(1.0)).is_ok());
    assert_eq!(
        edge.set_weight(Some(f64::NAN)),
        Err(ModelError::NonFiniteWeight)
    );
    assert_eq!(
        edge.set_weight(Some(f64::INFINITY)),
        Err(ModelError::NonFiniteWeight)
    );
}

#[test]
fn trust_revoke_rules() {
    let mut grant = TrustGrant::new(
        TrustGrantId::new(ts(1)),
        person(1),
        person(2),
        TrustScope::All,
        ts(10),
        envelope(),
    );
    assert!(grant.is_active());
    assert_eq!(grant.revoke(ts(9)), Err(ModelError::RevokedBeforeGranted));
    grant.revoke(ts(10)).expect("revoke");
    assert!(!grant.is_active());
    assert_eq!(grant.revoke(ts(11)), Err(ModelError::AlreadyRevoked));
}

#[test]
fn schema_acceptance_rules() {
    assert!(accepts_schema(&Version::new(0, 1, 99)));
    assert!(!accepts_schema(&Version::new(0, 2, 0)));
    assert!(!accepts_schema(&Version::new(1, 1, 0)));

    fn hypothetical_accepts(current: &Version, value: &Version) -> bool {
        if current.major == 0 || value.major == 0 {
            value.major == current.major && value.minor == current.minor
        } else {
            value.major == current.major
        }
    }
    assert!(hypothetical_accepts(
        &Version::new(1, 0, 0),
        &Version::new(1, 9, 9)
    ));
}

#[test]
fn ids_are_distinct_and_string_round_trip() {
    let at = ts(1_783_456_789_000);
    let first = EntityId::new(at);
    let second = EntityId::new(at);
    assert_ne!(first, second);
    let parsed = EntityId::from_str(&first.to_string()).expect("parse");
    assert_eq!(first, parsed);
}
