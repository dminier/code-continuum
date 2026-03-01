// Module JavaScript - Extraction et traversée AST
use std::collections::HashMap;
use tracing::debug;
use tree_sitter::Node;

use crate::semantic_graph::semantic_graph::{
    EdgeRelation, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};

use super::{DslExecutor, ScopeContext};

impl DslExecutor {
    /// Extrait le graphe JavaScript avec support des classes et méthodes
    pub(super) fn extract_javascript(
        &mut self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
    ) {
        // Phase 0: Construire la symbol table pour JavaScript
        self.build_javascript_symbol_table(node, source);
        // Phase 1: Extraire les nœuds
        let module_name = self.file_path.clone();
        let module_id = format!("{}::module", self.file_path);
        graph.add_node(SemanticNode {
            id: module_id.clone(),
            kind: NodeKind::Js,
            name: module_name.clone(),
            file_path: self.file_path.clone(),
            location: self.root_location(),
            metadata: self.metadata("javascript"),
        });

        let ctx = ScopeContext::new().with_module(module_name, module_id);
        self.traverse_javascript(node, source, graph, &ctx);
    }

    fn build_javascript_symbol_table(&mut self, node: Node, source: &str) {
        // Parcourir les field_definition dans les classes
        self.collect_javascript_fields(node, source, &ScopeContext::new());
    }

