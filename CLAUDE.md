# CLAUDE.md — AI Collaboration Guide

This document provides all context an AI agent needs to work effectively on this project. Follow every guideline here to maintain code quality, test coverage, and schema consistency.

---

## 0. Session Quickstart

**Read this first at the start of every session.**

1. Read `doc/SCHEMA_IA.md` for the current Neo4j schema (compact Cypher reference).
2. Read `src/semantic_graph/semantic_graph.rs` if you need to add or modify node/edge types.
3. Run `cargo test` before making any changes to establish a green baseline.
4. Follow the TDD workflow (Section 5) for every code change — no exceptions.

**Critical files to know:**

| File | Why it matters |
|---|---|
| `src/semantic_graph/semantic_graph.rs` | Source of truth for all node/edge types |
| `src/semantic_graph/neo4j_exporter.rs` | Cypher generation — always in sync with above |
| `doc/SCHEMA.md` | Human-readable schema reference |
| `doc/SCHEMA_IA.md` | Compact Cypher patterns for AI queries |
| `examples/` | Single source of truth for test fixtures |

---

## 1. Project Overview

- **Primary Goal:** Static analysis tool that parses multi-language codebases (Java, JavaScript/TypeScript, JSP, WebSphere XML) with Tree-Sitter, builds a semantic graph with qualified identifiers, and exports it to Neo4j. An MCP server exposes HTTP endpoints so AI agents can query the graph without re-running analysis.
- **Business Domain:** Developer tooling / code intelligence — dependency graph, call graph, and RAG for code understanding.
- **Pipeline:** `file discovery → AST extraction (Tree-Sitter) → unified semantic graph → Neo4j export → MCP HTTP API`

---

## 2. Tech Stack

| Layer | Technology |
|---|---|
| Language | Rust (edition 2021) |
| Async runtime | Tokio (full features) |
| Parser | Tree-Sitter + grammars: Java, JS/TS, Python, Rust, HTML |
| Database | Neo4j 5.x — bolt://localhost:7687, browser http://localhost:7474 |
| Neo4j client | neo4rs 0.7 |
| Serialization | serde 1.0 + serde_json 1.0 |
| Error handling | anyhow 1.0 |
| Logging | tracing 0.1 + tracing-subscriber + tracing-appender |
| Encoding | chardetng 0.1 + encoding_rs 0.8 |
| Regex | regex 1.10 |
| Package manager | Cargo |
| Dev environment | VS Code Dev Container + docker-compose |

---

## 3. Architecture & Key Files

### Module Map (`src/`)

```
src/
├── main.rs                      # CLI entrypoint + MCP mode switch
├── lib.rs
├── analysis/
│   ├── executor.rs              # analyze_repository() — top-level orchestrator
│   └── mod.rs
├── cli/                         # CLI argument parsing
├── config/
│   ├── mod.rs                   # Runtime configuration
│   └── package_filter.rs
├── encoding/                    # Multi-charset file reading
├── file_discovery/              # Recursive file discovery + filtering
├── graph_builder/
│   ├── builder.rs               # Dispatch to per-language extractors
│   ├── dsl_graph/
│   │   ├── java.rs              # Java AST → graph DSL
│   │   ├── javascript.rs        # JS/TS AST → graph DSL
│   │   └── mod.rs
│   └── dsl_executor/
│       ├── javascript_extractor.rs
│       ├── dependency_resolver.rs
│       └── mod.rs
├── semantic_graph/
│   ├── semantic_graph.rs        # NodeKind, EdgeRelation enums — source of truth for schema
│   ├── neo4j_exporter.rs        # Cypher generation for all node/edge types
│   ├── dsl.rs
│   └── mod.rs
├── neo4j_connectivity/          # Connection management, health checks
├── mcp/                         # MCP HTTP server (endpoints, handlers)
├── reporting/                   # Analysis reports
└── ui/                          # CLI progress/display utilities
```

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `NEO4J_URI` | `bolt://localhost:7687` | Neo4j Bolt URI |
| `NEO4J_USER` | `neo4j` | Neo4j username |
| `NEO4J_PASSWORD` | `password` | Neo4j password (dev only) |

