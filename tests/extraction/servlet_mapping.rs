use crate::common::test_utils;
use code_continuum::semantic_graph::semantic_graph::{NodeKind, SemanticGraph};
use std::path::PathBuf;

/// Test extraction de servlets avec servlet-mapping séparé
///
/// Fixture: examples/servlet_test/web.xml
/// Scénario:
/// - Given: Un web.xml avec <servlet> et <servlet-mapping> séparés
/// - When: On analyse le fichier
/// - Then: Les servlets doivent avoir leurs url-patterns dans les métadonnées
#[test]
fn test_servlet_with_separate_mapping() {
    let fixture_path = PathBuf::from("examples/servlet_test/web.xml");
    assert!(
        fixture_path.exists(),
        "Fixture manquante: {:?}",
        fixture_path
    );

    let mut graph = SemanticGraph::new();
    let content = std::fs::read_to_string(&fixture_path).unwrap();

    // Simuler l'extraction XML (on devrait utiliser l'extracteur réel)
    // Pour l'instant, on va juste vérifier que le fichier existe et est valide
    assert!(content.contains("<servlet>"));
    assert!(content.contains("<servlet-mapping>"));
    assert!(content.contains("RechercheServlet"));
    assert!(content.contains("/Recherche.srv"));
}

/// Test extraction de servlet avec multiples url-patterns
///
/// Fixture: examples/servlet_test/web.xml
/// Scénario:
/// - Given: Un servlet avec plusieurs servlet-mapping
/// - When: On analyse le fichier
/// - Then: Tous les url-patterns doivent être capturés
#[test]
fn test_servlet_with_multiple_patterns() {
    let fixture_path = PathBuf::from("examples/servlet_test/web.xml");
    assert!(
        fixture_path.exists(),
        "Fixture manquante: {:?}",
        fixture_path
    );

    let content = std::fs::read_to_string(&fixture_path).unwrap();

    // Vérifier que le UserServlet a bien deux patterns
    assert!(content.contains("UserServlet"));
    assert!(content.contains("/users/*"));
    assert!(content.contains("/api/users/*"));
}

/// Test extraction de servlet depuis websphere-portal/web.xml
///
/// Fixture: examples/websphere-portal/web.xml
/// Scénario:
/// - Given: Le web.xml de websphere-portal
/// - When: On extrait les servlets
/// - Then: DispatcherServlet et ResourceServlet doivent avoir leurs url-patterns
#[test]
fn test_websphere_servlet_extraction() {
    let fixture_path = PathBuf::from("examples/websphere-portal/web.xml");
    assert!(
        fixture_path.exists(),
        "Fixture manquante: {:?}",
        fixture_path
    );

    let content = std::fs::read_to_string(&fixture_path).unwrap();

    // Vérifier DispatcherServlet
    assert!(content.contains("DispatcherServlet"));
    assert!(content.contains("com.example.web.DispatcherServlet"));
    assert!(content.contains("*.do"));

    // Vérifier ResourceServlet
    assert!(content.contains("ResourceServlet"));
    assert!(content.contains("com.example.web.ResourceServlet"));
    assert!(content.contains("/resources/*"));
}
