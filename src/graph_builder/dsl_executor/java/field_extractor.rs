// Field extractor - Extraction des champs, paramètres et variables locales Java

use std::collections::HashMap;
use tree_sitter::Node;

use crate::semantic_graph::semantic_graph::{
    EdgeRelation, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};

use super::super::{DslExecutor, ScopeContext};

impl DslExecutor {
    /// Extrait une déclaration de champ de classe
    pub(in crate::graph_builder::dsl_executor) fn extract_field_declaration(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        // Class field -> Variable, parent = class
        if let Some(class_id) = &ctx.class_id {
            let type_name = node
                .child_by_field_name("type")
                .map(|t| self.get_text(t, source));

            if let Some(declarator) = node.child_by_field_name("declarator") {
                if let Some(name_node) = declarator.child_by_field_name("name") {
                    let field_name = self.get_text(name_node, source);
                    let field_id = ctx.qualified_id("variable", &field_name, &self.file_path);

                    let mut metadata = self.metadata("java");
                    if let Some(module) = &ctx.module {
                        metadata.insert("module".to_string(), module.clone());
                    }
                    if let Some(pkg) = &ctx.package {
                        metadata.insert("package".to_string(), pkg.clone());
                    }
                    if let Some(cls) = &ctx.class {
                        metadata.insert("class".to_string(), cls.clone());
                    }
                    if let Some(tn) = type_name.clone() {
                        metadata.insert("object_type".to_string(), tn);
                    }

                    graph.add_node(SemanticNode {
                        id: field_id.clone(),
                        kind: NodeKind::Variable,
                        name: field_name,
                        file_path: self.file_path.clone(),
                        location: self.node_location(node),
                        metadata,
                    });

                    graph.add_edge(SemanticEdge {
                        from: class_id.clone(),
                        to: field_id.clone(),
                        relation: EdgeRelation::Contains,
                        metadata: HashMap::new(),
                    });
                }
            }
        }
    }

    /// Extrait un paramètre formel de méthode
    pub(in crate::graph_builder::dsl_executor) fn extract_formal_parameter(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        if let Some(function_id) = &ctx.current_function {
            if let Some(name_node) = node.child_by_field_name("name") {
                let param_name = self.get_text(name_node, source);
                let param_id = ctx.qualified_id("param", &param_name, &self.file_path);

                let mut metadata = self.metadata("java");
                if let Some(module) = &ctx.module {
                    metadata.insert("module".to_string(), module.clone());
                }
                if let Some(pkg) = &ctx.package {
                    metadata.insert("package".to_string(), pkg.clone());
                }
                if let Some(cls) = &ctx.class {
                    metadata.insert("class".to_string(), cls.clone());
                }

                graph.add_node(SemanticNode {
                    id: param_id.clone(),
                    kind: NodeKind::Parameter,
                    name: param_name,
                    file_path: self.file_path.clone(),
                    location: self.node_location(node),
                    metadata,
                });
                graph.add_edge(SemanticEdge {
                    from: function_id.clone(),
                    to: param_id,
                    relation: EdgeRelation::Defines,
                    metadata: HashMap::new(),
                });
            }
        }
    }

    /// Extrait une déclaration de variable locale
    pub(in crate::graph_builder::dsl_executor) fn extract_local_variable_declaration(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        if let Some(function_id) = &ctx.current_function {
            if let Some(declarator) = node.child_by_field_name("declarator") {
                if let Some(name_node) = declarator.child_by_field_name("name") {
                    let var_name = self.get_text(name_node, source);
                    let var_id = ctx.qualified_id("variable", &var_name, &self.file_path);

                    let mut metadata = self.metadata("java");
                    if let Some(module) = &ctx.module {
                        metadata.insert("module".to_string(), module.clone());
                    }
                    if let Some(pkg) = &ctx.package {
                        metadata.insert("package".to_string(), pkg.clone());
                    }
                    if let Some(cls) = &ctx.class {
                        metadata.insert("class".to_string(), cls.clone());
                    }

                    graph.add_node(SemanticNode {
                        id: var_id.clone(),
                        kind: NodeKind::Variable,
                        name: var_name,
                        file_path: self.file_path.clone(),
                        location: self.node_location(node),
                        metadata,
                    });
                    graph.add_edge(SemanticEdge {
                        from: function_id.clone(),
                        to: var_id,
                        relation: EdgeRelation::Defines,
                        metadata: HashMap::new(),
                    });
                }
            }
        }
    }
}
