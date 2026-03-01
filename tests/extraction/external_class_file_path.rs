// Test d'intégration pour vérifier que les classes externes n'ont pas de file_path incorrects
// Bug: Les classes importées, superclasses et interfaces externes avaient le file_path du fichier les important

use code_continuum::graph_builder::MultiLanguageGraphBuilder;
use code_continuum::semantic_graph::semantic_graph::NodeKind;

#[test]
fn test_external_class_no_file_path() {
    let java_code = r#"
package backend.java;

import java.util.List;
import java.util.ArrayList;

public class DerivedClass extends BaseClass implements BaseInterface {
    private String derivedName;

    public DerivedClass() {
        this.derivedName = "DerivedClass";
    }

    @Override
    public void performAction() {
        System.out.println("Performing action in DerivedClass");
    }

    @Override
    public String getName() {
        return derivedName;
    }
}
"#;

    let builder = MultiLanguageGraphBuilder::new();
    let language = tree_sitter_java::language();
    let graph = builder
        .build_graph(
            "java",
            language,
            java_code,
            "examples/backend/java/DerivedClass.java",
        )
        .expect("graph should build");

    // Vérifier que la classe locale a un file_path correct
    let derived_class = graph
        .nodes
        .get("backend.java.DerivedClass")
        .expect("DerivedClass devrait exister");
    assert_eq!(
        derived_class.file_path, "examples/backend/java/DerivedClass.java",
        "DerivedClass devrait avoir le bon file_path"
    );
    assert_eq!(
        derived_class.metadata.get("package").map(|s| s.as_str()),
        Some("backend.java"),
        "DerivedClass devrait avoir le bon package"
    );

    // Vérifier que les classes importées (List, ArrayList) ont un file_path vide
    // Note: BaseClass et BaseInterface (extends/implements) ne sont créés que pendant
    // la résolution globale (resolve_extends_implements_global), pas pendant l'extraction initiale

    if let Some(list_class) = graph.nodes.get("java.util.List") {
        assert_eq!(
            list_class.file_path, "",
            "java.util.List importé devrait avoir un file_path VIDE, pas le file_path de DerivedClass"
        );
        assert_eq!(
            list_class.metadata.get("is_external").map(|s| s.as_str()),
            Some("true"),
            "java.util.List devrait être marqué comme externe"
        );
        assert_eq!(
            list_class.metadata.get("package").map(|s| s.as_str()),
            Some("java.util"),
            "java.util.List devrait avoir le package extrait de son FQN"
        );
    }

    if let Some(arraylist_class) = graph.nodes.get("java.util.ArrayList") {
        assert_eq!(
            arraylist_class.file_path, "",
            "java.util.ArrayList importé devrait avoir un file_path VIDE"
        );
        assert_eq!(
            arraylist_class
                .metadata
                .get("is_external")
                .map(|s| s.as_str()),
            Some("true"),
            "java.util.ArrayList devrait être marqué comme externe"
        );
    }
}

#[test]
fn test_no_duplicate_class_nodes() {
    let java_code = r#"
package com.example.portal.fo.web.portlets;

import com.example.portal.fo.web.portlets.GeneriquePortlet;

public class AbstractSouscriptionPortlet extends GeneriquePortlet {
    public void init() {
        super.init();
    }
}
"#;

    let builder = MultiLanguageGraphBuilder::new();
    let language = tree_sitter_java::language();
    let graph = builder
        .build_graph(
            "java",
            language,
            java_code,
            "demo-portlets-web/src/main/java/com/example/portal/fo/web/portlets/AbstractSouscriptionPortlet.java",
        )
        .expect("graph should build");

    // Compter combien de nœuds Class ont "GeneriquePortlet" dans leur ID
    let generique_portlet_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|(id, node)| node.kind == NodeKind::Class && id.contains("GeneriquePortlet"))
        .collect();

    assert_eq!(
        generique_portlet_nodes.len(),
        1,
        "Il ne devrait y avoir qu'UN SEUL nœud Class pour GeneriquePortlet, trouvé: {:?}",
        generique_portlet_nodes
            .iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>()
    );

    // Vérifier que le nœud GeneriquePortlet a le bon ID et un file_path vide (externe)
    let generique_node = generique_portlet_nodes[0].1;
    assert_eq!(
        generique_node.file_path, "",
        "GeneriquePortlet externe (superclasse ET importée) devrait avoir un file_path VIDE, pas le file_path de AbstractSouscriptionPortlet"
    );
    // Note: Ce test vérifie le comportement APRÈS correction du bug
    // Avant le fix, file_path serait "demo-portlets-web/src/main/java/.../AbstractSouscriptionPortlet.java"
}

#[test]
fn test_imported_classes_have_empty_file_path() {
    // Test simplifié: vérifier que les classes importées ont un file_path vide
    let java_code = r#"
package com.example.test;

import java.util.List;
import java.io.File;
import javax.servlet.http.HttpServlet;

public class TestClass {
    private List<String> items;
}
"#;

    let builder = MultiLanguageGraphBuilder::new();
    let language = tree_sitter_java::language();
    let graph = builder
        .build_graph("java", language, java_code, "src/TestClass.java")
        .expect("graph should build");

    // Vérifier que TestClass locale a le bon file_path
    let test_class = graph
        .nodes
        .get("com.example.test.TestClass")
        .expect("TestClass should exist");
    assert_eq!(test_class.file_path, "src/TestClass.java");

    // Vérifier que toutes les classes importées ont un file_path VIDE
    let imported_classes = vec![
        "java.util.List",
        "java.io.File",
        "javax.servlet.http.HttpServlet",
    ];
    for class_id in imported_classes {
        if let Some(imported_class) = graph.nodes.get(class_id) {
            assert_eq!(
                imported_class.file_path, "",
                "Classe importée {} devrait avoir file_path vide, trouvé: '{}'",
                class_id, imported_class.file_path
            );
            assert_eq!(
                imported_class.metadata.get("is_external"),
                Some(&"true".to_string()),
                "Classe importée {} devrait être marquée externe",
                class_id
            );
        }
    }
}
