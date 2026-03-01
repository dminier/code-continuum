/// Tests pour l'extraction XML (web.xml et portlet.xml)
///
/// Vérifie que xml_extractor.rs identifie correctement:
/// - web.xml: servlets, filters, servlet-mapping
/// - portlet.xml: portlets, modes, init-params, cache

#[cfg(test)]
mod xml_parsing_tests {
    use code_continuum::semantic_graph::semantic_graph::{EdgeRelation, UnifiedGraph};
    use code_continuum::graph_builder::dsl_executor::websphere_portal::XmlExtractor;

    fn extract_web_xml(xml_code: &str) -> UnifiedGraph {
        let extractor = XmlExtractor::new();
        let mut graph = UnifiedGraph::new();

        extractor
            .extract_web_xml("web.xml", xml_code, &mut graph)
            .expect("Extraction should succeed");

        graph
    }

    fn extract_portlet_xml(xml_code: &str) -> UnifiedGraph {
        let extractor = XmlExtractor::new();
        let mut graph = UnifiedGraph::new();

        extractor
            .extract_portlet_xml("portlet.xml", xml_code, &mut graph)
            .expect("Extraction should succeed");

        graph
    }

    fn count_relations(graph: &UnifiedGraph, relation: EdgeRelation) -> usize {
        graph
            .edges
            .iter()
            .filter(|e| e.relation == relation)
            .count()
    }

    #[test]
    fn test_web_xml_servlet_declaration() {
        let xml = r#"
<?xml version="1.0" encoding="UTF-8"?>
<web-app>
    <servlet>
        <servlet-name>DispatcherServlet</servlet-name>
        <servlet-class>com.example.web.DispatcherServlet</servlet-class>
    </servlet>
    
    <servlet-mapping>
        <servlet-name>DispatcherServlet</servlet-name>
        <url-pattern>*.do</url-pattern>
    </servlet-mapping>
</web-app>
"#;

        let graph = extract_web_xml(xml);

        // Vérifier qu'un nœud Servlet a été créé
        assert!(graph.nodes.values().any(|n| n.name == "DispatcherServlet"));

        // Vérifier la relation DECLARES
        let declares_count = count_relations(&graph, EdgeRelation::Declares);
        assert_eq!(declares_count, 1, "Should have 1 DECLARES relation");
    }

    #[test]
    fn test_web_xml_multiple_servlets() {
        let xml = r#"
<web-app>
    <servlet>
        <servlet-name>UserServlet</servlet-name>
        <servlet-class>com.example.UserServlet</servlet-class>
    </servlet>
    
    <servlet>
        <servlet-name>DocumentServlet</servlet-name>
        <servlet-class>com.example.DocumentServlet</servlet-class>
    </servlet>
    
    <servlet>
        <servlet-name>WorkflowServlet</servlet-name>
        <servlet-class>com.example.WorkflowServlet</servlet-class>
    </servlet>
</web-app>
"#;

        let graph = extract_web_xml(xml);

        let declares_count = count_relations(&graph, EdgeRelation::Declares);
        assert_eq!(declares_count, 3, "Should have 3 servlet declarations");

        assert!(graph.nodes.values().any(|n| n.name == "UserServlet"));
        assert!(graph.nodes.values().any(|n| n.name == "DocumentServlet"));
        assert!(graph.nodes.values().any(|n| n.name == "WorkflowServlet"));
    }

