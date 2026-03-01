// Exécuteur DSL avec contexte hiérarchique complet
// Génère des IDs qualifiés: package.Class.method pour éviter collisions

pub mod java;
mod javascript_extractor;
mod rust_extractor;

// Résolution des dépendances
pub mod dependency_resolver;

// WebSphere Portal extractors
pub mod websphere_portal;

use std::collections::HashMap;
use tracing::debug;
use tree_sitter::{Node, Tree};

use self::dependency_resolver::{DependencyResolver, DependencyTarget};
use crate::semantic_graph::semantic_graph::{
    EdgeRelation, Location, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};
use crate::config::PackageFilter;

/// Table de symboles pour résolution de type
/// Mappe les noms de variables vers leurs types réels
#[derive(Debug, Clone)]
struct SymbolTable {
    /// variable_name -> type_name
    types: HashMap<String, String>,
}

impl SymbolTable {
    fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    /// Enregistre une déclaration de variable avec son type
    fn insert(&mut self, var_name: String, type_name: String) {
        self.types.insert(var_name, type_name);
    }

    /// Résout le type d'une variable
    fn resolve_type(&self, var_name: &str) -> Option<&String> {
        self.types.get(var_name)
    }
}

/// Contexte hiérarchique pour tracking de la portée
#[derive(Debug, Clone)]
struct ScopeContext {
    module: Option<String>,           // Module (fichier)
    module_id: Option<String>,        // Node ID du module
    package: Option<String>,          // Java package, Python module
    package_id: Option<String>,       // Node ID du package
    class: Option<String>,            // Classe courante
    class_id: Option<String>,         // Node ID de la classe
    namespace: Option<String>,        // Namespace (Rust mod, JS module)
    current_function: Option<String>, // Fonction courante (pour portée lexicale)
}

impl ScopeContext {
    fn new() -> Self {
        ScopeContext {
            module: None,
            module_id: None,
            package: None,
            package_id: None,
            class: None,
            class_id: None,
            namespace: None,
            current_function: None,
        }
    }

    /// Génère un ID qualifié complet
    fn qualified_id(&self, kind: &str, name: &str, file_path: &str) -> String {
        let mut parts = vec![file_path.to_string()];

        if let Some(pkg) = &self.package {
            parts.push(pkg.clone());
        }
        if let Some(ns) = &self.namespace {
            parts.push(ns.clone());
        }
        if let Some(cls) = &self.class {
            parts.push(cls.clone());
        }

        parts.push(format!("{}:{}", kind, name));
        parts.join("::")
    }

    /// Clone avec nouvelle classe
    fn with_class(&self, class_name: String, class_id: String) -> Self {
        let mut ctx = self.clone();
        ctx.class = Some(class_name);
        ctx.class_id = Some(class_id);
        ctx
    }

    /// Clone avec nouveau package
    fn with_package(&self, package_name: String, package_id: String) -> Self {
        let mut ctx = self.clone();
        ctx.package = Some(package_name);
        ctx.package_id = Some(package_id);
        ctx
    }

    /// Clone avec nouveau module
    fn with_module(&self, module_name: String, module_id: String) -> Self {
        let mut ctx = self.clone();
        ctx.module = Some(module_name);
        ctx.module_id = Some(module_id);
        ctx
    }

    /// Clone avec la fonction courante (pour tracking de portée)
    fn with_function(&self, function_id: String) -> Self {
        let mut ctx = self.clone();
        ctx.current_function = Some(function_id);
        ctx
    }
}

pub struct DslExecutor {
    file_path: String,
    symbol_table: SymbolTable,
}

impl DslExecutor {
    pub fn new(file_path: String) -> Self {
        DslExecutor {
            file_path,
            symbol_table: SymbolTable::new(),
        }
    }

