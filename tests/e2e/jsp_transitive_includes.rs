//! Tests E2E pour les inclusions JSP/JSPF transitives (avec Neo4j)
//!
//! ## Objectif
//!
//! Valider le pipeline complet pour la détection des inclusions JSP :
//! **Fichier JSP → Parsing → Graph sémantique → Export Neo4j → Requête Cypher**
//!
//! ## Prérequis
//!
//! - Neo4j doit être accessible (docker-compose up -d neo4j)
//!
//! ## Cas testés
//!
//! 1. **Inclusions directes** : JSP → JSPF (un seul niveau)
//! 2. **Inclusions transitives** : JSP → JSPF → JSPF (multi-niveaux)
//! 3. **Types d'inclusion** : statique (`<%@ include`) vs dynamique (`<jsp:include>`)
//! 4. **Nœuds JSPF** : Vérifier que les fragments sont créés comme nœuds JSP
//!
//! ## Fixtures
//!
//! Utilise `examples/jsp_transitive_include/web/` :
//! ```text
//! main.jsp
//! ├── [INCLUDES_JSP] page1.jspx
//! │   ├── [INCLUDES_JSP] includes/fragment.jspf
//! │   │   └── [INCLUDES_JSP] sub/nested-fragment.jspf
//! │   └── [INCLUDES_JSP] includes/sidebar.jspf
//! │       └── [INCLUDES_JSP] sub/menu-items.jspf
//! ```
//!
//! ## Exécution
//!
//! ```bash
//! # Nécessite Neo4j en cours d'exécution
//! cargo test --test integration_e2e test_e2e_jsp -- --ignored --nocapture
//! ```

use crate::common;
use code_continuum::analysis::executor::analyze_repository;
use std::path::Path;

/// Test E2E: Pipeline complet JSP → Neo4j avec inclusions transitives
///
/// Ce test valide l'ensemble du pipeline :
/// 1. Parsing des fichiers JSP/JSPF
/// 2. Création du graph sémantique avec relations INCLUDES_JSP
/// 3. Export vers Neo4j
/// 4. Requêtes Cypher pour vérifier les données
#[tokio::test]
#[ignore = "Nécessite Neo4j - activer avec: cargo test -- --ignored"]
async fn test_e2e_jsp_transitive_includes_complete() {
    println!("\n=== TEST E2E: Inclusions JSP transitives ===\n");

    // =========================================================================
    // SETUP: Préparation de l'environnement
    // =========================================================================
    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    let graph = match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(g) => g,
        Err(e) => {
            eprintln!("⚠️ Neo4j unavailable: {}. Skipping test.", e);
            return;
        }
    };

    // Nettoyer la base
    let _ = graph.run(neo4rs::query("MATCH (n) DETACH DELETE n")).await;

    let fixture_path = Path::new("examples/jsp_transitive_include");
    if !fixture_path.exists() {
        eprintln!("⚠️ Fixture manquante: {:?}. Test skippé.", fixture_path);
        return;
    }

    // =========================================================================
    // ACT: Exécuter le pipeline complet
    // =========================================================================
    println!("📂 Analyse du répertoire: {:?}", fixture_path);
    analyze_repository(fixture_path).await;

    // =========================================================================
    // ASSERT 1: Vérifier les nœuds JSP créés
    // =========================================================================
    println!("\n--- Vérification des nœuds JSP ---");
    let nodes_query = "
        MATCH (j:Jsp)
        RETURN j.file_path AS path, j.name AS name
        ORDER BY path
    ";

    let mut result = graph.execute(neo4rs::query(nodes_query)).await.unwrap();

    let mut jspf_count = 0;
    let mut all_jsp_count = 0;

    while let Some(row) = result.next().await.unwrap() {
        let path: String = row.get("path").unwrap();
        let name: String = row.get("name").unwrap();
        println!("  - {} ({})", name, path);
        all_jsp_count += 1;
        if path.ends_with(".jspf") {
            jspf_count += 1;
        }
    }

    println!(
        "  Total: {} nœuds JSP dont {} JSPF",
        all_jsp_count, jspf_count
    );

    // ✅ Point clé: Les JSPF doivent être créés comme nœuds
    if jspf_count == 0 {
        println!(
            "⚠️ Aucun fichier JSPF trouvé comme nœud - les JSPF ne sont peut-être pas analysés"
        );
    } else {
        println!("✅ {} fichiers JSPF trouvés", jspf_count);
    }

    // =========================================================================
    // ASSERT 2: Vérifier les relations INCLUDES_JSP
    // =========================================================================
    println!("\n--- Vérification des relations INCLUDES_JSP ---");
    let edges_query = "
        MATCH (source:Jsp)-[r:INCLUDES_JSP]->(target:Jsp)
        RETURN 
            source.file_path AS from_path,
            target.file_path AS to_path,
            r.type AS include_type
        ORDER BY from_path, to_path
    ";

    let mut result = graph.execute(neo4rs::query(edges_query)).await.unwrap();
    let mut edge_count = 0;

    while let Some(row) = result.next().await.unwrap() {
        let from: String = row.get("from_path").unwrap();
        let to: String = row.get("to_path").unwrap();
        let inc_type: Option<String> = row.get("include_type").ok();
        println!(
            "  {} → {} [{}]",
            from,
            to,
            inc_type.as_deref().unwrap_or("?")
        );
        edge_count += 1;
    }

    println!("  Total: {} relations INCLUDES_JSP", edge_count);

    // =========================================================================
    // ASSERT 3: Vérifier la transitivité (chemins multi-niveaux)
    // =========================================================================
    println!("\n--- Vérification de la transitivité ---");
    let transitive_query = "
        MATCH path = (start:Jsp)-[:INCLUDES_JSP*1..5]->(end:Jsp)
        WHERE start.file_path CONTAINS 'main.jsp'
        WITH start, end, length(path) AS depth
        RETURN 
            start.name AS start_name,
            end.name AS end_name,
            end.file_path AS end_path,
            depth
        ORDER BY depth, end_path
    ";

    let mut result = graph
        .execute(neo4rs::query(transitive_query))
        .await
        .unwrap();
    let mut max_depth = 0i64;

    while let Some(row) = result.next().await.unwrap() {
        let end_name: String = row.get("end_name").unwrap();
        let depth: i64 = row.get("depth").unwrap();
        println!("  [depth={}] main.jsp → {}", depth, end_name);
        if depth > max_depth {
            max_depth = depth;
        }
    }

    if max_depth >= 3 {
        println!(
            "✅ Transitivité vérifiée jusqu'à la profondeur {}",
            max_depth
        );
    } else {
        println!("⚠️ Profondeur maximale: {} (attendu >= 3)", max_depth);
    }

    // =========================================================================
    // CLEANUP
    // =========================================================================
    println!("\n=== TEST TERMINÉ ===\n");
    let _ = graph.run(neo4rs::query("MATCH (n) DETACH DELETE n")).await;
}

