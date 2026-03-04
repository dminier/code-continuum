// Test de débogage : exploration de l'AST COBOL via tree-sitter-COBOL
// Exécuter avec: cargo test debug_cobol_ast -- --nocapture

use tree_sitter::Parser;

fn print_tree(node: tree_sitter::Node, source: &[u8], depth: usize) {
    let indent = "  ".repeat(depth);
    let node_text = if node.child_count() == 0 && node.end_byte() - node.start_byte() < 60 {
        let text = &source[node.start_byte()..node.end_byte()];
        format!(
            " = {:?}",
            String::from_utf8_lossy(text).replace('\n', "\\n")
        )
    } else {
        String::new()
    };

    // Afficher les champs si présents
    let mut field_info = String::new();
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(field_name) = node.field_name_for_child(
            node.children(&mut node.walk())
                .position(|c| c.id() == child.id())
                .unwrap_or(0) as u32,
        ) {
            field_info.push_str(&format!(" [field:{}]", field_name));
        }
    }

    println!(
        "{}[{}] kind={} named={} ({}-{}){}",
        indent,
        depth,
        node.kind(),
        node.is_named(),
        node.start_position().row + 1,
        node.end_position().row + 1,
        node_text
    );

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_tree(child, source, depth + 1);
    }
}

#[test]
#[ignore]
fn debug_cobol_ast() {
    let source = r#"       IDENTIFICATION DIVISION.
       PROGRAM-ID. SALESRPT.
       DATA DIVISION.
       WORKING-STORAGE SECTION.
       01 WS-COUNTER        PIC 9(4)  VALUE ZERO.
       01 WS-TOTAL-SALES    PIC 9(8)V99 VALUE ZERO.
       PROCEDURE DIVISION.
       MAIN-SECTION SECTION.
           PERFORM INIT-PARAGRAPH.
           CALL 'DBACCESS'.
       INIT-PARAGRAPH.
           MOVE ZERO TO WS-COUNTER.
       PROCESS-SECTION SECTION.
           PERFORM FETCH-PARAGRAPH.
       FETCH-PARAGRAPH.
           ADD 1 TO WS-COUNTER.
"#;

    let cobol_lang = tree_sitter_cobol::language();
    let mut parser = Parser::new();
    parser
        .set_language(cobol_lang)
        .expect("Failed to set COBOL language");

    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();

    println!("\n=== AST COBOL ===");
    println!("Root kind: {}", root.kind());
    println!("Has errors: {}", root.has_error());
    println!();
    print_tree(root, source.as_bytes(), 0);

    // Afficher aussi les erreurs de parsing
    if root.has_error() {
        println!("\n=== ERREURS DE PARSING ===");
        fn find_errors(node: tree_sitter::Node, source: &[u8]) {
            if node.is_error() || node.kind() == "ERROR" {
                let text = &source[node.start_byte()..node.end_byte()];
                println!(
                    "  ERROR at line {}: {:?}",
                    node.start_position().row + 1,
                    String::from_utf8_lossy(text)
                );
            }
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                find_errors(child, source);
            }
        }
        find_errors(root, source.as_bytes());
    }
}

#[test]
#[ignore]
fn debug_cobol_sections_and_paragraphs() {
    let source = r#"       IDENTIFICATION DIVISION.
       PROGRAM-ID. SALESRPT.
       PROCEDURE DIVISION.
       MAIN-SECTION SECTION.
           PERFORM INIT-PARAGRAPH.
       INIT-PARAGRAPH.
           STOP RUN.
"#;

    let cobol_lang = tree_sitter_cobol::language();
    let mut parser = Parser::new();
    parser
        .set_language(cobol_lang)
        .expect("Failed to set COBOL language");

    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();

    println!("\n=== Sections & Paragraphs ===");
    fn find_named_nodes(node: tree_sitter::Node, source: &[u8], depth: usize) {
        let kind = node.kind();
        if kind == "section"
            || kind == "paragraph"
            || kind == "section_header"
            || kind == "paragraph_header"
            || kind == "program_name"
            || kind == "copy_statement"
            || kind == "call_statement"
            || kind == "perform_statement_call_proc"
            || kind == "data_description"
        {
            let indent = "  ".repeat(depth);
            let text = &source[node.start_byte()..node.end_byte()];
            println!(
                "{}FOUND [{}]: {:?}",
                indent,
                kind,
                String::from_utf8_lossy(text)
                    .chars()
                    .take(80)
                    .collect::<String>()
            );

            // Afficher les champs
            let mut cursor = node.walk();
            let field_cursor = node.walk();
            for (i, child) in node.children(&mut cursor).enumerate() {
                let field_name = node.field_name_for_child(i as u32);
                let child_text = &source[child.start_byte()..child.end_byte()];
                println!(
                    "{}  child[{}] field={:?} kind={} text={:?}",
                    indent,
                    i,
                    field_name,
                    child.kind(),
                    String::from_utf8_lossy(child_text)
                        .chars()
                        .take(40)
                        .collect::<String>()
                );
                let _ = field_cursor;
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_named_nodes(child, source, depth + 1);
        }
    }

    find_named_nodes(root, source.as_bytes(), 0);
}
