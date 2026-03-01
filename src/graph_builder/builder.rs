// Builder - Orchestre l'analyse avec extraction AST hiérarchique
use tracing::debug;
use tree_sitter::Language;

use super::{dsl_executor::DslExecutor, dsl_graph};
use crate::semantic_graph::UnifiedGraph;

/// Builder - Analyse multi-langage avec extraction AST et contexte hiérarchique
pub struct MultiLanguageGraphBuilder;

impl MultiLanguageGraphBuilder {
    pub fn new() -> Self {
        MultiLanguageGraphBuilder
    }

    /// Construit un graphe en exécutant l'extraction AST
    pub fn build_graph(
        &self,
        language_name: &str,
        language: Language,
        source_code: &str,
        file_path: &str,
    ) -> Result<UnifiedGraph, String> {
        // Vérifier que le langage est supporté
        if !dsl_graph::supported_languages().contains(&language_name) {
            return Err(format!(
                "Langage non supporté: {}. Langages: {:?}",
                language_name,
                dsl_graph::supported_languages()
            ));
        }

        debug!(
            language = language_name,
            file = file_path,
            "Construction du graphe avec extraction AST hiérarchique"
        );

        // Parser le code source
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(language)
            .map_err(|e| format!("Erreur set_language: {:?}", e))?;

        let tree = parser.parse(source_code, None).ok_or("Erreur parsing")?;

        // Créer l'exécuteur DSL
        let mut executor = DslExecutor::new(file_path.to_string());
        let mut graph = UnifiedGraph::new();

        // Exécuter l'extraction
        executor.execute(language_name, &tree, source_code, language, &mut graph)?;

        debug!(
            nodes = graph.nodes.len(),
            edges = graph.edges.len(),
            "Graphe construit avec contexte hiérarchique"
        );

        Ok(graph)
    }
}

impl Default for MultiLanguageGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = MultiLanguageGraphBuilder::new();
        assert_eq!(std::mem::size_of_val(&builder), 0);
    }

    #[test]
    fn test_unsupported_language() {
        let builder = MultiLanguageGraphBuilder::new();
        let ts_lang = tree_sitter_python::language();

        let result = builder.build_graph("cobol", ts_lang, "print hello", "test.cob");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("non supporté"));
    }
}
