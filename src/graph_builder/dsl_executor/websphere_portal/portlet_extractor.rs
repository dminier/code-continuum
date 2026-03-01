use super::relations::RendersMetadata;
/// Extracteur pour les relations Portlet → JSP (RENDERS)
///
/// Parse les Portlets Java pour identifier les dispatch() vers JSP.
/// Utilise Tree-Sitter pour l'analyse AST précise.
///
/// Pattern recherché:
/// ```java
/// protected void doView(RenderRequest request, RenderResponse response) {
///     dispatch("/WEB-INF/portlets/user-list.jsp");
/// }
/// ```
use crate::semantic_graph::semantic_graph::{
    EdgeRelation, Location, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};
use tracing::debug;
use tree_sitter::{Node, Parser};

#[allow(dead_code)]
pub struct PortletExtractor {
    parser: Parser,
}

impl PortletExtractor {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        parser
            .set_language(tree_sitter_java::language())
            .expect("Failed to load Java grammar");
        Self { parser }
    }

    /// Extrait les relations RENDERS depuis un fichier Portlet
    pub fn extract_portlet_relations(
        &mut self,
        file_path: &str,
        content: &str,
        graph: &mut UnifiedGraph,
    ) -> Result<(), String> {
        debug!(file = file_path, "Extracting Portlet RENDERS relations");

        let tree = self
            .parser
            .parse(content, None)
            .ok_or("Failed to parse Java file")?;

        let root = tree.root_node();

        // Trouver la classe Portlet
        let portlet_class = self
            .find_portlet_class(root, content)
            .ok_or("No Portlet class found")?;

        let portlet_name = self.get_class_name(portlet_class, content);

        // Créer le nœud Portlet dans le graphe
        let portlet_id = format!("{}::{}", file_path, portlet_name);
        graph.add_node(SemanticNode {
            id: portlet_id.clone(),
            kind: NodeKind::Class,
            name: portlet_name.clone(),
            file_path: file_path.to_string(),
            location: Location {
                start_line: portlet_class.start_position().row + 1,
                end_line: portlet_class.end_position().row + 1,
                start_col: portlet_class.start_position().column,
                end_col: portlet_class.end_position().column,
            },
            metadata: [("type".to_string(), "Portlet".to_string())]
                .iter()
                .cloned()
                .collect(),
        });

        // Rechercher les appels dispatch()
        self.extract_dispatch_calls(portlet_class, content, &portlet_id, file_path, graph)?;

        Ok(())
    }

    /// Trouve la classe Portlet principale dans l'AST
    fn find_portlet_class<'a>(&self, node: Node<'a>, _content: &str) -> Option<Node<'a>> {
        if node.kind() == "class_declaration" {
            return Some(node);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = self.find_portlet_class(child, _content) {
                return Some(found);
            }
        }

        None
    }

    /// Extrait le nom de la classe
    fn get_class_name(&self, class_node: Node, content: &str) -> String {
        if let Some(name_node) = class_node.child_by_field_name("name") {
            content[name_node.byte_range()].to_string()
        } else {
            "UnknownPortlet".to_string()
        }
    }

    /// Recherche les appels dispatch() dans les méthodes du Portlet
    fn extract_dispatch_calls(
        &self,
        class_node: Node,
        content: &str,
        portlet_id: &str,
        file_path: &str,
        graph: &mut UnifiedGraph,
    ) -> Result<(), String> {
        // Trouver le class_body
        let class_body = class_node
            .child_by_field_name("body")
            .ok_or("No class body found")?;

        let mut cursor = class_body.walk();

        for child in class_body.children(&mut cursor) {
            if child.kind() == "method_declaration" {
                let method_name = self.get_method_name(child, content);
                self.find_dispatch_in_method(
                    child,
                    content,
                    portlet_id,
                    &method_name,
                    file_path,
                    graph,
                )?;
            }
        }

        Ok(())
    }

    /// Extrait le nom de la méthode
    fn get_method_name(&self, method_node: Node, content: &str) -> String {
        if let Some(name_node) = method_node.child_by_field_name("name") {
            content[name_node.byte_range()].to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Trouve les appels dispatch() dans une méthode
    fn find_dispatch_in_method(
        &self,
        method_node: Node,
        content: &str,
        portlet_id: &str,
        method_name: &str,
        file_path: &str,
        graph: &mut UnifiedGraph,
    ) -> Result<(), String> {
        self.traverse_for_dispatch(
            method_node,
            content,
            portlet_id,
            method_name,
            file_path,
            graph,
        );
        Ok(())
    }

    /// Traverse récursivement pour trouver les dispatch()
    fn traverse_for_dispatch(
        &self,
        node: Node,
        content: &str,
        portlet_id: &str,
        method_name: &str,
        file_path: &str,
        graph: &mut UnifiedGraph,
    ) {
        // Chercher method_invocation avec nom "dispatch"
        if node.kind() == "method_invocation" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = content[name_node.byte_range()].trim();

                if name == "dispatch" || name.ends_with(".dispatch") {
                    // Extraire l'argument (path JSP)
                    if let Some(args_node) = node.child_by_field_name("arguments") {
                        if let Some(jsp_path) = self.extract_string_argument(args_node, content) {
                            let mode = self.infer_mode_from_method(method_name);
                            self.create_renders_relation(
                                portlet_id,
                                &jsp_path,
                                mode,
                                method_name,
                                node.start_position().row + 1,
                                file_path,
                                graph,
                            );
                        }
                    }
                }
            }
        }

        // Parcourir les enfants
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_for_dispatch(child, content, portlet_id, method_name, file_path, graph);
        }
    }

    /// Extrait l'argument string d'un appel de méthode
    fn extract_string_argument(&self, args_node: Node, content: &str) -> Option<String> {
        let mut cursor = args_node.walk();
        for child in args_node.children(&mut cursor) {
            if child.kind() == "string_literal" {
                let text = content[child.byte_range()].trim();
                // Enlever les guillemets
                return Some(text.trim_matches('"').to_string());
            }
        }
        None
    }

    /// Déduit le mode (view, edit, help) depuis le nom de méthode
    fn infer_mode_from_method(&self, method_name: &str) -> &str {
        let lower = method_name.to_lowercase();
        if lower.contains("view") {
            "view"
        } else if lower.contains("edit") {
            "edit"
        } else if lower.contains("help") {
            "help"
        } else {
            "view" // default
        }
    }

    /// Crée la relation RENDERS et le nœud JSP
    fn create_renders_relation(
        &self,
        portlet_id: &str,
        jsp_path: &str,
        mode: &str,
        method_name: &str,
        line: usize,
        _file_path: &str,
        graph: &mut UnifiedGraph,
    ) {
        // Créer le nœud JSP (ou le trouver s'il existe)
        let jsp_id = format!("jsp::{}", jsp_path);
        let jsp_name = jsp_path
            .split('/')
            .last()
            .unwrap_or(jsp_path)
            .trim_end_matches(".jsp")
            .to_string();

        graph.add_node(SemanticNode {
            id: jsp_id.clone(),
            kind: NodeKind::Jsp,
            name: jsp_name,
            file_path: jsp_path.to_string(),
            location: Location::default(),
            metadata: [
                ("type".to_string(), "JSP".to_string()),
                ("path".to_string(), jsp_path.to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        });

        // Créer la relation RENDERS
        let _metadata = RendersMetadata {
            mode: mode.to_string(),
            line,
            method: Some(method_name.to_string()),
        };

        graph.add_edge(SemanticEdge {
            from: portlet_id.to_string(),
            to: jsp_id,
            relation: EdgeRelation::Renders,
            metadata: [
                ("mode".to_string(), mode.to_string()),
                ("method".to_string(), method_name.to_string()),
                ("line".to_string(), line.to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        });

        debug!(
            portlet = portlet_id,
            jsp = jsp_path,
            mode = mode,
            "RENDERS relation created"
        );
    }
}

impl Default for PortletExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_mode() {
        let extractor = PortletExtractor::new();
        assert_eq!(extractor.infer_mode_from_method("doView"), "view");
        assert_eq!(extractor.infer_mode_from_method("doEdit"), "edit");
        assert_eq!(extractor.infer_mode_from_method("doHelp"), "help");
        assert_eq!(extractor.infer_mode_from_method("processAction"), "view");
    }
}
