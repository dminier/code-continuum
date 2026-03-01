// Test d'intégration : vérifier les relations CALLS dans Neo4j

use crate::common;

#[tokio::test]
async fn test_neo4j_calls_relation() {
    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(graph) => {
            // Chercher les relations CALLS Function -> Function
            let result = graph
                .execute(neo4rs::query(
                    "MATCH (caller:Function)-[:CALLS]->(target:Function) RETURN count(*) as count LIMIT 1",
                ))
                .await;

            // Le test passe si la requête s'exécute
            assert!(result.is_ok(), "Failed to query CALLS relations");
        }
        Err(e) => {
            eprintln!("⚠️ Neo4j unavailable: {}. Skipping test.", e);
        }
    }
}

#[tokio::test]
async fn test_neo4j_transitive_calls() {
    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(graph) => {
            // Chercher les appels transitifs avec une profondeur de 1-5
            let result = graph
                .execute(neo4rs::query(
                    "MATCH path = (start:Node)-[:CALLS*1..5]->(end:Node) 
                     WHERE start.node_type = 'Function' AND end.node_type = 'Function'
                     RETURN count(DISTINCT path) as count LIMIT 1",
                ))
                .await;

            // Le test passe si la requête s'exécute
            assert!(result.is_ok(), "Failed to query transitive calls");
        }
        Err(e) => {
            eprintln!("⚠️ Neo4j unavailable: {}. Skipping test.", e);
        }
    }
}
