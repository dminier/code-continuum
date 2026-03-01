// Test d'intégration batch complet - Simule l'analyse multi-fichiers comme le vrai workflow
// Vérifie que les file_path sont corrects pour les classes locales et externes après la résolution globale

use code_continuum::graph_builder::dsl_executor::dependency_resolver::DependencyResolver;
use code_continuum::graph_builder::{DslExecutor, MultiLanguageGraphBuilder};
use code_continuum::semantic_graph::semantic_graph::{EdgeRelation, NodeKind, UnifiedGraph};

#[test]
fn test_batch_analysis_with_inheritance() {
    // Simuler l'analyse de 3 fichiers Java avec héritage
    let base_class_code = r#"
package backend.java;

public class BaseClass {
    protected String baseName = "BaseClass";

    public String getBaseName() {
        return baseName;
    }
}
"#;

    let base_interface_code = r#"
package backend.java;

public interface BaseInterface {
    void performAction();
    String getName();
}
"#;

    let derived_class_code = r#"
package backend.java;

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

    // Étape 1: Analyser chaque fichier individuellement (comme analyze_file)
    let mut unified_graph = UnifiedGraph::new();

    let files = vec![
        ("examples/backend/java/BaseClass.java", base_class_code),
        (
            "examples/backend/java/BaseInterface.java",
            base_interface_code,
        ),
        (
            "examples/backend/java/DerivedClass.java",
            derived_class_code,
        ),
    ];

    for (file_path, source_code) in files {
        let file_graph = builder
            .build_graph("java", language, source_code, file_path)
            .expect("graph should build");

        // Fusionner dans le graphe unifié
        for node in file_graph.nodes.values() {
            unified_graph.add_node(node.clone());
        }
        for edge in file_graph.edges {
            unified_graph.add_edge(edge);
        }
    }

    // Étape 2: Résolution globale (comme dans analyze_repository_with_filter)
    let mut resolver = DependencyResolver::new();
    DslExecutor::register_local_classes(&mut resolver, &unified_graph);
    DslExecutor::resolve_imports_global(&mut unified_graph, &resolver);
    DslExecutor::resolve_extends_implements_global(&mut unified_graph, &resolver);
    DslExecutor::resolve_calls_global(&mut unified_graph, &resolver);

    // Vérifications: Les classes locales doivent avoir leur file_path correct
    let base_class = unified_graph
        .nodes
        .get("backend.java.BaseClass")
        .expect("BaseClass should exist");
    assert_eq!(
        base_class.file_path, "examples/backend/java/BaseClass.java",
        "BaseClass locale devrait avoir son propre file_path"
    );
    assert!(
        base_class.metadata.get("is_external").is_none(),
        "BaseClass locale ne devrait pas être marquée externe"
    );

    let base_interface = unified_graph
        .nodes
        .get("backend.java.BaseInterface")
        .expect("BaseInterface should exist");
    assert_eq!(
        base_interface.file_path, "examples/backend/java/BaseInterface.java",
        "BaseInterface locale devrait avoir son propre file_path"
    );

    let derived_class = unified_graph
        .nodes
        .get("backend.java.DerivedClass")
        .expect("DerivedClass should exist");
    assert_eq!(
        derived_class.file_path, "examples/backend/java/DerivedClass.java",
        "DerivedClass locale devrait avoir son propre file_path"
    );

    // Vérifier qu'il n'y a PAS de duplication de BaseClass (chercher uniquement les nœuds Class)
    let base_class_nodes: Vec<_> = unified_graph
        .nodes
        .iter()
        .filter(|(id, node)| {
            node.kind == NodeKind::Class && id.as_str() == "backend.java.BaseClass"
        })
        .collect();
    assert_eq!(
        base_class_nodes.len(),
        1,
        "Il ne devrait y avoir qu'UN SEUL nœud Class BaseClass, trouvé: {:?}",
        base_class_nodes
            .iter()
            .map(|(id, _)| id)
            .collect::<Vec<_>>()
    );

    // Vérifier les relations EXTENDS et IMPLEMENTS créées par la résolution globale
    let has_extends = unified_graph.edges.iter().any(|e| {
        e.from == "backend.java.DerivedClass"
            && e.to == "backend.java.BaseClass"
            && e.relation == EdgeRelation::Extends
    });
    assert!(
        has_extends,
        "DerivedClass -> BaseClass EXTENDS devrait exister"
    );

    let has_implements = unified_graph.edges.iter().any(|e| {
        e.from == "backend.java.DerivedClass"
            && e.to == "backend.java.BaseInterface"
            && e.relation == EdgeRelation::Implements
    });
    assert!(
        has_implements,
        "DerivedClass -> BaseInterface IMPLEMENTS devrait exister"
    );
}

