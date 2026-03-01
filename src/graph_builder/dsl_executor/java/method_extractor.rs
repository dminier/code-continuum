// Method extractor - Extraction des méthodes et appels de méthodes Java

use std::collections::HashMap;
use tracing::debug;
use tree_sitter::Node;

use crate::semantic_graph::semantic_graph::{
    EdgeRelation, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};

use super::super::{DslExecutor, ScopeContext};

impl DslExecutor {
    /// Extrait une déclaration de méthode Java
    pub(in crate::graph_builder::dsl_executor) fn extract_method_declaration(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let method_name = self.get_text(name_node, source);
            let method_id = ctx.qualified_id("function", &method_name, &self.file_path);

            let mut metadata = self.metadata("java");
            if let Some(module) = &ctx.module {
                metadata.insert("module".to_string(), module.clone());
            }
            if let Some(pkg) = &ctx.package {
                metadata.insert("package".to_string(), pkg.clone());
            }
            if let Some(cls) = &ctx.class {
                metadata.insert("class".to_string(), cls.clone());
                metadata.insert(
                    "qualified_name".to_string(),
                    format!("{}.{}", cls, method_name),
                );
            }

            debug!(
                method = method_name,
                id = method_id,
                class = ?ctx.class,
                package = ?ctx.package,
                "Méthode Java extraite avec ID qualifié"
            );

            graph.add_node(SemanticNode {
                id: method_id.clone(),
                kind: NodeKind::Function,
                name: method_name,
                file_path: self.file_path.clone(),
                location: self.node_location(node),
                metadata,
            });

            // Relier la méthode à son parent structurel
            if let Some(class_id) = &ctx.class_id {
                graph.add_edge(SemanticEdge {
                    from: class_id.clone(),
                    to: method_id.clone(),
                    relation: EdgeRelation::Contains,
                    metadata: HashMap::new(),
                });
            } else if let Some(pkg_id) = &ctx.package_id {
                graph.add_edge(SemanticEdge {
                    from: pkg_id.clone(),
                    to: method_id.clone(),
                    relation: EdgeRelation::Contains,
                    metadata: HashMap::new(),
                });
            } else if let Some(module_id) = &ctx.module_id {
                graph.add_edge(SemanticEdge {
                    from: module_id.clone(),
                    to: method_id.clone(),
                    relation: EdgeRelation::Contains,
                    metadata: HashMap::new(),
                });
            }

            // Parcourir le body avec le contexte de cette fonction (pour portée lexicale)
            let func_ctx = ctx.with_function(method_id);
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                self.traverse_java(child, source, graph, &func_ctx);
            }
        }
    }

    /// Extrait un appel de méthode Java
    pub(in crate::graph_builder::dsl_executor) fn extract_method_invocation(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let called_name = self.get_text(name_node, source);

            let mut metadata = self.metadata("java");
            metadata.insert("target".to_string(), called_name.clone());

            // Extraire et résoudre le type de l'objet
            self.resolve_method_invocation_object(node, source, graph, ctx, &mut metadata);

            // Créer directement la relation CALLS si on a le caller
            if let Some(caller_id) = &ctx.current_function {
                let target_class = metadata.get("object_type");

                // Note: La résolution finale vers la fonction cible se fera dans resolve_call_edges
                // On stocke les métadonnées nécessaires dans l'edge
                let mut edge_metadata = HashMap::new();
                edge_metadata.insert("method_name".to_string(), called_name.clone());
                if let Some(class_name) = target_class {
                    edge_metadata.insert("object_type".to_string(), class_name.clone());
                }
                if let Some(obj) = metadata.get("object") {
                    edge_metadata.insert("object".to_string(), obj.clone());
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

    /// Résout le type de l'objet pour un appel de méthode
    fn resolve_method_invocation_object(
        &self,
        node: Node,
        source: &str,
        graph: &UnifiedGraph,
        ctx: &ScopeContext,
        metadata: &mut HashMap<String, String>,
    ) {
        // Extraire l'objet si présent (ex: serviceB.getStatus())
        if let Some(object_node) = node.child_by_field_name("object") {
            let object_name = self.get_text(object_node, source);
            metadata.insert("object".to_string(), object_name.clone());

            if object_name == "super" {
                // Super: typer avec la superclass connue
                if let Some(class_id) = &ctx.class_id {
                    if let Some(class_node) = graph.nodes.get(class_id) {
                        if let Some(super_name) = class_node.metadata.get("superclass") {
                            metadata.insert("object_type".to_string(), super_name.clone());
                        }
                    }
                }
            } else if object_name == "this" {
                // This: typer avec la classe courante - utiliser le FQN qualifié complet
                if let Some(class_id) = &ctx.class_id {
                    metadata.insert("object_type".to_string(), class_id.clone());
                } else if let Some(cls) = &ctx.class {
                    // Fallback: construire le FQN avec le package
                    let fqn = if let Some(pkg) = &ctx.package {
                        format!("{}.{}", pkg, cls)
                    } else {
                        cls.clone()
                    };
                    metadata.insert("object_type".to_string(), fqn);
                }
            } else if let Some(resolved_type) = self.symbol_table.resolve_type(&object_name) {
                // Résoudre le type depuis la symbol table
                metadata.insert("object_type".to_string(), resolved_type.clone());
                debug!(
                    object = object_name,
                    resolved_type = resolved_type,
                    "✅ Type résolu depuis symbol table"
                );
            } else if Self::looks_like_class_identifier(&object_name) {
                // Cas des appels statiques: l'objet peut être un FQN ou nom simple
                // Essayer de qualifier le nom si c'est un simple nom
                let qualified_name = if object_name.contains('.') {
                    // Déjà un FQN
                    object_name.clone()
                } else if let Some(pkg) = &ctx.package {
                    // Ajouter le package courant
                    format!("{}.{}", pkg, object_name)
                } else {
                    // Juste le nom simple
                    object_name.clone()
                };
                metadata.insert("object_type".to_string(), qualified_name.clone());
                debug!(
                    object = object_name,
                    qualified = qualified_name,
                    "✅ Type déduit (appel statique)"
                );
            }
        }

        // Cas où Tree-Sitter encode l'appel "super.foo()" sans champ object explicite
        if metadata.get("object_type").is_none() {
            let invocation_text = self.get_text(node, source);
            if invocation_text.starts_with("super.") {
                if let Some(class_id) = &ctx.class_id {
                    if let Some(class_node) = graph.nodes.get(class_id) {
                        if let Some(super_name) = class_node.metadata.get("superclass") {
                            metadata.insert("object_type".to_string(), super_name.clone());
                        }
                    }
                }
            }
        }
    }
}
