use anyhow::Result;
use cheugy_core::pipeline;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cheugy")]
#[command(about = "Code archaeology engine: browse code by meaning")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Init,
    Scan { path: Option<PathBuf> },
    Build { path: Option<PathBuf> },
    Inspect {
        kind: String,
        name: Option<String>,
    },
    Query {
        text: String,
    },
    Explore,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;

    match cli.command {
        Commands::Init => {
            pipeline::init_repo(&cwd)?;
            println!("Initialized .cheugy/");
        }
        Commands::Scan { path } => {
            let root = path.unwrap_or(cwd);
            let evidence = pipeline::scan(&root)?;
            println!("Scan completed. Evidence records: {}", evidence.len());
            println!("Output: {}", root.join(".cheugy/evidence.jsonl").display());
        }
        Commands::Build { path } => {
            let root = path.unwrap_or(cwd);
            let build = pipeline::build(&root)?;
            println!("Build completed");
            println!("Observations: {}", build.observations.len());
            println!("Entities: {}", build.entities.len());
            println!("Relations: {}", build.relations.len());
            println!("Relics: {}", build.relics.len());
        }
        Commands::Inspect { kind, name } => {
            if kind == "service" {
                let service_name = name.unwrap_or_else(|| "unknown".to_string());
                let routes = pipeline::inspect_entity_type(&cwd, "http_route")?;
                let tables = pipeline::inspect_entity_type(&cwd, "db_table")?;
                let configs = pipeline::inspect_entity_type(&cwd, "env_var_use")?;

                println!("Service: {service_name}");
                println!();
                println!("Routes");
                for r in routes {
                    println!("  {}", r.canonical_name);
                }
                println!();
                println!("Dependencies");
                for c in configs {
                    println!("  {}", c.canonical_name);
                }
                println!();
                println!("Tables");
                for t in tables {
                    println!("  {}", t.canonical_name);
                }
            } else {
                let entities = pipeline::inspect_entity_type(&cwd, &kind)?;
                if entities.is_empty() {
                    println!("No entities found for type '{}'.", kind);
                } else {
                    println!("Entities of type '{}':", kind);
                    for e in entities {
                        println!("- {} ({})", e.canonical_name, e.id);
                    }
                }
            }
        }
        Commands::Query { text } => {
            let relics = pipeline::query(&cwd, &text)?;
            if relics.is_empty() {
                println!("No relics matched '{}'.", text);
            } else {
                println!("Relic results for '{}':", text);
                for c in relics {
                    println!("- {} :: {}", c.label, c.theme);
                }
            }
        }
        Commands::Explore => {
            cheugy_tui::explorer::run(&cwd)?;
        }
    }

    Ok(())
}
