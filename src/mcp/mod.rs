use axum::{routing::post, Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tracing::{error, info, warn};

use crate::analysis::executor;
use crate::config::PackageFilter;
use crate::semantic_graph::Neo4jExporter;

/// Construit le Router axum du serveur MCP (utilisable en test)
pub fn make_app() -> Router {
    Router::new().route("/api/mcp/", post(handle_mcp_request))
}

/// Démarre le serveur MCP HTTP sur le port MCP_PORT (défaut: 8001)
pub async fn run_mcp_server() {
    let port = std::env::var("MCP_PORT").unwrap_or_else(|_| "8001".to_string());
    let addr = format!("0.0.0.0:{}", port);

    info!("Démarrage du serveur MCP sur {}", addr);

    let app = make_app();

    let listener = match TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Impossible de démarrer le serveur MCP sur {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    info!("Serveur MCP prêt sur http://{}/api/mcp/", addr);

    if let Err(e) = axum::serve(listener, app).await {
        error!("Erreur serveur MCP: {}", e);
    }
}

async fn handle_mcp_request(Json(request): Json<Value>) -> Json<Value> {
    let method = request["method"].as_str().unwrap_or("");
    let id = request.get("id").cloned();

    // Notification (pas d'id) → pas de réponse requise, retourner null
    if id.is_none() {
        return Json(Value::Null);
    }

    let result = match method {
        "initialize" => handle_initialize(),
        "tools/list" => handle_tools_list(),
        "tools/call" => handle_tools_call(&request["params"]).await,
        _ => json!({"error": {"code": -32601, "message": format!("Method not found: {}", method)}}),
    };

    Json(json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }))
}

fn handle_initialize() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "code-continuum",
            "version": "0.1.0"
        }
    })
}

fn handle_tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "add_project",
                "description": "Analyse un projet et ajoute ses nœuds/relations au graphe Neo4j. N'efface pas les autres projets.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_path": {
                            "type": "string",
                            "description": "Chemin absolu du projet à analyser (ex: /app/data/monprojet)"
                        },
                        "project_name": {
                            "type": "string",
                            "description": "Nom du projet (optionnel, défaut: dernier segment du chemin)"
                        },
                        "include_packages": {
                            "type": "string",
                            "description": "Patterns CSV pour filtrer les packages à inclure (optionnel, ex: 'com.example,org.myapp')"
                        },
                        "clear_project": {
                            "type": "boolean",
                            "description": "Supprimer les données existantes du projet avant l'analyse (défaut: false)"
                        }
                    },
                    "required": ["project_path"]
                }
            },
            {
                "name": "remove_project",
                "description": "Supprime tous les nœuds et relations d'un projet du graphe Neo4j.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "project_path": {
                            "type": "string",
                            "description": "Chemin du projet à supprimer (doit correspondre au project_path utilisé lors de l'ajout)"
                        }
                    },
                    "required": ["project_path"]
                }
            }
        ]
    })
}

async fn handle_tools_call(params: &Value) -> Value {
    let tool_name = params["name"].as_str().unwrap_or("");

    match tool_name {
        "add_project" => handle_add_project(&params["arguments"]).await,
        "remove_project" => handle_remove_project(&params["arguments"]).await,
        _ => json!({
            "content": [{"type": "text", "text": format!("Outil inconnu: {}", tool_name)}],
            "isError": true
        }),
    }
}

async fn handle_add_project(args: &Value) -> Value {
    let project_path = match args["project_path"].as_str() {
        Some(p) => p.to_string(),
        None => {
            return json!({
                "content": [{"type": "text", "text": "Paramètre requis manquant: project_path"}],
                "isError": true
            });
        }
    };

    // project_name: dernier segment du chemin si non fourni
    let project_name = args["project_name"]
        .as_str()
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            std::path::Path::new(&project_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        });

    let include_packages = args["include_packages"].as_str().map(|s| s.to_string());
    let clear_project = args["clear_project"].as_bool().unwrap_or(false);

    let path = std::path::Path::new(&project_path);

    if !path.exists() {
        return json!({
            "content": [{"type": "text", "text": format!("Chemin introuvable: {}", project_path)}],
            "isError": true
        });
    }

    if !path.is_dir() {
        return json!({
            "content": [{"type": "text", "text": format!("Le chemin n'est pas un répertoire: {}", project_path)}],
            "isError": true
        });
    }

    let filter = if let Some(patterns_str) = include_packages {
        let patterns: Vec<String> = patterns_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if patterns.is_empty() {
            None
        } else {
            Some(PackageFilter::with_patterns(patterns, vec![], true))
        }
    } else {
        None
    };

    info!(
        project = %project_name,
        path = %project_path,
        clear = clear_project,
        "MCP: add_project"
    );

    match executor::analyze_repository_for_project(
        path,
        &project_path,
        &project_name,
        filter,
        clear_project,
    )
    .await
    {
        Ok(summary) => json!({
            "content": [{"type": "text", "text": summary}],
            "isError": false
        }),
        Err(e) => {
            warn!(error = %e, "add_project failed");
            json!({
                "content": [{"type": "text", "text": format!("Erreur: {}", e)}],
                "isError": true
            })
        }
    }
}

async fn handle_remove_project(args: &Value) -> Value {
    let project_path = match args["project_path"].as_str() {
        Some(p) => p.to_string(),
        None => {
            return json!({
                "content": [{"type": "text", "text": "Paramètre requis manquant: project_path"}],
                "isError": true
            });
        }
    };

    info!(path = %project_path, "MCP: remove_project");

    // Convertir Box<dyn Error> en String immédiatement pour éviter de la tenir à travers un await
    let exporter = match Neo4jExporter::new().await {
        Ok(e) => e,
        Err(e) => {
            return json!({
                "content": [{"type": "text", "text": format!("Impossible de se connecter à Neo4j: {}", e)}],
                "isError": true
            });
        }
    };

    match exporter.delete_project(&project_path).await {
        Ok(count) => json!({
            "content": [{
                "type": "text",
                "text": format!("Projet supprimé: {} nœuds supprimés (project_path: {})", count, project_path)
            }],
            "isError": false
        }),
        Err(e) => {
            warn!(error = %e, "remove_project failed");
            json!({
                "content": [{"type": "text", "text": format!("Erreur lors de la suppression: {}", e)}],
                "isError": true
            })
        }
    }
}
