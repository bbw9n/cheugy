use cheugy_core::perl_bridge::load_manifest;
use cheugy_core::schema::Evidence;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct TempRepo {
    root: PathBuf,
}

impl TempRepo {
    fn new() -> Self {
        let unique = format!(
            "cheugy-extractors-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time before unix epoch")
                .as_nanos()
        );
        let root = std::env::temp_dir().join(unique);
        fs::create_dir_all(&root).expect("failed to create temp repo");
        Self { root }
    }

    fn path(&self) -> &Path {
        &self.root
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.root.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("failed to create parent directory");
        }
        fs::write(path, contents).expect("failed to write fixture file");
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("core crate should live under workspace root")
        .to_path_buf()
}

fn run_extractor(script_path: &Path, repo_root: &Path) -> Vec<Evidence> {
    let output = Command::new("perl")
        .arg(script_path)
        .arg(repo_root)
        .output()
        .expect("failed to run perl extractor");

    assert!(
        output.status.success(),
        "extractor failed: {}\n{}",
        script_path.display(),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout)
        .expect("extractor stdout should be utf-8")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<Evidence>(line).expect("invalid evidence json"))
        .collect()
}

#[test]
fn all_enabled_extractors_emit_expected_evidence() {
    let repo = TempRepo::new();
    repo.write(
        "src/main.rs",
        r#"fn main() {
    println!("boot");
}
"#,
    );
    repo.write(
        "src/app.ts",
        r#"if (require.main === module) {
  console.log("boot");
}

router.POST("/payments/:id", handlePayment);
const apiBase = "https://api.example.com/v1";
const topicName = kafka.publish("billing.events");
const flag = feature_flag("checkout_redesign");
const counter = prometheus.counter("billing_requests_total");
const query = "select * from invoices";
const grpcClient = grpc.connect("BillingService");
const cron = "@hourly";
"#,
    );
    repo.write(
        "src/config.py",
        r#"import os

token = os.getenv("PAYMENTS_API_KEY")

if __name__ == "__main__":
    print(token)
"#,
    );
    repo.write(
        "src/app.rb",
        r#"if __FILE__ == $PROGRAM_NAME
  puts "boot"
end

post "/ruby-payments/:id" do
  ENV["RUBY_API_KEY"]
  feature_flag("ruby_rollout")
  prometheus.counter("ruby_requests_total")
  kafka.publish("ruby.events")
  grpc.connect("RubyBillingService")
  Faraday.get("https://ruby.example.com/v1")
  sql = "select * from ruby_invoices"
  cron = "@daily"
end
"#,
    );
    repo.write(
        "proto/billing.proto",
        r#"service BillingService {
  rpc Charge (ChargeRequest) returns (ChargeReply);
}
"#,
    );
    repo.write(
        "deploy/cron.yaml",
        r#"schedule: "0 * * * *"
"#,
    );

    let workspace = workspace_root();
    let manifest = load_manifest(&workspace.join("extractors/manifest.json"))
        .expect("workspace manifest should parse");

    let mut outputs = HashMap::new();
    for spec in manifest.extractors.iter().filter(|spec| spec.enabled) {
        let evidence = run_extractor(&workspace.join(&spec.script), repo.path());
        outputs.insert(spec.name.as_str(), evidence);
    }

    assert_eq!(outputs.len(), 10, "expected coverage for every enabled extractor");

    assert_has_evidence(&outputs, "entrypoints", "src/main.rs", Some(("kind", "entrypoint")));
    assert_has_evidence(&outputs, "entrypoints", "src/app.ts", Some(("kind", "entrypoint")));
    assert_has_evidence(&outputs, "entrypoints", "src/app.rb", Some(("kind", "entrypoint")));
    assert_has_evidence(
        &outputs,
        "http_routes",
        "src/app.ts",
        Some(("route", "/payments/:id")),
    );
    assert_has_evidence(
        &outputs,
        "http_routes",
        "src/app.rb",
        Some(("route", "/ruby-payments/:id")),
    );
    assert_has_evidence(
        &outputs,
        "env_vars",
        "src/config.py",
        Some(("name", "PAYMENTS_API_KEY")),
    );
    assert_has_evidence(
        &outputs,
        "env_vars",
        "src/app.rb",
        Some(("name", "RUBY_API_KEY")),
    );
    assert_has_evidence(
        &outputs,
        "db_tables",
        "src/app.ts",
        Some(("table", "invoices")),
    );
    assert_has_evidence(
        &outputs,
        "db_tables",
        "src/app.rb",
        Some(("table", "ruby_invoices")),
    );
    assert_has_evidence(
        &outputs,
        "queues",
        "src/app.ts",
        Some(("topic", "billing.events")),
    );
    assert_has_evidence(
        &outputs,
        "queues",
        "src/app.rb",
        Some(("topic", "ruby.events")),
    );
    assert_has_evidence(
        &outputs,
        "rpc_services",
        "proto/billing.proto",
        Some(("name", "BillingService")),
    );
    assert_has_evidence(
        &outputs,
        "rpc_services",
        "src/app.ts",
        Some(("name", "BillingService")),
    );
    assert_has_evidence(
        &outputs,
        "rpc_services",
        "src/app.rb",
        Some(("name", "RubyBillingService")),
    );
    assert_has_evidence(
        &outputs,
        "scheduled_jobs",
        "deploy/cron.yaml",
        Some(("marker", "schedule")),
    );
    assert_has_evidence(
        &outputs,
        "scheduled_jobs",
        "src/app.ts",
        Some(("marker", "schedule")),
    );
    assert_has_evidence(
        &outputs,
        "scheduled_jobs",
        "src/app.rb",
        Some(("marker", "schedule")),
    );
    assert_has_evidence(
        &outputs,
        "feature_flags",
        "src/app.ts",
        Some(("name", "checkout_redesign")),
    );
    assert_has_evidence(
        &outputs,
        "feature_flags",
        "src/app.rb",
        Some(("name", "ruby_rollout")),
    );
    assert_has_evidence(
        &outputs,
        "metrics",
        "src/app.ts",
        Some(("name", "billing_requests_total")),
    );
    assert_has_evidence(
        &outputs,
        "metrics",
        "src/app.rb",
        Some(("name", "ruby_requests_total")),
    );
    assert_has_evidence(
        &outputs,
        "external_urls",
        "src/app.ts",
        Some(("url", "https://api.example.com/v1")),
    );
    assert_has_evidence(
        &outputs,
        "external_urls",
        "src/app.rb",
        Some(("url", "https://ruby.example.com/v1")),
    );
}

fn assert_has_evidence(
    outputs: &HashMap<&str, Vec<Evidence>>,
    extractor: &str,
    path: &str,
    capture: Option<(&str, &str)>,
) {
    let evidence = outputs
        .get(extractor)
        .unwrap_or_else(|| panic!("missing extractor output for {extractor}"));

    let found = evidence.iter().find(|ev| {
        ev.extractor == extractor
            && ev.path == path
            && capture.is_none_or(|(key, value)| {
                ev.captures.get(key).and_then(|v| v.as_str()) == Some(value)
            })
    });

    assert!(
        found.is_some(),
        "expected evidence from extractor {extractor} for path {path} with capture {capture:?}, got {evidence:#?}"
    );
}
