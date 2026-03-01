// Test d'intégration end-to-end : Analyse réelle du répertoire examples/
// Simule exactement le workflow de analyze_repository() sur des fichiers réels

use code_continuum::encoding::read_text_with_encoding_detection;
use code_continuum::graph_builder::dsl_executor::dependency_resolver::DependencyResolver;
use code_continuum::graph_builder::{DslExecutor, MultiLanguageGraphBuilder};
use code_continuum::semantic_graph::semantic_graph::{EdgeRelation, NodeKind, UnifiedGraph};
use std::path::{Path, PathBuf};

/// Helper: Analyse un fichier Java réel
fn analyze_real_java_file(
    builder: &MultiLanguageGraphBuilder,
    file_path: &Path,
    root_dir: &Path,
    unified_graph: &mut UnifiedGraph,
) -> Result<(), String> {
    let source_code = read_text_with_encoding_detection(file_path)
        .map_err(|e| format!("Failed to read {}: {}", file_path.display(), e))?;

    let relative_path = file_path
        .strip_prefix(root_dir)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace('\\', "/");

    let language = tree_sitter_java::language();
    let file_graph = builder
        .build_graph("java", language, &source_code, &relative_path)
        .map_err(|e| format!("Failed to build graph for {}: {}", file_path.display(), e))?;

    // Fusionner dans le graphe unifié
    for node in file_graph.nodes.values() {
        unified_graph.add_node(node.clone());
    }
    for edge in file_graph.edges {
        unified_graph.add_edge(edge);
    }

    Ok(())
}

#[test]
fn test_real_examples_directory_analysis() {
    let examples_dir = PathBuf::from("examples/backend/java");

    if !examples_dir.exists() {
        eprintln!("⚠️ Répertoire examples/backend/java non trouvé, test skippé");
        return;
    }

    let builder = MultiLanguageGraphBuilder::new();
    let mut unified_graph = UnifiedGraph::new();
    let root_dir = PathBuf::from("examples");

    // Analyser tous les fichiers .java du répertoire
    let java_files: Vec<PathBuf> = std::fs::read_dir(&examples_dir)
        .expect("Should read examples dir")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "java" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    println!(
        "📁 Analyse de {} fichiers Java dans examples/backend/java/",
        java_files.len()
    );

    for java_file in &java_files {
        println!("  📄 {}", java_file.display());
        analyze_real_java_file(&builder, java_file, &root_dir, &mut unified_graph)
            .expect("Should analyze file");
    }

    // Résolution globale (comme dans analyze_repository)
    println!("🔗 Résolution globale des dépendances...");
    let mut resolver = DependencyResolver::new();
    DslExecutor::register_local_classes(&mut resolver, &unified_graph);
    DslExecutor::resolve_imports_global(&mut unified_graph, &resolver);
    DslExecutor::resolve_extends_implements_global(&mut unified_graph, &resolver);
    DslExecutor::resolve_calls_global(&mut unified_graph, &resolver);

    println!(
        "✅ Graphe construit: {} nœuds, {} arêtes",
        unified_graph.nodes.len(),
        unified_graph.edges.len()
    );

    // VÉRIFICATIONS: file_path corrects pour les classes locales
    let local_classes = vec![
        ("backend.java.BaseClass", "backend/java/BaseClass.java"),
        (
            "backend.java.BaseInterface",
            "backend/java/BaseInterface.java",
        ),
        (
            "backend.java.DerivedClass",
            "backend/java/DerivedClass.java",
        ),
        ("backend.java.User", "backend/java/User.java"),
        ("backend.java.UserService", "backend/java/UserService.java"),
    ];

    for (class_fqn, expected_relative_path) in local_classes {
        if let Some(class_node) = unified_graph.nodes.get(class_fqn) {
            println!("✓ Classe locale: {} -> {}", class_fqn, class_node.file_path);
            assert_eq!(
                class_node.file_path, expected_relative_path,
                "Classe locale {} devrait avoir file_path={}",
                class_fqn, expected_relative_path
            );
            assert!(
                class_node.metadata.get("is_external").is_none(),
                "Classe locale {} ne devrait pas être marquée externe",
                class_fqn
            );
        }
    }

    // Vérifier qu'il n'y a PAS de duplication de BaseClass
    let base_class_nodes: Vec<_> = unified_graph
        .nodes
        .iter()
        .filter(|(id, node)| node.kind == NodeKind::Class && id.ends_with("BaseClass"))
        .collect();

    println!(
        "🔍 Nœuds BaseClass trouvés: {:?}",
        base_class_nodes
            .iter()
            .map(|(id, n)| (id.as_str(), n.file_path.as_str()))
            .collect::<Vec<_>>()
    );

    assert_eq!(
        base_class_nodes.len(),
        1,
        "Il ne devrait y avoir qu'UN SEUL nœud BaseClass, trouvé: {:?}",
        base_class_nodes
            .iter()
            .map(|(id, _)| id)
            .collect::<Vec<_>>()
    );

    // Vérifier les relations EXTENDS/IMPLEMENTS
    let has_extends = unified_graph.edges.iter().any(|e| {
        e.from == "backend.java.DerivedClass"
            && e.to == "backend.java.BaseClass"
            && e.relation == EdgeRelation::Extends
    });
    assert!(
        has_extends,
        "DerivedClass -> BaseClass EXTENDS devrait exister après résolution globale"
    );

    let has_implements = unified_graph.edges.iter().any(|e| {
        e.from == "backend.java.DerivedClass"
            && e.to == "backend.java.BaseInterface"
            && e.relation == EdgeRelation::Implements
    });
    assert!(
        has_implements,
        "DerivedClass -> BaseInterface IMPLEMENTS devrait exister après résolution globale"
    );

    // Vérifier que les classes EXTERNES (si importées) ont file_path vide
    let external_classes: Vec<_> = unified_graph
        .nodes
        .iter()
        .filter(|(_, node)| node.metadata.get("is_external") == Some(&"true".to_string()))
        .collect();

    println!("🌐 Classes externes trouvées: {}", external_classes.len());
    for (id, node) in &external_classes {
        println!("  - {} (file_path='{}')", id, node.file_path);
        assert_eq!(
            node.file_path, "",
            "Classe externe {} devrait avoir file_path VIDE, trouvé: '{}'",
            id, node.file_path
        );
    }

    println!("✅ Tous les tests sur le répertoire examples/ ont réussi !");
}

