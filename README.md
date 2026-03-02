# code-continuum

Static analysis tool that builds a semantic graph from source code and exposes it to AI agents via MCP servers.

## What it does

1. Parses source code using Tree-Sitter
2. Builds a semantic graph (classes, functions, imports, dependencies)
3. Stores the graph in Neo4j, tagged by project
4. Exposes two MCP servers so AI agents can manage projects and query the graph

## Architecture

```
Source code (mounted at /app/data)
    │
    ├─→ code-continuum MCP (port 8001)
    │   ├─ list_projects
    │   ├─ add_project (Tree-Sitter AST parsing + Cypher INSERT)
    │   └─ remove_project
    │
    ▼
Neo4j (bolt://localhost:7687)
    ▲
    │
    └─ Neo4j MCP (port 8000)
       └─ Cypher queries
            │
            ▼
        AI agents (Claude, Copilot, …)
```

## MCP Integration

Two MCP servers run alongside Neo4j:

| Service | Port | Purpose |
|---|---|---|
| `mcp-code-continuum` | 8001 | Add / remove projects in the graph |
| `mcp-neo4j` | 8000 | Query the graph with Cypher |

### Configuration

**Devcontainer:** Use `.mcp.json` at the root (service names):

```json
{
  "mcpServers": {
    "neo4j": {
      "type": "http",
      "url": "http://mcp-neo4j:8000/api/mcp/"
    },
    "code-continuum": {
      "type": "http",
      "url": "http://localhost:8001/api/mcp/"
    }
  }
}
```

**Production:** Use `production/.mcp.json` (localhost):

Just open Claude inside `production` directory.

```json
{
  "mcpServers": {
    "neo4j": {
      "type": "http",
      "url": "http://localhost:8040/api/mcp/"
    },
    "code-continuum": {
      "type": "http",
      "url": "http://localhost:8041/api/mcp/"
    }
  }
}

```

### Tools — code-continuum MCP

#### `list_projects`

Lists all available projects (subdirectories) mounted under CODE_PATH (`/app/data`).

Use this first to discover what projects are available, then pass a project name to `add_project`.

**No parameters required.**

**Returns:** List of available project names relative to CODE_PATH.

#### `add_project`

Analyses a source directory and inserts its nodes/relations into Neo4j.
Does **not** clear the whole database — each project is isolated by `project_path` / `project_name`.

The project path must be **relative to CODE_PATH** (e.g., `backend/java`, `my-app`). Use `list_projects` to discover available projects first.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `project_path` | string | ✅ | Path relative to CODE_PATH (e.g. `backend/java` or `my-app`) |
| `project_name` | string | | Friendly name (defaults to last segment of the path) |
| `include_packages` | string | | CSV filter: only index matching packages (e.g. `com.example,org.app`) |
| `clear_project` | boolean | | Delete existing data for this project before re-indexing (default: `false`) |

#### `remove_project`

Deletes all nodes and relations belonging to a project.

| Parameter | Type | Required | Description |
|---|---|---|---|
| `project_path` | string | ✅ | Must be relative to CODE_PATH and match the `project_path` used when adding |

### Tools — Neo4j MCP

Once a project is indexed, query it with Cypher through the `neo4j` MCP server:

```cypher
-- All functions of a project
MATCH (f:Function {project_name: "my-project"}) RETURN f.name, f.file_path

-- Call chains
MATCH (a:Function)-[:CALLS*1..5]->(b:Function)
WHERE a.project_name = "my-project"
RETURN a.name, b.name

-- Cross-project: find classes shared by two projects
MATCH (n:Node) WHERE n.name = "UserService"
RETURN n.project_name, n.file_path
```

## Quick start

> ⚠️ Choose your setup:
> - **Devcontainer:** Development inside VS Code container (recommended for contributors)
> - **Production:** Standalone Docker on any machine (ready for deployment)

### Development — VS Code Devcontainer

Open the workspace in VS Code, and it will automatically prompt to reopen in the Dev Container:

```bash
# 1. Clone and open in VS Code
git clone <repo>
cd code-continuum
# Click "Reopen in Container" when prompted

# 2. Inside the container, run tests or analyses
cargo test
cargo run -- examples/backend/java

# 3. Neo4j + MCP servers are auto-started by devcontainer config
# MCP endpoints available at:
#   http://mcp-neo4j:8000/api/mcp/      (Neo4j queries)
#   http://mcp-code-continuum:8001/api/mcp/ (project management)
```

**Use root `.mcp.json`** (already configured for service names).

### Production — Docker Compose (localhost)

Deploy on any machine with Docker. All services communicate via `localhost`.

#### 1 — Start the stack

```bash
cd production
cp .env.example .env # Edit .env — set NEO4J_PASSWORD and CODE_PATH if needed

docker compose up -d 
```


#### 2 — Mount your codebase

By default, the `CODE_PATH` is set to `../examples`. To analyze a different project:

```bash
# Option A: Edit production/.env
CODE_PATH=/absolute/path/to/your/project docker compose up -d

# Option B: Provide inline
docker compose run --rm code-continuum /path/to/project
```

The directory is mounted read-only at `/app/data` inside the container.

#### 3 — Use the MCP endpoints

From Claude Code or any MCP client, point to **localhost**:

```bash
# Copy production/.mcp.json to your Claude Code config
# It already points to localhost endpoints
cat production/.mcp.json
```

First, ask "list available projects"

```
list_projects()
```

Then : "add projects frontend and backend":

```
add_project(
  project_path = "backend",
  project_name = "frontend"
)
```

Query with the `neo4j` tool : "give me the sequence diagram of java function getClient ?"

