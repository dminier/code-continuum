// Tests d'extraction COBOL via DslExecutor
//
// Fixture: examples/backend/cobol/SalesReport.cbl
//
// Scénario général:
//   Given: fichier COBOL valide avec PROGRAM-ID, SECTIONs, PARAGRAPHs, COPY, CALL, PERFORM
//   When:  DslExecutor::extract() est appelé avec le langage "cobol"
//   Then:  le graphe sémantique contient les nœuds et arêtes attendus
use code_continuum::graph_builder::DslExecutor;
use code_continuum::semantic_graph::dsl::DslRegistry;
use code_continuum::semantic_graph::semantic_graph::{EdgeRelation, NodeKind};
use std::path::PathBuf;
use tree_sitter::Parser;

/// Helper: parse + extract pour COBOL
fn parse_and_extract(
    source: &str,
    file_path: &str,
) -> code_continuum::semantic_graph::semantic_graph::UnifiedGraph {
    let cobol_lang = DslRegistry::get_tree_sitter_language("cobol")
        .expect("COBOL language doit être enregistré dans DslRegistry");
    let mut parser = Parser::new();
    parser
        .set_language(cobol_lang)
        .expect("Impossible d'initialiser le parser COBOL");
    let tree = parser.parse(source, None).expect("Parse COBOL échoué");
    let mut executor = DslExecutor::new(file_path.to_string());
    executor.extract(&tree, source, "cobol")
}

/// Tests [Détection langage COBOL par extension]
///
/// Fixture: DslRegistry
/// Scénario:
///   Given: extensions .cbl, .cob, .cobol, .cpy
///   When:  DslRegistry::detect_language_from_path() est appelé
///   Then:  le langage "cobol" est retourné
#[test]
fn test_cobol_language_detection() {
    use std::path::Path;

    assert_eq!(
        DslRegistry::detect_language_from_path(Path::new("program.cbl")),
        Some("cobol"),
        ".cbl doit être détecté comme cobol"
    );
    assert_eq!(
        DslRegistry::detect_language_from_path(Path::new("program.cob")),
        Some("cobol"),
        ".cob doit être détecté comme cobol"
    );
    assert_eq!(
        DslRegistry::detect_language_from_path(Path::new("program.cobol")),
        Some("cobol"),
        ".cobol doit être détecté comme cobol"
    );
    assert_eq!(
        DslRegistry::detect_language_from_path(Path::new("copybook.cpy")),
        Some("cobol"),
        ".cpy doit être détecté comme cobol"
    );
}

/// Tests [PROGRAM-ID extrait comme Module]
///
/// Fixture: examples/backend/cobol/SalesReport.cbl
/// Scénario:
///   Given: programme COBOL avec PROGRAM-ID. SALESRPT.
///   When:  extraction du graphe
///   Then:  un nœud Module "SALESRPT" est présent
#[test]
fn test_cobol_program_id_as_module() {
    let fixture = PathBuf::from("examples/backend/cobol/SalesReport.cbl");
    assert!(fixture.exists(), "Fixture manquante: {:?}", fixture);

    let source = std::fs::read_to_string(&fixture).expect("Lecture fixture échouée");
    let graph = parse_and_extract(&source, "examples/backend/cobol/SalesReport.cbl");

    let modules: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Module)
        .map(|n| n.name.as_str())
        .collect();

    assert!(
        modules.contains(&"SALESRPT"),
        "PROGRAM-ID 'SALESRPT' doit être extrait comme Module. Trouvé: {:?}",
        modules
    );
}

