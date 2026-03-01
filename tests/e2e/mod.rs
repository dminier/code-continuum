//! # Tests End-to-End (E2E)
//!
//! Ces tests valident le pipeline complet :
//! **Fichier source → Parsing → Graph sémantique → Export Neo4j → Requête Cypher**
//!
//! ## Prérequis
//! - Neo4j doit être accessible (docker-compose up -d neo4j)
//! - Les fixtures sont dans `examples/`
//!
//! ## Structure d'un test E2E
//!
//! ```rust
//! #[tokio::test]
//! async fn test_e2e_nom_du_cas() {
//!     // 1. Setup : Connexion Neo4j + nettoyage
//!     let graph = setup_neo4j().await;
//!     cleanup_test_data(&graph).await;
//!
//!     // 2. Arrange : Charger la fixture depuis examples/
//!     let fixture = PathBuf::from("examples/backend/java/ServiceA.java");
//!
//!     // 3. Act : Exécuter le pipeline complet
//!     analyze_and_export(&fixture, &graph).await;
//!
//!     // 4. Assert : Vérifier via requête Cypher
//!     let result = query_cypher(&graph, "MATCH (n:Class) RETURN n.name").await;
//!     assert!(result.contains("ServiceA"));
//! }
//! ```
//!
//! ## Exécution
//!
//! ```bash
//! # Tous les tests E2E
//! cargo test --test integration_e2e
//!
//! # Un test spécifique
//! cargo test --test integration_e2e test_e2e_java_class_extraction
//! ```

// Modules de tests E2E
pub mod jsp_transitive_includes;
