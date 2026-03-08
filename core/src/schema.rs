use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: String,
    pub record_type: String,
    pub extractor: String,
    pub path: String,
    pub line: usize,
    pub raw: String,
    #[serde(default)]
    pub captures: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,
    pub record_type: String,
    pub kind: String,
    pub canonical_name: String,
    pub path: String,
    #[serde(default)]
    pub details: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    pub record_type: String,
    pub entity_type: String,
    pub canonical_name: String,
    pub observations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub record_type: String,
    pub relation_type: String,
    pub src_entity: String,
    pub dst_entity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticCluster {
    pub record_type: String,
    pub label: String,
    pub theme: String,
    pub distinguishing_feature: String,
    pub paths: Vec<String>,
}
