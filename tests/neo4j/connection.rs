// Test d'intégration : connexion à Neo4j

use crate::common;

#[tokio::test]
async fn test_neo4j_connection() {
    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    // Essayer de se connecter à Neo4j
    match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(graph) => {
            // Exécuter une requête de test simple
            let result = graph
                .execute(neo4rs::query(
                    "RETURN 'Hello from Neo4j!' AS message, datetime() AS timestamp",
                ))
                .await;

            // Le test passe si la connexion et la requête réussissent
            assert!(result.is_ok(), "Failed to execute query");
        }
        Err(e) => {
            eprintln!("⚠️ Neo4j unavailable: {}. Skipping test.", e);
            // On ne failé pas le test si Neo4j n'est pas disponible
        }
    }
}