    #[test]
    fn test_web_xml_filters() {
        let xml = r#"
<web-app>
    <filter>
        <filter-name>AuthenticationFilter</filter-name>
        <filter-class>com.example.filters.AuthenticationFilter</filter-class>
    </filter>
    
    <filter>
        <filter-name>LoggingFilter</filter-name>
        <filter-class>com.example.filters.LoggingFilter</filter-class>
    </filter>
    
    <filter-mapping>
        <filter-name>AuthenticationFilter</filter-name>
        <url-pattern>/*</url-pattern>
    </filter-mapping>
    
    <filter-mapping>
        <filter-name>LoggingFilter</filter-name>
        <url-pattern>/*</url-pattern>
    </filter-mapping>
</web-app>
"#;

        let graph = extract_web_xml(xml);

        // Vérifier les nœuds Filter
        assert!(graph
            .nodes
            .values()
            .any(|n| n.name == "AuthenticationFilter"));
        assert!(graph.nodes.values().any(|n| n.name == "LoggingFilter"));

        // Vérifier les relations FILTERS
        let filters_count = count_relations(&graph, EdgeRelation::Filters);
        assert_eq!(filters_count, 2, "Should have 2 filter mappings");

        // Vérifier l'ordre
        let filter_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Filters)
            .collect();

        let orders: Vec<usize> = filter_edges
            .iter()
            .filter_map(|e| e.metadata.get("order")?.parse().ok())
            .collect();

        assert!(orders.contains(&0));
        assert!(orders.contains(&1));
    }

    #[test]
    fn test_portlet_xml_simple() {
        let xml = r#"
<?xml version="1.0" encoding="UTF-8"?>
<portlet-app>
    <portlet>
        <portlet-name>UserManagementPortlet</portlet-name>
        <portlet-class>com.example.portlets.UserManagementPortlet</portlet-class>
        <supports>
            <mime-type>text/html</mime-type>
            <portlet-mode>view</portlet-mode>
            <portlet-mode>edit</portlet-mode>
        </supports>
    </portlet>
</portlet-app>
"#;

        let graph = extract_portlet_xml(xml);

        // Vérifier nœud Portlet
        assert!(graph
            .nodes
            .values()
            .any(|n| n.name == "UserManagementPortlet"));

        // Vérifier relation CONFIGURES
        let configures_count = count_relations(&graph, EdgeRelation::Configures);
        assert_eq!(configures_count, 1, "Should have 1 CONFIGURES relation");

        // Vérifier métadonnées
        let config_edge = graph
            .edges
            .iter()
            .find(|e| e.relation == EdgeRelation::Configures)
            .unwrap();

        let modes = config_edge.metadata.get("modes").unwrap();
        assert!(modes.contains("view"));
        assert!(modes.contains("edit"));
    }

    #[test]
    fn test_portlet_xml_with_params() {
        let xml = r#"
<portlet-app>
    <portlet>
        <portlet-name>DocumentPortlet</portlet-name>
        <portlet-class>com.example.portlets.DocumentPortlet</portlet-class>
        
        <init-param>
            <name>template-view</name>
            <value>/WEB-INF/portlets/document-list.jsp</value>
        </init-param>
        
        <init-param>
            <name>template-edit</name>
            <value>/WEB-INF/portlets/document-edit.jsp</value>
        </init-param>
        
        <init-param>
            <name>pageSize</name>
            <value>10</value>
        </init-param>
        
        <supports>
            <mime-type>text/html</mime-type>
            <portlet-mode>view</portlet-mode>
            <portlet-mode>edit</portlet-mode>
        </supports>
        
        <expiration-cache>300</expiration-cache>
    </portlet>
</portlet-app>
"#;

        let graph = extract_portlet_xml(xml);

        let config_edge = graph
            .edges
            .iter()
            .find(|e| e.relation == EdgeRelation::Configures)
            .unwrap();

        // Vérifier cache
        assert_eq!(
            config_edge.metadata.get("cacheSeconds"),
            Some(&"300".to_string())
        );

        // Vérifier template params
        assert!(config_edge
            .metadata
            .get("param_template-view")
            .is_some_and(|v| v.contains("document-list.jsp")));
    }

    #[test]
    fn test_portlet_xml_multiple_portlets() {
        let xml = r#"
<portlet-app>
    <portlet>
        <portlet-name>Portlet1</portlet-name>
        <portlet-class>com.example.Portlet1</portlet-class>
        <supports>
            <portlet-mode>view</portlet-mode>
        </supports>
    </portlet>
    
    <portlet>
        <portlet-name>Portlet2</portlet-name>
        <portlet-class>com.example.Portlet2</portlet-class>
        <supports>
            <portlet-mode>view</portlet-mode>
        </supports>
    </portlet>
</portlet-app>
"#;

        let graph = extract_portlet_xml(xml);

        let configures_count = count_relations(&graph, EdgeRelation::Configures);
        assert_eq!(configures_count, 2, "Should configure 2 portlets");
    }

    #[test]
    fn test_real_web_xml_example() {
        let example_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/websphere-portal/config/web.xml");

        if example_path.exists() {
            let xml = std::fs::read_to_string(&example_path).unwrap();
            let graph = extract_web_xml(&xml);

            let declares_count = count_relations(&graph, EdgeRelation::Declares);
            let filters_count = count_relations(&graph, EdgeRelation::Filters);

            println!(
                "✅ web.xml: {} servlets, {} filters",
                declares_count, filters_count
            );

            assert!(declares_count > 0, "Should have servlet declarations");
        }
    }

    #[test]
    fn test_real_portlet_xml_example() {
        let example_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/websphere-portal/config/portlet.xml");

        if example_path.exists() {
            let xml = std::fs::read_to_string(&example_path).unwrap();
            let graph = extract_portlet_xml(&xml);

            let configures_count = count_relations(&graph, EdgeRelation::Configures);

            println!("✅ portlet.xml: {} portlets configured", configures_count);

            assert!(configures_count > 0, "Should have portlet configurations");
        }
    }
}
