use std::path::Path;
use tracing::error;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod analysis;
mod cli;
mod config;
mod encoding;
mod file_discovery;
mod graph_builder;
mod mcp;
mod neo4j_connectivity;
mod reporting;
mod semantic_graph;
mod ui;

use analysis::executor;
use config::PackageFilter;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Mode MCP HTTP
    if args.contains(&"--mcp".to_string()) {
        // Logs sur stderr pour ne pas interférer avec stdout du serveur HTTP
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"));
        let console_layer = fmt::layer()
            .with_target(false)
            .with_ansi(false)
            .with_writer(std::io::stderr);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .init();

        mcp::run_mcp_server().await;
        return;
    }

    // Initialiser la sortie logs: console + fichier .output/app.log
    let _ = std::fs::create_dir_all(".output");

    let file_appender = tracing_appender::rolling::daily(".output", "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let console_layer = fmt::layer().with_target(true).with_ansi(true);
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    // ASCII Art Banner
    println!(
        "\n{}",
        r#"
    CODE CONTINUUM 

    Semantic Code Analysis & Knowledge Graph Extraction
    Powered by Tree-Sitter + Neo4j + Rust

        ┌─────────────────────┐
        │ EnhanceUnderstanding│
        └──────────┬──────────┘
                   │ USES
                   ▼
        ┌─────────────────────┐
        │ Semantic Code Graph │
        └──────────┬──────────┘
                   │ ENABLES
                   ▼
        ┌─────────────────────┐
        │ TrackMigration      │──────▶ Function-level Traceability
        └──────────┬──────────┘
                   │
                   ▼
        ┌─────────────────────┐
        │ Annotate            │──────▶ Migration Metadata
        └─────────────────────┘

            ╔═══════════════════════════════════════════════╗
            ║  Supported Languages:                         ║
            ║  • Java                                       ║
            ║  • JavaScript/TypeScript                      ║
            ║  • Rust                                       ║
            ║  • JSP                                        ║
            ║  • Websphere Portal                           ║
            ║  • And More ...                               ║                   
            ╚═══════════════════════════════════════════════╝
"#
    );
    let args = match cli::parse_args() {
        Ok(args) => args,
        Err(e) => {
            error!("{}", e);
            eprintln!(
                "Usage: {} <chemin_repertoire>",
                std::env::args().next().unwrap_or_default()
            );
            std::process::exit(1);
        }
    };

    // Valider le chemin
    let path = Path::new(&args.source_directory);
    if let Err(e) = cli::validate_path(path) {
        error!("{}", e);
        std::process::exit(1);
    }

    // Vérifier la connectivité à Neo4j
    if let Err(e) = neo4j_connectivity::test_connection().await {
        error!("Erreur critique: {}", e);
        eprintln!("ERREUR: Impossible de se connecter à Neo4j. Veuillez vérifier:");
        eprintln!("  - Neo4j est en cours d'exécution");
        eprintln!(
            "  - Variables d'environnement configurées (NEO4J_URI, NEO4J_USER, NEO4J_PASSWORD)"
        );
        eprintln!("  - Les identifiants sont corrects");
        std::process::exit(1);
    }

    // Charger le filtre personnalisé si disponible via env var
    let filter = if let Ok(include_packages) = std::env::var("INCLUDE_PACKAGES") {
        let patterns: Vec<String> = include_packages
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        PackageFilter::with_patterns(patterns, vec![], true)
    } else {
        PackageFilter::default()
    };

    // Analyser le répertoire avec le filtre
    executor::analyze_repository_with_filter(path, Some(filter)).await;
}
