//! MCP (Model Context Protocol) Server pour Code Graph
//! Expose deux fonctions: parse_project et query_graph

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::io::{self, BufRead, Write};
use std::path::Path;
use tracing::{debug, error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct McpMessage {
    jsonrpc: String,
    id: Option<u64>,
    method: Option<String>,
    params: Option<Value>,
    result: Option<Value>,
    error: Option<McpError>,
}

#[derive(Debug, Serialize, Deserialize)]
struct McpError {
    code: i32,
    message: String,
}

pub async fn run_mcp_server() {
    info!("MCP Server started in stdio mode");

    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let reader = stdin.lock();

    for line in reader.lines() {
        if let Ok(request_line) = line {
            let request_line = request_line.trim();
            if request_line.is_empty() {
                continue;
            }

            match serde_json::from_str::<McpMessage>(&request_line) {
                Ok(req) => {
                    debug!("📥 Reçu: {:?}", req.method);
                    let response = handle_request(req).await;
                    if let Ok(response_json) = serde_json::to_string(&response) {
                        let _ = writeln!(stdout, "{}", response_json);
                        let _ = stdout.flush();
                    }
                }
                Err(e) => {
                    error!("❌ Erreur de parsing JSON: {}", e);
                    let error_response = McpMessage {
                        jsonrpc: "2.0".to_string(),
                        id: None,
                        method: None,
                        params: None,
                        result: None,
                        error: Some(McpError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                        }),
                    };
                    if let Ok(response_json) = serde_json::to_string(&error_response) {
                        let _ = writeln!(stdout, "{}", response_json);
                        let _ = stdout.flush();
                    }
                }
            }
        }
    }
}

async fn handle_request(mut req: McpMessage) -> McpMessage {
    let method = req.method.take().unwrap_or_default();
    let params = req.params.take().unwrap_or(json!({}));
    let id = req.id;

    let result = match method.as_str() {
        "initialize" => initialize(&params),
        "initialized" => Ok(json!({})),
        "tools/list" => list_tools(),
        "prompts/list" => list_prompts(),
        "prompts/get" => get_prompt(&params),
        "tools/call" => {
            let tool_name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let default_args = json!({});
            let args = params.get("arguments").unwrap_or(&default_args);
            let tool_result = match tool_name {
                "parse_project" => parse_project(args).await,
                "execute_cypher" => execute_cypher_tool(args).await,
                _ => Err(format!("Unknown tool: {}", tool_name)),
            };
            // Wrap result in MCP tools/call format
            tool_result.map(|content| json!({
                "content": [
                    {
                        "type": "text",
                        "text": serde_json::to_string_pretty(&content).unwrap_or_else(|_| content.to_string())
                    }
                ]
            }))
        }
        _ => Err(format!("Unknown method: {}", method)),
    };

    match result {
        Ok(result_value) => McpMessage {
            jsonrpc: "2.0".to_string(),
            id,
            method: None,
            params: None,
            result: Some(result_value),
            error: None,
        },
        Err(error_message) => McpMessage {
            jsonrpc: "2.0".to_string(),
            id,
            method: None,
            params: None,
            result: None,
            error: Some(McpError {
                code: -32603,
                message: error_message,
            }),
        },
    }
}

fn initialize(params: &Value) -> Result<Value, String> {
    let protocol_version = params
        .get("protocolVersion")
        .and_then(|v| v.as_str())
        .unwrap_or("2024-11-05");
    debug!(
        "🤝 Initialize MCP server with protocol {}",
        protocol_version
    );

    Ok(json!({
        "protocolVersion": protocol_version,
        "capabilities": {
            "tools": {},
            "prompts": {}
        },
        "serverInfo": {
            "name": "code-continuum",
            "version": "0.1.0"
        }
    }))
}

fn list_tools() -> Result<Value, String> {
    Ok(json!({
        "tools": [
            {
                "name": "parse_project",
                "description": "Parse a project directory and build the semantic code graph",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the project directory to analyze"
                        },
                        "languages": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Optional: filter by specific languages (java, javascript, python, rust)"
                        }
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "execute_cypher",
                "description": "Execute a raw Cypher query on the Neo4j graph database. Use this to search for classes, functions, analyze call graphs, find transitive calls, etc.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "The Cypher query to execute. Nodes have TWO labels: generic :Node and typed labels (:Function, :Class, :Variable, etc.). Use typed labels for simpler queries. Examples: 'MATCH (c:Class) WHERE c.language = 'java' RETURN c.name, c.file_path', 'MATCH path = (start:Function {name: 'startChain'})-[:CALLS*1..5]->(called:Function) RETURN start.name, called.name, length(path) as depth'"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results to return (default: 100)"
                        }
                    },
                    "required": ["query"]
                }
            }
        ]
    }))
}

fn list_prompts() -> Result<Value, String> {
    Ok(json!({
        "prompts": [
            {
                "name": "parse_project_guide",
                "description": "Guide to use parse_project tool to analyze a codebase",
                "arguments": []
            },
            {
                "name": "code_continuum_schema",
                "description": "Get the Neo4j graph schema with node types, relationships, and properties for querying the codebase",
                "arguments": []
            }
        ]
    }))
}