Remove a project when done:

```
remove_project(project_path = "backend/java")
```

#### 4 — Stop the stack

```bash
cd production
docker compose down
# Remove data volumes (if needed): docker compose down -v
```

## Development

All development happens in the **VS Code Devcontainer** — no local Rust/Neo4j installation required.

```bash
git clone <repo>
cd code-continuum
# VS Code → "Reopen in Container"
```

Once inside the container, the devcontainer's own `docker-compose.yml` automatically starts Neo4j and both MCP servers. The root `.mcp.json` is pre-configured for service names.

The last thing to do is to launch the `Debug: MCP Server` on VSCODE.

**For production deployment**, see [Production — Docker Compose (localhost)](#production--docker-compose-localhost) above.

### Run tests

```bash
# Unit and extraction tests (no Neo4j needed)
cargo test

# MCP E2E tests (Neo4j runs automatically in the devcontainer)
cargo test --test integration_mcp -- --ignored --nocapture

# All ignored tests (Neo4j required)
cargo test -- --ignored --nocapture
```

### Analyse locally (development only)

The CLI batch mode (direct invocation) clears the entire database on each run. **Use this only for testing, not for multi-project work.**

For multi-project analysis, always use the MCP `add_project` tool (see [Production — Docker Compose (localhost)](#production--docker-compose-localhost)).

```bash
# Direct CLI — clears the whole database and imports the directory
cargo run -- examples/backend/java

# With package filter
INCLUDE_PACKAGES=com.example,org.myapp cargo run -- /path/to/project

# Query (Neo4j browser at http://localhost:7474)
MATCH (f:Function)-[:CALLS*1..5]->(g:Function) RETURN f.name, g.name
```

> **Note:** The CLI batch mode clears the entire database on each run.
> For multi-project use, always go through the MCP `add_project` tool instead.

## Supported languages

| Language | Extensions | Extraction |
|---|---|---|
| Java | `.java` | Specialized (classes, methods, imports, fields) |
| JavaScript / TypeScript | `.js` `.jsx` `.ts` `.tsx` | Specialized (classes, methods) |
| Rust | `.rs` | Specialized (functions, structs, enums, traits, impl blocks, use declarations) |
| JSP / JSPX / JSPF | `.jsp` `.jspx` `.jspf` | Specialized (include relations) |
| XML (WebSphere Portal) | `portlet.xml` `web.xml` | Specialized (servlet/portlet mappings) |
| HTML | `.html` `.htm` | Indexed only (no extraction) |

### Adding a new language

Adding support for a new language is surprisingly straightforward : mostly because this project cheats *a lot* with Vibe Coding. That was actually the goal of the project: **don’t spend too much time managing new language**.

A TDD approach is still a good practice.

## AI Agents

The project ships custom Claude Code sub-agents in [.claude/agents/](.claude/agents/).

### retrodoc

The [`retrodoc`](.claude/agents/retrodoc.md) agent generates complete reverse documentation
by querying the live Neo4j graph and producing Mermaid diagrams from actual data.

```
generate the retrodoc for this project
```

The agent:
1. Calls `add_project` via the MCP server to self-import the codebase into Neo4j
2. Executes discovery Cypher queries (module hierarchy, call chains, data structures, …)
3. Synthesizes Mermaid diagrams from live graph data
4. Writes the result to `doc/RETRODOC.md`

**Output:** [doc/RETRODOC_GENERATED.md](doc/RETRODOC_GENERATED.md)
**todo update this part**
---

## Docs

| | |
|---|---|
| [doc/SCHEMA.md](doc/SCHEMA.md) | Full Neo4j schema reference (nodes, relations, properties, query examples) |
| [doc/SCHEMA_IA.md](doc/SCHEMA_IA.md) | Condensed Cypher cheatsheet — intended as AI agent knowledge base |
| [doc/RETRODOC_GENERATED.md](doc/RETRODOC_GENERATED.md) | Reverse documentation generated by the retrodoc agent |

## Project structure

```
src/
├── analysis/           # Orchestration
├── semantic_graph/     # Core types (graph nodes, edges, Neo4j export)
├── graph_builder/      # AST extraction per language
├── neo4j_connectivity/ # Neo4j connection helper
├── mcp/                # MCP HTTP server (add_project, remove_project)
└── ...
tests/
├── mcp/                # E2E tests for the MCP endpoint
├── neo4j/              # Neo4j integration tests
├── extraction/         # Parsing / extraction unit tests
└── e2e/                # Full pipeline tests
.devcontainer/          # Dev Container (Rust + Neo4j)
docker-compose.yml      # Production: neo4j + mcp-neo4j + mcp-code-continuum
```

---

## Motivation

I tested this approach on a real Java project during a legacy migration. The AST + Graph + Agentic combo is incredibly powerful ; the AI maps the application architecture almost instantly and generates Cypher queries that would take a human forever 😄. This approach uses fewer tokens.

An explored use case is API retrodocumentation. By giving the AI a complete view of the sequence diagram, it can retrodocument both technically and functionally. The diagram acts as a guiding thread, preventing the AI from getting lost in the complexity of the system. This same technique can also be applied for audits.

This project is just a part of the core technique and help creating complex multi-agent orchestration (coordinator, retro-spec → new spec → code generation, all using a code-continuum expert), which is a whole other story. A complementary schema can be used with relations or new property discovery by agents. For example: "functional description", "security issue", "call backend", ... what you need to maintain traceability. 


The next DSL: **COBOL** — the king of legacies.

If you're interested in this approach or want to discuss it further, feel free to contact me and exchange ideas.
