// Test E2E : Export de servlets avec url-pattern vers Neo4j
//
// Fixture: web.xml avec servlet + servlet-mapping
// Scénario:
// - Given: Un web.xml avec RechercheServlet et son url-pattern
// - When: On extrait et exporte vers Neo4j
// - Then: Le nœud Servlet doit avoir la propriété url_pattern dans Neo4j

use code_continuum::graph_builder::dsl_executor::websphere_portal::XmlExtractor;
use code_continuum::semantic_graph::neo4j_exporter::Neo4jExporter;
use code_continuum::semantic_graph::semantic_graph::{NodeKind, UnifiedGraph};
use neo4rs::{query, Graph};

#[tokio::test]
async fn test_servlet_url_pattern_exported_to_neo4j() {
    // Vérifier la connexion Neo4j
    let neo4j_available = check_neo4j_connection().await;
    if !neo4j_available {
        println!("⚠️ Neo4j non disponible, test ignoré");
        return;
    }

    let web_xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<web-app>
    <servlet>
        <servlet-name>RechercheServlet</servlet-name>
        <servlet-class>com.example.servlet.RechercheServlet</servlet-class>
    </servlet>
    
    <servlet-mapping>
        <servlet-name>RechercheServlet</servlet-name>
        <url-pattern>/Recherche.srv</url-pattern>
    </servlet-mapping>
    
    <servlet>
        <servlet-name>TarifPjServlet</servlet-name>
        <servlet-class>com.example.servlet.TarifPjServlet</servlet-class>
    </servlet>
    
    <servlet-mapping>
        <servlet-name>TarifPjServlet</servlet-name>
        <url-pattern>/TarifPjServlet.srv</url-pattern>
    </servlet-mapping>
</web-app>
"#;

    // 1. Extraire le graphe depuis web.xml
    let mut graph = UnifiedGraph::new();
    let extractor = XmlExtractor::new();

    extractor
        .extract_web_xml("config/web.xml", web_xml_content, &mut graph)
        .expect("Should extract web.xml");

    // Vérifier que le graphe contient bien les servlets avec url-pattern
    let recherche = graph
        .nodes
        .get("servlet::RechercheServlet")
        .expect("RechercheServlet should exist");

    assert_eq!(recherche.kind, NodeKind::Servlet);
    assert_eq!(
        recherche.metadata.get("url-pattern"),
        Some(&"/Recherche.srv".to_string()),
        "Servlet should have url-pattern in graph"
    );

    // 2. Exporter vers Neo4j
    let exporter = Neo4jExporter::new().await.expect("Should connect to Neo4j");

    exporter
        .export_graph(&graph)
        .await
        .expect("Should export to Neo4j");

    println!("✅ Graphe exporté vers Neo4j");

    // 3. Requêter Neo4j pour vérifier que url_pattern est présent
    let neo4j_graph = connect_neo4j().await.expect("Should connect to Neo4j");

    let mut result = neo4j_graph
        .execute(query(
            "MATCH (s:Servlet {name: 'RechercheServlet'})
             RETURN s.id as id, s.name as name, s.url_pattern as url_pattern, s.class as class",
        ))
        .await
        .expect("Should execute query");

    if let Some(row) = result.next().await.expect("Should get row") {
        let id: String = row.get("id").expect("Should have id");
        let name: String = row.get("name").expect("Should have name");
        let url_pattern: String = row.get("url_pattern").expect("Should have url_pattern");
        let class: String = row.get("class").expect("Should have class");

        println!("✅ Servlet trouvé dans Neo4j:");
        println!("   ID: {}", id);
        println!("   Name: {}", name);
        println!("   URL Pattern: {}", url_pattern);
        println!("   Class: {}", class);

        assert_eq!(id, "servlet::RechercheServlet");
        assert_eq!(name, "RechercheServlet");
        assert_eq!(url_pattern, "/Recherche.srv");
        assert_eq!(class, "com.example.servlet.RechercheServlet");
    } else {
        panic!("RechercheServlet not found in Neo4j");
    }

    // Vérifier aussi TarifPjServlet
    let mut result2 = neo4j_graph
        .execute(query(
            "MATCH (s:Servlet {name: 'TarifPjServlet'})
             RETURN s.url_pattern as url_pattern",
        ))
        .await
        .expect("Should execute query");

    if let Some(row) = result2.next().await.expect("Should get row") {
        let url_pattern: String = row.get("url_pattern").expect("Should have url_pattern");
        println!("✅ TarifPjServlet URL Pattern: {}", url_pattern);
        assert_eq!(url_pattern, "/TarifPjServlet.srv");
    } else {
        panic!("TarifPjServlet not found in Neo4j");
    }

    println!("🎉 Test réussi: url-pattern correctement exporté vers Neo4j!");
}

async fn check_neo4j_connection() -> bool {
    connect_neo4j().await.is_ok()
}

async fn connect_neo4j() -> Result<Graph, Box<dyn std::error::Error>> {
    let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://neo4j:7687".to_string());
    let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());

    let graph = Graph::new(&uri, &user, &password).await?;
    Ok(graph)
}
