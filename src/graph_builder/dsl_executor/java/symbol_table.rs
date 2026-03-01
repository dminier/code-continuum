// Symbol table builder - Construction de la table de symboles pour résolution de types

use tracing::debug;
use tree_sitter::Node;

use super::super::DslExecutor;

impl DslExecutor {
    /// Construit la symbol table en extrayant les déclarations de champs
    pub(in crate::graph_builder::dsl_executor) fn build_java_symbol_table(
        &mut self,
        node: Node,
        source: &str,
    ) {
        match node.kind() {
            "field_declaration" => {
                // Extraire le type: <type> <var_name>;
                if let Some(type_node) = node.child_by_field_name("type") {
                    let type_name = self.get_text(type_node, source);

                    // Le declarator contient le nom de la variable
                    if let Some(declarator) = node.child_by_field_name("declarator") {
                        // Peut être variable_declarator avec field "name"
                        if let Some(name_node) = declarator.child_by_field_name("name") {
                            let var_name = self.get_text(name_node, source);
                            debug!(
                                var = %var_name,
                                type_name = %type_name,
                                "✅ Symbol table (field): {} -> {}",
                                var_name,
                                type_name
                            );
                            self.symbol_table.insert(var_name, type_name);
                        }
                    }
                }
            }
            "local_variable_declaration" => {
                if let Some(type_node) = node.child_by_field_name("type") {
                    let type_name = self.get_text(type_node, source);
                    if let Some(declarator) = node.child_by_field_name("declarator") {
                        if let Some(name_node) = declarator.child_by_field_name("name") {
                            let var_name = self.get_text(name_node, source);
                            debug!(
                                var = %var_name,
                                type_name = %type_name,
                                "✅ Symbol table (local): {} -> {}",
                                var_name,
                                type_name
                            );
                            self.symbol_table.insert(var_name, type_name);
                        }
                    }
                }
            }
            "formal_parameter" => {
                if let Some(type_node) = node.child_by_field_name("type") {
                    let type_name = self.get_text(type_node, source);
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let var_name = self.get_text(name_node, source);
                        debug!(
                            var = %var_name,
                            type_name = %type_name,
                            "✅ Symbol table (param): {} -> {}",
                            var_name,
                            type_name
                        );
                        self.symbol_table.insert(var_name, type_name);
                    }
                }
            }
            _ => {}
        }

        // Parcourir récursivement tous les enfants
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.build_java_symbol_table(child, source);
        }
    }
}
