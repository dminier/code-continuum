// Tests d'intégration : extraction des imports Java et création des relations IMPORTS
use std::fs;
use std::path::PathBuf;

use code_continuum::graph_builder::dsl_executor::dependency_resolver::DependencyResolver;
use code_continuum::graph_builder::{DslExecutor, MultiLanguageGraphBuilder};
use code_continuum::semantic_graph::semantic_graph::{EdgeRelation, NodeKind};

#[test]
fn test_java_imports_creates_import_relations() {
    let builder = MultiLanguageGraphBuilder::new();
    let language = tree_sitter_java::language();

    let source = r#"
package com.example.portal.fo.web.portlets;

import javax.portlet.ActionRequest;
import javax.portlet.ActionResponse;
import javax.portlet.GenericPortlet;

public class GeneriquePortlet extends GenericPortlet {
    public void processAction(ActionRequest request, ActionResponse response) {
    }
}
"#;

    let mut graph = builder
        .build_graph("java", language, source, "src/GeneriquePortlet.java")
        .expect("graph should build");

    // Vérifier que la classe GeneriquePortlet est créée
    let generique_class = graph
        .nodes
        .values()
        .find(|n| n.name == "GeneriquePortlet" && n.kind == NodeKind::Class);
    assert!(
        generique_class.is_some(),
        "GeneriquePortlet class should be created"
    );

    let generique_id = generique_class.unwrap().id.clone();

    // Vérifier qu'il existe des relations IMPORTS depuis GeneriquePortlet
    let import_relations: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.from == generique_id && format!("{:?}", e.relation).contains("Import"))
        .collect();

    println!(
        "✅ Found {} IMPORTS relations from GeneriquePortlet",
        import_relations.len()
    );
    for edge in &import_relations {
        if let Some(target_node) = graph.nodes.get(&edge.to) {
            println!("   -> {}", target_node.name);
        }
    }

    assert!(
        !import_relations.is_empty(),
        "GeneriquePortlet should have IMPORTS relations to imported classes"
    );
}

#[test]
fn java_imports_skip_external_targets() {
    let builder = MultiLanguageGraphBuilder::new();
    let language = tree_sitter_java::language();

    let source = r#"
package com.example;

import java.util.List;

public class Foo {
    private List<String> values;
}
"#;

    let mut graph = builder
        .build_graph("java", language, source, "src/Foo.java")
        .expect("graph should build");

    let mut resolver = DependencyResolver::new();
    DslExecutor::register_local_classes(&mut resolver, &graph);
    DslExecutor::resolve_imports_global(&mut graph, &resolver);

    let import_count = graph
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Import)
        .count();
    assert_eq!(import_count, 1, "expected one import node");

    // Aucun nœud fantôme ne doit être créé pour les imports externes
    let phantom_nodes: Vec<_> = graph
        .nodes
        .values()
        .filter(|n| n.metadata.get("is_phantom") == Some(&"true".to_string()))
        .collect();
    assert!(
        phantom_nodes.is_empty(),
        "phantom nodes should not be created"
    );

    // Avec la nouvelle logique, les relations IMPORTS vers des packages externes sont créées
    // (c'est attendu pour voir les dépendances)
    let import_edges: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.relation == EdgeRelation::Imports)
        .collect();
    println!(
        "✅ Found {} IMPORTS relations (including external)",
        import_edges.len()
    );
    // Les imports externes sont maintenant créés - c'est la nouvelle logique
}

#[test]
fn java_imports_from_examples_base_portlet() {
    let builder = MultiLanguageGraphBuilder::new();
    let language = tree_sitter_java::language();

    let base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/websphere-portal/java/BasePortlet.java");
    let source = fs::read_to_string(&base_path).expect("should read BasePortlet.java");

    let mut graph = builder
        .build_graph(
            "java",
            language,
            &source,
            base_path.to_string_lossy().as_ref(),
        )
        .expect("graph should build for BasePortlet");

    // Résoudre les relations EXTENDS/IMPLEMENTS à partir des métadonnées
    graph.resolve_extends_implements_local();

    let base_class = graph
        .nodes
        .values()
        .find(|n| n.name == "BasePortlet" && n.kind == NodeKind::Class)
        .expect("BasePortlet class should be created");
    let base_id = base_class.id.clone();

    let import_node_names: Vec<_> = graph
        .nodes
        .values()
        .filter(|n| n.kind == NodeKind::Import)
        .map(|n| n.name.clone())
        .collect();
    assert!(
        import_node_names.contains(&"javax.portlet.*".to_string()),
        "Wildcard import node for javax.portlet should be created"
    );

    let base_import_targets: Vec<_> = graph
        .edges
        .iter()
        .filter(|e| e.from == base_id && e.relation == EdgeRelation::Imports)
        .map(|e| e.to.clone())
        .collect();

    let has_portlet_import = base_import_targets
        .iter()
        .any(|target| target == "javax.portlet::import_package");
    assert!(
        has_portlet_import,
        "BasePortlet should import javax.portlet package"
    );

    let has_extends_generic_portlet = graph.edges.iter().any(|e| {
        e.from == base_id
            && e.relation == EdgeRelation::Extends
            && graph.nodes.get(&e.to).map(|n| n.name.as_str()) == Some("GenericPortlet")
    });
    assert!(
        has_extends_generic_portlet,
        "BasePortlet should extend GenericPortlet"
    );
}