    pub fn extract(&mut self, tree: &Tree, source: &str, language: &str) -> UnifiedGraph {
        let mut graph = UnifiedGraph::new();
        debug!(
            file = %self.file_path,
            language = %language,
            "Extraction avec contexte hiérarchique"
        );

        // Utiliser le registre DSL pour déterminer le type d'extracteur
        use crate::semantic_graph::dsl::{DslRegistry, ExtractorType};

        match DslRegistry::get_extractor_type(language) {
            ExtractorType::Java => {
                let root = tree.root_node();
                self.extract_java(root, source, &mut graph);
            }
            ExtractorType::JavaScript => {
                let root = tree.root_node();
                self.extract_javascript(root, source, &mut graph);
            }
            ExtractorType::Xml => {
                self.extract_xml(source, &mut graph);
            }
            ExtractorType::Jsp => {
                self.extract_jsp(source, &mut graph);
            }
            ExtractorType::Rust => {
                let root = tree.root_node();
                self.extract_rust(root, source, &mut graph);
            }
            ExtractorType::TreeSitter => {
                // Future implémentation: extraction générique via DSL tree-sitter
                debug!("Extraction Tree-Sitter générique (non implémentée)");
            }
            ExtractorType::None => {
                // Pas d'extraction pour ce langage
                debug!("Pas d'extraction pour ce langage");
            }
        }

        debug!(
            nodes = graph.nodes.len(),
            edges = graph.edges.len(),
            "Extraction terminée (CALLS seront résolus globalement)"
        );

        // 🔍 Résolution globale des CALLS différée
        debug!(
            "🔍 Symbol table construite (résolution CALLS globale différée) symbol_table_size={}",
            self.symbol_table.types.len()
        );

        graph
    }

    /// Compatibilité avec l'ancienne API
    pub fn execute(
        &mut self,
        _language_name: &str,
        tree: &Tree,
        source: &str,
        _language: tree_sitter::Language,
        graph: &mut UnifiedGraph,
    ) -> Result<(), String> {
        let extracted = self.extract(tree, source, _language_name);
        *graph = extracted;
        Ok(())
    }

    /// Enregistre toutes les classes/interfaces locales dans le resolver pour résolution ultérieure
    /// Ignore les nœuds marqués is_external pour éviter collision avec classes locales
    pub fn register_local_classes(resolver: &mut DependencyResolver, graph: &UnifiedGraph) {
        for (id, node) in &graph.nodes {
            if node.kind == NodeKind::Class || node.kind == NodeKind::Interface {
                // ⚠️ FILTRER les nœuds externes: on n'enregistre que les classes LOCALES avec file_path
                if node.metadata.get("is_external") == Some(&"true".to_string()) {
                    continue; // Ignorer les nœuds externes
                }

                let fqn = node
                    .metadata
                    .get("qualified_name")
                    .cloned()
                    .unwrap_or_else(|| node.name.clone());
                resolver.register_local(&fqn, id);
            }
        }
    }

    /// Résout globalement les imports Java en créant les relations IMPORTS vers des cibles locales uniquement
    pub fn resolve_imports_global(graph: &mut UnifiedGraph, resolver: &DependencyResolver) {
        let import_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|(_, node)| node.kind == NodeKind::Import)
            .map(|(id, node)| (id.clone(), node.clone()))
            .collect();

        let filter = resolver.filter();

