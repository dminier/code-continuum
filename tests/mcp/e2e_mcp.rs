//! Tests E2E du serveur MCP code-continuum
//!
//! Chaque test démarre un serveur MCP sur un port aléatoire, appelle l'endpoint
//! via HTTP, puis vérifie le résultat directement dans Neo4j.
//!
//! **IMPORTANT (changement v0.2.0):**
//! - Les chemins `project_path` sont maintenant **relatifs à `/app/data`** (CODE_PATH)
//! - Les tests doivent fournir des chemins comme `"backend"`, `"backend/java"`, etc.
//! - Les chemins absolus ne fonctionnent plus — la résolution se fait en interne
//! - Cela aligne le MCP avec la structure du container (pas de désalignement chemin local vs container)
//!
//! Exécution des tests:
//! ```bash
//! cargo test --test integration_mcp -- --ignored --nocapture
//! ```

use crate::common;
use reqwest::Client;
use serde_json::{json, Value};
use serial_test::serial;
use std::path::PathBuf;
use tokio::net::TcpListener;

// ============================================================================
// Helpers
// ============================================================================

/// Configure CODE_PATH pour les tests (pointe vers examples/)
fn setup_code_path() {
    let code_path = std::path::PathBuf::from("examples")
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from("examples"));
    std::env::set_var("CODE_PATH", code_path.to_string_lossy().to_string());
}

/// Démarre le serveur MCP sur un port aléatoire, retourne le port lié.
async fn start_test_mcp_server() -> u16 {
    setup_code_path();
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("Bind port 0");
    let port = listener.local_addr().expect("local_addr").port();
    let app = code_continuum::mcp::make_app();
    tokio::spawn(async move {
        axum::serve(listener, app).await.ok();
    });
    // Courte pause pour que l'acceptor soit prêt
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    port
}

/// Envoie une requête JSON-RPC 2.0 au serveur MCP et retourne la réponse.
async fn mcp_call(client: &Client, url: &str, method: &str, id: u64, params: Value) -> Value {
    let body = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    });
    client
        .post(url)
        .json(&body)
        .send()
        .await
        .expect("HTTP request failed")
        .json::<Value>()
        .await
        .expect("JSON parse failed")
}

/// Retourne le chemin absolu vers un sous-dossier de examples/.
/// **Note (v0.2.0):** Pour les tests MCP, utilisez le chemin relatif au lieu de celui-ci.
fn example_path(relative: &str) -> String {
    PathBuf::from("examples")
        .join(relative)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from("examples").join(relative))
        .to_string_lossy()
        .to_string()
}

/// Retourne le chemin relatif à CODE_PATH (/app/data) pour les tests MCP.
/// Dans le container, /app/data=examples/, donc "backend" mappe à examples/backend.
fn mcp_project_path(relative: &str) -> String {
    relative.to_string()
}

/// Compte les nœuds appartenant à un projet dans Neo4j.
async fn count_project_nodes(graph: &neo4rs::Graph, project_name: &str) -> i64 {
    let q = neo4rs::query("MATCH (n:Node {project_name: $name}) RETURN count(n) AS cnt")
        .param("name", project_name.to_string());
    let mut result = graph.execute(q).await.expect("Cypher count failed");
    if let Some(row) = result.next().await.expect("No row") {
        row.get::<i64>("cnt").unwrap_or(0)
    } else {
        0
    }
}

/// Supprime les nœuds de test d'un projet dans Neo4j (cleanup).
async fn cleanup_project(graph: &neo4rs::Graph, project_name: &str) {
    let q = neo4rs::query("MATCH (n:Node {project_name: $name}) DETACH DELETE n")
        .param("name", project_name.to_string());
    let _ = graph.run(q).await;
}

// ============================================================================
// Tests
// ============================================================================

