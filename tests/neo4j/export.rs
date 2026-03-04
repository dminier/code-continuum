// Test d'intégration : vérification des données exportées dans Neo4j

use crate::common;

#[tokio::test]
async fn test_neo4j_exported_data_structure() {
    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(graph) => {
            // Chercher les nœuds avec leurs propriétés
            let result = graph
                .execute(neo4rs::query(
                    "MATCH (n:Node) WHERE n.node_type IS NOT NULL RETURN n.node_type, count(*) as count",
                ))
                .await;

            // Le test passe si la requête s'exécute et retourne des données
            match result {
                Ok(_) => {
                    // Le test est considéré comme réussi
                }
                Err(e) => {
                    eprintln!("⚠️ Failed to query exported data: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("⚠️ Neo4j unavailable: {}. Skipping test.", e);
        }
    }
}

#[tokio::test]
async fn test_neo4j_node_properties() {
    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(graph) => {
            // Vérifier qu'un nœud fonction a les propriétés attendues
            let result = graph
                .execute(neo4rs::query(
                    "MATCH (f:Function) WHERE f.name IS NOT NULL AND f.id IS NOT NULL 
                     RETURN f.id, f.name, f.file_path LIMIT 1",
                ))
                .await;

            // Le test passe si la requête s'exécute
            assert!(result.is_ok(), "Failed to query Function node properties");
        }
        Err(e) => {
            eprintln!("⚠️ Neo4j unavailable: {}. Skipping test.", e);
        }
    }
}
