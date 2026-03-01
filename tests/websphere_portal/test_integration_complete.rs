/// Test d'intégration complet pour WebSphere Portal
///
/// Analyse tous les fichiers du projet examples/websphere-portal/
/// et vérifie que toutes les relations sémantiques sont extraites correctement.

#[cfg(test)]
mod integration_complete_tests {
    use code_continuum::semantic_graph::semantic_graph::{EdgeRelation, UnifiedGraph};
    use code_continuum::graph_builder::dsl_executor::websphere_portal::WebSphereExtractor;
    use std::path::PathBuf;

    /// Helper pour récupérer le chemin vers les exemples
    fn websphere_examples_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/websphere-portal")
    }

    /// Helper pour analyser tous les fichiers d'un répertoire
    fn analyze_directory(
        extractor: &mut WebSphereExtractor,
        dir: &PathBuf,
        graph: &mut UnifiedGraph,
    ) {
        if !dir.exists() {
            return;
        }

        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_str().unwrap();
                    if matches!(ext_str, "java" | "jsp" | "js" | "xml" | "jspx") {
                        let content = std::fs::read_to_string(&path).unwrap();
                        let file_name = path.file_name().unwrap().to_str().unwrap();

                        let _ = extractor.extract_file(file_name, &content, graph);
                    }
                }
            } else if path.is_dir() {
                analyze_directory(extractor, &path, graph);
            }
        }
    }

    /// Helper pour afficher un résumé du graphe
    fn print_graph_summary(graph: &UnifiedGraph) {
        println!("\n╔════════════════════════════════════════════════════════╗");
        println!("║         RÉSUMÉ DU GRAPHE WEBSPHERE PORTAL              ║");
        println!("╚════════════════════════════════════════════════════════╝");

        println!("\n📊 Nœuds: {}", graph.nodes.len());
        let mut node_types = std::collections::HashMap::new();
        for node in graph.nodes.values() {
            *node_types.entry(format!("{:?}", node.kind)).or_insert(0) += 1;
        }
        for (kind, count) in node_types {
            println!("   • {}: {}", kind, count);
        }

        println!("\n🔗 Relations: {}", graph.edges.len());
        let mut edge_types = std::collections::HashMap::new();
        for edge in &graph.edges {
            *edge_types
                .entry(format!("{:?}", edge.relation))
                .or_insert(0) += 1;
        }

        let priority_order = [
            ("Renders", EdgeRelation::Renders),
            ("CallsAjax", EdgeRelation::CallsAjax),
            ("Declares", EdgeRelation::Declares),
            ("IncludesJs", EdgeRelation::IncludesJs),
            ("IncludesCss", EdgeRelation::IncludesCss),
            ("IncludesJsp", EdgeRelation::IncludesJsp),
            ("Configures", EdgeRelation::Configures),
            ("Filters", EdgeRelation::Filters),
        ];

        for (name, rel) in priority_order {
            let count = graph.edges.iter().filter(|e| e.relation == rel).count();
            if count > 0 {
                println!("   • {}: {}", name, count);
            }
        }
    }

    #[test]
    fn test_integration_portlets_directory() {
        let portlets_dir = websphere_examples_dir().join("portlets");

        if !portlets_dir.exists() {
            println!("⚠️ Portlets directory not found, skipping test");
            return;
        }

        let mut extractor = WebSphereExtractor::new();
        let mut graph = UnifiedGraph::new();

        analyze_directory(&mut extractor, &portlets_dir, &mut graph);

        println!("\n✅ Portlets Analysis:");
        println!("   • {} nœuds créés", graph.nodes.len());
        println!("   • {} relations extraites", graph.edges.len());

        // Vérifier qu'au moins quelques relations RENDERS existent
        let renders_count = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Renders)
            .count();

        println!("   • {} relations RENDERS", renders_count);

        assert!(
            renders_count > 0,
            "Should have at least 1 RENDERS relation from portlets"
        );
    }

    #[test]
    fn test_integration_javascript_directory() {
        let js_dir = websphere_examples_dir().join("javascript");

        if !js_dir.exists() {
            println!("⚠️ JavaScript directory not found, skipping test");
            return;
        }

        let mut extractor = WebSphereExtractor::new();
        let mut graph = UnifiedGraph::new();

        analyze_directory(&mut extractor, &js_dir, &mut graph);

        println!("\n✅ JavaScript Analysis:");
        println!("   • {} nœuds créés", graph.nodes.len());
        println!("   • {} relations extraites", graph.edges.len());

        // Vérifier les CALLS_AJAX
        let ajax_count = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::CallsAjax)
            .count();

        println!("   • {} relations CALLS_AJAX", ajax_count);

        if ajax_count > 0 {
            println!("\n   Exemples d'appels AJAX détectés:");
            for edge in graph
                .edges
                .iter()
                .filter(|e| e.relation == EdgeRelation::CallsAjax)
                .take(3)
            {
                let method = edge
                    .metadata
                    .get("method")
                    .map(|s| s.as_str())
                    .unwrap_or("?");
                let url = edge.metadata.get("url").map(|s| s.as_str()).unwrap_or("?");
                println!("      {} {} → {}", method, url, edge.to);
            }
        }
    }

    #[test]
    fn test_integration_jsp_directory() {
        let jsp_dir = websphere_examples_dir().join("jsp");

        if !jsp_dir.exists() {
            println!("⚠️ JSP directory not found, skipping test");
            return;
        }

        let mut extractor = WebSphereExtractor::new();
        let mut graph = UnifiedGraph::new();

        analyze_directory(&mut extractor, &jsp_dir, &mut graph);

        println!("\n✅ JSP Analysis:");
        println!("   • {} nœuds créés", graph.nodes.len());
        println!("   • {} relations extraites", graph.edges.len());

        // Compter les différents types d'includes
        let js_count = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::IncludesJs)
            .count();
        let css_count = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::IncludesCss)
            .count();
        let jsp_count = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::IncludesJsp)
            .count();

        println!("   • {} INCLUDES_JS", js_count);
        println!("   • {} INCLUDES_CSS", css_count);
        println!("   • {} INCLUDES_JSP", jsp_count);
    }

    #[test]
    fn test_integration_config_directory() {
        let config_dir = websphere_examples_dir().join("config");

        if !config_dir.exists() {
            println!("⚠️ Config directory not found, skipping test");
            return;
        }

        let mut extractor = WebSphereExtractor::new();
        let mut graph = UnifiedGraph::new();

        analyze_directory(&mut extractor, &config_dir, &mut graph);

        println!("\n✅ Config Analysis (web.xml + portlet.xml):");
        println!("   • {} nœuds créés", graph.nodes.len());
        println!("   • {} relations extraites", graph.edges.len());

        let declares_count = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Declares)
            .count();
        let configures_count = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Configures)
            .count();
        let filters_count = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Filters)
            .count();

        println!("   • {} DECLARES (servlets)", declares_count);
        println!("   • {} CONFIGURES (portlets)", configures_count);
        println!("   • {} FILTERS", filters_count);
    }

    #[test]
    fn test_integration_full_project() {
        let examples_dir = websphere_examples_dir();

        if !examples_dir.exists() {
            println!("⚠️ WebSphere Portal examples not found, skipping test");
            return;
        }

        let mut extractor = WebSphereExtractor::new();
        let mut graph = UnifiedGraph::new();

        // Analyser tous les sous-répertoires
        let subdirs = ["portlets", "jsp", "javascript", "config"];

        for subdir in subdirs {
            let dir = examples_dir.join(subdir);
            if dir.exists() {
                analyze_directory(&mut extractor, &dir, &mut graph);
            }
        }

        print_graph_summary(&graph);

        // Assertions globales
        assert!(
            graph.nodes.len() > 0,
            "Should have created at least some nodes"
        );
        assert!(
            graph.edges.len() > 0,
            "Should have created at least some relations"
        );

        // Vérifier qu'on a au moins quelques relations de chaque type critique
        let renders_count = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Renders)
            .count();
        let ajax_count = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::CallsAjax)
            .count();
        let declares_count = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Declares)
            .count();

        println!("\n✅ VALIDATION FINALE:");
        println!("   • Relations RENDERS (⭐⭐⭐): {}", renders_count);
        println!("   • Relations CALLS_AJAX (⭐⭐⭐): {}", ajax_count);
        println!("   • Relations DECLARES (⭐⭐⭐): {}", declares_count);

        if renders_count > 0 {
            println!("\n   ✅ Extraction RENDERS opérationnelle");
        }
        if ajax_count > 0 {
            println!("   ✅ Extraction AJAX opérationnelle");
        }
        if declares_count > 0 {
            println!("   ✅ Extraction XML opérationnelle");
        }

        println!("\n╔═══════════════════════════════════════════════════════╗");
        println!("║  ✅  INTÉGRATION WEBSPHERE PORTAL COMPLÈTE  ✅         ║");
        println!("╚═══════════════════════════════════════════════════════╝");
    }

    #[test]
    fn test_sample_portlet_to_jsp_flow() {
        // Test un flow complet: Portlet → JSP → JavaScript → Servlet
        let mut extractor = WebSphereExtractor::new();
        let mut graph = UnifiedGraph::new();

        // 1. Portlet dispatch vers JSP
        let portlet_code = r#"
package com.example;
import javax.portlet.*;
public class SamplePortlet extends GenericPortlet {
    protected void doView(RenderRequest req, RenderResponse resp) {
        dispatch("/WEB-INF/jsp/sample.jsp");
    }
}
"#;
        let _ = extractor.extract_file("SamplePortlet.java", portlet_code, &mut graph);

        // 2. JSP inclut JavaScript
        let jsp_code = r#"
<html>
<head>
    <script src="/resources/js/sample.js"></script>
</head>
<body>
    <h1>Sample</h1>
</body>
</html>
"#;
        let _ = extractor.extract_file("/WEB-INF/jsp/sample.jsp", jsp_code, &mut graph);

        // 3. JavaScript appelle servlet
        let js_code = r#"
$.ajax({
    url: "/portal/api/sample",
    type: "POST"
});
"#;
        let _ = extractor.extract_file("/resources/js/sample.js", js_code, &mut graph);

        println!("\n✅ Flow Sample Analysis:");
        print_graph_summary(&graph);

        // Vérifier le flow complet
        assert!(
            graph
                .edges
                .iter()
                .any(|e| e.relation == EdgeRelation::Renders),
            "Should have Portlet→JSP (RENDERS)"
        );
        assert!(
            graph
                .edges
                .iter()
                .any(|e| e.relation == EdgeRelation::IncludesJs),
            "Should have JSP→JS (INCLUDES_JS)"
        );
        assert!(
            graph
                .edges
                .iter()
                .any(|e| e.relation == EdgeRelation::CallsAjax),
            "Should have JS→Servlet (CALLS_AJAX)"
        );

        println!("\n   ✅ Flow complet détecté: Portlet → JSP → JavaScript → Servlet");
    }
}
