use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::{ModelError, Timestamp};

fn uuid_timestamp(at: Timestamp) -> uuid::Timestamp {
    let seconds = at.0.div_euclid(1_000) as u64;
    let millis = at.0.rem_euclid(1_000) as u32;
    uuid::Timestamp::from_unix(uuid::NoContext, seconds, millis * 1_000_000)
}

macro_rules! uuid_id {
    ($name:ident) => {
        #[doc = concat!("UUIDv7-backed identifier for ", stringify!($name), ".")]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Creates a new UUIDv7 identifier using the injected timestamp.
            pub fn new(at: Timestamp) -> Self {
                Self(Uuid::new_v7(uuid_timestamp(at)))
            }

            /// Wraps an existing UUID as this identifier type.
            pub fn from_uuid(u: Uuid) -> Self {
                Self(u)
            }

            /// Returns the underlying UUID.
            pub fn as_uuid(&self) -> &Uuid {
                &self.0
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        impl FromStr for $name {
            type Err = ModelError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s)
                    .map(Self)
                    .map_err(|_| ModelError::InvalidId(s.to_string()))
            }
        }
    };
}

uuid_id!(EntityId);
uuid_id!(EdgeId);
uuid_id!(GroupId);
uuid_id!(PersonId);
uuid_id!(OpId);
uuid_id!(StoryId);
uuid_id!(CustodyEventId);
uuid_id!(MembershipId);
uuid_id!(TrustGrantId);

fn valid_slug(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

macro_rules! slug_id {
    ($name:ident, $what:literal) => {
        #[doc = concat!("Validated slug identifier for ", $what, ".")]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Creates a validated slug identifier.
            pub fn new(value: impl Into<String>) -> Result<Self, ModelError> {
                let value = value.into();
                if valid_slug(&value) {
                    Ok(Self(value))
                } else {
                    Err(ModelError::InvalidSlug { what: $what, value })
                }
            }

            /// Returns this identifier as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(serde::de::Error::custom)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}

slug_id!(KindId, "kind");
slug_id!(AttrId, "attribute");
slug_id!(TemplateId, "template");
