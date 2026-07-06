use std::collections::BTreeSet;

use serde::{Deserialize, Deserializer, Serialize};

use crate::{Circle, ModelError, ProvenanceEnvelope, SensitivityTier};

/// Attribute value carried by an entity or edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum AttributeValue {
    /// Text value.
    Text(String),
    /// Numeric value.
    Number(f64),
    /// Template-validated enum symbol.
    Enum(String),
    /// Ordered set of tags.
    Tags(BTreeSet<String>),
    /// ISO calendar date.
    Date(IsoDate),
    /// Geographic value.
    Geo(GeoValue),
    /// Link value.
    Link(LinkValue),
    /// Media reference.
    Media(MediaRefId),
}

/// Calendar date in YYYY-MM-DD form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct IsoDate(String);

impl IsoDate {
    /// Creates a validated ISO calendar date.
    pub fn new(value: impl Into<String>) -> Result<Self, ModelError> {
        let value = value.into();
        if valid_iso_date(&value) {
            Ok(Self(value))
        } else {
            Err(ModelError::InvalidDate(value))
        }
    }

    /// Returns this date as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for IsoDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Geographic value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeoValue {
    /// Latitude and longitude point.
    Point { lat: f64, lon: f64 },
    /// Named geographic region.
    Region { name: String },
}

impl GeoValue {
    /// Creates a validated point.
    pub fn point(lat: f64, lon: f64) -> Result<Self, ModelError> {
        if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lon) {
            Ok(Self::Point { lat, lon })
        } else {
            Err(ModelError::InvalidGeo { lat, lon })
        }
    }
}

/// Link value with an optional format hint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LinkValue {
    url: String,
    format: Option<LinkFormat>,
}

impl LinkValue {
    /// Creates a validated link value.
    pub fn new(url: impl Into<String>, format: Option<LinkFormat>) -> Result<Self, ModelError> {
        let mut url = url.into();
        if matches!(format, Some(LinkFormat::Email)) && !url.starts_with("mailto:") {
            url = format!("mailto:{url}");
        }
        if valid_url(&url, format) {
            Ok(Self { url, format })
        } else {
            Err(ModelError::InvalidUrl(url))
        }
    }

    /// Returns the stored URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the optional format hint.
    pub fn format(&self) -> Option<LinkFormat> {
        self.format
    }
}

impl<'de> Deserialize<'de> for LinkValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Raw {
            url: String,
            format: Option<LinkFormat>,
        }

        let raw = Raw::deserialize(deserializer)?;
        Self::new(raw.url, raw.format).map_err(serde::de::Error::custom)
    }
}

/// Link format hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkFormat {
    /// Email link stored as a mailto URL.
    Email,
    /// General URL.
    Url,
}

/// Opaque media reference identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct MediaRefId(String);

impl MediaRefId {
    /// Creates a non-empty media reference.
    pub fn new(value: impl Into<String>) -> Result<Self, ModelError> {
        let value = value.into();
        if value.is_empty() {
            Err(ModelError::InvalidId(value))
        } else {
            Ok(Self(value))
        }
    }

    /// Returns this media reference as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for MediaRefId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

/// Attribute instance with visibility, provenance, and optional tier override.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttributeInstance {
    /// Attribute value.
    pub value: AttributeValue,
    /// Attribute visibility.
    pub visibility: Circle,
    tier_override: Option<SensitivityTier>,
    /// Attribute provenance.
    pub provenance: ProvenanceEnvelope,
}

impl AttributeInstance {
    /// Creates an attribute instance without a tier override.
    pub fn new(value: AttributeValue, visibility: Circle, provenance: ProvenanceEnvelope) -> Self {
        Self {
            value,
            visibility,
            tier_override: None,
            provenance,
        }
    }

    /// Tightens the tier override relative to the current effective tier.
    pub fn tighten_tier(
        &mut self,
        entity_tier: SensitivityTier,
        new_tier: SensitivityTier,
    ) -> Result<(), ModelError> {
        let current = self.effective_tier(entity_tier);
        if new_tier > current {
            self.tier_override = Some(new_tier);
            Ok(())
        } else {
            Err(ModelError::TierLoosenAttempt {
                current,
                attempted: new_tier,
            })
        }
    }

    /// Returns the effective tier for an entity context.
    pub fn effective_tier(&self, entity_tier: SensitivityTier) -> SensitivityTier {
        self.tier_override.map_or(entity_tier, |tier| {
            SensitivityTier::most_restrictive(entity_tier, tier)
        })
    }
}

fn valid_iso_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    let Some(year) = parse_digits(&value[0..4]) else {
        return false;
    };
    let Some(month) = parse_digits(&value[5..7]) else {
        return false;
    };
    let Some(day) = parse_digits(&value[8..10]) else {
        return false;
    };
    let max_day = days_in_month(year, month);
    max_day != 0 && day >= 1 && day <= max_day
}

fn parse_digits(value: &str) -> Option<u32> {
    value.chars().try_fold(0_u32, |acc, c| {
        c.to_digit(10)
            .map(|digit| acc.saturating_mul(10).saturating_add(digit))
    })
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn leap_year(year: u32) -> bool {
    year.is_multiple_of(4) && !year.is_multiple_of(100) || year.is_multiple_of(400)
}

fn valid_url(url: &str, format: Option<LinkFormat>) -> bool {
    if url.is_empty() || url.chars().any(char::is_whitespace) {
        return false;
    }
    match format {
        Some(LinkFormat::Email) => valid_mailto(url),
        _ => url.starts_with("https://") || url.starts_with("http://") || valid_mailto(url),
    }
}

fn valid_mailto(url: &str) -> bool {
    let Some(email) = url.strip_prefix("mailto:") else {
        return false;
    };
    let parts: Vec<&str> = email.split('@').collect();
    parts.len() == 2 && !parts[0].is_empty() && parts[1].contains('.')
}
