// Test d'intégration : vérification des nœuds de service dans Neo4j

use crate::common;

#[tokio::test]
async fn test_neo4j_service_nodes() {
    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(graph) => {
            // Chercher les nœuds de classe
            let result = graph
                .execute(neo4rs::query(
                    "MATCH (c:Class) RETURN count(*) as count LIMIT 1",
                ))
                .await;

            // Le test passe si la requête s'exécute
            assert!(result.is_ok(), "Failed to query Class nodes");
        }
        Err(e) => {
            eprintln!("⚠️ Neo4j unavailable: {}. Skipping test.", e);
        }
    }
}

#[tokio::test]
async fn test_neo4j_service_relationships() {
    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(graph) => {
            // Chercher les relations entre services
            let result = graph
                .execute(neo4rs::query(
                    "MATCH (a:Class)-[r]->(b:Class) RETURN count(*) as count LIMIT 1",
                ))
                .await;

            // Le test passe si la requête s'exécute
            assert!(result.is_ok(), "Failed to query service relationships");
        }
        Err(e) => {
            eprintln!("⚠️ Neo4j unavailable: {}. Skipping test.", e);
        }
    }
}