#[test]
fn test_batch_analysis_with_external_imports() {
    // Test avec imports externes (java.util, javax.portlet)
    let portlet_code = r#"
package com.example.portal.fo.web.portlets;

import javax.portlet.GenericPortlet;
import javax.portlet.PortletException;
import java.util.List;
import java.util.ArrayList;

public class GeneriquePortlet extends GenericPortlet {
    private List<String> data;

    public GeneriquePortlet() {
        this.data = new ArrayList<>();
    }
}
"#;

    let builder = MultiLanguageGraphBuilder::new();
    let language = tree_sitter_java::language();
    let mut unified_graph = UnifiedGraph::new();

    // Analyser le fichier
    let file_graph = builder
        .build_graph(
            "java",
            language,
            portlet_code,
            "demo-portlets-web/src/main/java/com/example/portal/fo/web/portlets/GeneriquePortlet.java",
        )
        .expect("graph should build");

    for node in file_graph.nodes.values() {
        unified_graph.add_node(node.clone());
    }
    for edge in file_graph.edges {
        unified_graph.add_edge(edge);
    }

    // Résolution globale
    let mut resolver = DependencyResolver::new();
    DslExecutor::register_local_classes(&mut resolver, &unified_graph);
    DslExecutor::resolve_imports_global(&mut unified_graph, &resolver);
    DslExecutor::resolve_extends_implements_global(&mut unified_graph, &resolver);

    // Vérifier la classe locale
    let generique_portlet = unified_graph
        .nodes
        .get("com.example.portal.fo.web.portlets.GeneriquePortlet")
        .expect("GeneriquePortlet should exist");
    assert_eq!(
        generique_portlet.file_path,
        "demo-portlets-web/src/main/java/com/example/portal/fo/web/portlets/GeneriquePortlet.java",
        "GeneriquePortlet locale devrait avoir son propre file_path"
    );
    assert!(
        generique_portlet.metadata.get("is_external").is_none(),
        "GeneriquePortlet locale ne devrait pas être externe"
    );

    // Vérifier les classes importées externes (file_path vide)
    let external_classes = vec![
        "javax.portlet.GenericPortlet",
        "javax.portlet.PortletException",
        "java.util.List",
        "java.util.ArrayList",
    ];

    for class_fqn in external_classes {
        if let Some(ext_class) = unified_graph.nodes.get(class_fqn) {
            assert_eq!(
                ext_class.file_path, "",
                "Classe externe {} devrait avoir file_path VIDE, trouvé: '{}'",
                class_fqn, ext_class.file_path
            );
            assert_eq!(
                ext_class.metadata.get("is_external"),
                Some(&"true".to_string()),
                "Classe externe {} devrait être marquée is_external=true",
                class_fqn
            );

            // Vérifier que le package est correct
            let expected_package = class_fqn.rsplitn(2, '.').nth(1).unwrap_or("");
            assert_eq!(
                ext_class.metadata.get("package").map(|s| s.as_str()),
                Some(expected_package),
                "Classe externe {} devrait avoir le bon package",
                class_fqn
            );
        }
    }

    // Vérifier qu'il n'y a PAS de duplication de GenericPortlet
    let generic_portlet_nodes: Vec<_> = unified_graph
        .nodes
        .iter()
        .filter(|(id, _)| id.contains("GenericPortlet"))
        .collect();

    println!(
        "Nœuds GenericPortlet trouvés: {:?}",
        generic_portlet_nodes
            .iter()
            .map(|(id, n)| (id.as_str(), n.file_path.as_str()))
            .collect::<Vec<_>>()
    );

    // On devrait avoir 2 nœuds: GeneriquePortlet (locale) et GenericPortlet (javax, externe)
    assert_eq!(
        generic_portlet_nodes.len(),
        2,
        "Devrait avoir GeneriquePortlet locale ET javax.portlet.GenericPortlet externe"
    );

    // Vérifier que l'un est local et l'autre externe
    let local_count = generic_portlet_nodes
        .iter()
        .filter(|(_, n)| n.file_path != "")
        .count();
    let external_count = generic_portlet_nodes
        .iter()
        .filter(|(_, n)| n.file_path == "")
        .count();

    assert_eq!(local_count, 1, "Devrait avoir 1 GeneriquePortlet locale");
    assert_eq!(external_count, 1, "Devrait avoir 1 GenericPortlet externe");
}

#[test]
fn test_batch_no_duplicate_for_same_class() {
    // Test de non-régression: vérifier qu'une classe référencée plusieurs fois
    // ne crée qu'UN SEUL nœud externe
    let file1_code = r#"
package com.example;

import java.util.List;

public class ServiceA {
    private List<String> data;
}
"#;

    let file2_code = r#"
package com.example;

import java.util.List;

public class ServiceB {
    private List<Integer> numbers;
}
"#;

    let builder = MultiLanguageGraphBuilder::new();
    let language = tree_sitter_java::language();
    let mut unified_graph = UnifiedGraph::new();

    // Analyser les 2 fichiers
    for (file_path, source) in vec![
        ("src/ServiceA.java", file1_code),
        ("src/ServiceB.java", file2_code),
    ] {
        let file_graph = builder
            .build_graph("java", language, source, file_path)
            .expect("graph should build");

        for node in file_graph.nodes.values() {
            unified_graph.add_node(node.clone());
        }
        for edge in file_graph.edges {
            unified_graph.add_edge(edge);
        }
    }

    // Résolution globale
    let mut resolver = DependencyResolver::new();
    DslExecutor::register_local_classes(&mut resolver, &unified_graph);
    DslExecutor::resolve_imports_global(&mut unified_graph, &resolver);

    // Vérifier qu'il n'y a qu'UN SEUL nœud pour java.util.List
    let list_nodes: Vec<_> = unified_graph
        .nodes
        .iter()
        .filter(|(id, _)| *id == "java.util.List")
        .collect();

    assert_eq!(
        list_nodes.len(),
        1,
        "java.util.List ne devrait exister qu'UNE SEULE fois malgré 2 imports"
    );

    let list_node = list_nodes[0].1;
    assert_eq!(
        list_node.file_path, "",
        "java.util.List devrait avoir file_path vide"
    );
}