/// Tests [SECTIONs extraites comme Functions avec metadata kind=section]
///
/// Fixture: examples/backend/cobol/SalesReport.cbl
/// Scénario:
///   Given: MAIN-SECTION SECTION. et PROCESS-SALES-SECTION SECTION.
///   When:  extraction
///   Then:  nœuds Function avec metadata kind=section
#[test]
fn test_cobol_sections_as_functions() {
    let fixture = PathBuf::from("examples/backend/cobol/SalesReport.cbl");
    assert!(fixture.exists(), "Fixture manquante: {:?}", fixture);

    let source = std::fs::read_to_string(&fixture).expect("Lecture fixture échouée");
    let graph = parse_and_extract(&source, "examples/backend/cobol/SalesReport.cbl");

    let section_names: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| {
            n.kind == NodeKind::Function
                && n.metadata
                    .get("kind")
                    .map(|k| k == "section")
                    .unwrap_or(false)
        })
        .map(|n| n.name.as_str())
        .collect();

    assert!(
        section_names.contains(&"MAIN-SECTION"),
        "MAIN-SECTION doit être extraite. Trouvé: {:?}",
        section_names
    );
    assert!(
        section_names.contains(&"PROCESS-SALES-SECTION"),
        "PROCESS-SALES-SECTION doit être extraite. Trouvé: {:?}",
        section_names
    );
}

/// Tests [PARAGRAPHs extraits comme Functions avec metadata kind=paragraph]
///
/// Fixture: examples/backend/cobol/SalesReport.cbl
/// Scénario:
///   Given: INIT-PARAGRAPH. FETCH-CUSTOMER-PARAGRAPH. etc.
///   When:  extraction
///   Then:  nœuds Function avec metadata kind=paragraph
#[test]
fn test_cobol_paragraphs_as_functions() {
    let fixture = PathBuf::from("examples/backend/cobol/SalesReport.cbl");
    assert!(fixture.exists(), "Fixture manquante: {:?}", fixture);

    let source = std::fs::read_to_string(&fixture).expect("Lecture fixture échouée");
    let graph = parse_and_extract(&source, "examples/backend/cobol/SalesReport.cbl");

    let paragraph_names: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| {
            n.kind == NodeKind::Function
                && n.metadata
                    .get("kind")
                    .map(|k| k == "paragraph")
                    .unwrap_or(false)
        })
        .map(|n| n.name.as_str())
        .collect();

    assert!(
        paragraph_names.contains(&"INIT-PARAGRAPH"),
        "INIT-PARAGRAPH doit être extrait. Trouvé: {:?}",
        paragraph_names
    );
    assert!(
        paragraph_names.contains(&"FETCH-CUSTOMER-PARAGRAPH"),
        "FETCH-CUSTOMER-PARAGRAPH doit être extrait. Trouvé: {:?}",
        paragraph_names
    );
    assert!(
        paragraph_names.contains(&"CLOSE-PARAGRAPH"),
        "CLOSE-PARAGRAPH doit être extrait. Trouvé: {:?}",
        paragraph_names
    );
}

/// Tests [COPY extrait comme Import]
///
/// Fixture: examples/backend/cobol/SalesReport.cbl
/// Scénario:
///   Given: COPY WSCONSTANTS.
///   When:  extraction
///   Then:  un nœud Import "WSCONSTANTS" est présent
#[test]
fn test_cobol_copy_as_import() {
    let fixture = PathBuf::from("examples/backend/cobol/SalesReport.cbl");
    assert!(fixture.exists(), "Fixture manquante: {:?}", fixture);

    let source = std::fs::read_to_string(&fixture).expect("Lecture fixture échouée");
    let graph = parse_and_extract(&source, "examples/backend/cobol/SalesReport.cbl");

    let import_names: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Import)
        .map(|n| n.name.as_str())
        .collect();

    assert!(
        import_names
            .iter()
            .any(|n| n.to_uppercase() == "WSCONSTANTS"),
        "COPY WSCONSTANTS doit être extrait comme Import. Trouvé: {:?}",
        import_names
    );
}

