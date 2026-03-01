// Import extractor - Extraction et résolution des imports Java

use std::collections::HashMap;
use tree_sitter::Node;

use crate::semantic_graph::semantic_graph::{
    EdgeRelation, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};

use super::super::{DslExecutor, ScopeContext};

impl DslExecutor {
    /// Extrait les déclarations d'import et crée des nœuds Import
    pub(in crate::graph_builder::dsl_executor) fn extract_java_imports(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_declaration" {
                let import_text = self.get_text(child, source);
                let mut import_path: Option<String> = None;
                let mut is_static = false;
                let mut wildcard = import_text.contains(".*");

                let mut import_cursor = child.walk();
                for part in child.children(&mut import_cursor) {
                    match part.kind() {
                        "scoped_identifier" | "identifier" => {
                            import_path = Some(self.get_text(part, source));
                        }
                        "*" => wildcard = true,
                        "static" => is_static = true,
                        _ => {}
                    }
                }

                if let Some(mut path) = import_path {
                    if wildcard {
                        path.push_str(".*");
                    }

                    let import_id = format!("{}::import:{}", self.file_path, path);

                    let mut metadata = self.metadata("java");
                    metadata.insert("import_path".to_string(), path.clone());
                    metadata.insert("is_static".to_string(), is_static.to_string());
                    if let Some(module_id) = &ctx.module_id {
                        metadata.insert("module_id".to_string(), module_id.clone());
                    }
                    if let Some(pkg) = &ctx.package {
                        metadata.insert("package".to_string(), pkg.clone());
                    }

                    graph.add_node(SemanticNode {
                        id: import_id.clone(),
                        kind: NodeKind::Import,
                        name: path.clone(),
                        file_path: self.file_path.clone(),
                        location: self.node_location(child),
                        metadata,
                    });

                    if let Some(module_id) = &ctx.module_id {
                        graph.add_edge(SemanticEdge {
                            from: module_id.clone(),
                            to: import_id,
                            relation: EdgeRelation::Contains,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }
    }

    /// Crée les relations IMPORTS entre les classes et les packages/classes qu'elles importent
    pub(in crate::graph_builder::dsl_executor) fn create_import_relations_java(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        // Collecter tous les imports du fichier
        let mut imports = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "import_declaration" {
                let import_text = self.get_text(child, source);
                let mut import_path: Option<String> = None;
                let mut wildcard = import_text.contains(".*");
                let mut import_cursor = child.walk();
                for part in child.children(&mut import_cursor) {
                    match part.kind() {
                        "scoped_identifier" | "identifier" => {
                            import_path = Some(self.get_text(part, source));
                        }
                        "*" => wildcard = true,
                        _ => {}
                    }
                }
                if let Some(mut path) = import_path {
                    if wildcard {
                        path.push_str(".*");
                    }
                    imports.push(path);
                }
            }
        }

        // Pour chaque classe du fichier, créer une relation IMPORTS vers les classes importées
        self.create_class_import_relations(node, source, graph, ctx, &imports);
    }

    /// Crée les relations IMPORTS depuis les classes vers les imports
    pub(in crate::graph_builder::dsl_executor) fn create_class_import_relations(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
        imports: &[String],
    ) {
        match node.kind() {
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let class_name = self.get_text(name_node, source);
                    let class_id = if let Some(pkg) = &ctx.package {
                        format!("{}.{}", pkg, class_name)
                    } else {
                        class_name.clone()
                    };

                    for import_path in imports {
                        if import_path.ends_with(".*") {
                            // Import wildcard: créer relation vers package
                            let package_name = &import_path[..import_path.len() - 2];
                            let package_id = format!("{}::import_package", package_name);

                            if !graph.nodes.contains_key(&package_id) {
                                let mut metadata = self.metadata("java");
                                metadata.insert("is_external".to_string(), "true".to_string());
                                graph.add_node(SemanticNode {
                                    id: package_id.clone(),
                                    kind: NodeKind::Package,
                                    name: package_name.to_string(),
                                    file_path: String::new(),
                                    location: self.root_location(),
                                    metadata,
                                });
                            }

                            graph.add_edge(SemanticEdge {
                                from: class_id.clone(),
                                to: package_id,
                                relation: EdgeRelation::Imports,
                                metadata: HashMap::new(),
                            });
                        } else {
                            // Import spécifique: créer relation vers classe
                            let class_id_imported = import_path.clone();
                            if !graph.nodes.contains_key(&class_id_imported) {
                                let simple_name = import_path
                                    .split('.')
                                    .last()
                                    .unwrap_or("Unknown")
                                    .to_string();

                                let mut metadata = self.metadata("java");
                                metadata.insert("is_external".to_string(), "true".to_string());

                                // Extraire et stocker le package du FQN importé
                                if let Some(last_dot) = import_path.rfind('.') {
                                    let package = &import_path[..last_dot];
                                    metadata.insert("package".to_string(), package.to_string());
                                }

                                graph.add_node(SemanticNode {
                                    id: class_id_imported.clone(),
                                    kind: NodeKind::Class,
                                    name: simple_name,
                                    file_path: String::new(),
                                    location: self.root_location(),
                                    metadata,
                                });
                            }

                            graph.add_edge(SemanticEdge {
                                from: class_id.clone(),
                                to: class_id_imported.clone(),
                                relation: EdgeRelation::Imports,
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.create_class_import_relations(child, source, graph, ctx, imports);
        }
    }
}
