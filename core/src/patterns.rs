use crate::schema::{Evidence, Observation};
use serde_json::Value;
use std::collections::HashMap;

pub trait ObservationAdapter: Send + Sync {
    fn supports(&self, extractor: &str) -> bool;
    fn adapt(&self, idx: usize, evidence: &Evidence) -> Observation;
}

pub struct DefaultAdapter;

impl ObservationAdapter for DefaultAdapter {
    fn supports(&self, _extractor: &str) -> bool {
        true
    }

    fn adapt(&self, idx: usize, evidence: &Evidence) -> Observation {
        Observation {
            id: format!("ob_{idx}"),
            record_type: "observation".to_string(),
            kind: evidence.extractor.clone(),
            canonical_name: evidence.extractor.clone(),
            path: evidence.path.clone(),
            details: HashMap::new(),
        }
    }
}

pub struct HttpRouteAdapter;

impl ObservationAdapter for HttpRouteAdapter {
    fn supports(&self, extractor: &str) -> bool {
        extractor == "http_routes"
    }

    fn adapt(&self, idx: usize, evidence: &Evidence) -> Observation {
        let method = str_capture(&evidence.captures, "method").unwrap_or("GET");
        let route = str_capture(&evidence.captures, "route").unwrap_or("/");
        Observation {
            id: format!("ob_{idx}"),
            record_type: "observation".to_string(),
            kind: "http_route".to_string(),
            canonical_name: format!("{method} {route}"),
            path: evidence.path.clone(),
            details: HashMap::new(),
        }
    }
}

pub struct EnvVarAdapter;

impl ObservationAdapter for EnvVarAdapter {
    fn supports(&self, extractor: &str) -> bool {
        extractor == "env_vars"
    }

    fn adapt(&self, idx: usize, evidence: &Evidence) -> Observation {
        let name = str_capture(&evidence.captures, "name").unwrap_or("UNKNOWN");
        Observation {
            id: format!("ob_{idx}"),
            record_type: "observation".to_string(),
            kind: "env_var_use".to_string(),
            canonical_name: name.to_string(),
            path: evidence.path.clone(),
            details: HashMap::new(),
        }
    }
}

pub struct DbTableAdapter;

impl ObservationAdapter for DbTableAdapter {
    fn supports(&self, extractor: &str) -> bool {
        extractor == "db_tables"
    }

    fn adapt(&self, idx: usize, evidence: &Evidence) -> Observation {
        let table = str_capture(&evidence.captures, "table").unwrap_or("unknown_table");
        Observation {
            id: format!("ob_{idx}"),
            record_type: "observation".to_string(),
            kind: "db_table".to_string(),
            canonical_name: table.to_string(),
            path: evidence.path.clone(),
            details: HashMap::new(),
        }
    }
}

pub struct AdapterRegistry {
    adapters: Vec<Box<dyn ObservationAdapter>>,
    fallback: Box<dyn ObservationAdapter>,
}

impl AdapterRegistry {
    pub fn default() -> Self {
        Self {
            adapters: vec![
                Box::new(HttpRouteAdapter),
                Box::new(EnvVarAdapter),
                Box::new(DbTableAdapter),
            ],
            fallback: Box::new(DefaultAdapter),
        }
    }

    pub fn adapt(&self, idx: usize, evidence: &Evidence) -> Observation {
        for adapter in &self.adapters {
            if adapter.supports(&evidence.extractor) {
                return adapter.adapt(idx, evidence);
            }
        }
        self.fallback.adapt(idx, evidence)
    }
}

fn str_capture<'a>(captures: &'a HashMap<String, Value>, key: &str) -> Option<&'a str> {
    captures.get(key).and_then(|v| v.as_str())
}
