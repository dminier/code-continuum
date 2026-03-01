// Tests d'extraction Rust via DslExecutor
use code_continuum::graph_builder::DslExecutor;
use code_continuum::semantic_graph::semantic_graph::{EdgeRelation, NodeKind};
use tree_sitter::Parser;

fn parse_and_extract(source: &str) -> code_continuum::semantic_graph::semantic_graph::UnifiedGraph {
    let rust_lang = tree_sitter_rust::language();
    let mut parser = Parser::new();
    parser
        .set_language(rust_lang)
        .expect("Failed to set rust language");

    let tree = parser.parse(source, None).expect("Parse failed");
    let mut executor = DslExecutor::new("test_file.rs".to_string());
    executor.extract(&tree, source, "rust")
}

#[test]
fn test_rust_function_extraction() {
    let source = r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
"#;

    let graph = parse_and_extract(source);

    let func_names: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Function)
        .map(|n| n.name.as_str())
        .collect();

    assert!(
        func_names.contains(&"add"),
        "Function 'add' should be extracted. Found: {:?}",
        func_names
    );
    assert!(
        func_names.contains(&"subtract"),
        "Function 'subtract' should be extracted. Found: {:?}",
        func_names
    );
}

#[test]
fn test_rust_struct_extraction() {
    let source = r#"
struct Point {
    x: f64,
    y: f64,
}

struct Circle {
    center: Point,
    radius: f64,
}
"#;

    let graph = parse_and_extract(source);

    let class_names: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Class)
        .map(|n| n.name.as_str())
        .collect();

    assert!(
        class_names.contains(&"Point"),
        "Struct 'Point' should be extracted as Class. Found: {:?}",
        class_names
    );
    assert!(
        class_names.contains(&"Circle"),
        "Struct 'Circle' should be extracted as Class. Found: {:?}",
        class_names
    );
}

#[test]
fn test_rust_trait_extraction() {
    let source = r#"
trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}

trait Drawable {
    fn draw(&self);
}
"#;

    let graph = parse_and_extract(source);

    let trait_names: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Trait)
        .map(|n| n.name.as_str())
        .collect();

    assert!(
        trait_names.contains(&"Shape"),
        "Trait 'Shape' should be extracted. Found: {:?}",
        trait_names
    );
    assert!(
        trait_names.contains(&"Drawable"),
        "Trait 'Drawable' should be extracted. Found: {:?}",
        trait_names
    );
}

#[test]
fn test_rust_impl_block_methods() {
    let source = r#"
struct Counter {
    value: i32,
}

impl Counter {
    fn new() -> Self {
        Counter { value: 0 }
    }

    fn increment(&mut self) {
        self.value += 1;
    }

    fn get(&self) -> i32 {
        self.value
    }
}
"#;

    let graph = parse_and_extract(source);

    let func_names: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Function)
        .map(|n| n.name.as_str())
        .collect();

    assert!(
        func_names.contains(&"new"),
        "Method 'new' should be extracted. Found: {:?}",
        func_names
    );
    assert!(
        func_names.contains(&"increment"),
        "Method 'increment' should be extracted. Found: {:?}",
        func_names
    );
    assert!(
        func_names.contains(&"get"),
        "Method 'get' should be extracted. Found: {:?}",
        func_names
    );

    // Vérifier que les méthodes sont liées au struct via Contains
    let counter_class_id = graph
        .nodes
        .values()
        .find(|n| n.kind == NodeKind::Class && n.name == "Counter")
        .map(|n| n.id.clone());

    if let Some(class_id) = counter_class_id {
        let contained_functions: Vec<&str> = graph
            .edges
            .iter()
            .filter(|e| e.from == class_id && e.relation == EdgeRelation::Contains)
            .filter_map(|e| graph.nodes.get(&e.to))
            .filter(|n| n.kind == NodeKind::Function)
            .map(|n| n.name.as_str())
            .collect();

        assert!(
            contained_functions.contains(&"new"),
            "Method 'new' should be contained in 'Counter'. Found: {:?}",
            contained_functions
        );
    }
}

#[test]
fn test_rust_use_declaration_extraction() {
    let source = r#"
use std::collections::HashMap;
use std::io::{Read, Write};
"#;

    let graph = parse_and_extract(source);

    let import_names: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Import)
        .map(|n| n.name.as_str())
        .collect();

    assert!(
        !import_names.is_empty(),
        "At least one import should be extracted. Found: {:?}",
        import_names
    );

    let has_hashmap = import_names
        .iter()
        .any(|name| name.contains("HashMap") || name.contains("std::collections"));
    assert!(
        has_hashmap,
        "Import for HashMap should be found. Found: {:?}",
        import_names
    );
}

#[test]
fn test_rust_call_expression_detection() {
    let source = r#"
fn helper() -> i32 {
    42
}

fn main() {
    let result = helper();
    println!("{}", result);
}
"#;

    let graph = parse_and_extract(source);

    // Vérifier qu'il existe des edges CALLS temporaires
    let calls_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.relation == EdgeRelation::Calls)
        .collect();

    assert!(
        !calls_edges.is_empty(),
        "At least one CALLS edge should be detected. Edges: {:?}",
        calls_edges
            .iter()
            .map(|e| (&e.from, &e.to))
            .collect::<Vec<_>>()
    );

    // Vérifier qu'un appel à 'helper' a été détecté
    let has_helper_call = calls_edges.iter().any(|e| {
        e.to.contains("helper")
            || e.metadata.get("method_name").map(|s| s.as_str()) == Some("helper")
    });

    assert!(
        has_helper_call,
        "Call to 'helper' should be detected. CALLS edges: {:?}",
        calls_edges
            .iter()
            .map(|e| (&e.from, &e.to, &e.metadata))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_rust_module_node_created() {
    let source = r#"fn dummy() {}"#;

    let graph = parse_and_extract(source);

    let module_node = graph
        .nodes
        .values()
        .find(|n| n.kind == NodeKind::Module && n.file_path == "test_file.rs");

    assert!(
        module_node.is_some(),
        "A Module node should be created for the file. Nodes: {:?}",
        graph
            .nodes
            .values()
            .map(|n| (&n.kind, &n.name))
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_rust_enum_extraction() {
    let source = r#"
enum Direction {
    North,
    South,
    East,
    West,
}
"#;

    let graph = parse_and_extract(source);

    let type_names: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Type)
        .map(|n| n.name.as_str())
        .collect();

    assert!(
        type_names.contains(&"Direction"),
        "Enum 'Direction' should be extracted as Type. Found: {:?}",
        type_names
    );
}
