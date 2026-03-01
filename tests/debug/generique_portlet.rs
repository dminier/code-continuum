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
        }
        Err(e) => {
            panic!("❌ Erreur d'extraction: {}", e);
        }
    }
}
