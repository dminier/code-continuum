// Class extractor - Extraction des classes et interfaces Java

use std::collections::HashMap;
use tracing::debug;
use tree_sitter::Node;

use crate::semantic_graph::semantic_graph::{
    EdgeRelation, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};

use super::super::{DslExecutor, ScopeContext};

/// Cherche le FQN d'un nom simple dans les nœuds Import du graphe.
/// Retourne le FQN si un import exact correspond (ex: `com.foo.Bar` pour `Bar`).
fn resolve_via_imports(simple_name: &str, graph: &UnifiedGraph) -> Option<String> {
    let suffix = format!(".{}", simple_name);
    for node in graph.nodes.values() {
        if node.kind == NodeKind::Import {
            if let Some(import_path) = node.metadata.get("import_path") {
                if import_path.ends_with(&suffix) && !import_path.ends_with(".*") {
                    return Some(import_path.clone());
                }
            }
        }
    }
    None
}

impl DslExecutor {
    /// Extrait une déclaration de classe Java
    pub(in crate::graph_builder::dsl_executor) fn extract_class_declaration(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let class_name = self.get_text(name_node, source);

            // Utiliser le FQN comme ID pour faciliter les recherches
            let class_id_for_node = if let Some(pkg) = &ctx.package {
                format!("{}.{}", pkg, class_name)
            } else {
                class_name.clone()
            };

            let mut metadata = self.metadata("java");
            if let Some(module) = &ctx.module {
                metadata.insert("module".to_string(), module.clone());
            }
            if let Some(pkg) = &ctx.package {
                metadata.insert("package".to_string(), pkg.clone());
                metadata.insert(
                    "qualified_name".to_string(),
                    format!("{}.{}", pkg, class_name),
                );
            }

            // Extraire les relations extends et implements et les stocker en métadonnées
            self.extract_class_inheritance_metadata(
                node,
                source,
                &class_name,
                &mut metadata,
                graph,
            );

            graph.add_node(SemanticNode {
                id: class_id_for_node.clone(),
                kind: NodeKind::Class,
                name: class_name.clone(),
                file_path: self.file_path.clone(),
                location: self.node_location(node),
                metadata,
            });

            // Relier la classe à son parent (package ou module)
            self.link_class_to_parent(graph, ctx, &class_id_for_node);

            // Parcourir les méthodes avec contexte de classe
            let class_ctx = ctx.with_class(class_name, class_id_for_node);
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.traverse_java(child, source, graph, &class_ctx);
            }
        }
    }

    /// Extrait une déclaration d'interface Java
    pub(in crate::graph_builder::dsl_executor) fn extract_interface_declaration(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let interface_name = self.get_text(name_node, source);

            let interface_id_for_node = if let Some(pkg) = &ctx.package {
                format!("{}.{}", pkg, interface_name)
            } else {
                interface_name.clone()
            };

            let mut metadata = self.metadata("java");
            if let Some(module) = &ctx.module {
                metadata.insert("module".to_string(), module.clone());
            }
            if let Some(pkg) = &ctx.package {
                metadata.insert("package".to_string(), pkg.clone());
                metadata.insert(
                    "qualified_name".to_string(),
                    format!("{}.{}", pkg, interface_name),
                );
            }

            // Extraire les relations extends (interfaces peuvent étendre d'autres interfaces)
            if let Some(superclass_node) = node.child_by_field_name("extends") {
                let mut cursor = superclass_node.walk();
                for child in superclass_node.children(&mut cursor) {
                    if child.kind() == "type_identifier" || child.kind() == "identifier" {
                        let superinterface_name = self.get_text(child, source);
                        debug!(
                            interface = %interface_name,
                            superinterface = %superinterface_name,
                            "Java interface extends detected"
                        );
                        metadata.insert("superclass".to_string(), superinterface_name);
                        break;
                    }
                }
            }

            graph.add_node(SemanticNode {
                id: interface_id_for_node.clone(),
                kind: NodeKind::Interface,
                name: interface_name.clone(),
                file_path: self.file_path.clone(),
                location: self.node_location(node),
                metadata,
            });

            // Relier l'interface à son parent (package ou module)
            self.link_class_to_parent(graph, ctx, &interface_id_for_node);

            // Parcourir les méthodes avec contexte d'interface
            let interface_ctx = ctx.with_class(interface_name, interface_id_for_node);
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.traverse_java(child, source, graph, &interface_ctx);
            }
        }
    }

    /// Extrait les métadonnées d'héritage (extends et implements)
    /// Stocke les FQNs complets pour éviter les ambiguïtés lors de la résolution.
    /// Priorité de résolution : FQN explicite > import > même package.
    fn extract_class_inheritance_metadata(
        &self,
        node: Node,
        source: &str,
        class_name: &str,
        metadata: &mut HashMap<String, String>,
        graph: &UnifiedGraph,
    ) {
        let current_package = metadata.get("package").cloned();

        // Superclass (extends)
        if let Some(superclass_node) = node.child_by_field_name("superclass") {
            let mut cursor = superclass_node.walk();
            for child in superclass_node.children(&mut cursor) {
                if matches!(
                    child.kind(),
                    "type_identifier" | "identifier" | "scoped_identifier"
                ) {
                    let superclass_simple = self.get_text(child, source);

                    // Priorité : FQN explicite > import > même package
                    let superclass_fqn = if superclass_simple.contains('.') {
                        superclass_simple.clone()
                    } else if let Some(fqn) = resolve_via_imports(&superclass_simple, graph) {
                        fqn
                    } else if let Some(pkg) = &current_package {
                        format!("{}.{}", pkg, superclass_simple)
                    } else {
                        superclass_simple.clone()
                    };

                    debug!(
                        class = %class_name,
                        superclass_simple = %superclass_simple,
                        superclass_fqn = %superclass_fqn,
                        "Java class extends detected"
                    );
                    metadata.insert("superclass".to_string(), superclass_fqn);
                    metadata.insert("superclass_simple".to_string(), superclass_simple);
                    break;
                }
            }

            if !metadata.contains_key("superclass") {
                let mut superclass_text = self.get_text(superclass_node, source);
                if let Some(stripped) = superclass_text.strip_prefix("extends ") {
                    superclass_text = stripped.trim().to_string();
                } else {
                    superclass_text = superclass_text.trim().to_string();
                }

                let superclass_fqn = if superclass_text.contains('.') {
                    superclass_text.clone()
                } else if let Some(fqn) = resolve_via_imports(&superclass_text, graph) {
                    fqn
                } else if let Some(pkg) = &current_package {
                    format!("{}.{}", pkg, superclass_text)
                } else {
                    superclass_text.clone()
                };

                metadata.insert("superclass".to_string(), superclass_fqn);
                metadata.insert("superclass_simple".to_string(), superclass_text);
            }
        }

        // Interfaces (implements)
        let mut interfaces = Vec::new();
        let mut interfaces_fqn = Vec::new();

        if let Some(interfaces_node) = node.child_by_field_name("interfaces") {
            let mut cursor = interfaces_node.walk();
            for iface_child in interfaces_node.children(&mut cursor) {
                if iface_child.kind() == "type_list" {
                    let mut list_cursor = iface_child.walk();
                    for type_item in iface_child.children(&mut list_cursor) {
                        if type_item.kind() == "type_identifier" || type_item.kind() == "identifier"
                        {
                            let interface_simple = self.get_text(type_item, source);

                            // Priorité : FQN explicite > import > même package
                            let interface_fqn = if interface_simple.contains('.') {
                                interface_simple.clone()
                            } else if let Some(fqn) = resolve_via_imports(&interface_simple, graph)
                            {
                                fqn
                            } else if let Some(pkg) = &current_package {
                                format!("{}.{}", pkg, interface_simple)
                            } else {
                                interface_simple.clone()
                            };

                            debug!(
                                class = %class_name,
                                interface_simple = %interface_simple,
                                interface_fqn = %interface_fqn,
                                "Java class implements detected"
                            );

                            interfaces.push(interface_simple);
                            interfaces_fqn.push(interface_fqn);
                        }
                    }
                }
            }
        }

        if !interfaces.is_empty() {
            metadata.insert("interfaces".to_string(), interfaces.join(","));
            metadata.insert("interfaces_fqn".to_string(), interfaces_fqn.join(","));
        }
    }

    /// Relie une classe/interface à son parent (package ou module)
    fn link_class_to_parent(&self, graph: &mut UnifiedGraph, ctx: &ScopeContext, class_id: &str) {
        if let Some(pkg_id) = &ctx.package_id {
            graph.add_edge(SemanticEdge {
                from: pkg_id.clone(),
                to: class_id.to_string(),
                relation: EdgeRelation::Contains,
                metadata: HashMap::new(),
            });
        } else if let Some(module_id) = &ctx.module_id {
            graph.add_edge(SemanticEdge {
                from: module_id.clone(),
                to: class_id.to_string(),
                relation: EdgeRelation::Contains,
                metadata: HashMap::new(),
            });
        }
    }
}
