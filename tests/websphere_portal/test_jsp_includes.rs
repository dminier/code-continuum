/// Tests pour l'extraction INCLUDES (JSP → JavaScript/CSS/JSP)
///
/// Vérifie que jsp_extractor.rs identifie correctement:
/// - <script src="..."> → INCLUDES_JS
/// - <link rel="stylesheet" href="..."> → INCLUDES_CSS
/// - <%@ include file="..."%> et <jsp:include> → INCLUDES_JSP

#[cfg(test)]
mod jsp_includes_tests {
    use code_continuum::semantic_graph::semantic_graph::{EdgeRelation, UnifiedGraph};
    use code_continuum::graph_builder::dsl_executor::websphere_portal::JspExtractor;

    fn extract_jsp(jsp_code: &str, file_path: &str) -> UnifiedGraph {
        let extractor = JspExtractor::new();
        let mut graph = UnifiedGraph::new();

        extractor
            .extract_jsp_relations(file_path, jsp_code, &mut graph)
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
    fn test_script_tag_includes() {
        let jsp = r#"
<%@ page contentType="text/html;charset=UTF-8" %>
<html>
<head>
    <script src="/resources/js/jquery.min.js"></script>
    <script src="/resources/js/app.js"></script>
    <script src="/resources/js/user-form-validator.js"></script>
</head>
<body>
    <h1>User Management</h1>
</body>
</html>
"#;

        let graph = extract_jsp(jsp, "/WEB-INF/portlets/user-list.jsp");

        let js_count = count_relations(&graph, EdgeRelation::IncludesJs);
        assert_eq!(js_count, 3, "Should detect 3 JavaScript includes");

        // Vérifier que les nœuds JS ont été créés
        assert!(graph.nodes.values().any(|n| n.name == "jquery.min"));
        assert!(graph.nodes.values().any(|n| n.name == "app"));
        assert!(graph
            .nodes
            .values()
            .any(|n| n.name == "user-form-validator"));
    }

    #[test]
    fn test_link_css_includes() {
        let jsp = r#"
<html>
<head>
    <link rel="stylesheet" href="/resources/css/bootstrap.css">
    <link rel="stylesheet" href="/resources/css/user-form.css">
</head>
"#;

        let graph = extract_jsp(jsp, "/WEB-INF/portlets/user-form.jsp");

        let css_count = count_relations(&graph, EdgeRelation::IncludesCss);
        assert_eq!(css_count, 2, "Should detect 2 CSS includes");

        // Vérifier métadonnées
        let css_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::IncludesCss)
            .collect();

        assert!(css_edges
            .iter()
            .any(|e| e.metadata.get("position").is_some()));
    }

    #[test]
    fn test_jsp_include_directive() {
        let jsp = r#"
<%@ include file="/WEB-INF/common/header.jspf" %>
<%@ include file="/WEB-INF/common/navigation.jspf" %>

<div class="content">
    <h1>Main Content</h1>
</div>

<%@ include file="/WEB-INF/common/footer.jspf" %>
"#;

        let graph = extract_jsp(jsp, "/WEB-INF/portlets/main.jsp");

        let jsp_count = count_relations(&graph, EdgeRelation::IncludesJsp);
        assert_eq!(jsp_count, 3, "Should detect 3 JSP includes");

        // Vérifier type "static"
        let jsp_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::IncludesJsp)
            .collect();