/// Tests [CALL génère une arête Calls vers le programme appelé]
///
/// Fixture: examples/backend/cobol/SalesReport.cbl
/// Scénario:
///   Given: CALL 'DBACCESS' USING WS-DB-HANDLE
///   When:  extraction
///   Then:  une arête EdgeRelation::Calls vers temp_call:DBACCESS existe
#[test]
fn test_cobol_call_generates_calls_edge() {
    let fixture = PathBuf::from("examples/backend/cobol/SalesReport.cbl");
    assert!(fixture.exists(), "Fixture manquante: {:?}", fixture);

    let source = std::fs::read_to_string(&fixture).expect("Lecture fixture échouée");
    let graph = parse_and_extract(&source, "examples/backend/cobol/SalesReport.cbl");

    let calls_targets: Vec<&str> = graph
        .edges
        .iter()
        .filter(|e| e.relation == EdgeRelation::Calls)
        .map(|e| e.to.as_str())
        .collect();

    assert!(
        calls_targets.iter().any(|t| t.contains("DBACCESS")),
        "CALL 'DBACCESS' doit générer un edge Calls. Cibles trouvées: {:?}",
        calls_targets
    );
    assert!(
        calls_targets.iter().any(|t| t.contains("CUSTPROC")),
        "CALL 'CUSTPROC' doit générer un edge Calls. Cibles trouvées: {:?}",
        calls_targets
    );
}

/// Tests [PERFORM génère une arête Calls vers la section/paragraph cible]
///
/// Fixture: examples/backend/cobol/SalesReport.cbl
/// Scénario:
///   Given: PERFORM INIT-PARAGRAPH. PERFORM PROCESS-SALES-SECTION.
///   When:  extraction
///   Then:  arêtes EdgeRelation::Calls vers les cibles PERFORM
#[test]
fn test_cobol_perform_generates_calls_edge() {
    let fixture = PathBuf::from("examples/backend/cobol/SalesReport.cbl");
    assert!(fixture.exists(), "Fixture manquante: {:?}", fixture);

    let source = std::fs::read_to_string(&fixture).expect("Lecture fixture échouée");
    let graph = parse_and_extract(&source, "examples/backend/cobol/SalesReport.cbl");

    let calls_targets: Vec<&str> = graph
        .edges
        .iter()
        .filter(|e| e.relation == EdgeRelation::Calls)
        .map(|e| e.to.as_str())
        .collect();

    assert!(
        calls_targets.iter().any(|t| t.contains("INIT-PARAGRAPH")),
        "PERFORM INIT-PARAGRAPH doit générer un edge Calls. Cibles trouvées: {:?}",
        calls_targets
    );
    assert!(
        calls_targets
            .iter()
            .any(|t| t.contains("PROCESS-SALES-SECTION")),
        "PERFORM PROCESS-SALES-SECTION doit générer un edge Calls. Cibles trouvées: {:?}",
        calls_targets
    );
}

/// Tests [Variables de niveau 01 extraites comme Variable]
///
/// Fixture: examples/backend/cobol/SalesReport.cbl
/// Scénario:
///   Given: 01 WS-COUNTER PIC 9(4) VALUE ZERO.
///   When:  extraction
///   Then:  nœuds Variable pour les items de niveau 01
#[test]
fn test_cobol_data_items_as_variables() {
    let fixture = PathBuf::from("examples/backend/cobol/SalesReport.cbl");
    assert!(fixture.exists(), "Fixture manquante: {:?}", fixture);

    let source = std::fs::read_to_string(&fixture).expect("Lecture fixture échouée");
    let graph = parse_and_extract(&source, "examples/backend/cobol/SalesReport.cbl");

    let var_names: Vec<&str> = graph
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Variable)
        .map(|n| n.name.as_str())
        .collect();

    assert!(
        var_names.contains(&"WS-COUNTER"),
        "WS-COUNTER (niveau 01) doit être extrait comme Variable. Trouvé: {:?}",
        var_names
    );
    assert!(
        var_names.contains(&"WS-TOTAL-SALES"),
        "WS-TOTAL-SALES (niveau 01) doit être extrait comme Variable. Trouvé: {:?}",
        var_names
    );
}