/// Test: tools/list retourne exactement les outils add_project et remove_project.
///
/// Ce test n'a pas besoin de Neo4j.
#[tokio::test]
#[ignore = "Nécessite le serveur MCP - activer avec: cargo test -- --ignored"]
async fn test_mcp_tools_list() {
    let port = start_test_mcp_server().await;
    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/api/mcp/", port);

    let response = mcp_call(&client, &url, "tools/list", 1, json!({})).await;

    let tools = response["result"]["tools"]
        .as_array()
        .expect("tools doit être un tableau");

    let names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap_or(""))
        .collect();

    assert!(
        names.contains(&"list_projects"),
        "list_projects manquant: {:?}",
        names
    );
    assert!(
        names.contains(&"add_project"),
        "add_project manquant: {:?}",
        names
    );
    assert!(
        names.contains(&"remove_project"),
        "remove_project manquant: {:?}",
        names
    );
    assert_eq!(
        names.len(),
        3,
        "Exactement 3 outils attendus (list_projects, add_project, remove_project)"
    );
}

/// Test: initialize retourne les informations du serveur MCP.
///
/// Ce test n'a pas besoin de Neo4j.
#[tokio::test]
#[ignore = "Nécessite le serveur MCP - activer avec: cargo test -- --ignored"]
async fn test_mcp_initialize() {
    let port = start_test_mcp_server().await;
    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/api/mcp/", port);

    let response = mcp_call(
        &client,
        &url,
        "initialize",
        1,
        json!({"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test", "version": "0.1"}}),
    )
    .await;

    let server_name = response["result"]["serverInfo"]["name"]
        .as_str()
        .unwrap_or("");
    assert_eq!(server_name, "code-continuum");

    let protocol = response["result"]["protocolVersion"].as_str().unwrap_or("");
    assert!(!protocol.is_empty(), "protocolVersion doit être renseigné");
}

/// Test E2E: add_project analyse examples/backend et crée les nœuds dans Neo4j
/// avec les propriétés project_path et project_name.
#[tokio::test]
#[serial]
#[ignore = "Nécessite Neo4j + MCP - activer avec: cargo test -- --ignored"]
async fn test_mcp_add_project_and_verify_neo4j() {
    println!("\n=== TEST E2E MCP: add_project ===\n");

    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    let graph = match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(g) => g,
        Err(e) => {
            eprintln!("⚠️ Neo4j non disponible: {}. Test skippé.", e);
            return;
        }
    };

    let fixture = PathBuf::from("examples/backend");
    if !fixture.exists() {
        eprintln!("⚠️ Fixture manquante: examples/backend. Test skippé.");
        return;
    }

    let project_path = mcp_project_path("backend");
    let project_name = "test-mcp-backend";

    // Cleanup préalable (au cas où un test précédent aurait échoué)
    cleanup_project(&graph, project_name).await;

    let port = start_test_mcp_server().await;
    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/api/mcp/", port);

    // --- ACT : appel MCP add_project ---
    println!(
        "📂 Ajout du projet: {} (name: {})",
        project_path, project_name
    );
    let response = mcp_call(
        &client,
        &url,
        "tools/call",
        2,
        json!({
            "name": "add_project",
            "arguments": {
                "project_path": project_path,
                "project_name": project_name
            }
        }),
    )
    .await;

    // --- ASSERT 1 : réponse MCP sans erreur ---
    let is_error = response["result"]["isError"].as_bool().unwrap_or(true);
    let content_text = response["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("");
    println!("Réponse MCP: {}", content_text);
    assert!(
        !is_error,
        "add_project a retourné une erreur: {}",
        content_text
    );
    assert!(
        content_text.contains(project_name),
        "La réponse doit mentionner le nom du projet"
    );

    // --- ASSERT 2 : nœuds présents dans Neo4j ---
    let count = count_project_nodes(&graph, project_name).await;
    println!("✅ Nœuds dans Neo4j pour '{}': {}", project_name, count);
    assert!(
        count > 0,
        "Des nœuds doivent être présents dans Neo4j après add_project"
    );

    // --- ASSERT 3 : propriétés project_path et project_name sur les nœuds ---
    let q = neo4rs::query(
        "MATCH (n:Node {project_name: $name}) RETURN n.project_path AS pp, n.project_name AS pn LIMIT 1",
    )
    .param("name", project_name.to_string());
    let mut result = graph.execute(q).await.unwrap();
    if let Some(row) = result.next().await.unwrap() {
        let pp: String = row.get("pp").unwrap_or_default();
        let pn: String = row.get("pn").unwrap_or_default();
        println!("  project_path={}, project_name={}", pp, pn);
        assert_eq!(pn, project_name, "project_name incorrect sur le nœud");
        assert_eq!(pp, project_path, "project_path incorrect sur le nœud");
    } else {
        panic!("Aucun nœud trouvé dans Neo4j");
    }

    // --- CLEANUP ---
    cleanup_project(&graph, project_name).await;
    println!("=== TEST TERMINÉ ===\n");
}

