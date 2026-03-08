use crate::schema::Evidence;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorSpec {
    pub name: String,
    pub script: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorManifest {
    pub extractors: Vec<ExtractorSpec>,
}

pub fn load_manifest(path: &Path) -> Result<ExtractorManifest> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read extractor manifest at {}", path.display()))?;
    let manifest = serde_json::from_str::<ExtractorManifest>(&raw)
        .with_context(|| "failed to parse extractor manifest JSON")?;
    Ok(manifest)
}

pub fn run_extractors(repo_root: &Path, manifest: &ExtractorManifest) -> Result<Vec<Evidence>> {
    let mut all = Vec::new();
    for ex in manifest.extractors.iter().filter(|x| x.enabled) {
        let script_path = repo_root.join(&ex.script);
        let output = run_one_extractor(repo_root, &script_path)
            .with_context(|| format!("extractor '{}' failed", ex.name))?;
        all.extend(output);
    }
    Ok(all)
}

fn run_one_extractor(repo_root: &Path, script_path: &Path) -> Result<Vec<Evidence>> {
    let output = Command::new("perl")
        .arg(script_path)
        .arg(repo_root)
        .output()
        .with_context(|| format!("failed to spawn perl for {}", script_path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "perl extractor failed: {}\n{}",
            script_path.display(),
            stderr.trim()
        );
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut out = Vec::new();
    for (idx, line) in stdout.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let ev = serde_json::from_str::<Evidence>(line).with_context(|| {
            format!(
                "invalid JSONL from {} at line {}",
                script_path.display(),
                idx + 1
            )
        })?;
        out.push(ev);
    }
    Ok(out)
}

pub fn manifest_path(repo_root: &Path) -> PathBuf {
    repo_root.join("extractors/manifest.json")
}
