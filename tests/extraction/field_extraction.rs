// Test d'intégration : extraction des field declarations en Java
use tree_sitter::Parser;

#[test]
fn test_java_field_declaration_extraction() {
    let java = tree_sitter_java::language();
    let mut parser = Parser::new();
    parser.set_language(java).expect("Failed to set language");

    let source = r#"
package backend.java;

public class ServiceA {
    private String name;
    private ServiceB serviceB;
}
"#;

    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();

    // Chercher field_declaration récursivement
    let mut found_fields = Vec::new();

    fn find_fields(node: tree_sitter::Node, source: &str, _depth: usize, found: &mut Vec<String>) {
        if node.kind() == "field_declaration" {
            if let Some(decl) = node.child_by_field_name("declarator") {
                if let Some(name) = decl.child_by_field_name("name") {
                    if let Ok(text) = name.utf8_text(source.as_bytes()) {
                        found.push(text.to_string());
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_fields(child, source, _depth + 1, found);
        }
    }

    find_fields(root, source, 0, &mut found_fields);

    // Vérifier que les fields ont été trouvés
    assert!(found_fields.contains(&"name".to_string()));
    assert!(found_fields.contains(&"serviceB".to_string()));
    assert_eq!(found_fields.len(), 2);
}