    fn collect_javascript_fields(&mut self, node: Node, source: &str, ctx: &ScopeContext) {
        match node.kind() {
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let class_name = self.get_text(name_node, source);
                    let class_id = ctx.qualified_id("class", &class_name, &self.file_path);
                    let class_ctx = ctx.with_class(class_name, class_id);

                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        self.collect_javascript_fields(child, source, &class_ctx);
                    }
                    return;
                }
            }
            "field_definition" | "public_field_definition" => {
                if let Some(name_node) = node.child_by_field_name("property") {
                    let field_name = self.get_text(name_node, source);

                    // Essayer de déduire le type depuis l'initialisation
                    if let Some(value_node) = node.child_by_field_name("value") {
                        let value_text = self.get_text(value_node, source);

                        // Détection de type simple : new ClassName()
                        if value_text.starts_with("new ") {
                            let type_name = value_text
                                .trim_start_matches("new ")
                                .split('(')
                                .next()
                                .unwrap_or("")
                                .trim();

                            if !type_name.is_empty() {
                                self.symbol_table.insert(field_name, type_name.to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_javascript_fields(child, source, ctx);
        }
    }

    fn traverse_javascript(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        match node.kind() {
            "function_declaration" | "function" | "arrow_function" => {
                let func_name = if let Some(name_node) = node.child_by_field_name("name") {
                    self.get_text(name_node, source)
                } else {
                    format!("anonymous_{}", node.start_position().row)
                };

                let func_id = ctx.qualified_id("function", &func_name, &self.file_path);

                let mut metadata = self.metadata("javascript");
                if let Some(module) = &ctx.module {
                    metadata.insert("module".to_string(), module.clone());
                }
                if let Some(cls) = &ctx.class {
                    metadata.insert("class".to_string(), cls.clone());
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
                if let Some(parent_fn) = &ctx.current_function {
                    graph.add_edge(SemanticEdge {
                        from: parent_fn.clone(),
                        to: func_id.clone(),
                        relation: EdgeRelation::Contains,
                        metadata: HashMap::new(),
                    });
                } else if let Some(class_id) = &ctx.class_id {
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
                    self.traverse_javascript(child, source, graph, &func_ctx);
                }
                return;
            }
            "class_declaration" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let class_name = self.get_text(name_node, source);
                    let class_id = ctx.qualified_id("class", &class_name, &self.file_path);
                    let class_id_for_node = class_id.clone();

                    let mut metadata = self.metadata("javascript");

                    // Extraire les relations extends et implements et les stocker en métadonnées
                    // (elles seront résolues plus tard dans resolve_extends_implements_global)

                    // Superclass (extends) - accéder au nœud class_heritage directement
                    // Le nœud class_heritage est généralement le 3e enfant (après class, identifier)
                    for i in 0..node.child_count() {
                        if let Some(child) = node.child(i) {
                            if child.kind() == "class_heritage" {
                                debug!(
                                    heritage_text = %self.get_text(child, source),
                                    "class_heritage found"
                                );
                                // Chercher "extends" suivi d'un identifiant
                                let mut cursor = child.walk();
                                let mut next_is_superclass = false;
                                for grandchild in child.children(&mut cursor) {
                                    if grandchild.kind() == "extends" {
                                        next_is_superclass = true;
                                    } else if next_is_superclass
                                        && (grandchild.kind() == "identifier"
                                            || grandchild.kind() == "type_identifier")
                                    {
                                        let superclass_name = self.get_text(grandchild, source);
                                        debug!(
                                            class = %class_name,
                                            superclass = %superclass_name,
                                            "JavaScript class extends detected"
                                        );
                                        metadata.insert("superclass".to_string(), superclass_name);
                                        break;
                                    }
                                }
                                break; // Pas besoin de regarder d'autres enfants
                            }
                        }
                    }

                    // Implements clause (pour les interfaces TypeScript)
                    let mut interfaces = Vec::new();
                    if let Some(implements_node) = node.child_by_field_name("implements") {
                        let mut cursor = implements_node.walk();
                        for iface_node in implements_node.children(&mut cursor) {
                            if iface_node.kind() == "type_identifier"
                                || iface_node.kind() == "identifier"
                            {
                                let interface_name = self.get_text(iface_node, source);
                                debug!(
                                    class = %class_name,
                                    interface = %interface_name,
                                    "JavaScript/TypeScript class implements detected"
                                );
                                interfaces.push(interface_name);
                            }
                        }
                    }
                    if !interfaces.is_empty() {
                        metadata.insert("interfaces".to_string(), interfaces.join(","));
                    }

                    graph.add_node(SemanticNode {
                        id: class_id_for_node.clone(),
                        kind: NodeKind::Class,
                        name: class_name.clone(),
                        file_path: self.file_path.clone(),
                        location: self.node_location(node),
                        metadata,
                    });

                    if let Some(module_id) = &ctx.module_id {
                        graph.add_edge(SemanticEdge {
                            from: module_id.clone(),
                            to: class_id_for_node.clone(),
                            relation: EdgeRelation::Contains,
                            metadata: HashMap::new(),
                        });
                    }

                    let class_ctx = ctx.with_class(class_name, class_id_for_node);
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        self.traverse_javascript(child, source, graph, &class_ctx);
                    }
                    return;
                }
            }
            "method_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let method_name = self.get_text(name_node, source);
                    let func_id = ctx.qualified_id("function", &method_name, &self.file_path);

                    let mut metadata = self.metadata("javascript");
                    if let Some(module) = &ctx.module {
                        metadata.insert("module".to_string(), module.clone());
                    }
                    if let Some(class_name) = &ctx.class {
                        metadata.insert("class".to_string(), class_name.clone());
                    }

                    graph.add_node(SemanticNode {
                        id: func_id.clone(),
                        kind: NodeKind::Function,
                        name: method_name,
                        file_path: self.file_path.clone(),
                        location: self.node_location(node),
                        metadata,
                    });

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

                    // Parcourir le body avec le contexte de cette méthode
                    let func_ctx = ctx.with_function(func_id);
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        self.traverse_javascript(child, source, graph, &func_ctx);
                    }
                    return;
                }
            }
            // 🆕 Object literal avec méthodes : var obj = { method: function() {} }
            "variable_declarator" => {
                // Chercher si c'est un object literal
                if let Some(name_node) = node.child_by_field_name("name") {
                    if let Some(value_node) = node.child_by_field_name("value") {
                        if value_node.kind() == "object" {
                            let obj_name = self.get_text(name_node, source);
                            let obj_id = ctx.qualified_id("object", &obj_name, &self.file_path);

                            // Créer un nœud pour l'objet (comme un Module/Namespace)
                            let mut metadata = self.metadata("javascript");
                            metadata.insert("object_literal".to_string(), "true".to_string());
                            if let Some(module) = &ctx.module {
                                metadata.insert("module".to_string(), module.clone());
                            }

                            graph.add_node(SemanticNode {
                                id: obj_id.clone(),
                                kind: NodeKind::Module,
                                name: obj_name.clone(),
                                file_path: self.file_path.clone(),
                                location: self.node_location(node),
                                metadata,
                            });

                            // Relier au module parent
                            if let Some(module_id) = &ctx.module_id {
                                graph.add_edge(SemanticEdge {
                                    from: module_id.clone(),
                                    to: obj_id.clone(),
                                    relation: EdgeRelation::Contains,
                                    metadata: HashMap::new(),
                                });
                            }

                            // Traverser l'objet avec le contexte de cet objet
                            let obj_ctx = ctx.with_class(obj_name.clone(), obj_id);
                            self.traverse_javascript(value_node, source, graph, &obj_ctx);
                            return;
                        }
                    }
                }
            }
            // 🆕 Méthode dans un object literal : { methodName: function() {} }
            "pair" => {
                if let Some(key_node) = node.child_by_field_name("key") {
                    if let Some(value_node) = node.child_by_field_name("value") {
                        let value_kind = value_node.kind();
                        // Si la valeur est une fonction (function_expression pour anonyme, arrow_function)
                        if value_kind == "function"
                            || value_kind == "function_expression"
                            || value_kind == "arrow_function"
                        {
                            let method_name = self.get_text(key_node, source);
                            let func_id =
                                ctx.qualified_id("function", &method_name, &self.file_path);

                            let mut metadata = self.metadata("javascript");
                            metadata.insert("object_method".to_string(), "true".to_string());
                            if let Some(module) = &ctx.module {
                                metadata.insert("module".to_string(), module.clone());
                            }
                            if let Some(class_name) = &ctx.class {
                                metadata.insert("class".to_string(), class_name.clone());
                                metadata.insert("object".to_string(), class_name.clone());
                            }

                            graph.add_node(SemanticNode {
                                id: func_id.clone(),
                                kind: NodeKind::Function,
                                name: method_name,
                                file_path: self.file_path.clone(),
                                location: self.node_location(node),
                                metadata,
                            });

                            // Relier la méthode à son objet parent
                            if let Some(class_id) = &ctx.class_id {
                                graph.add_edge(SemanticEdge {
                                    from: class_id.clone(),
                                    to: func_id.clone(),
                                    relation: EdgeRelation::Defines,
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

                            // Parcourir le body de la fonction avec le contexte de cette méthode
                            let func_ctx = ctx.with_function(func_id);
                            self.traverse_javascript(value_node, source, graph, &func_ctx);
                            return;
                        }
                    }
                }
            }
            "call_expression" => {
                if let Some(func_node) = node.child_by_field_name("function") {
                    let full_call = self.get_text(func_node, source);

                    // Extraire méthode et objet (ex: api.fetch ou this.method)
                    let (called_name, object_name) = if full_call.contains('.') {
                        let parts: Vec<&str> = full_call.split('.').collect();
                        if parts.len() >= 2 {
                            let method = parts.last().unwrap();
                            let obj = parts[parts.len() - 2];
                            if obj == "this" {
                                (method.to_string(), None)
                            } else {
                                (method.to_string(), Some(obj.to_string()))
                            }
                        } else {
                            (full_call.clone(), None)
                        }
                    } else {
                        (full_call.clone(), None)
                    };

                    // Créer directement la relation CALLS si on a le caller
                    if let Some(caller_id) = &ctx.current_function {
                        let mut edge_metadata = HashMap::new();
                        edge_metadata.insert("method_name".to_string(), called_name.clone());

                        if let Some(obj) = &object_name {
                            edge_metadata.insert("object".to_string(), obj.clone());
                            // Résoudre le type de l'objet depuis la symbol table
                            if let Some(resolved_type) = self.symbol_table.resolve_type(obj) {
                                edge_metadata
                                    .insert("object_type".to_string(), resolved_type.clone());
                            }
                        }

                        // Pour l'instant, on crée un edge temporaire avec le nom de méthode comme cible
                        // resolve_call_edges le remplacera par le vrai ID de fonction
                        graph.add_edge(SemanticEdge {
                            from: caller_id.clone(),
                            to: format!("temp_call:{}", called_name),
                            relation: EdgeRelation::Calls,
                            metadata: edge_metadata,
                        });
                    }
                }
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_javascript(child, source, graph, ctx);
        }
    }
}
