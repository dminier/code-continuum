use std::path::Path;
use tracing::{debug, error, info};

use super::{AnalysisReport, FileError, ParseError};
use crate::config::PackageFilter;
use crate::encoding::read_text_with_encoding_detection;
use crate::graph_builder::dsl_executor::dependency_resolver::DependencyResolver;
use crate::graph_builder::MultiLanguageGraphBuilder;
use crate::semantic_graph::{dsl::DslRegistry, Neo4jExporter, UnifiedGraph};

/// Détecte la langue d'un fichier basé sur son extension
pub fn detect_language(file_path: &Path) -> Option<&'static str> {
    DslRegistry::detect_language_from_path(file_path)
}

/// Analyse un fichier individuel et ajoute ses nœuds au graphe
pub fn analyze_file(
    builder: &MultiLanguageGraphBuilder,
    unified_graph: &mut UnifiedGraph,
    file_path: &Path,
    _root_dir: &Path,
    report: &mut AnalysisReport,
) {
    // Déterminer la langue
    let language = match detect_language(file_path) {
        Some(lang) => lang,
        None => {
            error!(file = %file_path.display(), "Impossible de déterminer la langue");
            return;
        }
    };

    // Lire le fichier en détectant l'encodage
    let source_code = match read_text_with_encoding_detection(file_path) {
        Ok(content) => content,
        Err(e) => {
            error!(file = %file_path.display(), error = %e, "Erreur de lecture du fichier");
            report.read_errors.push(FileError {
                file: file_path.display().to_string(),
                error: e.to_string(),
            });
            return;
        }
    };

    debug!(file = %file_path.display(), language = language, "Analyzing file");

    // Obtenir le parser Tree-sitter approprié
    let tree_sitter_language = match DslRegistry::get_tree_sitter_language(language) {
        Some(lang) => lang,
        None => {
            error!(language = language, "Langage non supporté");
            return;
        }
    };

    // Construire le graphe pour ce fichier avec V2
    // Utiliser le chemin absolu canonicalisé pour garantir l'unicité
    let file_path_str = file_path
        .canonicalize()
        .unwrap_or_else(|_| file_path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");

    match builder.build_graph(language, tree_sitter_language, &source_code, &file_path_str) {
        Ok(file_graph) => {
            debug!(
                file = %file_path.display(),
                count = file_graph.nodes.len(),
                edges = file_graph.edges.len(),
                "Nœuds extraits"
            );
            // Ajouter tous les nœuds au graphe unifié
            for node in file_graph.nodes.values() {
                unified_graph.add_node(node.clone());
            }
            // Ajouter toutes les arêtes
            for edge in file_graph.edges {
                unified_graph.add_edge(edge);
            }
        }
        Err(e) => {
            error!(
                file = %file_path.display(),
                language = language,
                error = %e,
                "Erreur lors de l'analyse"
            );
            report.parse_errors.push(ParseError {
                file: file_path.display().to_string(),
                language: language.to_string(),
                error: e.to_string(),
            });
        }
    }
}

/// Orchestre l'analyse complète du dépôt
pub async fn analyze_repository(path: &Path) {
    analyze_repository_with_filter(path, None).await;
}

/// Orchestre l'analyse complète du dépôt avec un filtre personnalisé
pub async fn analyze_repository_with_filter(path: &Path, filter: Option<PackageFilter>) {
    debug!(directory = ?path, "Scanning repository");

    // Collecter tous les fichiers source
    let mut source_files = Vec::new();
    let mut unsupported_files = Vec::new();
    crate::file_discovery::collect_source_files(path, &mut source_files, &mut unsupported_files);

    info!(count = source_files.len(), directory = ?path, "Found source files");

    if source_files.is_empty() {
        info!("Aucun fichier source trouvé dans le répertoire");
    }

    // Trier les fichiers pour traiter les JSP/JSPX/JSPF en dernier
    // Cela évite de créer des nœuds JS fantômes avant que les vrais fichiers JS soient parsés
    source_files.sort_by(|a, b| {
        let a_is_jsp = a.extension().map_or(false, |ext| {
            matches!(ext.to_str(), Some("jsp") | Some("jspx") | Some("jspf"))
        });
        let b_is_jsp = b.extension().map_or(false, |ext| {
            matches!(ext.to_str(), Some("jsp") | Some("jspx") | Some("jspf"))
        });

        match (a_is_jsp, b_is_jsp) {
            (true, false) => std::cmp::Ordering::Greater, // JSP après non-JSP
            (false, true) => std::cmp::Ordering::Less,    // non-JSP avant JSP
            _ => a.cmp(b),                                // Ordre alphabétique sinon
        }
    });

    // Afficher les fichiers trouvés (détails en debug)
    debug!("Fichiers à analyser:");
    for file in &source_files {
        debug!(file = %file.display(), "Fichier source");
    }

    // Créer le builder et le graphe unifié
    let builder = MultiLanguageGraphBuilder::new();
    let mut unified_graph = UnifiedGraph::new();

    // Initialiser le rapport
    let mut report = AnalysisReport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        directory: path.display().to_string(),
        supported_languages: crate::graph_builder::dsl_graph::supported_languages(),
        processed_files: source_files.len(),
        unsupported_files: unsupported_files
            .into_iter()
            .map(|p| p.display().to_string())
            .collect(),
        ..Default::default()
    };

    info!("Starting code analysis");
    crate::ui::phase_start("Code Analysis");

    // Traiter chaque fichier
    for (idx, file_path) in source_files.iter().enumerate() {
        analyze_file(&builder, &mut unified_graph, file_path, path, &mut report);

        // Afficher la progression tous les 10 fichiers ou au début/fin
        crate::ui::show_progress_stepped(idx + 1, source_files.len(), "Analyzing files", 10);
    }

    crate::ui::phase_complete("Code Analysis");
    // Initialiser le resolver de dépendances avec les classes locales indexées
    debug!("Indexing classes for dependency resolution");
    let mut resolver = if let Some(f) = filter {
        DependencyResolver::with_filter(f)
    } else {
        DependencyResolver::new()
    };
    crate::graph_builder::DslExecutor::register_local_classes(&mut resolver, &unified_graph);

    // Résolution globale des imports (création d'arêtes IMPORTS et nœuds fantômes)
    debug!("Resolving IMPORTS relationships");
    crate::graph_builder::DslExecutor::resolve_imports_global(&mut unified_graph, &resolver);

    // Résolution globale des relations EXTENDS et IMPLEMENTS
    debug!("Resolving EXTENDS/IMPLEMENTS relationships");
    crate::graph_builder::DslExecutor::resolve_extends_implements_global(
        &mut unified_graph,
        &resolver,
    );

    // Résolution globale des relations CALLS après avoir tous les fichiers
    debug!("Resolving CALLS relationships");
    crate::graph_builder::DslExecutor::resolve_calls_global(&mut unified_graph, &resolver);
    // Afficher le résumé
    info!("Graph analysis complete");
    unified_graph.print_summary();

    debug!("=== Fonctions détectées ===");
    for func in unified_graph.find_functions() {
        let language = func
            .metadata
            .get("language")
            .unwrap_or(&"unknown".to_string())
            .clone();
        debug!(
            name = %func.name,
            language = %language,
            line = func.location.start_line,
            "Fonction"
        );
    }

    // Écrire le rapport dans .output/report.json
    if let Err(e) = crate::reporting::write_report(&report) {
        error!(error = %e, "Impossible d'écrire le rapport .output/report.json");
    } else {
        debug!("Report written to .output/report.json");
    }

    // Exporter le graphe vers Neo4j
    info!("Exporting to Neo4j");
    match Neo4jExporter::new().await {
        Ok(exporter) => match exporter.export_graph(&unified_graph).await {
            Ok(_) => {
                info!("Graph exported to Neo4j successfully");
            }
            Err(e) => {
                error!(error = %e, "Erreur lors de l'export du graphe vers Neo4j");
            }
        },
        Err(e) => {
            error!(error = %e, "Impossible de créer la connexion à Neo4j pour l'export");
        }
    }
}
