//! # Tests End-to-End (E2E)
//!
//! Pipeline complet : fichier source → parsing → graph → Neo4j → requête Cypher
//!
//! Ces tests utilisent les fixtures du dossier `examples/` et vérifient
//! que les données sont correctement exportées et requêtables dans Neo4j.

mod common;
mod e2e;

use std::path::PathBuf;

/// Helper pour obtenir le chemin d'une fixture
fn fixture_path(relative: &str) -> PathBuf {
    let path = PathBuf::from("examples").join(relative);
    assert!(
        path.exists(),
        "Fixture manquante: {:?}. Tous les tests E2E doivent utiliser examples/",
        path
    );
    path
}

// =============================================================================
// TESTS E2E - Java
// =============================================================================

#[cfg(test)]
mod e2e_java {
    use super::*;

    /// E2E: Extraction d'une classe Java simple et vérification dans Neo4j
    ///
    /// Fixture: examples/backend/java/User.java
    /// Pipeline: Parse → Graph → Neo4j → Cypher query
    #[test]
    #[ignore = "Nécessite Neo4j - activer avec: cargo test --ignored"]
    fn test_e2e_java_simple_class() {
        let _fixture = fixture_path("backend/java/User.java");
        // TODO: Implémenter le test E2E complet
        // 1. Parser le fichier
        // 2. Construire le graph
        // 3. Exporter vers Neo4j
        // 4. Requêter et vérifier
    }

    /// E2E: Chaîne d'appels transitifs Java → Neo4j
    ///
    /// Fixture: examples/backend/java/TransitiveChain.java
    /// Vérifie: Les relations CALLS sont correctement exportées
    #[test]
    #[ignore = "Nécessite Neo4j - activer avec: cargo test --ignored"]
    fn test_e2e_java_transitive_calls() {
        let _fixture = fixture_path("backend/java/TransitiveChain.java");
        // TODO: Implémenter
        // Vérifier que MATCH (a)-[:CALLS*]->(b) retourne la chaîne complète
    }

    /// E2E: Héritage Java → Neo4j
    ///
    /// Fixtures: examples/backend/java/BaseClass.java, DerivedClass.java
    /// Vérifie: Les relations EXTENDS sont correctement exportées
    #[test]
    #[ignore = "Nécessite Neo4j - activer avec: cargo test --ignored"]
    fn test_e2e_java_inheritance() {
        let _base = fixture_path("backend/java/BaseClass.java");
        let _derived = fixture_path("backend/java/DerivedClass.java");
        // TODO: Implémenter
        // Vérifier MATCH (d:Class)-[:EXTENDS]->(b:Class) RETURN d.name, b.name
    }
}

// =============================================================================
// TESTS E2E - JavaScript
// =============================================================================

#[cfg(test)]
mod e2e_javascript {
    use super::*;

    /// E2E: Service JavaScript avec héritage → Neo4j
    ///
    /// Fixtures: examples/backend/javascript/BaseService.js, DerivedService.js
    #[test]
    #[ignore = "Nécessite Neo4j - activer avec: cargo test --ignored"]
    fn test_e2e_js_service_inheritance() {
        let _base = fixture_path("backend/javascript/BaseService.js");
        let _derived = fixture_path("backend/javascript/DerivedService.js");
        // TODO: Implémenter
    }

    /// E2E: Pipeline transitif JavaScript → Neo4j
    ///
    /// Fixture: examples/backend/javascript/transitive-pipeline.js
    #[test]
    #[ignore = "Nécessite Neo4j - activer avec: cargo test --ignored"]
    fn test_e2e_js_transitive_pipeline() {
        let _fixture = fixture_path("backend/javascript/transitive-pipeline.js");
        // TODO: Implémenter
    }
}

// =============================================================================
// TESTS E2E - Multi-fichiers / Projet complet
// =============================================================================

#[cfg(test)]
mod e2e_project {
    use super::*;

    /// E2E: Analyse d'un projet Java complet → Neo4j
    ///
    /// Fixture: examples/backend/java/ (tout le dossier)
    /// Vérifie: Toutes les classes et relations sont exportées
    #[test]
    #[ignore = "Nécessite Neo4j - activer avec: cargo test --ignored"]
    fn test_e2e_full_java_project() {
        let _project = fixture_path("backend/java");
        // TODO: Implémenter
        // Analyser tout le dossier et vérifier le graph complet
    }

    /// E2E: Analyse WebSphere Portal → Neo4j
    ///
    /// Fixture: examples/websphere-portal/
    #[test]
    #[ignore = "Nécessite Neo4j - activer avec: cargo test --ignored"]
    fn test_e2e_websphere_portal() {
        let _project = fixture_path("websphere-portal");
        // TODO: Implémenter
    }
}
