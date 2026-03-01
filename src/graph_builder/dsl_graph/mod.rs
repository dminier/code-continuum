// DSL tree-sitter-graph par langage
// Architecture modulaire

use crate::semantic_graph::dsl::DslRegistry;

pub mod java;
pub mod javascript;

/// Liste des langages supportés par DslRegistry
pub fn supported_languages() -> Vec<&'static str> {
    DslRegistry::supported_languages()
}
