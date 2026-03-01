//! # Tests End-to-End MCP
//!
//! Pipeline complet via le serveur MCP HTTP :
//! **HTTP POST → MCP handler → analyse → Neo4j → vérification Cypher**
//!
//! ## Prérequis
//! - Neo4j accessible (docker-compose up -d neo4j)
//! - Fixtures présentes sous `examples/`
//!
//! ## Exécution
//! ```bash
//! cargo test --test integration_mcp -- --ignored --nocapture
//! ```

pub mod e2e_mcp;