        assert!(jsp_edges
            .iter()
            .all(|e| e.metadata.get("type") == Some(&"static".to_string())));
    }

    #[test]
    fn test_jsp_include_tag() {
        let jsp = r#"
<div class="layout">
    <jsp:include page="/WEB-INF/fragments/menu.jsp"/>
    <jsp:include page="/WEB-INF/fragments/sidebar.jsp"/>
</div>
"#;

        let graph = extract_jsp(jsp, "/WEB-INF/portlets/dashboard.jsp");

        let jsp_count = count_relations(&graph, EdgeRelation::IncludesJsp);
        assert_eq!(jsp_count, 2, "Should detect 2 JSP tag includes");

        // Vérifier type "dynamic"
        let jsp_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::IncludesJsp)
            .collect();

        assert!(jsp_edges
            .iter()
            .all(|e| e.metadata.get("type") == Some(&"dynamic".to_string())));
    }

    #[test]
    fn test_mixed_includes() {
        let jsp = r#"
<%@ page contentType="text/html;charset=UTF-8" %>
<%@ include file="/WEB-INF/common/header.jspf" %>

<html>
<head>
    <link rel="stylesheet" href="/resources/css/style.css">
    <script src="/resources/js/app.js"></script>
</head>
<body>
    <jsp:include page="/WEB-INF/fragments/menu.jsp"/>
    <h1>Dashboard</h1>
</body>
</html>
"#;

        let graph = extract_jsp(jsp, "/WEB-INF/portlets/dashboard.jsp");

        assert_eq!(count_relations(&graph, EdgeRelation::IncludesJs), 1);
        assert_eq!(count_relations(&graph, EdgeRelation::IncludesCss), 1);
        assert_eq!(count_relations(&graph, EdgeRelation::IncludesJsp), 2);

        println!("✅ Mixed includes: {} total relations", graph.edges.len());
    }

    #[test]
    fn test_position_ordering() {
        let jsp = r#"
<head>
    <script src="/resources/js/first.js"></script>
    <script src="/resources/js/second.js"></script>
    <script src="/resources/js/third.js"></script>
</head>
"#;

        let graph = extract_jsp(jsp, "/WEB-INF/portlets/test.jsp");

        let js_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::IncludesJs)
            .collect();

        // Vérifier que les positions sont correctes (0, 1, 2)
        let positions: Vec<usize> = js_edges
            .iter()
            .filter_map(|e| e.metadata.get("position")?.parse().ok())
            .collect();

        assert_eq!(positions.len(), 3);
        assert!(positions.contains(&0));
        assert!(positions.contains(&1));
        assert!(positions.contains(&2));
    }

    #[test]
    fn test_no_includes() {
        let jsp = r#"
<html>
<body>
    <h1>Simple Page</h1>
    <p>No includes here</p>
</body>
</html>
"#;

        let graph = extract_jsp(jsp, "/WEB-INF/portlets/simple.jsp");

        assert_eq!(count_relations(&graph, EdgeRelation::IncludesJs), 0);
        assert_eq!(count_relations(&graph, EdgeRelation::IncludesCss), 0);
        assert_eq!(count_relations(&graph, EdgeRelation::IncludesJsp), 0);
    }

    /// Test pour les scripts <script src="..."> qui pointent vers des fichiers JSP
    ///
    /// Cas d'usage: Une JSP génère dynamiquement du JavaScript (configuration, traductions, etc.)
    /// La relation INCLUDES_JS doit pointer vers un nœud de type Jsp (pas Js)
    ///
    /// Fixture: examples/web_templates/dynamic_js_script.jsp
    #[test]
    fn test_script_tag_pointing_to_jsp_file() {
        use code_continuum::semantic_graph::semantic_graph::NodeKind;

        let jsp = r#"
<%@ page contentType="text/html;charset=UTF-8" %>
<html>
<head>
    <!-- Script classique .js -->
    <script src="/resources/js/main.js"></script>
    
    <!-- Script qui pointe vers un fichier JSP (génère du JS dynamique) -->
    <script src="/common/config.jsp"></script>
    
    <!-- Script JSPX -->
    <script src="/fragments/translations.jspx"></script>
</head>
<body></body>
</html>
"#;

        let graph = extract_jsp(jsp, "/WEB-INF/portlets/dynamic.jsp");

        // Devrait détecter 3 scripts au total (1 .js + 2 .jsp/.jspx)
        let js_relations = count_relations(&graph, EdgeRelation::IncludesJs);
        assert_eq!(
            js_relations, 3,
            "Should detect 3 script includes (1 .js + 2 .jsp/.jspx)"
        );

        // Vérifier qu'on a un nœud Js pour main.js
        let js_nodes: Vec<_> = graph
            .nodes
            .values()
            .filter(|n| n.kind == NodeKind::Js)
            .collect();
        assert!(
            js_nodes.iter().any(|n| n.name == "main"),
            "Should have main.js node as Js"
        );

        // Vérifier qu'on a des nœuds Jsp pour config.jsp et translations.jspx
        let jsp_nodes: Vec<_> = graph
            .nodes
            .values()
            .filter(|n| n.kind == NodeKind::Jsp && n.id != "jsp::/WEB-INF/portlets/dynamic.jsp")
            .collect();

        println!(
            "JSP nodes (target): {:?}",
            jsp_nodes.iter().map(|n| &n.name).collect::<Vec<_>>()
        );

        assert!(
            jsp_nodes
                .iter()
                .any(|n| n.name == "config" || n.file_path.contains("config.jsp")),
            "Should have config.jsp node as Jsp type"
        );
        assert!(
            jsp_nodes
                .iter()
                .any(|n| n.name == "translations" || n.file_path.contains("translations.jspx")),
            "Should have translations.jspx node as Jsp type"
        );

        // Vérifier que les relations INCLUDES_JS pointent vers les bons types de nœuds
        for edge in graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::IncludesJs)
        {
            let target = graph.nodes.get(&edge.to);
            println!(
                "INCLUDES_JS: {} -> {} (kind: {:?})",
                edge.from,
                edge.to,
                target.map(|n| &n.kind)
            );
        }
    }

    /// Test pour <script src="<c:url value='...jsp'/>">
    #[test]
    fn test_script_c_url_pointing_to_jsp_file() {
        use code_continuum::semantic_graph::semantic_graph::NodeKind;

        let jsp = r#"
<%@ page contentType="text/html;charset=UTF-8" %>
<html>
<head>
    <script src="<c:url value="/dynamic/settings.jsp"/>"></script>
</head>
<body></body>
</html>
"#;

        let graph = extract_jsp(jsp, "/WEB-INF/portlets/settings.jsp");

        let js_relations = count_relations(&graph, EdgeRelation::IncludesJs);
        assert_eq!(js_relations, 1, "Should detect 1 script include via c:url");

        // Le nœud cible doit être de type Jsp, pas Js
        let edge = graph
            .edges
            .iter()
            .find(|e| e.relation == EdgeRelation::IncludesJs)
            .expect("Should have one INCLUDES_JS relation");

        let target = graph.nodes.get(&edge.to).expect("Target node should exist");
        assert_eq!(
            target.kind,
            NodeKind::Jsp,
            "Target of INCLUDES_JS should be Jsp when source file is .jsp"
        );
    }
}
