// Test de l'extraction des url-pattern depuis web.xml et relations IMPLEMENTED_BY

use code_continuum::graph_builder::dsl_executor::websphere_portal::XmlExtractor;
use code_continuum::semantic_graph::semantic_graph::{EdgeRelation, NodeKind, UnifiedGraph};

#[test]
fn test_servlet_url_pattern_extraction() {
    let web_xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<web-app>
    <servlet>
        <servlet-name>ErreurSigma</servlet-name>
        <servlet-class>com.example.servlets.ErreurSigmaServlet</servlet-class>
    </servlet>
    
    <servlet>
        <servlet-name>TarifPj</servlet-name>
        <servlet-class>com.example.servlets.TarifPjServlet</servlet-class>
    </servlet>
    
    <servlet-mapping>
        <servlet-name>ErreurSigma</servlet-name>
        <url-pattern>/ErreurSigma.srv</url-pattern>
    </servlet-mapping>
    
    <servlet-mapping>
        <servlet-name>TarifPj</servlet-name>
        <url-pattern>/TarifPjServlet.srv</url-pattern>
    </servlet-mapping>
</web-app>
"#;

    let mut graph = UnifiedGraph::new();
    let extractor = XmlExtractor::new();

    extractor
        .extract_web_xml("config/web.xml", web_xml_content, &mut graph)
        .expect("Should extract web.xml");

    // Vérifier que les servlets sont créés
    let erreur_servlet = graph
        .nodes
        .get("servlet::ErreurSigma")
        .expect("ErreurSigma servlet should exist");

    assert_eq!(erreur_servlet.kind, NodeKind::Servlet);
    assert_eq!(erreur_servlet.name, "ErreurSigma");

    // Vérifier que url-pattern est dans les métadonnées
    assert_eq!(
        erreur_servlet.metadata.get("url-pattern"),
        Some(&"/ErreurSigma.srv".to_string()),
        "Servlet should have url-pattern in metadata"
    );

    let tarif_servlet = graph
        .nodes
        .get("servlet::TarifPj")
        .expect("TarifPj servlet should exist");

    assert_eq!(
        tarif_servlet.metadata.get("url-pattern"),
        Some(&"/TarifPjServlet.srv".to_string()),
        "TarifPj should have url-pattern"
    );

    // Vérifier les relations IMPLEMENTED_BY
    let erreur_impl = graph.edges.iter().find(|e| {
        e.from == "servlet::ErreurSigma"
            && e.to == "com.example.servlets.ErreurSigmaServlet"
            && e.relation == EdgeRelation::ImplementedBy
    });
    assert!(
        erreur_impl.is_some(),
        "IMPLEMENTED_BY relation should exist for ErreurSigma"
    );

    let tarif_impl = graph.edges.iter().find(|e| {
        e.from == "servlet::TarifPj"
            && e.to == "com.example.servlets.TarifPjServlet"
            && e.relation == EdgeRelation::ImplementedBy
    });
    assert!(
        tarif_impl.is_some(),
        "IMPLEMENTED_BY relation should exist for TarifPj"
    );

    println!("✅ Servlets extraits avec url-pattern et relations IMPLEMENTED_BY");
}

#[test]
fn test_servlet_with_java_class_resolution() {
    // Test end-to-end : web.xml + classe Java servlet pour vérifier que IMPLEMENTED_BY pointe vers le bon nœud
    use code_continuum::graph_builder::MultiLanguageGraphBuilder;

    let web_xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<web-app>
    <servlet>
        <servlet-name>ErreurSigma</servlet-name>
        <servlet-class>com.example.ErreurSigmaServlet</servlet-class>
    </servlet>
    
    <servlet-mapping>
        <servlet-name>ErreurSigma</servlet-name>
        <url-pattern>/ErreurSigma.srv</url-pattern>
    </servlet-mapping>
</web-app>
"#;

    let java_servlet_code = r#"
package com.example;

import javax.servlet.http.HttpServlet;

public class ErreurSigmaServlet extends HttpServlet {
    public void doGet() {
        // handle error
    }
}
"#;

    let mut graph = UnifiedGraph::new();
    let extractor = XmlExtractor::new();

    // 1. Extraire web.xml
    extractor
        .extract_web_xml("config/web.xml", web_xml_content, &mut graph)
        .expect("Should extract web.xml");

    // 2. Extraire la classe Java du servlet
    let builder = MultiLanguageGraphBuilder::new();
    let lang = tree_sitter_java::language();
    let file_graph = builder
        .build_graph(
            "java",
            lang,
            java_servlet_code,
            "com/example/ErreurSigmaServlet.java",
        )
        .expect("Should build graph");

    for node in file_graph.nodes.values() {
        graph.add_node(node.clone());
    }
    for edge in file_graph.edges {
        graph.add_edge(edge);
    }

    // 3. Vérifier que le nœud Servlet existe avec url-pattern
    let servlet = graph
        .nodes
        .get("servlet::ErreurSigma")
        .expect("Servlet should exist");

    assert_eq!(
        servlet.metadata.get("url-pattern"),
        Some(&"/ErreurSigma.srv".to_string())
    );

    // 4. Vérifier que la classe Java existe
    let class = graph
        .nodes
        .get("com.example.ErreurSigmaServlet")
        .expect("Java class should exist");

    assert_eq!(class.kind, NodeKind::Class);
    assert_eq!(class.name, "ErreurSigmaServlet");
    assert_eq!(class.file_path, "com/example/ErreurSigmaServlet.java");

    // 5. Vérifier la relation IMPLEMENTED_BY : Servlet -> Class
    let impl_relation = graph.edges.iter().find(|e| {
        e.from == "servlet::ErreurSigma"
            && e.to == "com.example.ErreurSigmaServlet"
            && e.relation == EdgeRelation::ImplementedBy
    });

    assert!(
        impl_relation.is_some(),
        "IMPLEMENTED_BY relation should connect Servlet to Java Class"
    );

    println!("✅ Servlet URL pattern + IMPLEMENTED_BY -> Java Class fonctionne !");
    println!("   La requête Cypher suivante devrait fonctionner :");
    println!("   MATCH (s:Servlet)-[:IMPLEMENTED_BY]->(c:Class)");
    println!("   WHERE s.metadata.`url-pattern` = '/ErreurSigma.srv'");
    println!("   RETURN s.name, c.id, c.file_path");
}