        for (import_id, import_node) in import_nodes {
            let import_path = match import_node.metadata.get("import_path") {
                Some(path) => path.clone(),
                None => continue,
            };

            // Ignorer l'import si son package est exclu
            if let Some(pkg) = PackageFilter::extract_package(&import_path) {
                if !filter.should_process(&pkg) {
                    continue;
                }
            }

            let module_id = import_node
                .metadata
                .get("module_id")
                .cloned()
                .unwrap_or_else(|| format!("{}::module", import_node.file_path));

            let target_id = match resolver.resolve(&import_path) {
                DependencyTarget::Local(node_id) => Some(node_id),
                DependencyTarget::External(_) | DependencyTarget::Unresolved(_) => None,
            };

            if let Some(target) = target_id {
                graph.add_edge(SemanticEdge {
                    from: module_id.clone(),
                    to: target.clone(),
                    relation: EdgeRelation::Imports,
                    metadata: HashMap::new(),
                });

                // Lier aussi le nœud Import au nœud cible pour conserver le contexte source
                graph.add_edge(SemanticEdge {
                    from: import_id.clone(),
                    to: target,
                    relation: EdgeRelation::References,
                    metadata: HashMap::new(),
                });
            }
        }
    }

    /// Résout globalement les Call nodes en relations Function -> Function (local uniquement)
    /// À appeler APRÈS extraction de tous les fichiers
    pub fn resolve_calls_global(graph: &mut UnifiedGraph, resolver: &DependencyResolver) {
        debug!("🔍 Résolution CALLS globale: Function -[CALLS]-> Function (local only)");

        // Construire index: function_name -> [function_ids]
        let mut function_index: HashMap<String, Vec<String>> = HashMap::new();
        for (id, node) in &graph.nodes {
            if node.kind == NodeKind::Function {
                function_index
                    .entry(node.name.clone())
                    .or_insert_with(Vec::new)
                    .push(id.clone());
            }
        }

        // Collecter les relations CALLS temporaires à résoudre
        let temp_call_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Calls && e.to.starts_with("temp_call:"))
            .cloned()
            .collect();

        let mut resolved_count = 0;

        // Retirer les edges temporaires
        graph
            .edges
            .retain(|e| !(e.relation == EdgeRelation::Calls && e.to.starts_with("temp_call:")));

        for edge in temp_call_edges {
            let method_name = match edge.metadata.get("method_name") {
                Some(name) => name,
                None => {
                    tracing::warn!(edge_to = %edge.to, "Edge temp_call sans métadonnée method_name, ignoré");
                    continue;
                }
            };
            let caller_id = &edge.from;

            // Chercher le type de l'objet dans les métadonnées de l'edge
            let target_class = edge.metadata.get("object_type");

            // Résoudre la fonction cible
            let target_function_id = if let Some(class_name) = target_class {
                let package_allowed = PackageFilter::extract_package(class_name)
                    .map(|pkg| resolver.filter().should_process(&pkg))
                    .unwrap_or(true);
                let allow_phantom = resolver.filter().should_create_phantom(class_name);
                if !package_allowed && !allow_phantom {
                    continue;
                }
                // Appel avec type : serviceB -> ServiceB
                if let Some(candidates) = function_index.get(method_name) {
                    if let Some(found) = candidates
                        .iter()
                        .find(|id| id.contains(&format!("::{class_name}::")))
                    {
                        Some(found.clone())
                    } else if allow_phantom {
                        Some(create_phantom_function(
                            graph,
                            class_name,
                            method_name,
                            allow_phantom,
                        ))
                    } else {
                        None
                    }
                } else if allow_phantom {
                    Some(create_phantom_function(
                        graph,
                        class_name,
                        method_name,
                        allow_phantom,
                    ))
                } else {
                    None
                }
            } else {
                // Appel sans objet : chercher dans le même fichier que le caller
                if let Some(candidates) = function_index.get(method_name) {
                    let parts: Vec<&str> = caller_id.split("::").collect();
                    if parts.len() >= 2 {
                        let file = parts[0];
                        candidates
                            .iter()
                            .find(|id| id.starts_with(&format!("{file}::")))
                            .cloned()
                    } else {
                        candidates.first().cloned()
                    }
                } else {
                    None
                }
            };

            // Créer la relation Function -> Function si trouvée
            if let Some(target) = target_function_id {
                graph.add_edge(SemanticEdge {
                    from: caller_id.clone(),
                    to: target,
                    relation: EdgeRelation::Calls,
                    metadata: HashMap::new(),
                });
                resolved_count += 1;
            }
        }

        debug!(
            resolved = resolved_count,
            "✅ Relations CALLS créées (Function -[CALLS]-> Function)"
        );
    }

    /// Résout globalement les relations extends et implements sans créer de nœuds fantômes
    /// À appeler APRÈS resolve_calls_global
    pub fn resolve_extends_implements_global(
        graph: &mut UnifiedGraph,
        resolver: &DependencyResolver,
    ) {
        debug!("🔍 Résolution EXTENDS/IMPLEMENTS globale: Class -[EXTENDS]-> Class, Class -[IMPLEMENTS]-> Interface");

        let class_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|(_, node)| node.kind == NodeKind::Class)
            .map(|(id, node)| (id.clone(), node.clone()))
            .collect();

        let mut extends_count = 0;
        let mut implements_count = 0;
        let mut unresolved_extends = 0;
        let mut unresolved_implements = 0;

        for (class_id, class_node) in class_nodes {
            // Résoudre EXTENDS - utiliser d'abord le FQN complet, sinon le fallback
            if let Some(superclass_fqn) = class_node.metadata.get("superclass") {
                let reference = superclass_fqn.as_str();
                match Self::resolve_class_reference(resolver, reference) {
                    Some(target_id) => {
                        graph.add_edge(SemanticEdge {
                            from: class_id.clone(),
                            to: target_id.clone(),
                            relation: EdgeRelation::Extends,
                            metadata: HashMap::new(),
                        });
                        extends_count += 1;
                        debug!(
                            from = %class_id,
                            to = %target_id,
                            reference = %reference,
                            "✅ EXTENDS relation créée (FQN trouvé)"
                        );
                    }
                    None => {
                        // Fallback: essayer avec le nom simple si on a
                        if let Some(superclass_simple) =
                            class_node.metadata.get("superclass_simple")
                        {
                            if let Some(target_id) =
                                Self::resolve_class_reference(resolver, superclass_simple)
                            {
                                graph.add_edge(SemanticEdge {
                                    from: class_id.clone(),
                                    to: target_id.clone(),
                                    relation: EdgeRelation::Extends,
                                    metadata: HashMap::new(),
                                });
                                extends_count += 1;
                                debug!(
                                    from = %class_id,
                                    to = %target_id,
                                    reference = %superclass_simple,
                                    "✅ EXTENDS relation créée (nom simple fallback)"
                                );
                            } else {
                                unresolved_extends += 1;
                                debug!(
                                    from = %class_id,
                                    superclass = %reference,
                                    "⚠️ Superclass non résolue"
                                );
                            }
                        } else {
                            unresolved_extends += 1;
                            debug!(
                                from = %class_id,
                                superclass = %reference,
                                "⚠️ Superclass non résolue (pas de fallback)"
                            );
                        }
                    }
                }
            }

            // Résoudre IMPLEMENTS - utiliser le FQN complet avec fallback
            if let Some(interfaces_fqn_str) = class_node.metadata.get("interfaces_fqn") {
                for interface_fqn in interfaces_fqn_str.split(',') {
                    let interface_fqn = interface_fqn.trim();
                    match Self::resolve_class_reference(resolver, interface_fqn) {
                        Some(target_id) => {
                            graph.add_edge(SemanticEdge {
                                from: class_id.clone(),
                                to: target_id.clone(),
                                relation: EdgeRelation::Implements,
                                metadata: HashMap::new(),
                            });
                            implements_count += 1;
                            debug!(
                                from = %class_id,
                                to = %target_id,
                                interface = %interface_fqn,
                                "✅ IMPLEMENTS relation créée (FQN trouvé)"
                            );
                        }
                        None => {
                            // Fallback: essayer avec le nom simple
                            if let Some(interfaces_simple) = class_node.metadata.get("interfaces") {
                                // Récupérer le nom simple correspondant
                                if let Some(pos) = interfaces_fqn_str
                                    .split(',')
                                    .position(|i| i.trim() == interface_fqn)
                                {
                                    if let Some(simple) = interfaces_simple.split(',').nth(pos) {
                                        if let Some(target_id) =
                                            Self::resolve_class_reference(resolver, simple.trim())
                                        {
                                            graph.add_edge(SemanticEdge {
                                                from: class_id.clone(),
                                                to: target_id.clone(),
                                                relation: EdgeRelation::Implements,
                                                metadata: HashMap::new(),
                                            });
                                            implements_count += 1;
                                            debug!(
                                                from = %class_id,
                                                to = %target_id,
                                                interface = %simple,
                                                "✅ IMPLEMENTS relation créée (fallback)"
                                            );
                                        } else {
                                            unresolved_implements += 1;
                                            debug!(
                                                from = %class_id,
                                                interface = %interface_fqn,
                                                "⚠️ Interface non résolue"
                                            );
                                        }
                                    }
                                }
                            } else {
                                unresolved_implements += 1;
                                debug!(
                                    from = %class_id,
                                    interface = %interface_fqn,
                                    "⚠️ Interface non résolue (pas de fallback)"
                                );
                            }
                        }
                    }
                }
            }
        }

        debug!(
            extends = extends_count,
            implements = implements_count,
            unresolved_extends = unresolved_extends,
            unresolved_implements = unresolved_implements,
            "✅ Relations EXTENDS et IMPLEMENTS créées"
        );
    }
    /// Résout une référence de classe en utilisant le resolver (pas de nœuds fantômes)
    fn resolve_class_reference(resolver: &DependencyResolver, reference: &str) -> Option<String> {
        // Respecter le filtre de packages
        if let Some(pkg) = PackageFilter::extract_package(reference) {
            if !resolver.filter().should_process(&pkg) {
                return None;
            }
        }

        match resolver.resolve(reference) {
            DependencyTarget::Local(id) => Some(id),
            DependencyTarget::External(_) | DependencyTarget::Unresolved(_) => None,
        }
    }

    /// Extraction XML avec XmlExtractor spécialisé
    fn extract_xml(&mut self, source: &str, graph: &mut UnifiedGraph) {
        use crate::graph_builder::dsl_executor::websphere_portal::XmlExtractor;

        let extractor = XmlExtractor::new();
        let file_name = std::path::Path::new(&self.file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Déterminer le type de fichier XML
        let result = if file_name.eq_ignore_ascii_case("portlet.xml") {
            debug!("Extraction portlet.xml avec XmlExtractor");
            extractor.extract_portlet_xml(&self.file_path, source, graph)
        } else if file_name.eq_ignore_ascii_case("web.xml") {
            debug!("Extraction web.xml avec XmlExtractor");
            extractor.extract_web_xml(&self.file_path, source, graph)
        } else {
            // Fichier XML générique - pas d'extraction spécifique
            debug!("Fichier XML générique - pas d'extraction");
            Ok(())
        };

        if let Err(e) = result {
            tracing::warn!(file = %self.file_path, error = %e, "Erreur extraction XML");
        }
    }

    /// Extraction JSP/JSPX/JSPF avec JspExtractor spécialisé
    fn extract_jsp(&mut self, source: &str, graph: &mut UnifiedGraph) {
        use crate::graph_builder::dsl_executor::websphere_portal::JspExtractor;

        let extractor = JspExtractor::new();

        debug!("Extraction JSP avec JspExtractor (INCLUDES relations)");

        let result = extractor.extract_jsp_relations(&self.file_path, source, graph);

        if let Err(e) = result {
            tracing::warn!(file = %self.file_path, error = %e, "Erreur extraction JSP");
        }
    }

    // ===== UTILITAIRES =====
    fn get_text(&self, node: Node, source: &str) -> String {
        node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
    }

    fn node_location(&self, node: Node) -> Location {
        Location {
            start_line: node.start_position().row + 1,
            start_col: node.start_position().column + 1,
            end_line: node.end_position().row + 1,
            end_col: node.end_position().column + 1,
        }
    }

    /// Location par défaut pour les nœuds synthétiques (module/package)
    fn root_location(&self) -> Location {
        Location {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 1,
        }
    }

    fn metadata(&self, language: &str) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("language".to_string(), language.to_string());
        meta.insert("extractor".to_string(), "hierarchical".to_string());
        meta
    }
}

