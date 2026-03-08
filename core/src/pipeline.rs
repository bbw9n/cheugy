use crate::cluster_engine::build_semantic_clusters;
use crate::entity_graph::build_entities;
use crate::patterns::AdapterRegistry;
use crate::perl_bridge::{load_manifest, manifest_path, run_extractors};
use crate::relation_engine::infer_relations;
use crate::schema::{Entity, Evidence, Observation, Relation, SemanticCluster};
use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::json;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct BuildArtifacts {
    pub observations: Vec<Observation>,
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
    pub clusters: Vec<SemanticCluster>,
}

pub fn init_repo(root: &Path) -> Result<()> {
    fs::create_dir_all(root.join(".cheugy"))?;
    Ok(())
}

pub fn scan(root: &Path) -> Result<Vec<Evidence>> {
    init_repo(root)?;
    let manifest = load_manifest(&manifest_path(root))?;
    let evidence = run_extractors(root, &manifest)?;
    write_jsonl(root.join(".cheugy/evidence.jsonl"), &evidence)?;
    Ok(evidence)
}

pub fn build(root: &Path) -> Result<BuildArtifacts> {
    let evidence = read_jsonl::<Evidence>(&root.join(".cheugy/evidence.jsonl"))?;
    let observations = normalize_observations(&evidence);
    let entities = build_entities(&observations);
    let relations = infer_relations(&entities);
    let clusters = build_semantic_clusters(&observations);

    write_jsonl(root.join(".cheugy/observations.jsonl"), &observations)?;
    write_jsonl(root.join(".cheugy/entities.jsonl"), &entities)?;
    write_jsonl(root.join(".cheugy/relations.jsonl"), &relations)?;
    write_jsonl(root.join(".cheugy/clusters.jsonl"), &clusters)?;

    fs::write(
        root.join(".cheugy/index.json"),
        serde_json::to_vec_pretty(&json!({
            "evidence": evidence.len(),
            "observations": observations.len(),
            "entities": entities.len(),
            "relations": relations.len(),
            "clusters": clusters.len(),
        }))?,
    )?;

    Ok(BuildArtifacts {
        observations,
        entities,
        relations,
        clusters,
    })
}

pub fn inspect_entity_type(root: &Path, entity_type: &str) -> Result<Vec<Entity>> {
    let entities = read_jsonl::<Entity>(&root.join(".cheugy/entities.jsonl"))?;
    Ok(entities
        .into_iter()
        .filter(|e| e.entity_type == entity_type)
        .collect())
}

pub fn query(root: &Path, q: &str) -> Result<Vec<SemanticCluster>> {
    let query = q.to_lowercase();
    let clusters = read_jsonl::<SemanticCluster>(&root.join(".cheugy/clusters.jsonl"))?;
    Ok(clusters
        .into_iter()
        .filter(|c| {
            c.label.to_lowercase().contains(&query)
                || c.theme.to_lowercase().contains(&query)
                || c
                    .distinguishing_feature
                    .to_lowercase()
                    .contains(&query)
                || c.paths.iter().any(|p| p.to_lowercase().contains(&query))
        })
        .collect())
}

fn normalize_observations(evidence: &[Evidence]) -> Vec<Observation> {
    let registry = AdapterRegistry::default();
    evidence
        .iter()
        .enumerate()
        .map(|(i, ev)| registry.adapt(i, ev))
        .collect()
}

fn write_jsonl<T: Serialize>(path: impl AsRef<Path>, records: &[T]) -> Result<()> {
    let mut file = File::create(path.as_ref())
        .with_context(|| format!("failed to create {}", path.as_ref().display()))?;
    for record in records {
        writeln!(file, "{}", serde_json::to_string(record)?)?;
    }
    Ok(())
}

pub fn read_jsonl<T: for<'de> serde::Deserialize<'de>>(path: &Path) -> Result<Vec<T>> {
    let file = File::open(path).with_context(|| format!("failed to open {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut out = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        out.push(serde_json::from_str(&line)?);
    }
    Ok(out)
}
