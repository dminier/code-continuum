/// Tests pour l'extraction RENDERS (Portlet → JSP)
///
/// Vérifie que portlet_extractor.rs identifie correctement les appels dispatch()
/// et crée les relations RENDERS avec les métadonnées appropriées.

#[cfg(test)]
mod portlet_jsp_tests {
    use code_continuum::semantic_graph::semantic_graph::{EdgeRelation, UnifiedGraph};
    use code_continuum::graph_builder::dsl_executor::websphere_portal::PortletExtractor;

    /// Helper pour créer un graphe et extraire un Portlet
    fn extract_portlet(portlet_code: &str, file_path: &str) -> UnifiedGraph {
        let mut extractor = PortletExtractor::new();
        let mut graph = UnifiedGraph::new();

        extractor
            .extract_portlet_relations(file_path, portlet_code, &mut graph)
            .expect("Extraction should succeed");

        graph
    }

    /// Helper pour compter les relations d'un type donné
    fn count_relations(graph: &UnifiedGraph, relation: EdgeRelation) -> usize {
        graph
            .edges
            .iter()
            .filter(|e| e.relation == relation)
            .count()
    }

    #[test]
    fn test_simple_dispatch_view() {
        let code = r#"
package com.example.portlets;

import javax.portlet.*;
import java.io.IOException;

public class UserManagementPortlet extends GenericPortlet {
    
    @Override
    protected void doView(RenderRequest request, RenderResponse response) 
            throws PortletException, IOException {
        dispatch("/WEB-INF/portlets/user-list.jsp");
    }
}
"#;

        let graph = extract_portlet(code, "UserManagementPortlet.java");

        // Vérifier qu'un nœud Portlet a été créé
        assert!(
            graph
                .nodes
                .values()
                .any(|n| n.name == "UserManagementPortlet"),
            "Portlet node should be created"
        );

        // Vérifier qu'un nœud JSP a été créé
        assert!(
            graph.nodes.values().any(|n| n.name == "user-list"),
            "JSP node should be created"
        );

        // Vérifier qu'une relation RENDERS existe
        let renders_count = count_relations(&graph, EdgeRelation::Renders);
        assert_eq!(renders_count, 1, "Should have exactly 1 RENDERS relation");

        // Vérifier les métadonnées
        let renders_edge = graph
            .edges
            .iter()
            .find(|e| e.relation == EdgeRelation::Renders)
            .expect("RENDERS edge should exist");

        assert_eq!(
            renders_edge.metadata.get("mode"),
            Some(&"view".to_string()),
            "Mode should be 'view'"
        );
        assert_eq!(
            renders_edge.metadata.get("method"),
            Some(&"doView".to_string()),
            "Method should be 'doView'"
        );
    }

    #[test]
    fn test_multiple_dispatches_different_modes() {
        let code = r#"
package com.example.portlets;

import javax.portlet.*;

public class DocumentPortlet extends GenericPortlet {
    
    @Override
    protected void doView(RenderRequest request, RenderResponse response) {
        dispatch("/WEB-INF/portlets/document-list.jsp");
    }
    
    @Override
    protected void doEdit(RenderRequest request, RenderResponse response) {
        dispatch("/WEB-INF/portlets/document-edit.jsp");
    }
}
"#;

        let graph = extract_portlet(code, "DocumentPortlet.java");

        // Vérifier 2 relations RENDERS
        let renders_count = count_relations(&graph, EdgeRelation::Renders);
        assert_eq!(renders_count, 2, "Should have 2 RENDERS relations");

        // Vérifier les modes
        let view_edge = graph
            .edges
            .iter()
            .find(|e| {
                e.relation == EdgeRelation::Renders
                    && e.metadata.get("mode") == Some(&"view".to_string())
            })
            .expect("View RENDERS should exist");

        let edit_edge = graph
            .edges
            .iter()
            .find(|e| {
                e.relation == EdgeRelation::Renders
                    && e.metadata.get("mode") == Some(&"edit".to_string())
            })
            .expect("Edit RENDERS should exist");

        assert!(view_edge.to.contains("document-list"));
        assert!(edit_edge.to.contains("document-edit"));
    }

    #[test]
    fn test_dispatch_with_full_qualified_class() {
        let code = r#"
package com.example.portlets;

public class DashboardPortlet extends javax.portlet.GenericPortlet {
    
    protected void doView(javax.portlet.RenderRequest request, 
                         javax.portlet.RenderResponse response) {
        this.getPortletContext()
            .getRequestDispatcher("/WEB-INF/portlets/dashboard.jsp")
            .include(request, response);
        
        // Simple dispatch aussi
        dispatch("/WEB-INF/portlets/widgets.jsp");
    }
}
"#;

        let graph = extract_portlet(code, "DashboardPortlet.java");

        // Au moins 1 dispatch détecté (le simple)
        let renders_count = count_relations(&graph, EdgeRelation::Renders);
        assert!(
            renders_count >= 1,
            "Should detect at least the simple dispatch"
        );
    }

    #[test]
    fn test_real_example_usermanagement() {
        // Charger le vrai fichier d'exemple
        let example_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("examples/websphere-portal/portlets/UserManagementPortlet.java");

        if example_path.exists() {
            let code = std::fs::read_to_string(&example_path).expect("Should read example file");

            let graph = extract_portlet(&code, "UserManagementPortlet.java");

            // Le fichier exemple devrait avoir au moins 1 dispatch
            let renders_count = count_relations(&graph, EdgeRelation::Renders);
            assert!(
                renders_count > 0,
                "Example file should have RENDERS relations"
            );

            println!(
                "✅ UserManagementPortlet: {} RENDERS relations found",
                renders_count
            );
        } else {
            println!("⚠️ Example file not found, skipping real example test");
        }
    }

    #[test]
    fn test_no_dispatch_no_relations() {
        let code = r#"
package com.example.portlets;

public class EmptyPortlet extends GenericPortlet {
    // Pas de dispatch
}
"#;

        let graph = extract_portlet(code, "EmptyPortlet.java");

        let renders_count = count_relations(&graph, EdgeRelation::Renders);
        assert_eq!(renders_count, 0, "No dispatch = no RENDERS relations");
    }
}
