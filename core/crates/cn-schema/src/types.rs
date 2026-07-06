use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

use cn_model::{AttrId, Circle, KindId, LinkFormat, TemplateId};
use serde::{Deserialize, Deserializer, Serialize};

/// Parsed group template document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupTemplate {
    /// Persisted template schema version.
    pub schema_version: semver::Version,
    /// Stable template id.
    pub template_id: TemplateId,
    /// Human-readable template name.
    pub name: String,
    /// Human-readable template description.
    pub description: String,
    /// Template-specific vocabulary labels.
    #[serde(default)]
    pub vocabulary: BTreeMap<String, String>,
    /// Entity kind definitions.
    pub kinds: Vec<KindDef>,
    /// Edge kind definitions.
    pub edge_kinds: Vec<EdgeKindDef>,
    /// Optional visual theme.
    #[serde(default)]
    pub theme: Option<Theme>,
}

/// Entity kind definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KindDef {
    /// Entity kind id.
    pub id: KindId,
    /// Human-readable kind label.
    pub label: String,
    /// Preferred 3D shape.
    pub shape: Shape,
    /// Theme role used to color this kind.
    pub color_role: ColorRole,
    /// Attribute definitions for this kind.
    pub attributes: Vec<AttrDef>,
}

/// Supported entity shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shape {
    /// Sphere shape.
    Sphere,
    /// Cube shape.
    Cube,
    /// Octahedron shape.
    Octahedron,
    /// Tetrahedron shape.
    Tetrahedron,
    /// Torus shape.
    Torus,
    /// Cone shape.
    Cone,
}

/// Theme color role for an entity kind.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ColorRole(String);

impl ColorRole {
    /// Creates a validated kind color role.
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if valid_color_role(&value) {
            Ok(Self(value))
        } else {
            Err(format!("invalid color role: {value}"))
        }
    }

    /// Returns this color role as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ColorRole {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl Display for ColorRole {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Attribute definition for an entity kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttrDef {
    /// Attribute id.
    pub id: AttrId,
    /// Attribute value type.
    #[serde(rename = "type")]
    pub attr_type: AttrType,
    /// Whether this attribute must be present.
    #[serde(default)]
    pub required: bool,
    /// Allowed enum values.
    #[serde(default)]
    pub values: Option<Vec<String>>,
    /// Link format hint.
    #[serde(default)]
    pub format: Option<LinkFormat>,
    /// Default visibility for instances of this attribute.
    #[serde(default = "default_visibility_group")]
    pub default_visibility: Circle,
}

/// Attribute value type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttrType {
    /// Text value.
    Text,
    /// Numeric value.
    Number,
    /// Enum symbol value.
    Enum,
    /// Tags value.
    Tags,
    /// Calendar date value.
    Date,
    /// Geographic value.
    Geo,
    /// Link value.
    Link,
    /// Media reference value.
    Media,
}

/// Edge kind definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdgeKindDef {
    /// Edge kind id.
    pub id: KindId,
    /// Human-readable edge label.
    pub label: String,
    /// Allowed source entity kinds.
    pub from: Vec<KindId>,
    /// Allowed target entity kinds.
    pub to: Vec<KindId>,
    /// Whether this edge is directed.
    pub directed: bool,
    /// Edge weight policy.
    #[serde(default)]
    pub weighted: Weighted,
}

/// Edge weight policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Weighted {
    /// Edge must carry a weight.
    Required,
    /// Edge may carry a weight.
    Optional,
    /// Edge must not carry a weight.
    #[default]
    Forbidden,
}

/// Template visual theme.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Theme {
    /// Theme mode name.
    pub mode: String,
    /// Color role mapping.
    pub roles: BTreeMap<String, HexColor>,
}

/// Lowercase hex color in #rrggbb form.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct HexColor(String);

impl HexColor {
    /// Creates a validated lowercase hex color.
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if valid_hex_color(&value) {
            Ok(Self(value))
        } else {
            Err(format!("invalid hex color: {value}"))
        }
    }

    /// Returns this color as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl Display for HexColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

pub(crate) fn default_visibility_group() -> Circle {
    Circle::Group
}

fn valid_color_role(value: &str) -> bool {
    let Some(rest) = value.strip_prefix("kind-") else {
        return false;
    };
    let mut chars = rest.chars();
    match chars.next() {
        Some(first) if ('1'..='9').contains(&first) => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_digit())
}

fn valid_hex_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7
        && bytes[0] == b'#'
        && bytes[1..]
            .iter()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(b))
}
