// Architecture d'extraction de graphe sémantique avec contexte hiérarchique
// - dsl_graph: Définitions DSL par langage (référence)
// - dsl_executor: Extraction AST manuelle avec contexte (package, classe, namespace)
// - builder: Orchestration de l'analyse

mod builder;
pub mod dsl_executor; // Public pour les tests d'intégration
pub mod dsl_graph;

pub use builder::MultiLanguageGraphBuilder;
pub use dsl_executor::DslExecutor;
