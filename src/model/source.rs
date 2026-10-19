//! Original package XML addresses, independent of rendering and extraction order.
use serde::{Deserialize, Serialize};

/// One expanded-name step; index counts siblings with the same namespace and name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XmlPathSegment {
    pub namespace: String,
    pub local_name: String,
    pub index: u32,
}

/// A single original XML node. Synthetic content has no source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XmlSourceLocation {
    pub part: String,
    pub path: Vec<XmlPathSegment>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paragraph_id: Option<String>,
}