Logs are written to `.output/app.log`.

---

## 4. Coding Conventions

### Formatting & Linting

```bash
cargo fmt                                        # Format all code
cargo clippy --all-targets -- -D warnings        # Zero warnings policy
```

Both must pass before any commit. No exceptions.

### Naming

| Context | Convention | Example |
|---|---|---|
| Functions / variables / modules | `snake_case` | `analyze_repository` |
| Types / structs / enums | `PascalCase` | `NodeKind`, `EdgeRelation` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_DEPTH` |
| Graph node IDs | Qualified dot-notation | `com.example.ServiceA.doCall` |

### Error Handling

- Use `Result<T, E>` with `anyhow::Error` for fallible operations.
- Propagate errors with `?`; never use `.unwrap()` in production code paths.
- Log meaningful context before returning errors (e.g., file path, node ID).
- MCP endpoint handlers must return structured JSON errors, never panic.

### Logging

| Level | When to use |
|---|---|
| `error!` | Unrecoverable failures |
| `warn!` | Recoverable issues, skipped files |
| `info!` | Key pipeline milestones (start, end, counts) — keep minimal |
| `debug!` | Flow details, per-file events |
| `trace!` | Deep parser / AST debugging |

Use structured fields: `tracing::info!(file = %path, nodes = count, "Extraction complete")`.

### Documentation

- New docs go under `doc/` with `UPPERCASE_NAME.md` filenames.
- All new doc files must be linked from `doc/SCHEMA.md` or the relevant schema reference.

---

## 5. TDD Workflow (MANDATORY)

**Every new feature and every bug fix MUST follow the Red-Green-Refactor cycle.**

### Priority Order

```
E2E tests  >  Integration tests  >  Unit tests
```

**Why E2E first?** E2E tests validate the complete pipeline:
`source file → parsing → semantic graph → Neo4j export → Cypher query`

They catch regressions across the entire chain and use real files from `examples/` as fixtures.

Unit tests remain useful for pure functions and isolated utilities only.

### Test Directory Structure

```
tests/
├── e2e/                         # End-to-End: full pipeline against Neo4j (HIGHEST PRIORITY)
│   └── *.rs
├── extraction/                  # Integration: AST parsing + graph building
│   ├── javascript.rs
│   ├── java_ast.rs
│   ├── java_imports.rs
│   ├── jsp_transitive_includes.rs
│   ├── field_extraction.rs
│   ├── servlet_mapping.rs
│   └── ...
├── neo4j/                       # Integration: Neo4j export + Cypher queries
│   ├── connection.rs
│   ├── calls.rs
│   ├── services.rs
│   └── export.rs
├── debug/                       # Debug/exploratory tests
└── common/                      # Shared helpers and fixtures helpers
```

---

### Step 1: RED — Write the failing test first

1. Clarify the exact expected behavior before writing any implementation code.
2. Place the test in the right directory:
   - Unit tests: `#[cfg(test)] mod tests { ... }` in the same `.rs` file.
   - Integration/E2E tests: new file under `tests/` in the appropriate subdirectory.
3. Write a test that **compiles but fails at runtime**.
4. Confirm the failure:

```bash
cargo test test_my_new_feature -- --nocapture
```

The test must fail for the right reason (behavior not yet implemented, not a compile error).

---

### Step 2: GREEN — Make the test pass

1. Write **only the minimum code** to make the test pass.
2. No premature optimization. The code can be ugly at this stage.
3. Verify:

```bash
cargo test test_my_new_feature
```

---

### Step 3: REFACTOR — Improve without breaking

1. Clean up duplication, improve readability.
2. After every change, re-run tests to stay green.
3. Final check:

```bash
cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
```

---

### Test Fixtures: The `examples/` Directory

**ABSOLUTE RULE: All integration and E2E tests MUST use files from `examples/` as fixtures.**

`examples/` is the single source of truth for test fixtures. Never create ad-hoc source files inside `tests/`.

#### Structure

