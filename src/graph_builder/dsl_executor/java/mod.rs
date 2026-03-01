// Module Java - Extraction et traversée AST
// Organisation modulaire par responsabilité

mod class_extractor;
mod field_extractor;
mod import_extractor;
mod method_extractor;
mod package_extractor;
mod symbol_table;

use crate::semantic_graph::semantic_graph::UnifiedGraph;
use tree_sitter::Node;

use super::{DslExecutor, ScopeContext};

impl DslExecutor {
    /// Heuristic: detect class-like identifiers (used for static calls)
    pub(super) fn looks_like_class_identifier(name: &str) -> bool {
        name.chars()
            .next()
            .map(|c| c.is_uppercase())
            .unwrap_or(false)
    }

    /// Extrait le graphe Java avec support des packages, classes et méthodes
    pub(super) fn extract_java(&mut self, node: Node, source: &str, graph: &mut UnifiedGraph) {
        // Créer le nœud Module (fichier) comme racine
        let module_name = self.file_path.clone();
        let module_id = format!("{}::module", self.file_path);
        self.create_module_node(graph, &module_id, &module_name);

        // Extraire le package et créer son nœud éventuel
        let ctx = self.extract_java_package(
            node,
            source,
            graph,
            &ScopeContext::new().with_module(module_name, module_id),
        );

        // Extraire les imports avant les symboles
        self.extract_java_imports(node, source, graph, &ctx);

        // Phase 0: Construire la symbol table (nécessaire AVANT traverse pour résoudre les types)
        self.build_java_symbol_table(node, source);

        // Phase 1: Extraire tous les nœuds (classes, méthodes, appels, variables, etc.)
        self.traverse_java(node, source, graph, &ctx);

        // Phase 2: Créer les relations IMPORTS (nécessite que les classes existent déjà)
        self.create_import_relations_java(node, source, graph, &ctx);
    }

    /// Traverse l'AST Java et délègue aux extractors spécialisés
    pub(super) fn traverse_java(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        match node.kind() {
            "method_declaration" => {
                self.extract_method_declaration(node, source, graph, ctx);
                return;
            }
            "class_declaration" => {
                self.extract_class_declaration(node, source, graph, ctx);
                return;
            }
            "interface_declaration" => {
                self.extract_interface_declaration(node, source, graph, ctx);
                return;
            }
            "field_declaration" => {
                self.extract_field_declaration(node, source, graph, ctx);
            }
            "formal_parameter" => {
                self.extract_formal_parameter(node, source, graph, ctx);
            }
            "local_variable_declaration" => {
                self.extract_local_variable_declaration(node, source, graph, ctx);
            }
            "method_invocation" => {
                self.extract_method_invocation(node, source, graph, ctx);
            }
            _ => {}
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.traverse_java(child, source, graph, ctx);
        }
    }
}