fn get_prompt(params: &Value) -> Result<Value, String> {
    let prompt_name = params
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or("Missing 'name' parameter")?;

    match prompt_name {
        "parse_project_guide" => Ok(json!({
            "description": "How to use parse_project",
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "# Using parse_project Tool

## Purpose
Analyzes a codebase and exports the semantic graph to Neo4j.

## Parameters
- **path** (required): Absolute path to the project directory
- **languages** (optional): Array of languages to analyze, e.g. [\"java\", \"javascript\", \"python\", \"rust\"]

## Example Usage
```json
{
  \"path\": \"/workspaces/code-continuum/examples\",
  \"languages\": [\"java\", \"javascript\"]
}
```

## What it does
1. Scans the directory for source files
2. Parses code to extract:
   - Classes and their methods
   - Functions and their calls
   - Variables and their usage
   - Import statements
   - Parameters
3. Builds relationships (CALLS, DEFINES, USES, etc.)
4. Exports everything to Neo4j at bolt://neo4j:7687

## After parsing
Verify with execute_cypher:
```cypher
MATCH (n) RETURN labels(n)[0] as type, count(*) as count ORDER BY count DESC
```

Then explore with code_continuum_schema prompt for query examples."
                    }
                }
            ]
        })),
        "code_continuum_schema" => Ok(json!({
            "description": "Code Graph Database Schema",
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": "# Code Graph Neo4j Schema

## Node Structure
All nodes have **TWO labels** (mode 'both' by default):
- Generic label **:Node** (for common queries)
- Typed label **:Function**, **:Class**, **:Variable**, etc. (for specific queries)

Example: A function has labels `:Node:Function`

This allows two query styles:
```cypher
// Style 1: Use typed label directly (RECOMMENDED - simpler & faster)
MATCH (f:Function) WHERE f.language = 'java' RETURN f.name

// Style 2: Use Node with node_type property
MATCH (f:Node) WHERE f.node_type = 'Function' RETURN f.name
```

### Core Properties (all nodes):

MATCH (f:Node) WHERE f.node_type = 'Function' RETURN f.name
```

### Core Properties (all nodes)
- **id**: Unique identifier (e.g., 'User.java::User::getStatus')
- **name**: Element name (e.g., 'getStatus', 'User')
- **node_type**: Type of node (see below)
- **language**: Programming language ('java', 'javascript', 'rust', etc.)
- **file_path**: Source file path

### Optional Properties
- **module**: Module or namespace
- **package**: Package or module path
- **class**: Containing class name
- **caller_id**: ID of calling node (for calls)
- **object_type**: Object type (for variables/fields)
- **start_line, start_col, end_line, end_col**: Location in file

### Available Labels
All nodes have both labels simultaneously:
- **:Node** - Generic label (all nodes)
- **:Function, :Class, :Interface, :Module, :Variable, :Parameter, :Call, :Package** - Typed labels

### Node Types (node_type property)
- **Function**: Functions/methods
- **Class**: Classes
- **Interface**: Interfaces
- **Module**: Modules/namespaces
- **Type**: Custom types
- **Trait**: Traits (Rust)
- **Parameter**: Function parameters
- **Variable**: Local variables/fields
- **Call**: Function calls
- **Import**: Imports
- **Package**: External packages
- **Expression**: Expressions
- **Operator**: Operators

## Relationships
- **CALLS**: Function calls another Function
- **DEFINES**: Defines an element (function parameter, etc.)
- **IMPORTS**: Module imports another module
- **EXTENDS**: Class extends another class
- **IMPLEMENTS**: Class implements an interface
- **USES**: Uses a variable/type
- **CONTAINS**: Contains another element (class contains method)

## Example Queries

### Find all Java classes:
```cypher
// Recommended: Use typed label (simpler & faster)
MATCH (c:Class) WHERE c.language = 'java' 
RETURN c.name, c.package, c.file_path, c.start_line

// Alternative: Use Node with node_type
MATCH (c:Node) WHERE c.node_type = 'Class' AND c.language = 'java' 
RETURN c.name, c.package, c.file_path, c.start_line
```

### Find transitive function calls (up to depth 5):
```cypher
// Recommended: Use typed labels for better performance
// Recommended: Use typed labels for better performance
MATCH path = (start:Function {name: 'functionName'})-[:CALLS*1..5]->(called:Function)
RETURN start.name, start.class, start.package, start.file_path,
       called.name, called.class, called.package, called.file_path,
       length(path) as depth
ORDER BY depth
```

### Find methods in a class:
```cypher
MATCH (c:Class {name: 'ClassName'})-[:CONTAINS]->(f:Function)
RETURN f.name, f.start_line, f.id
ORDER BY f.start_line
```

### Find who calls a function:
```cypher
MATCH (caller:Function)-[:CALLS]->(target:Function {name: 'targetFunction'})
RETURN caller.name, caller.file_path, caller.start_line
```

### Graph statistics:
```cypher
MATCH (n:Node) 
RETURN n.node_type as type, count(*) as count 
ORDER BY count DESC
```

**Recommendation**: Use typed labels (`:Function`, `:Class`, etc.) for simpler and faster queries.

Use the `execute_cypher` tool to run any of these queries."
                    }
                }
            ]
        })),
        _ => Err(format!("Unknown prompt: {}", prompt_name)),
    }
}

async fn parse_project(args: &Value) -> Result<Value, String> {
    let path_str = args
        .get("path")
        .and_then(|p| p.as_str())
        .ok_or("Missing 'path' parameter")?;

    info!(path = path_str, "Parsing project");

    let path = Path::new(path_str);
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path_str));
    }

    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", path_str));
    }

    // Analyser le projet
    crate::analysis::executor::analyze_repository(path).await;

    let response = json!({
        "status": "success",
        "message": format!("✅ Project parsed successfully from {}", path_str),
        "details": {
            "path": path_str,
            "note": "Graph has been extracted and exported to Neo4j"
        }
    });
    Ok(response)
}
async fn execute_cypher_tool(args: &Value) -> Result<Value, String> {
    let query = args
        .get("query")
        .and_then(|q| q.as_str())
        .ok_or("Missing 'query' parameter")?;

    let limit = args.get("limit").and_then(|l| l.as_i64()).unwrap_or(100);

    debug!(query = query, "Executing Cypher query");

    // Connexion à Neo4j
    let uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://neo4j:7687".to_string());
    let user = env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());

    let graph = match neo4rs::Graph::new(&uri, &user, &password).await {
        Ok(g) => g,
        Err(e) => {
            warn!("Neo4j connection failed: {}", e);
            return Ok(json!({
                "status": "error",
                "query": query,
                "message": format!("Neo4j connection failed: {}", e),
                "results": []
            }));
        }
    };

    // Ajouter une limite si elle n'est pas déjà présente
    let final_query = if query.to_uppercase().contains("LIMIT") {
        query.to_string()
    } else {
        format!("{} LIMIT {}", query, limit)
    };

    // Exécuter la requête
    match graph.execute(neo4rs::query(&final_query)).await {
        Ok(mut result) => {
            let mut results = Vec::new();
            let mut columns: Vec<String> = Vec::new();

            while let Ok(Some(row)) = result.next().await {
                let record = extract_row_as_json(&row, &mut columns);
                if !record.is_empty() {
                    results.push(json!(record));
                }
            }

            Ok(json!({
                "status": "success",
                "query": final_query,
                "columns": columns,
                "results": results,
                "count": results.len()
            }))
        }
        Err(e) => {
            error!("Failed to execute Cypher query: {}", e);
            Ok(json!({
                "status": "error",
                "query": final_query,
                "message": format!("Query execution failed: {}", e),
                "results": []
            }))
        }
    }
}

/// Extrait une ligne Neo4j et la convertit en JSON de manière générique
fn extract_row_as_json(
    row: &neo4rs::Row,
    columns: &mut Vec<String>,
) -> serde_json::Map<String, Value> {
    let mut record = serde_json::Map::new();

    // Liste des colonnes à essayer (basée sur votre schéma)
    let potential_columns = [
        // Propriétés de base
        "id",
        "name",
        "node_type",
        "language",
        "file_path",
        "module",
        "package",
        "class",
        "caller_id",
        "object_type",
        "start_line",
        "start_col",
        "end_line",
        "end_col",
        // Aliases courants
        "file",
        "line",
        "type",
        "value",
        "count",
        "total",
        // Relations
        "caller",
        "callee",
        "relation",
        "user",
        // Queries transitives
        "depth",
        "call_depth",
        "start_function",
        "start_file",
        "start_class",
        "start_package",
        "start_language",
        "called_function",
        "called_file",
        "called_class",
        "called_package",
        "called_language",
        // Noms de variables Cypher courants
        "n",
        "m",
        "r",
        "c",
        "f",
        "v",
        "p",
    ];

    for col in &potential_columns {
        if let Some(value) = try_extract_value(row, col) {
            record.insert(col.to_string(), value);

            // Ajouter à la liste des colonnes si première ligne
            if columns.is_empty() || !columns.contains(&col.to_string()) {
                if columns.is_empty() {
                    columns.push(col.to_string());
                }
            }
        }
    }

    record
}

/// Essaie d'extraire une valeur d'une colonne avec différents types
fn try_extract_value(row: &neo4rs::Row, column: &str) -> Option<Value> {
    // Essayer String
    if let Ok(val) = row.get::<String>(column) {
        return Some(json!(val));
    }

    // Essayer i64
    if let Ok(val) = row.get::<i64>(column) {
        return Some(json!(val));
    }

    // Essayer f64
    if let Ok(val) = row.get::<f64>(column) {
        return Some(json!(val));
    }

    // Essayer bool
    if let Ok(val) = row.get::<bool>(column) {
        return Some(json!(val));
    }

    // Essayer Vec<String>
    if let Ok(val) = row.get::<Vec<String>>(column) {
        return Some(json!(val));
    }

    None
}
