// ============================================================
// Integration Tests - Neo4j
// ============================================================
// Ce fichier est un point d'entrée pour les tests d'intégration Neo4j
// Cargo va le découvrir automatiquement et lancer les tests qu'il contient

#[path = "common/mod.rs"]
mod common;

#[path = "neo4j/calls.rs"]
mod neo4j_calls;

#[path = "neo4j/connection.rs"]
mod neo4j_connection;

#[path = "neo4j/export.rs"]
mod neo4j_export;

#[path = "neo4j/services.rs"]
mod neo4j_services;

#[path = "neo4j/servlet_url_pattern_export.rs"]
mod neo4j_servlet_url_pattern_export;