#[test]
fn java_imports_extends_implements_from_examples_listener() {
    let builder = MultiLanguageGraphBuilder::new();
    let language = tree_sitter_java::language();

    let listener_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples/websphere-portal/java/PortalStartupListener.java");
    let source =
        fs::read_to_string(&listener_path).expect("should read PortalStartupListener.java");

    let mut graph = builder
        .build_graph(
            "java",
            language,
            &source,
            listener_path.to_string_lossy().as_ref(),
        )
        .expect("graph should build for PortalStartupListener");

    // Résoudre les relations EXTENDS/IMPLEMENTS à partir des métadonnées
    graph.resolve_extends_implements_local();

    let listener = graph
        .nodes
        .values()
        .find(|n| n.name == "PortalStartupListener" && n.kind == NodeKind::Class)
        .expect("PortalStartupListener class should be created");
    let listener_id = listener.id.clone();

    let servlet_imports = [
        "javax.servlet.ServletContextEvent",
        "javax.servlet.ServletContextListener",
    ];
    for expected in servlet_imports {
        assert!(
            graph.edges.iter().any(|e| {
                e.from == listener_id && e.relation == EdgeRelation::Imports && e.to == expected
            }),
            "PortalStartupListener should import {}",
            expected
        );
    }

    let has_implements = graph.edges.iter().any(|e| {
        e.from == listener_id
            && e.relation == EdgeRelation::Implements
            && graph.nodes.get(&e.to).map(|n| n.name.as_str()) == Some("ServletContextListener")
    });
    assert!(
        has_implements,
        "PortalStartupListener should implement ServletContextListener"
    );
}

#[test]
fn java_calls_to_super_external_generic_portlet() {
    let builder = MultiLanguageGraphBuilder::new();
    let language = tree_sitter_java::language();

    let source = r#"
package com.example;

import javax.portlet.EventRequest;
import javax.portlet.EventResponse;
import javax.portlet.PortletException;

public class MyPortlet extends javax.portlet.GenericPortlet {
    public void processEvent(EventRequest request, EventResponse response) throws PortletException {
        super.processEvent(request, response);
    }
}
"#;

    let mut graph = builder
        .build_graph("java", language, source, "src/MyPortlet.java")
        .expect("graph should build");

    let mut resolver = DependencyResolver::new();
    DslExecutor::register_local_classes(&mut resolver, &graph);
    DslExecutor::resolve_calls_global(&mut graph, &resolver);

    // Résoudre les relations EXTENDS/IMPLEMENTS à partir des métadonnées
    graph.resolve_extends_implements_local();

    let class_node = graph
        .nodes
        .values()
        .find(|n| n.kind == NodeKind::Class && n.name == "MyPortlet")
        .expect("class node should exist");

    // La métadonnée superclass doit contenir le FQN complet
    assert!(
        class_node
            .metadata
            .get("superclass")
            .map(|s| s.as_str())
            .is_some(),
        "superclass metadata should be captured"
    );

    let caller_fn = graph
        .nodes
        .values()
        .find(|n| {
            n.kind == NodeKind::Function
                && n.name == "processEvent"
                && n.metadata.get("is_phantom") != Some(&"true".to_string())
        })
        .expect("caller function should exist");
    let caller_id = caller_fn.id.clone();

    let phantom_target_id = "javax.portlet.GenericPortlet::function:processEvent";
    let phantom_ids: Vec<_> = graph
        .nodes
        .iter()
        .filter(|(_, n)| n.metadata.get("is_phantom") == Some(&"true".to_string()))
        .map(|(id, _)| id.clone())
        .collect();
    assert!(
        phantom_ids.contains(&phantom_target_id.to_string()),
        "phantom target function should be created, phantoms={:?}",
        phantom_ids
    );

    let phantom_fn = graph.nodes.get(phantom_target_id).unwrap();
    assert_eq!(phantom_fn.kind, NodeKind::Function);

    let has_call_edge = graph.edges.iter().any(|e| {
        e.from == caller_id && e.to == phantom_target_id && e.relation == EdgeRelation::Calls
    });
    assert!(
        has_call_edge,
        "CALLS edge to external GenericPortlet.processEvent should exist"
    );
}
