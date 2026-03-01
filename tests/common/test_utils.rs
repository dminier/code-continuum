//! Utilitaires de test pour Neo4j
#![allow(dead_code)]

use neo4rs::{query, Graph, Row};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Crée et retourne une connexion à la base de données.
pub async fn get_graph() -> Option<Graph> {
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let pass = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());

    match Graph::new(&uri, &user, &pass).await {
        Ok(g) => Some(g),
        Err(e) => {
            eprintln!("⚠️ Neo4j unavailable: {}. Test will be skipped.", e);
            None
        }
    }
}

/// Nettoie la base de données par lots pour éviter les erreurs de mémoire.
pub async fn clear_db_with_graph(graph: &Graph) {
    loop {
        let mut result = graph
            .execute(query(
                "CALL { MATCH (n) RETURN n LIMIT 10000 } DETACH DELETE n RETURN count(n) AS deleted_count",
            ))
            .await
            .expect("Failed to execute batch delete query");

        let row = result
            .next()
            .await
            .expect("Failed to get next row from batch delete result")
            .expect("Batch delete result stream was empty");

        let deleted_count: i64 = row
            .get("deleted_count")
            .expect("Failed to get deleted_count from row");

        if deleted_count == 0 {
            break;
        }
    }
}

/// Exécute une requête Cypher et retourne les résultats sous forme de vecteur de lignes.
pub async fn execute_cypher_query_with_graph(
    graph: &Graph,
    query_str: &str,
) -> Result<Vec<Row>, neo4rs::Error> {
    let q = query(query_str);
    let mut result_stream = graph.execute(q).await?;
    let mut rows = Vec::new();
    while let Some(row) = result_stream.next().await? {
        rows.push(row);
    }
    Ok(rows)
}

/// Vide complètement la base de données Neo4j.
/// Retourne false si Neo4j n'est pas disponible.
pub async fn clear_db() -> bool {
    if let Some(graph) = get_graph().await {
        clear_db_with_graph(&graph).await;
        true
    } else {
        false
    }
}

/// Exécute une requête Cypher.
/// Retourne None si Neo4j n'est pas disponible.
pub async fn execute_cypher_query(
    query_str: &str,
    _params: Option<&[(&str, &str)]>,
) -> Option<Vec<Row>> {
    let graph = get_graph().await?;
    match execute_cypher_query_with_graph(&graph, query_str).await {
        Ok(rows) => Some(rows),
        Err(e) => {
            eprintln!("⚠️ Cypher query failed: {}", e);
            None
        }
    }
}

// Pour rétro-compatibilité avec les anciens tests
#[allow(dead_code)]
pub async fn clear_db_internal(graph: Arc<Mutex<Graph>>) {
    let g = graph.lock().await;
    clear_db_with_graph(&g).await;
}