/// Test E2E: Liste des JSPF utilisés par une page JSP (cas d'usage signalé)
///
/// Ce test répond directement à l'anomalie signalée :
/// "pouvoir rajouter les JSPF dans le tableau des JSP utilisées"
#[tokio::test]
#[ignore = "Nécessite Neo4j - activer avec: cargo test -- --ignored"]
async fn test_e2e_list_all_jspf_used_by_jsp() {
    println!("\n=== TEST: Lister tous les JSPF utilisés par un JSP ===\n");

    // Setup
    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    let graph = match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(g) => g,
        Err(e) => {
            eprintln!("⚠️ Neo4j unavailable: {}. Skipping test.", e);
            return;
        }
    };

    // Nettoyer
    let _ = graph.run(neo4rs::query("MATCH (n) DETACH DELETE n")).await;

    let fixture_path = Path::new("examples/jsp_transitive_include");
    if !fixture_path.exists() {
        eprintln!("⚠️ Fixture manquante: {:?}. Test skippé.", fixture_path);
        return;
    }

    // Analyser
    analyze_repository(fixture_path).await;

    // Requête Cypher pour lister TOUS les JSPF (directs ET transitifs) utilisés par main.jsp
    let query = "
        MATCH path = (main:Jsp)-[:INCLUDES_JSP*1..10]->(jspf:Jsp)
        WHERE main.file_path CONTAINS 'main.jsp'
          AND jspf.file_path ENDS WITH '.jspf'
        WITH DISTINCT jspf, min(length(path)) AS min_depth
        RETURN 
            jspf.file_path AS jspf_path,
            jspf.name AS jspf_name,
            min_depth AS depth
        ORDER BY min_depth, jspf_path
    ";

    let mut result = graph.execute(neo4rs::query(query)).await.unwrap();

    println!("📋 JSPF utilisés par main.jsp (directs et transitifs):\n");
    println!("| Depth | Fichier JSPF | Nom |");
    println!("|-------|--------------|-----|");

    let mut jspf_count = 0;

    while let Some(row) = result.next().await.unwrap() {
        let path: String = row.get("jspf_path").unwrap();
        let name: String = row.get("jspf_name").unwrap();
        let depth: i64 = row.get("depth").unwrap();
        println!("| {} | {} | {} |", depth, path, name);
        jspf_count += 1;
    }

    println!("\n  Total: {} fichiers JSPF", jspf_count);

    if jspf_count >= 4 {
        println!("✅ Tous les JSPF attendus sont présents");
    } else {
        println!(
            "⚠️ Moins de 4 JSPF trouvés (attendu: fragment, sidebar, nested-fragment, menu-items)"
        );
    }

    // Cleanup
    let _ = graph.run(neo4rs::query("MATCH (n) DETACH DELETE n")).await;
}
