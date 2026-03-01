// Module d'utilitaires partagés pour les tests d'intégration
#![allow(dead_code)]

use std::env;

// Module avec les utilitaires de test Neo4j
pub mod test_utils;

/// Configure les variables d'environnement pour les tests
pub fn setup_env() {
    if env::var("NEO4J_URI").is_err() {
        env::set_var("NEO4J_URI", "bolt://localhost:7687");
    }
    if env::var("NEO4J_USER").is_err() {
        env::set_var("NEO4J_USER", "neo4j");
    }
    if env::var("NEO4J_PASSWORD").is_err() {
        env::set_var("NEO4J_PASSWORD", "password");
    }
}

/// Récupère la configuration Neo4j depuis les variables d'environnement
pub fn get_neo4j_config() -> (String, String, String) {
    let uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let pass = env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());
    (uri, user, pass)
}