/// Crée un nœud de classe fantôme et un nœud de fonction fantôme dans le graphe.
/// Retourne l'ID du nœud de fonction créé.
fn create_phantom_function(
    graph: &mut UnifiedGraph,
    class_name: &str,
    method_name: &str,
    _allow_phantom: bool,
) -> String {
    let phantom_class_id = class_name.to_string();
    if !graph.nodes.contains_key(&phantom_class_id) {
        let simple = class_name.split('.').last().unwrap_or(class_name);
        let mut meta = HashMap::new();
        meta.insert("is_phantom".to_string(), "true".to_string());
        meta.insert("language".to_string(), "java".to_string());

        if let Some(last_dot) = class_name.rfind('.') {
            let package = &class_name[..last_dot];
            meta.insert("package".to_string(), package.to_string());
        }

        graph.add_node(SemanticNode {
            id: phantom_class_id.clone(),
            kind: NodeKind::Class,
            name: simple.to_string(),
            file_path: String::new(),
            location: Location::default(),
            metadata: meta,
        });
    }

    let phantom_fn_id = format!("{}::function:{}", class_name, method_name);
    if !graph.nodes.contains_key(&phantom_fn_id) {
        let mut meta = HashMap::new();
        meta.insert("is_phantom".to_string(), "true".to_string());
        meta.insert("language".to_string(), "java".to_string());
        meta.insert("object_type".to_string(), class_name.to_string());

        if let Some(last_dot) = class_name.rfind('.') {
            let package = &class_name[..last_dot];
            meta.insert("package".to_string(), package.to_string());
        }

        graph.add_node(SemanticNode {
            id: phantom_fn_id.clone(),
            kind: NodeKind::Function,
            name: method_name.to_string(),
            file_path: String::new(),
            location: Location::default(),
            metadata: meta,
        });

        if graph.nodes.contains_key(&phantom_class_id) {
            graph.add_edge(SemanticEdge {
                from: phantom_class_id.clone(),
                to: phantom_fn_id.clone(),
                relation: EdgeRelation::Contains,
                metadata: HashMap::new(),
            });
        }
    }

    phantom_fn_id
}