```
examples/
├── backend/
│   ├── java/                    # Java: classes, services, inheritance, static calls
│   └── javascript/              # JS backend: services, transitive pipelines
├── frontend/
│   └── javascript/              # JS frontend: components, API calls
├── web_templates/               # JSP/JSPX templates, WEB-INF configs
├── config/                      # XML configuration files
└── websphere-portal/            # WebSphere Portal: Java + XML configs
```

#### Fixture Mapping

| Use Case | Fixture Files | Related Tests |
|---|---|---|
| Java class extraction | `backend/java/*.java` | `extraction/java_ast` |
| Inheritance / interfaces | `backend/java/Base*.java`, `Derived*.java` | `extraction/java_ast` |
| Method calls | `backend/java/Service*.java` | `neo4j/calls` |
| Transitive calls | `backend/java/TransitiveChain.java` | `neo4j/transitive` |
| Static calls | `backend/java/StaticCallsExample.java` | `extraction/java_ast` |
| JavaScript modules | `backend/javascript/*.js` | `extraction/javascript` |
| JSP templates | `web_templates/*.jsp` | `extraction/jsp_*` |
| XML configs | `config/*.xml`, `websphere-portal/*.xml` | `extraction/servlet_*` |

#### Fixture Rules

1. **No source files inside `tests/`** — all fixtures live in `examples/`.
2. **Examples = living documentation** — every example file must be valid, readable code.
3. **One file = one concept** — `TransitiveChain.java`, `StaticCallsExample.java`, etc.
4. **Descriptive naming** — file name must make the tested concept obvious.

#### Test Template

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Tests [EXPECTED BEHAVIOR DESCRIPTION]
    ///
    /// Fixture: examples/backend/java/ServiceA.java
    /// Scenario:
    ///   Given: [initial conditions]
    ///   When:  [action under test]
    ///   Then:  [expected result]
    #[test]
    fn test_descriptive_name() {
        // Arrange (Given) — always use examples/
        let fixture = PathBuf::from("examples/backend/java/ServiceA.java");
        assert!(fixture.exists(), "Missing fixture: {:?}", fixture);

        // Act (When)
        let result = function_under_test(&fixture);

        // Assert (Then)
        assert_eq!(result, expected_value);
    }
}
```

---

### TDD Rules for AI Agents

| FORBIDDEN | REQUIRED |
|---|---|
| Write implementation code before a test | Write the test BEFORE any implementation |
| Modify existing code without a regression test | Add a test that captures the bug before fixing it |
| Implement multiple features in one cycle | One test = one atomic behavior |
| Refactor while tests are red | All tests must be green before refactoring |
| Delete tests to make them pass | Tests define the contract — never remove them |
| Create fixtures inside `tests/` | All fixtures go in `examples/` |

---

### Bug Fix Workflow

1. **Reproduce** — write a test that reproduces the bug (RED)
2. **Confirm** — the test must fail in the same way as the reported bug
3. **Fix** — implement the minimal fix (GREEN)
4. **Verify** — the test passes, other tests still pass
5. **Protect** — this test now guards against regressions

---

### Pre-Commit Checklist

```bash
# 1. All tests pass
cargo test

# 2. Code is formatted
cargo fmt --check

# 3. No clippy warnings
cargo clippy --all-targets -- -D warnings

