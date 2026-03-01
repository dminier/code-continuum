// Package extractor - Extraction des déclarations de package Java

use std::collections::HashMap;
use tracing::debug;
use tree_sitter::Node;

use crate::semantic_graph::semantic_graph::{
    EdgeRelation, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};

use super::super::{DslExecutor, ScopeContext};

impl DslExecutor {
    /// Extrait la déclaration de package et crée le nœud Package
    pub(in crate::graph_builder::dsl_executor) fn extract_java_package(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        base_ctx: &ScopeContext,
    ) -> ScopeContext {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "package_declaration" {
                // Le package n'a pas de field "name", mais un enfant direct de type scoped_identifier
                let mut pkg_cursor = child.walk();
                for pkg_child in child.children(&mut pkg_cursor) {
                    if pkg_child.kind() == "scoped_identifier" || pkg_child.kind() == "identifier" {
                        let package_name = self.get_text(pkg_child, source);
                        debug!(package = package_name, "Package Java extrait");
                        let package_id = format!("{}::package:{}", self.file_path, package_name);

                        // Créer le nœud Package
                        let mut metadata = self.metadata("java");
                        metadata.insert("package".to_string(), package_name.clone());
                        graph.add_node(SemanticNode {
                            id: package_id.clone(),
                            kind: NodeKind::Package,
                            name: package_name.clone(),
                            file_path: self.file_path.clone(),
                            location: self.root_location(),
                            metadata,
                        });

                        // Relier le module racine au package
                        if let Some(module_id) = &base_ctx.module_id {
                            graph.add_edge(SemanticEdge {
                                from: module_id.clone(),
                                to: package_id.clone(),
                                relation: EdgeRelation::Contains,
                                metadata: HashMap::new(),
                            });
                        }

                        return base_ctx.with_package(package_name, package_id);
                    }
                }
            }
        }
        base_ctx.clone()
    }

    /// Crée le nœud Module (fichier) comme racine
    pub(in crate::graph_builder::dsl_executor) fn create_module_node(
        &self,
        graph: &mut UnifiedGraph,
        module_id: &str,
        module_name: &str,
    ) {
        graph.add_node(SemanticNode {
            id: module_id.to_string(),
            kind: NodeKind::Module,
            name: module_name.to_string(),
            file_path: self.file_path.clone(),
            location: self.root_location(),
            metadata: self.metadata("java"),
        });
    }
}
