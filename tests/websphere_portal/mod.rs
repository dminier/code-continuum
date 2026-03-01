mod test_ajax_calls;
mod test_integration_complete;
mod test_jsp_includes;
/// Tests WebSphere Portal - Configuration commune
///
/// Ce module fournit des helpers et fixtures pour tester
/// l'extraction des relations sémantiques WebSphere Portal
// Déclaration des modules de tests
mod test_portlet_jsp;
mod test_xml_parsing;

use code_continuum::semantic_graph::semantic_graph::{EdgeRelation, UnifiedGraph};
use code_continuum::graph_builder::dsl_executor::websphere_portal::*;
use std::path::PathBuf;

/// Chemin vers le dossier d'exemples WebSphere Portal
pub fn websphere_examples_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("websphere-portal")
}

/// Crée un graphe vide pour les tests
pub fn create_test_graph() -> UnifiedGraph {
    UnifiedGraph::new()
}

/// Helper: compte le nombre de relations d'un type donné
pub fn count_relations(graph: &UnifiedGraph, relation_type: &str) -> usize {
    graph
        .edges
        .iter()
        .filter(|edge| match &edge.relation {
            EdgeRelation::Custom(rel) => rel == relation_type,
            _ => false,
        })
        .count()
}

/// Helper: trouve un nœud par nom
pub fn find_node_by_name<'a>(graph: &'a UnifiedGraph, name: &str) -> Option<&'a String> {
    graph
        .nodes
        .iter()
        .find(|(_, node)| node.name == name)
        .map(|(id, _)| id)
}

/// Helper: vérifie qu'une relation existe
pub fn has_relation(
    graph: &UnifiedGraph,
    from_name: &str,
    to_name: &str,
    relation_type: &str,
) -> bool {
    if let (Some(from_id), Some(to_id)) = (
        find_node_by_name(graph, from_name),
        find_node_by_name(graph, to_name),
    ) {
        graph.edges.iter().any(|edge| {
            edge.from == *from_id
                && edge.to == *to_id
                && match &edge.relation {
                    EdgeRelation::Custom(rel) => rel == relation_type,
                    _ => false,
                }
        })
    } else {
        false
    }
}

/// Helper: lit un fichier d'exemple
pub fn read_example_file(relative_path: &str) -> String {
    let path = websphere_examples_dir().join(relative_path);
    std::fs::read_to_string(path).expect("Failed to read example file")
}

/// Helper: affiche le graphe pour debug
pub fn print_graph_summary(graph: &UnifiedGraph) {
    println!("\n=== Graph Summary ===");
    println!("Nodes: {}", graph.nodes.len());
    for (id, node) in &graph.nodes {
        println!("  {} ({}) - {}", node.name, node.kind, id);
    }

    println!("\nEdges: {}", graph.edges.len());
    for edge in &graph.edges {
        println!(
            "  {} -[{:?}]-> {}",
            graph
                .nodes
                .get(&edge.from)
                .map(|n| &n.name)
                .unwrap_or(&edge.from),
            edge.relation,
            graph
                .nodes
                .get(&edge.to)
                .map(|n| &n.name)
                .unwrap_or(&edge.to)
        );
    }
    println!("===================\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websphere_examples_dir_exists() {
        let dir = websphere_examples_dir();
        assert!(dir.exists(), "WebSphere examples directory should exist");
    }

    #[test]
    fn test_create_test_graph() {
        let graph = create_test_graph();
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }
}
