use crate::schema::{Observation, SemanticCluster};
use std::collections::{BTreeSet, HashMap};

pub fn build_semantic_clusters(observations: &[Observation]) -> Vec<SemanticCluster> {
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
        .map(|(label, paths)| SemanticCluster {
            record_type: "semantic_cluster".into(),
            theme: format!("{} theme", label),
            distinguishing_feature: "Grouped using path + evidence heuristics".into(),
            label,
            paths: paths.into_iter().collect(),
        })
        .collect()
}
