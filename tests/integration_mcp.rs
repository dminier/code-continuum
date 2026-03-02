// ============================================================
// Integration Tests - MCP (Model Context Protocol)
// ============================================================
// Tests E2E du serveur MCP HTTP :
//   add_project → Neo4j → vérification Cypher
//   remove_project → Neo4j → vérification absence
//
// Prérequis: Neo4j accessible (docker-compose up -d neo4j)
//
// Exécution:
//   cargo test --test integration_mcp -- --ignored --nocapture

#[path = "common/mod.rs"]
mod common;

#[path = "mcp/mod.rs"]
mod mcp_tests;