# 4. If schema changed: verify SCHEMA.md and SCHEMA_IA.md are updated
```

All four checks must pass. No exceptions.

---

## 6. Neo4j Schema Maintenance (CRITICAL)

Any change to the Neo4j schema — new node label, new relation type, new property — requires **synchronized updates across exactly four files in the same commit**:

| File | What to update |
|---|---|
| `src/semantic_graph/semantic_graph.rs` | `NodeKind` and `EdgeRelation` enums |
| `src/semantic_graph/neo4j_exporter.rs` | Cypher generation for the new type |
| `doc/SCHEMA.md` | Full human-readable documentation with examples |
| `doc/SCHEMA_IA.md` | Compact Cypher patterns reference for AI agents |

**All four must remain in sync. Never update one without the others.**

### When This Applies

- Adding a new `NodeKind` variant (new language construct)
- Adding a new `EdgeRelation` variant (new relationship type)
- Adding or renaming a node/edge property
- Changing how a Cypher query generates IDs or labels

### Commit Rule

Schema changes must ship in an atomic commit that includes all modified files. Use the `schema` commit type and reference all four modified files in the commit body.

---

## 7. MCP Server

The MCP server (`src/mcp/`) exposes an HTTP API for AI agents to query the Neo4j graph without re-running the analysis.

### Endpoints (under `/api/mcp/`)

| Endpoint | Description |
|---|---|
| `execute_cypher` | Run an arbitrary Cypher query |
| `search_nodes` | Full-text search on node names/IDs |
| `find_calls` | Find callers/callees of a function |
| `analyze_dependencies` | Dependency graph for a module/class |

### Querying the Graph as an AI Agent

Prefer MCP endpoints over direct Neo4j access. Use `execute_cypher` for ad-hoc exploration:

```json
// Find all callers of a method
{
  "query": "MATCH (caller)-[:CALLS]->(callee {id: 'com.example.ServiceA.doCall'}) RETURN caller.id, caller.kind"
}

// Find all classes in a package
{
  "query": "MATCH (n:Class) WHERE n.id STARTS WITH 'com.example.' RETURN n.id, n.name LIMIT 50"
}

// Inspect outgoing dependencies of a module
{
  "query": "MATCH (m {id: 'com.example.ModuleX'})-[r]->(dep) RETURN type(r), dep.id, dep.kind"
}
```

Refer to `doc/SCHEMA_IA.md` for the full list of node labels, relation types, and property names.

### Request Format

All endpoints accept and return JSON. Errors are returned as structured JSON, never as plain text or panics.

### Logging in MCP Mode

Use `warn` as the default log level in MCP stdio mode to avoid polluting the JSON stream. Never print to stdout in MCP handlers — use `tracing` only.

---

## 8. Commit Conventions

Use **Conventional Commits** format:

```
<type>(<scope>): <short description>

[optional body]
```

| Type | When to use |
|---|---|
| `feat` | New feature or new language support |
| `fix` | Bug fix |
| `test` | Adding or updating tests |
| `refactor` | Code restructuring without behavior change |
| `docs` | Documentation updates |
| `chore` | Tooling, dependencies, CI configuration |
| `schema` | Neo4j schema changes (requires 4-file sync) |

**Schema changes must use the `schema` type** and reference all four modified files in the commit body.

---

## 9. Security

- **Never commit secrets.** Neo4j credentials in `.env.example` are for local development only.
- The `.env` file (if it exists) is gitignored — keep it that way.
- MCP endpoints must validate all inputs before executing Cypher. Never interpolate raw strings into Cypher queries.
- Review authentication and connection handling any time `mcp/` or `neo4j_connectivity/` code changes.

---

## 10. Infrastructure

### Development Environment

The canonical development environment is the **VS Code Dev Container** (`.devcontainer/`). All contributors should use it to ensure reproducible builds.

```bash
# Build
cargo build

# Build (release)
cargo build --release

# Run analysis on examples
cargo run -- examples/backend/

# Run all tests
cargo test

# Run E2E tests only
cargo test --test e2e

# Run extraction integration tests
cargo test --test extraction

# Run Neo4j integration tests
cargo test --test neo4j
```

### Services (docker-compose.yml)

| Service | Image | Ports |
|---|---|---|
| `neo4j` | neo4j:5.15 | 7474 (browser), 7687 (bolt) |
| `code-continuum` | Local build | — |
| `mcp-neo4j` | Local build | MCP HTTP API |

### Deployment

Use `scripts/deploy.sh` with the following commands:

```bash
./scripts/deploy.sh build      # Build Docker images
./scripts/deploy.sh start      # Start all services
./scripts/deploy.sh stop       # Stop all services
./scripts/deploy.sh logs       # Tail service logs
./scripts/deploy.sh analyze    # Run analysis
./scripts/deploy.sh test       # Run tests
./scripts/deploy.sh status     # Check service health
./scripts/deploy.sh clean      # Remove containers and volumes
```

Neo4j health is checked automatically before running the analyzer. The MCP server is verified after startup.