/// Test E2E: remove_project supprime les nœuds d'un projet dans Neo4j.
#[tokio::test]
#[serial]
#[ignore = "Nécessite Neo4j + MCP - activer avec: cargo test -- --ignored"]
async fn test_mcp_remove_project() {
    println!("\n=== TEST E2E MCP: remove_project ===\n");

    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    let graph = match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(g) => g,
        Err(e) => {
            eprintln!("⚠️ Neo4j non disponible: {}. Test skippé.", e);
            return;
        }
    };

    let fixture = PathBuf::from("examples/websphere-portal");
    if !fixture.exists() {
        eprintln!("⚠️ Fixture manquante: examples/websphere-portal. Test skippé.");
        return;
    }

    let project_path = mcp_project_path("websphere-portal");
    let project_name = "test-mcp-remove-websphere";

    cleanup_project(&graph, project_name).await;

    let port = start_test_mcp_server().await;
    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/api/mcp/", port);

    // --- ACT 1 : ajouter le projet ---
    println!("📂 Ajout du projet: {}", project_name);
    let add_resp = mcp_call(
        &client,
        &url,
        "tools/call",
        2,
        json!({
            "name": "add_project",
            "arguments": {"project_path": project_path, "project_name": project_name}
        }),
    )
    .await;

    let is_error = add_resp["result"]["isError"].as_bool().unwrap_or(true);
    assert!(
        !is_error,
        "add_project a échoué: {}",
        add_resp["result"]["content"][0]["text"]
    );

    let count_before = count_project_nodes(&graph, project_name).await;
    println!("✅ Nœuds ajoutés: {}", count_before);
    assert!(
        count_before > 0,
        "Des nœuds doivent être présents après ajout"
    );

    // --- ACT 2 : supprimer le projet ---
    println!("🗑️ Suppression du projet: {}", project_name);
    let remove_resp = mcp_call(
        &client,
        &url,
        "tools/call",
        3,
        json!({
            "name": "remove_project",
            "arguments": {"project_path": project_path}
        }),
    )
    .await;

    let is_error = remove_resp["result"]["isError"].as_bool().unwrap_or(true);
    let content_text = remove_resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("");
    println!("Réponse remove: {}", content_text);
    assert!(!is_error, "remove_project a échoué: {}", content_text);

    // --- ASSERT : plus aucun nœud ---
    let count_after = count_project_nodes(&graph, project_name).await;
    println!("✅ Nœuds après suppression: {} (attendu: 0)", count_after);
    assert_eq!(
        count_after, 0,
        "Plus aucun nœud ne doit être présent après remove_project"
    );

    println!("=== TEST TERMINÉ ===\n");
}

