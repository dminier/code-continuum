use std::env;
use tracing::{debug, error, info};

/// Teste la connectivité avec Neo4j et retourne un résultat
pub async fn test_connection() -> Result<(), String> {
    let uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());

    debug!(uri = %uri, "Testing Neo4j connection");

    match neo4rs::Graph::new(&uri, &user, &password).await {
        Ok(graph) => {
            // Test simple: exécuter une requête Cypher basique
            match graph.run(neo4rs::query("RETURN 1 as test")).await {
                Ok(_) => {
                    info!("Neo4j connection successful");
                    Ok(())
                }
                Err(e) => {
                    let msg = format!(
                        "Échec du test de requête Neo4j: {} (URI: {}, user: {})",
                        e, uri, user
                    );
                    error!("{}", msg);
                    Err(msg)
                }
            }
        }
        Err(e) => {
            let msg = format!(
                "Impossible de se connecter à Neo4j: {} (URI: {}, user: {}, password: ***)\nVérifiez que Neo4j est en cours d'exécution et les paramètres de connexion.",
                e, uri, user
            );
            error!("{}", msg);
            Err(msg)
        }
    }
}