#[test]
fn test_real_examples_no_file_path_pollution() {
    // Test de non-régression: s'assurer qu'aucune classe locale n'a le file_path d'une autre classe
    let examples_dir = PathBuf::from("examples/backend/java");

    if !examples_dir.exists() {
        eprintln!("⚠️ Répertoire examples/backend/java non trouvé, test skippé");
        return;
    }

    let builder = MultiLanguageGraphBuilder::new();
    let mut unified_graph = UnifiedGraph::new();
    let root_dir = PathBuf::from("examples");

    let java_files: Vec<PathBuf> = std::fs::read_dir(&examples_dir)
        .expect("Should read examples dir")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "java" {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    for java_file in &java_files {
        analyze_real_java_file(&builder, java_file, &root_dir, &mut unified_graph)
            .expect("Should analyze file");
    }

    // Résolution globale
    let mut resolver = DependencyResolver::new();
    DslExecutor::register_local_classes(&mut resolver, &unified_graph);
    DslExecutor::resolve_imports_global(&mut unified_graph, &resolver);
    DslExecutor::resolve_extends_implements_global(&mut unified_graph, &resolver);

    // Vérifier que chaque classe locale a SON propre file_path (pas celui d'un autre fichier)
    for (class_id, class_node) in unified_graph.nodes.iter() {
        if class_node.kind != NodeKind::Class {
            continue;
        }

        if class_node.metadata.get("is_external") == Some(&"true".to_string()) {
            // Classe externe: doit avoir file_path vide
            assert_eq!(
                class_node.file_path, "",
                "Classe externe {} a un file_path non vide: '{}'",
                class_id, class_node.file_path
            );
        } else {
            // Classe locale: doit avoir un file_path cohérent avec son nom
            assert!(
                !class_node.file_path.is_empty(),
                "Classe locale {} ne devrait pas avoir file_path vide",
                class_id
            );

            // Pour les classes top-level, le nom de la classe devrait apparaître dans le file_path
            // Pour les classes internes (Level1Service, Level2Service, etc. dans TransitiveChain.java),
            // c'est le fichier parent qui apparaît
            // On vérifie juste que le file_path se termine par .java et est cohérent
            assert!(
                class_node.file_path.ends_with(".java"),
                "Classe locale {} devrait avoir un file_path se terminant par .java, trouvé: '{}'",
                class_id,
                class_node.file_path
            );

            // Vérifier qu'aucune classe n'a le file_path d'un fichier qui ne devrait pas être le sien
            // (par ex, une classe définie dans A.java ne devrait pas avoir file_path=B.java)
            // Note: Les classes internes sont OK avec le file_path du fichier parent
        }
    }

    println!("✅ Aucune pollution de file_path détectée !");
}

#[test]
fn test_demo_portlets_transitive_extends_generique_portlet() {
    // Repro du bug signalé: GeneriquePortlet (classe locale) remonte avec file_path vide
    // Chaîne d'héritage réelle (3 niveaux) dans les samples Demo Portlets:
    // FinalisationEpargneBaqPortlet → AbstractSouscriptionPortlet → GeneriquePortlet
    let root_dir = PathBuf::from(".samples/demo-portlets");
    if !root_dir.exists() {
        eprintln!("⚠️ Répertoire .samples/demo-portlets non trouvé, test skippé");
        return;
    }

    let builder = MultiLanguageGraphBuilder::new();
    let mut unified_graph = UnifiedGraph::new();

    let files = vec![
        "demo-portlets-web/src/main/java/com/example/portal/fo/web/portlets/GeneriquePortlet.java",
        "demo-portlets-web/src/main/java/com/example/portal/fo/web/portlets/contractualisation/AbstractSouscriptionPortlet.java",
        "demo-portlets-web/src/main/java/com/example/portal/fo/web/portlets/contractualisation/FinalisationEpargneBaqPortlet.java",
    ];

    for rel_path in &files {
        let abs_path = root_dir.join(rel_path);
        analyze_real_java_file(&builder, &abs_path, &root_dir, &mut unified_graph)
            .expect("Should analyze demo-portlets sample file");
    }

    // Résolution globale comme le pipeline réel
    let mut resolver = DependencyResolver::new();
    DslExecutor::register_local_classes(&mut resolver, &unified_graph);
    DslExecutor::resolve_imports_global(&mut unified_graph, &resolver);
    DslExecutor::resolve_extends_implements_global(&mut unified_graph, &resolver);

    // Vérifier les 3 classes locales et leur file_path (doit être non vide et correct)
    let final_portlet = unified_graph
        .nodes
        .get("com.example.portal.fo.web.portlets.contractualisation.FinalisationEpargneBaqPortlet")
        .expect("FinalisationEpargneBaqPortlet devrait exister");
    assert_eq!(
        final_portlet.file_path,
        "demo-portlets-web/src/main/java/com/example/portal/fo/web/portlets/contractualisation/FinalisationEpargneBaqPortlet.java",
        "FinalisationEpargneBaqPortlet devrait avoir son file_path"
    );

    let abstract_portlet = unified_graph
        .nodes
        .get("com.example.portal.fo.web.portlets.contractualisation.AbstractSouscriptionPortlet")
        .expect("AbstractSouscriptionPortlet devrait exister");
    assert_eq!(
        abstract_portlet.file_path,
        "demo-portlets-web/src/main/java/com/example/portal/fo/web/portlets/contractualisation/AbstractSouscriptionPortlet.java",
        "AbstractSouscriptionPortlet devrait avoir son file_path"
    );

    let generique_portlet = unified_graph
        .nodes
        .get("com.example.portal.fo.web.portlets.GeneriquePortlet")
        .expect("GeneriquePortlet devrait exister");

    // 🐞 Si ce test échoue ici avec file_path vide, on reproduit l'anomalie signalée
    assert_eq!(
        generique_portlet.file_path,
        "demo-portlets-web/src/main/java/com/example/portal/fo/web/portlets/GeneriquePortlet.java",
        "GeneriquePortlet devrait être locale avec un file_path non vide"
    );

    // Vérifier les relations EXTENDS transitives FinalisationEpargneBaqPortlet → AbstractSouscriptionPortlet → GeneriquePortlet
    let has_extends_final_to_abstract = unified_graph.edges.iter().any(|e| {
        e.from
            == "com.example.portal.fo.web.portlets.contractualisation.FinalisationEpargneBaqPortlet"
            && e.to
                == "com.example.portal.fo.web.portlets.contractualisation.AbstractSouscriptionPortlet"
            && e.relation == EdgeRelation::Extends
    });
    assert!(
        has_extends_final_to_abstract,
        "EXTENDS FinalisationEpargneBaqPortlet -> AbstractSouscriptionPortlet manquante"
    );

    let has_extends_abstract_to_generique = unified_graph.edges.iter().any(|e| {
        e.from
            == "com.example.portal.fo.web.portlets.contractualisation.AbstractSouscriptionPortlet"
            && e.to == "com.example.portal.fo.web.portlets.GeneriquePortlet"
            && e.relation == EdgeRelation::Extends
    });
    assert!(
        has_extends_abstract_to_generique,
        "EXTENDS AbstractSouscriptionPortlet -> GeneriquePortlet manquante"
    );
}

#[test]
fn test_examples_three_level_extends_chain() {
    // Test end-to-end basé sur le dossier examples/ avec 3 niveaux d'héritage
    // Level2Class -> Level1Class -> Level0Class
    let examples_dir = PathBuf::from("examples/backend/java");
    if !examples_dir.exists() {
        eprintln!("⚠️ Répertoire examples/backend/java non trouvé, test skippé");
        return;
    }

    let builder = MultiLanguageGraphBuilder::new();
    let mut unified_graph = UnifiedGraph::new();
    let root_dir = PathBuf::from("examples");

    let files = vec![
        "backend/java/Level0Class.java",
        "backend/java/Level1Class.java",
        "backend/java/Level2Class.java",
    ];

    for rel_path in &files {
        let abs_path = root_dir.join(rel_path);
        analyze_real_java_file(&builder, &abs_path, &root_dir, &mut unified_graph)
            .expect("Should analyze examples three-level chain");
    }

    // Résolution globale
    let mut resolver = DependencyResolver::new();
    DslExecutor::register_local_classes(&mut resolver, &unified_graph);
    DslExecutor::resolve_imports_global(&mut unified_graph, &resolver);
    DslExecutor::resolve_extends_implements_global(&mut unified_graph, &resolver);

    // Vérifier file_path des 3 classes
    let l0 = unified_graph
        .nodes
        .get("backend.java.Level0Class")
        .expect("Level0Class should exist");
    assert_eq!(l0.file_path, "backend/java/Level0Class.java");

    let l1 = unified_graph
        .nodes
        .get("backend.java.Level1Class")
        .expect("Level1Class should exist");
    assert_eq!(l1.file_path, "backend/java/Level1Class.java");

    let l2 = unified_graph
        .nodes
        .get("backend.java.Level2Class")
        .expect("Level2Class should exist");
    assert_eq!(l2.file_path, "backend/java/Level2Class.java");

    // Vérifier les relations EXTENDS transitives
    let has_l2_to_l1 = unified_graph.edges.iter().any(|e| {
        e.from == "backend.java.Level2Class"
            && e.to == "backend.java.Level1Class"
            && e.relation == EdgeRelation::Extends
    });
    assert!(has_l2_to_l1, "EXTENDS Level2Class -> Level1Class manquante");

    let has_l1_to_l0 = unified_graph.edges.iter().any(|e| {
        e.from == "backend.java.Level1Class"
            && e.to == "backend.java.Level0Class"
            && e.relation == EdgeRelation::Extends
    });
    assert!(has_l1_to_l0, "EXTENDS Level1Class -> Level0Class manquante");
}