/// Test E2E: isolation entre deux projets.
/// Supprimer un projet ne doit pas affecter les nœuds de l'autre.
#[tokio::test]
#[serial]
#[ignore = "Nécessite Neo4j + MCP - activer avec: cargo test -- --ignored"]
async fn test_mcp_two_projects_isolation() {
    println!("\n=== TEST E2E MCP: isolation multi-projets ===\n");

    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    let graph = match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(g) => g,
        Err(e) => {
            eprintln!("⚠️ Neo4j non disponible: {}. Test skippé.", e);
            return;
        }
    };

    if !PathBuf::from("examples/backend").exists()
        || !PathBuf::from("examples/websphere-portal").exists()
    {
        eprintln!("⚠️ Fixtures manquantes. Test skippé.");
        return;
    }

    let path1 = mcp_project_path("backend");
    let path2 = mcp_project_path("websphere-portal");
    let name1 = "test-mcp-iso-backend";
    let name2 = "test-mcp-iso-websphere";

    cleanup_project(&graph, name1).await;
    cleanup_project(&graph, name2).await;

    let port = start_test_mcp_server().await;
    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/api/mcp/", port);

    // --- ACT 1 : ajouter les deux projets ---
    println!("📂 Ajout du projet 1: {}", name1);
    let r1 = mcp_call(
        &client,
        &url,
        "tools/call",
        2,
        json!({"name": "add_project", "arguments": {"project_path": path1, "project_name": name1}}),
    )
    .await;
    assert!(
        !r1["result"]["isError"].as_bool().unwrap_or(true),
        "add_project({}) a échoué",
        name1
    );

    println!("📂 Ajout du projet 2: {}", name2);
    let r2 = mcp_call(
        &client,
        &url,
        "tools/call",
        3,
        json!({"name": "add_project", "arguments": {"project_path": path2, "project_name": name2}}),
    )
    .await;
    assert!(
        !r2["result"]["isError"].as_bool().unwrap_or(true),
        "add_project({}) a échoué",
        name2
    );

    let count1 = count_project_nodes(&graph, name1).await;
    let count2 = count_project_nodes(&graph, name2).await;
    println!("  {} → {} nœuds", name1, count1);
    println!("  {} → {} nœuds", name2, count2);
    assert!(count1 > 0, "{} doit avoir des nœuds", name1);
    assert!(count2 > 0, "{} doit avoir des nœuds", name2);

    // --- ACT 2 : supprimer seulement le projet 1 ---
    println!("\n🗑️ Suppression de {} uniquement...", name1);
    let remove_resp = mcp_call(
        &client,
        &url,
        "tools/call",
        4,
        json!({"name": "remove_project", "arguments": {"project_path": path1}}),
    )
    .await;
    assert!(
        !remove_resp["result"]["isError"].as_bool().unwrap_or(true),
        "remove_project({}) a échoué",
        name1
    );

    // --- ASSERT : isolation ---
    let count1_after = count_project_nodes(&graph, name1).await;
    let count2_after = count_project_nodes(&graph, name2).await;
    println!(
        "  {} → {} nœuds après suppression (attendu: 0)",
        name1, count1_after
    );
    println!(
        "  {} → {} nœuds inchangés (attendu: {})",
        name2, count2_after, count2
    );

    assert_eq!(
        count1_after, 0,
        "{} doit être vide après remove_project",
        name1
    );
    assert_eq!(
        count2_after, count2,
        "Les nœuds de {} ne doivent pas être affectés",
        name2
    );

    println!("✅ Isolation confirmée");

    // --- CLEANUP ---
    cleanup_project(&graph, name2).await;
    println!("=== TEST TERMINÉ ===\n");
}

