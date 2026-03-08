use crate::schema::{Observation, Relic};
use std::collections::{BTreeSet, HashMap};

pub fn build_relics(observations: &[Observation]) -> Vec<Relic> {
    let mut buckets: HashMap<String, BTreeSet<String>> = HashMap::new();

    for ob in observations {
        let key = if ob.path.contains("/payout") {
            "Payout Logic"
        } else if ob.path.contains("/feed") {
            "Feed Logic"
        } else if ob.kind == "http_route" {
            "HTTP Surface"
        } else if ob.kind == "db_table" {
            "Persistence"
        } else {
            "Core Architecture"
        };
        buckets
            .entry(key.to_string())
            .or_default()
            .insert(ob.path.clone());
    }

    buckets
        .into_iter()
        .map(|(label, paths)| Relic {
            record_type: "relic".into(),
            theme: format!("{} theme", label),
            distinguishing_feature: "Grouped using path + evidence heuristics".into(),
            label,
            paths: paths.into_iter().collect(),
        })
        .collect()
}
