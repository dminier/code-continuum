# GEMINI.MD: AI Collaboration Guide

This document provides essential context for AI models interacting with this project. Adhering to these guidelines will ensure consistency and maintain code quality.

## 1. Project Overview & Purpose

- **Primary Goal:** Static analysis tool that parses multi-language code (Java, JavaScript/TypeScript, JSP/WebSphere XML,...) with Tree-Sitter, builds a semantic graph with qualified identifiers, and exports it to Neo4j; an MCP server exposes HTTP endpoints so AI agents can query the graph without re-running analysis.
- **Business Domain:** Developer tooling / code intelligence for dependency and call-graph analysis (RAG for code understanding).

## 2. Core Technologies & Stack

- **Languages:** Rust (edition 2021) for the app and MCP server; Tree-Sitter grammars for Java, JavaScript/TypeScript, Python, Rust, HTML; test fixtures in Java/JavaScript/JSP/XML.
- **Frameworks & Runtimes:** Tokio async runtime; tracing for structured logging; MCP server (custom Rust) for HTTP endpoints.
- **Databases:** Neo4j 5.x (bolt://7687, browser 7474) as the knowledge graph store.
- **Key Libraries/Dependencies:** tree-sitter and language packs, neo4rs for Neo4j, serde/serde_json, anyhow, regex, chrono, tracing + tracing-subscriber/appender.
- **Package Manager(s):** Cargo (Rust).

## 3. Architectural Patterns

- **Overall Architecture:** Monolithic Rust application with a pipeline: file discovery → AST extraction via Tree-Sitter → unified semantic graph → Neo4j export; optional MCP HTTP layer for agent access. Runs in a VS Code Dev Container with companion Neo4j and MCP services via docker-compose.
- **Directory Structure Philosophy:**
  - `/src`: Application code (CLI/main, analysis orchestrator, graph model/exporter, graph builder/extractors, Neo4j connectivity, MCP server, reporting, encoding utils).
  - `/tests`: Integration tests organized by feature (extraction, Neo4j, debug) plus shared helpers in `common/`.
  - `/doc`: French-first documentation (installation, commands, conception/architecture, schema, MCP server, deployment, tests, extractors).
  - `/examples`: Sample codebases (Java/JS, JSP, WebSphere portal configs) for analysis.
  - `/scripts`: Deployment helper scripts; docker-compose configs at root for production.

## 4. Coding Conventions & Style Guide

- **Formatting:** Use `cargo fmt`; keep code clippy-clean (`cargo clippy --all-targets`). Docs are in French; new docs go under `doc/` with UPPERCASE names and must be linked from `doc/README.md`.
- **Naming Conventions:** Rust defaults—snake_case for functions/variables/modules, PascalCase for types, SCREAMING_SNAKE_CASE for constants; qualified IDs for graph entities (`package.Class.method`).
- **API Design:** CLI entry via `src/main.rs`/`cli` module; MCP server exposes endpoints like `execute_cypher`, `search_nodes`, `find_calls`, `analyze_dependencies` under `/api/mcp/` (JSON over HTTP).
- **Error Handling:** Prefer `Result` with `?`; avoid `unwrap` in production paths; log meaningful errors before exit.
- **Logging:** Use `tracing` with structured fields; default `info` level for CLI, `warn` for MCP stdio mode; keep `info` logs minimal and actionable, `debug` for flow details, `trace` only for deep parser debugging. Logs go to console and `.output/app.log`.

## 5. Key Files & Entrypoints

- **Main Entrypoint(s):** `src/main.rs` (CLI + MCP switch), orchestrating `analysis::executor::analyze_repository` after Neo4j connectivity check.
- **Configuration:** `Cargo.toml` for dependencies; environment vars `NEO4J_URI`, `NEO4J_USER`, `NEO4J_PASSWORD` (defaults to bolt://localhost:7687, neo4j/password); Dev Container/docker-compose define Neo4j and MCP services; `.output/` for logs.
- **CI/CD Pipeline:** No CI config detected in repo; confirm before assuming automation.

## 6. Development & Testing Workflow

- **Local Development Environment:** Open in VS Code Dev Container; build with `cargo build` (or `--release` for speed/production); run analysis with `cargo run -- <path>` (examples under `examples/backend/`).
- **Testing:** `cargo test` runs unit + integration; category tests via `cargo test --test connection|calls|services|export|field_extraction|javascript|java_ast`; tests skip gracefully if Neo4j is unavailable. Use `-- --nocapture` for logs, `--test-threads=1` for sequential runs.
- **CI/CD Process:** Not defined; assume manual workflows (run fmt, clippy, test before PR).

## 7. 🔴🟢🔵 TDD Workflow (OBLIGATOIRE)

**Toute nouvelle fonctionnalité ou correction de bug DOIT suivre le cycle TDD Red-Green-Refactor.**

**🎯 PRIORITÉ : Tests E2E > Tests d'intégration > Tests unitaires**

Ce projet privilégie les **tests End-to-End (E2E)** car ils :
- Valident le pipeline complet : fichier source → parsing → graph → **export Neo4j** → requête Cypher
- Testent le comportement réel sur des fichiers concrets (`examples/`)
- Vérifient que les données sont correctement persistées et requêtables dans Neo4j
- Détectent les régressions sur toute la chaîne

**Structure des tests :**
```
tests/
├── e2e/                 # Tests End-to-End (PRIORITAIRE)
│   └── *.rs            # Pipeline complet avec Neo4j
├── extraction/          # Tests d'intégration parsing
├── neo4j/              # Tests connectivité Neo4j
└── common/             # Helpers partagés
```

Les tests unitaires restent utiles pour les fonctions pures et les utilitaires isolés.

### Étape 1: 🔴 RED — Écrire le test d'abord
1. **Comprendre le besoin** : Avant d'écrire du code, clarifier le comportement attendu
2. **Créer le fichier de test** :
   - Tests unitaires : dans le même fichier avec `#[cfg(test)] mod tests { ... }`
   - Tests d'intégration : dans `/tests/` selon la catégorie (extraction, neo4j, etc.)
3. **Écrire un test qui ÉCHOUE** : Le test doit compiler mais échouer à l'exécution
4. **Vérifier l'échec** : Exécuter `cargo test <nom_du_test>` et confirmer l'échec attendu

```bash
# Exemple: vérifier que le test échoue
cargo test test_nouvelle_fonctionnalite -- --nocapture
```

### Étape 2: 🟢 GREEN — Faire passer le test
1. **Implémenter le minimum** : Écrire UNIQUEMENT le code nécessaire pour faire passer le test
2. **Pas d'optimisation prématurée** : Le code peut être "laid" à cette étape
3. **Vérifier le succès** : `cargo test <nom_du_test>` doit passer

```bash
# Le test doit maintenant passer
cargo test test_nouvelle_fonctionnalite
```

### Étape 3: 🔵 REFACTOR — Améliorer le code
1. **Nettoyer** : Améliorer la lisibilité, supprimer les duplications
2. **Garder les tests verts** : Après chaque modification, relancer les tests
3. **Vérifier la qualité** : `cargo fmt && cargo clippy --all-targets`

```bash
# Cycle de refactoring
cargo fmt && cargo clippy --all-targets && cargo test
```

### 📁 Fixtures de Test : Le Dossier `examples/`

**RÈGLE ABSOLUE : Tous les tests d'intégration DOIVENT utiliser les fichiers du dossier `examples/`.**

Le dossier `examples/` est la **source unique de vérité** pour les fixtures de test. Il contient des exemples réalistes représentant les cas d'usage du projet.

#### Structure des Examples

```
examples/
├── backend/                    # Code serveur
│   ├── java/                   # Java: classes, services, héritage, appels statiques
│   └── javascript/             # JS backend: services, pipelines transitifs
├── frontend/                   # Code client
│   ├── html/                   # Templates HTML
│   └── javascript/             # JS frontend: composants, API calls
├── web_templates/              # Templates web dynamiques
│   ├── *.jsp, *.jspx          # JSP/JSPX templates
│   ├── WEB-INF/               # Configuration web Java
│   └── common/                # Includes partagés
├── config/                     # Fichiers de configuration
│   └── *.xml                  # XML (portlet, app config)
└── websphere-portal/          # Cas d'usage WebSphere Portal
    ├── java/                  # Code Java Portal
    └── *.xml                  # Configs Portal
```

#### Mapping Cas d'Usage → Fichiers Examples

| Cas d'Usage | Fichiers Examples | Tests Associés |
|-------------|-------------------|----------------|
| Extraction classes Java | `backend/java/*.java` | `extraction/java_*` |
| Héritage/interfaces | `backend/java/Base*.java, Derived*.java` | `extraction/inheritance` |
| Appels de méthodes | `backend/java/Service*.java` | `neo4j/calls` |
| Appels transitifs | `backend/java/TransitiveChain.java` | `neo4j/transitive` |
| Appels statiques | `backend/java/StaticCallsExample.java` | `extraction/static_calls` |
| JavaScript modules | `backend/javascript/*.js` | `extraction/javascript` |
| JSP/Templates | `web_templates/*.jsp` | `extraction/jsp` |
| Config XML | `config/*.xml, websphere-portal/*.xml` | `extraction/xml` |

#### Règles pour les Fixtures

1. **Pas de fixtures dans `/tests/`** : Jamais de code source de test dans le dossier tests
2. **Examples = Documentation vivante** : Les examples doivent être du code valide et lisible
3. **Un fichier = Un concept** : Chaque fichier example doit illustrer un concept précis
4. **Nommage explicite** : `TransitiveChain.java`, `StaticCallsExample.java`, etc.

#### Template de Test avec Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Test pour [DESCRIPTION DU COMPORTEMENT ATTENDU]
    /// 
    /// Fixture: examples/backend/java/ServiceA.java
    /// Scénario:
    /// - Given: [CONDITIONS INITIALES]
    /// - When: [ACTION TESTÉE]
    /// - Then: [RÉSULTAT ATTENDU]
    #[test]
    fn test_nom_descriptif() {
        // Arrange (Given) - Utiliser TOUJOURS examples/
        let fixture_path = PathBuf::from("examples/backend/java/ServiceA.java");
        assert!(fixture_path.exists(), "Fixture manquante: {:?}", fixture_path);
        
        // Act (When)
        let result = fonction_a_tester(&fixture_path);
        
        // Assert (Then)
        assert_eq!(result, expected);
    }
}
```

### Règles TDD Strictes pour l'IA

| ❌ INTERDIT | ✅ OBLIGATOIRE |
|------------|---------------|
| Écrire du code sans test | Écrire le test AVANT l'implémentation |
| Modifier du code existant sans test de régression | Ajouter un test qui capture le bug avant de corriger |
| Implémenter plusieurs fonctionnalités à la fois | Un test = une fonctionnalité atomique |
| Refactorer sans tests verts | Toujours avoir une suite de tests passante |
| Supprimer des tests pour "faire passer" | Les tests définissent le contrat, pas l'inverse |

### Workflow TDD pour Corrections de Bug

1. **Reproduire** : Écrire un test qui reproduit le bug (🔴 RED)
2. **Confirmer** : Le test doit échouer de la même manière que le bug
3. **Corriger** : Implémenter la correction minimale (🟢 GREEN)
4. **Valider** : Le test passe, le bug est corrigé
5. **Régresser** : Le test protège contre les régressions futures

### Checklist Avant Commit

```bash
# 1. Tous les tests passent
cargo test

# 2. Code formaté
cargo fmt --check

# 3. Pas de warnings clippy
cargo clippy --all-targets -- -D warnings

# 4. Documentation à jour (si changement de schéma)
# Vérifier SCHEMA.md et SCHEMA_IA.md
```

## 8. Specific Instructions for AI Collaboration

- **Contribution Guidelines:** Run `cargo fmt`, `cargo clippy --all-targets`, and `cargo test` before submitting changes. Keep logging structured and concise. 
  
  **⚠️ SCHEMA MAINTENANCE CRITICAL:** Any change to Neo4j schema (node types, relations, properties) requires synchronized updates across THREE documents:
  1. **Code source:** `src/code_continuum/semantic_graph.rs` (NodeKind, EdgeRelation enums) and `src/code_continuum/neo4j_exporter.rs` (Cypher export logic)
  2. **Human documentation:** `doc/SCHEMA.md` (complete, detailed reference with examples)
  3. **AI documentation:** `doc/SCHEMA_IA.md` (compact, patterns-focused reference for agent queries)
  
  All three must remain in sync. When any one changes, update the others immediately in the same commit.

- **Infrastructure (IaC):** Docker and docker-compose files affect runtime services (Neo4j, MCP); coordinate changes carefully. Dev Container is the expected environment.
- **Security:** Do not commit secrets; Neo4j defaults (neo4j/password) are for dev only. Validate authentication/connection handling when touching MCP or Neo4j code.
- **Dependencies:** Add Rust deps via `Cargo.toml`/`cargo add`; ensure reproducibility and review impact on parsing or graph schema.
- **Commit Messages:** No explicit policy observed; using clear, concise messages (or Conventional Commits) is recommended until a policy is stated.
