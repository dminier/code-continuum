// Module Rust - Extraction et traversée AST
use std::collections::HashMap;
use tracing::debug;
use tree_sitter::Node;

use crate::semantic_graph::semantic_graph::{
    EdgeRelation, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};

use super::{DslExecutor, ScopeContext};

impl DslExecutor {
    /// Extrait le graphe Rust avec support des fonctions, structs, traits, impl blocks, modules et use
    pub(super) fn extract_rust(&mut self, node: Node, source: &str, graph: &mut UnifiedGraph) {
        let module_name = self.file_path.clone();
        let module_id = format!("{}::module", self.file_path);

        graph.add_node(SemanticNode {
            id: module_id.clone(),
            kind: NodeKind::Module,
            name: module_name.clone(),
            file_path: self.file_path.clone(),
            location: self.root_location(),
            metadata: self.metadata("rust"),
        });

        let ctx = ScopeContext::new().with_module(module_name, module_id);
        self.traverse_rust(node, source, graph, &ctx);
    }

    fn traverse_rust(&self, node: Node, source: &str, graph: &mut UnifiedGraph, ctx: &ScopeContext) {
        match node.kind() {
            // fn name(...) -> ... { ... }
            "function_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let func_name = self.get_text(name_node, source);
                    let func_id = ctx.qualified_id("function", &func_name, &self.file_path);

                    let mut metadata = self.metadata("rust");
                    if let Some(cls) = &ctx.class {
                        metadata.insert("struct".to_string(), cls.clone());
                    }
                    if let Some(ns) = &ctx.namespace {
                        metadata.insert("module".to_string(), ns.clone());
                    }

                    graph.add_node(SemanticNode {
                        id: func_id.clone(),
                        kind: NodeKind::Function,
                        name: func_name,
                        file_path: self.file_path.clone(),
                        location: self.node_location(node),
                        metadata,
                    });

                    // Relier la fonction à son parent structurel
                    if let Some(class_id) = &ctx.class_id {
                        graph.add_edge(SemanticEdge {
                            from: class_id.clone(),
                            to: func_id.clone(),
                            relation: EdgeRelation::Contains,
                            metadata: HashMap::new(),
                        });
                    } else if let Some(module_id) = &ctx.module_id {
                        graph.add_edge(SemanticEdge {
                            from: module_id.clone(),
                            to: func_id.clone(),
                            relation: EdgeRelation::Contains,
                            metadata: HashMap::new(),
                        });
                    }

                    // Parcourir le body avec le contexte de cette fonction
                    let func_ctx = ctx.with_function(func_id);
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        self.traverse_rust(child, source, graph, &func_ctx);
                    }
                    return;
                }
            }

            // struct Name { ... }
            "struct_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let struct_name = self.get_text(name_node, source);
                    let struct_id = ctx.qualified_id("class", &struct_name, &self.file_path);

                    let mut metadata = self.metadata("rust");
                    metadata.insert("kind".to_string(), "struct".to_string());
                    if let Some(ns) = &ctx.namespace {
                        metadata.insert("module".to_string(), ns.clone());
                    }

                    graph.add_node(SemanticNode {
                        id: struct_id.clone(),
                        kind: NodeKind::Class,
                        name: struct_name,
                        file_path: self.file_path.clone(),
                        location: self.node_location(node),
                        metadata,
                    });

                    if let Some(module_id) = &ctx.module_id {
                        graph.add_edge(SemanticEdge {
                            from: module_id.clone(),
                            to: struct_id.clone(),
                            relation: EdgeRelation::Contains,
                            metadata: HashMap::new(),
                        });
                    }
                }
                // Pas de récursion dans les champs de struct (pas de corps exécutable)
                return;
            }

            // enum Name { ... }
            "enum_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let enum_name = self.get_text(name_node, source);
                    let enum_id = ctx.qualified_id("type", &enum_name, &self.file_path);

                    let mut metadata = self.metadata("rust");
                    metadata.insert("kind".to_string(), "enum".to_string());

                    graph.add_node(SemanticNode {
                        id: enum_id.clone(),
                        kind: NodeKind::Type,
                        name: enum_name,
                        file_path: self.file_path.clone(),
                        location: self.node_location(node),
                        metadata,
                    });

                    if let Some(module_id) = &ctx.module_id {
                        graph.add_edge(SemanticEdge {
                            from: module_id.clone(),
                            to: enum_id.clone(),
                            relation: EdgeRelation::Contains,
                            metadata: HashMap::new(),
                        });
                    }
                }
                return;
            }

            // trait Name { ... }
            "trait_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let trait_name = self.get_text(name_node, source);
                    let trait_id = ctx.qualified_id("trait", &trait_name, &self.file_path);

                    let mut metadata = self.metadata("rust");
                    if let Some(ns) = &ctx.namespace {
                        metadata.insert("module".to_string(), ns.clone());
                    }

                    graph.add_node(SemanticNode {
                        id: trait_id.clone(),
                        kind: NodeKind::Trait,
                        name: trait_name.clone(),
                        file_path: self.file_path.clone(),
                        location: self.node_location(node),
                        metadata,
                    });

                    if let Some(module_id) = &ctx.module_id {
                        graph.add_edge(SemanticEdge {
                            from: module_id.clone(),
                            to: trait_id.clone(),
                            relation: EdgeRelation::Contains,
                            metadata: HashMap::new(),
                        });
                    }

                    // Traverser les méthodes du trait avec le trait comme classe courante
                    let trait_ctx = ctx.with_class(trait_name, trait_id);
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        self.traverse_rust(child, source, graph, &trait_ctx);
                    }
                    return;
                }
            }

            // impl StructName { ... } ou impl Trait for StructName { ... }
            "impl_item" => {
                // Chercher le type sur lequel porte l'impl (field "type")
                let type_name = node
                    .child_by_field_name("type")
                    .map(|n| self.get_text(n, source));

                if let Some(type_name) = type_name {
                    // L'impl block hérite du contexte courant mais met à jour la classe
                    let impl_id = ctx.qualified_id("class", &type_name, &self.file_path);
                    let impl_ctx = ctx.with_class(type_name, impl_id);

                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        self.traverse_rust(child, source, graph, &impl_ctx);
                    }
                    return;
                }
            }

            // mod name { ... }
            "mod_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let mod_name = self.get_text(name_node, source);
                    let mod_id = ctx.qualified_id("module", &mod_name, &self.file_path);

                    let mut metadata = self.metadata("rust");
                    if let Some(ns) = &ctx.namespace {
                        metadata.insert("parent_module".to_string(), ns.clone());
                    }

                    graph.add_node(SemanticNode {
                        id: mod_id.clone(),
                        kind: NodeKind::Module,
                        name: mod_name.clone(),
                        file_path: self.file_path.clone(),
                        location: self.node_location(node),
                        metadata,
                    });

                    if let Some(module_id) = &ctx.module_id {
                        graph.add_edge(SemanticEdge {
                            from: module_id.clone(),
                            to: mod_id.clone(),
                            relation: EdgeRelation::Contains,
                            metadata: HashMap::new(),
                        });
                    }

                    // Traverser les enfants du mod avec ce mod comme nouveau module
                    let mod_ctx = ctx.with_class(mod_name, mod_id);
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        self.traverse_rust(child, source, graph, &mod_ctx);
                    }
                    return;
                }
            }

            // use path::to::item;
            "use_declaration" => {
                let import_path = self.get_text(node, source)
                    .trim_start_matches("use ")
                    .trim_end_matches(';')
                    .trim()
                    .to_string();

                if !import_path.is_empty() {
                    let import_id = format!(
                        "{}::import:{}",
                        self.file_path,
                        import_path.replace("::", ".")
                    );

                    let mut metadata = self.metadata("rust");
                    metadata.insert("import_path".to_string(), import_path.clone());
                    if let Some(module_id) = &ctx.module_id {
                        metadata.insert("module_id".to_string(), module_id.clone());
                    }

                    graph.add_node(SemanticNode {
                        id: import_id.clone(),
                        kind: NodeKind::Import,
                        name: import_path,
                        file_path: self.file_path.clone(),
                        location: self.node_location(node),
                        metadata,
                    });
                }
                return;
            }

            // func(args)
            "call_expression" => {
                if let Some(caller_id) = &ctx.current_function {
                    if let Some(func_node) = node.child_by_field_name("function") {
                        let full_call = self.get_text(func_node, source);

                        // Extraire le nom de la fonction appelée (dernier segment du chemin)
                        let called_name = full_call
                            .split("::")
                            .last()
                            .unwrap_or(&full_call)
                            .to_string();

                        if !called_name.is_empty() {
                            let mut edge_metadata = HashMap::new();
                            edge_metadata
                                .insert("method_name".to_string(), called_name.clone());

                            debug!(
                                caller = %caller_id,
                                callee = %called_name,
                                "Rust call_expression détecté"
                            );

                            graph.add_edge(SemanticEdge {
                                from: caller_id.clone(),
                                to: format!("temp_call:{}", called_name),
                                relation: EdgeRelation::Calls,
                                metadata: edge_metadata,
                            });
                        }
                    }
                }
                // Continuer la traversée pour détecter les appels imbriqués
            }

            // obj.method(args)
            "method_call_expression" => {
                if let Some(caller_id) = &ctx.current_function {
                    let method_name = node
                        .child_by_field_name("method")
                        .map(|n| self.get_text(n, source));

                    let receiver_name = node
                        .child_by_field_name("receiver")
                        .map(|n| self.get_text(n, source));

                    if let Some(method_name) = method_name {
                        let mut edge_metadata = HashMap::new();
                        edge_metadata
                            .insert("method_name".to_string(), method_name.clone());

                        if let Some(recv) = &receiver_name {
                            edge_metadata.insert("object".to_string(), recv.clone());
                            // Résolution de type depuis la symbol table
                            if let Some(resolved_type) = self.symbol_table.resolve_type(recv) {
                                edge_metadata
                                    .insert("object_type".to_string(), resolved_type.clone());
                            }
                        }

                        debug!(
                            caller = %caller_id,
                            method = %method_name,
                            receiver = ?receiver_name,
                            "Rust method_call_expression détecté"
                        );

                        graph.add_edge(SemanticEdge {
                            from: caller_id.clone(),
                            to: format!("temp_call:{}", method_name),
                            relation: EdgeRelation::Calls,
                            metadata: edge_metadata,
                        });
                    }
                }
                // Continuer la traversée pour détecter les appels imbriqués
            }

            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_rust(child, source, graph, ctx);
        }
    }
}
