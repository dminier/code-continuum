// Test d'intégration : débogage de l'AST Java
use tree_sitter::Parser;

#[test]
fn debug_generique_portlet_extraction() {
    use code_continuum::encoding::read_text_with_encoding_detection;
    use code_continuum::graph_builder::MultiLanguageGraphBuilder;
    use std::path::PathBuf;

    let path = PathBuf::from(".samples/demo-portlets/demo-portlets-web/src/main/java/com/example/portal/fo/web/portlets/GeneriquePortlet.java");

    if !path.exists() {
        eprintln!("File not found, test skipped");
        return;
    }

    let source = read_text_with_encoding_detection(&path).expect("read file");
    println!(
        "\n📄 Fichier lu, {} bytes, {} lignes",
        source.len(),
        source.lines().count()
    );

    let builder = MultiLanguageGraphBuilder::new();
    let lang = tree_sitter_java::language();

    match builder.build_graph("java", lang, &source, "test.java") {
        Ok(graph) => {
            println!("\n✅ Graph construit:");
            println!("  Nodes: {}", graph.nodes.len());
            println!("  Edges: {}", graph.edges.len());

            println!("\n🔍 Tous les nœuds Class:");
            for (id, node) in &graph.nodes {
                if node.kind == code_continuum::semantic_graph::semantic_graph::NodeKind::Class {
                    println!("  - ID: {}", id);
                    println!("    name: {}, file_path: '{}'", node.name, node.file_path);
                    println!(
                        "    qualified_name: {:?}",
                        node.metadata.get("qualified_name")
                    );
                    println!("    is_external: {:?}", node.metadata.get("is_external"));
                }
            }

            assert!(
                graph.nodes.values().any(|n| n.name == "GeneriquePortlet"),
                "GeneriquePortlet class should be extracted"
            );
        }
        Err(e) => {
            panic!("❌ Erreur d'extraction: {}", e);
        }
    }
}

#[test]
fn test_java_ast_structure() {
    let java = tree_sitter_java::language();
    let mut parser = Parser::new();
    parser.set_language(java).expect("Failed to set language");

    let source = r#"
package backend.java;

public class ServiceA {
    private ServiceB serviceB;
    
    public String processData(String input) {
        String transformed = serviceB.transformData(input);
        return "ServiceA[" + transformed + "]";
    }
    
    public String getStatus() {
        String otherStatus = serviceB.getStatus();
        return "ServiceA" + otherStatus;
    }
}
"#;

    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();

    // Chercher les méthodes
    let mut found_methods = Vec::new();

    fn find_methods(node: tree_sitter::Node, source: &str, found: &mut Vec<String>) {
        if node.kind() == "method_declaration" {
            // Le nom est généralement dans un enfant nommé "name"
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(text) = name_node.utf8_text(source.as_bytes()) {
                    found.push(text.to_string());
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_methods(child, source, found);
        }
    }

    find_methods(root, source, &mut found_methods);

    // Vérifier que les méthodes ont été trouvées
    assert!(
        found_methods.contains(&"processData".to_string()),
        "processData not found in: {:?}",
        found_methods
    );
    assert!(
        found_methods.contains(&"getStatus".to_string()),
        "getStatus not found in: {:?}",
        found_methods
    );
}

#[test]
fn test_java_method_calls() {
    let java = tree_sitter_java::language();
    let mut parser = Parser::new();
    parser.set_language(java).expect("Failed to set language");

    let source = r#"
public class ServiceA {
    private ServiceB serviceB;
    
    public void test() {
        serviceB.transformData("input");
        serviceB.getStatus();
    }
}
"#;

    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();

    // Chercher les appels de méthode
    let mut found_calls = Vec::new();

    fn find_calls(node: tree_sitter::Node, source: &str, found: &mut Vec<String>) {
        if node.kind() == "method_invocation" {
            // Le nom de la méthode est généralement dans un enfant nommé "name"
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(text) = name_node.utf8_text(source.as_bytes()) {
                    found.push(text.to_string());
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_calls(child, source, found);
        }
    }

    find_calls(root, source, &mut found_calls);

    // Vérifier que les appels ont été trouvés
    assert!(
        !found_calls.is_empty(),
        "No method calls found in: {:?}",
        found_calls
    );
}