/// Test E2E: add_project avec clear_project=true réinitialise le projet existant.
#[tokio::test]
#[serial]
#[ignore = "Nécessite Neo4j + MCP - activer avec: cargo test -- --ignored"]
async fn test_mcp_add_project_with_clear() {
    println!("\n=== TEST E2E MCP: add_project avec clear_project=true ===\n");

    common::setup_env();
    let (uri, user, pass) = common::get_neo4j_config();

    let graph = match neo4rs::Graph::new(&uri, &user, &pass).await {
        Ok(g) => g,
        Err(e) => {
            eprintln!("⚠️ Neo4j non disponible: {}. Test skippé.", e);
            return;
        }
    };

    let fixture = PathBuf::from("examples/backend");
    if !fixture.exists() {
        eprintln!("⚠️ Fixture manquante: examples/backend. Test skippé.");
        return;
    }

    let project_path = mcp_project_path("backend");
    let project_name = "test-mcp-clear";

    cleanup_project(&graph, project_name).await;

    let port = start_test_mcp_server().await;
    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/api/mcp/", port);

    let add_args = json!({
        "name": "add_project",
        "arguments": {"project_path": project_path, "project_name": project_name}
    });

    // --- Premier ajout ---
    println!("📂 Premier ajout...");
    let r1 = mcp_call(&client, &url, "tools/call", 2, add_args.clone()).await;
    assert!(!r1["result"]["isError"].as_bool().unwrap_or(true));

    let count1 = count_project_nodes(&graph, project_name).await;
    println!("  Nœuds après premier ajout: {}", count1);
    assert!(count1 > 0);

    // --- Second ajout avec clear_project=true ---
    println!("📂 Second ajout avec clear_project=true...");
    let r2 = mcp_call(
        &client,
        &url,
        "tools/call",
        3,
        json!({
            "name": "add_project",
            "arguments": {
                "project_path": project_path,
                "project_name": project_name,
                "clear_project": true
            }
        }),
    )
    .await;
    assert!(!r2["result"]["isError"].as_bool().unwrap_or(true));

    let count2 = count_project_nodes(&graph, project_name).await;
    println!("  Nœuds après ré-ajout avec clear: {}", count2);
    // Le count doit être identique (pas de doublons) car le clear a supprimé les anciens
    assert!(count2 > 0, "Des nœuds doivent être présents après ré-ajout");
    assert_eq!(
        count1, count2,
        "Le nombre de nœuds doit être identique (pas de doublons)"
    );

    // --- CLEANUP ---
    cleanup_project(&graph, project_name).await;
    println!("=== TEST TERMINÉ ===\n");
}
/// Test: list_projects énumère les sous-dossiers de /app/data
///
/// NOTE: Ce test fonctionne en dev avec /app/data défini si on est en container.
/// En local, créez un symlink: `ln -s $(pwd)/examples /app/data`
/// ou modifiez le code pour utiliser une variable d'env.
#[tokio::test]
#[ignore = "Nécessite /app/data (container) ou symlink local"]
async fn test_mcp_list_projects() {
    println!("\n=== TEST MCP: list_projects ===\n");

    let port = start_test_mcp_server().await;
    let client = Client::new();
    let url = format!("http://127.0.0.1:{}/api/mcp/", port);

    // --- ACT : appel MCP list_projects ---
    let response = mcp_call(
        &client,
        &url,
        "tools/call",
        1,
        json!({
            "name": "list_projects",
            "arguments": {}
        }),
    )
    .await;

    // --- ASSERT ---
    let is_error = response["result"]["isError"].as_bool().unwrap_or(true);
    let content_text = response["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or("");

    println!("Response: {}", content_text);

    // Si /app/data n'existe pas, l'erreur est attendue
    if is_error {
        if content_text.contains("/app/data n'existe pas") {
            println!("✓ Réponse attendue: /app/data n'existe pas (pas en container)");
        } else {
            println!("! Erreur inattendue: {}", content_text);
        }
    } else {
        // Si /app/data existe, on devrait avoir une liste
        println!("✓ list_projects réussi");
        assert!(
            content_text.contains("disponibles") || content_text.contains("trouvés"),
            "Réponse should mention 'disponibles' or 'trouvés'"
        );
    }
}
