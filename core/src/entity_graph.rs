use crate::schema::{Entity, Observation};
use std::collections::HashMap;

pub fn build_entities(observations: &[Observation]) -> Vec<Entity> {
    let mut grouped: HashMap<(String, String), Vec<String>> = HashMap::new();
    for ob in observations {
        grouped
            .entry((ob.kind.clone(), ob.canonical_name.clone()))
            .or_default()
            .push(ob.id.clone());
    }

    grouped
        .into_iter()
        .enumerate()
        .map(|(i, ((kind, name), obs_ids))| Entity {
            id: format!("ent_{i}"),
            record_type: "entity".to_string(),
            entity_type: kind,
            canonical_name: name,
            observations: obs_ids,
        })
        .collect()
}
