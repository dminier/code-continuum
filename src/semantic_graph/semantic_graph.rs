// Module pour la représentation et la gestion des graphes sémantiques
// Contient les structures de nœuds, arêtes et le graphe unifié

use std::collections::HashMap;
use tracing::info;

/// Représente un nœud dans le graphe sémantique
#[derive(Debug, Clone)]
pub struct SemanticNode {
    pub id: String,
    pub kind: NodeKind,
    pub name: String,
    pub file_path: String,
    pub location: Location,
    pub metadata: HashMap<String, String>,
}

/// Types de nœuds possibles dans le graphe sémantique
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    // Entités de code principales
    Function,
    Class,
    Interface,
    Module,
    Type,
    Trait,

    // Éléments de fonction
    Parameter,
    Variable,

    // Imports et dépendances
    Import,
    Package,

    // Expressions et opérateurs
    Expression,
    Operator,

    // Configuration WebSphere Portal
    /// Fichier de configuration portlet.xml
    PortletXml,
    /// Fichier de configuration web.xml
    WebXml,
    /// Classe Portlet (GenericPortlet, etc.)
    Portlet,
    /// Classe Servlet
    Servlet,
    /// Classe Filter
    Filter,

    // Fichiers JSP
    /// Fichier JSP (Java Server Page)
    Jsp,

    // Fichiers JavaScript
    /// Fichier JavaScript
    Js,
}

/// Position d'un nœud dans le code source
#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct Location {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

/// Représente une arête (relation) dans le graphe sémantique
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SemanticEdge {
    pub from: String,
    pub to: String,
    pub relation: EdgeRelation,
    pub metadata: HashMap<String, String>, // WebSphere Portal metadata
}

/// Types de relations possibles entre les nœuds
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum EdgeRelation {
    // Relations de définition
    /// Fonction définit un paramètre
    Defines,
    /// Fonction a un paramètre
    HasParam,
    /// Fonction retourne un type
    Returns,
    /// Variable est assignée par une expression
    AssignedBy,

    // Relations d'appel
    /// Fonction appelle une autre fonction
    Calls,
    /// Fonction déclenche un autre nœud
    Triggers,

    // Relations de structure
    /// Classe contient une méthode
    Contains,
    /// Classe hérite d'une classe
    Extends,
    /// Classe implémente une interface
    Implements,

    // Relations de dépendance
    /// Module importe un autre module
    Imports,
    /// Nœud référence une définition
    References,
    /// Fonction utilise une variable
    Uses,

    // Relations de type
    /// Nœud a un type
    HasType,
    /// Nœud déclare un type
    DeclareType,

    // Relations de visibilité
    /// Expose un symbole
    Exports,

    // Relations WebSphere Portal (nouvelles)
    /// Portlet rend une JSP
    Renders,
    /// JSP inclut un fichier JavaScript
    IncludesJs,
    /// JSP inclut un fichier CSS
    IncludesCss,
    /// JSP inclut une autre JSP
    IncludesJsp,
    /// web.xml déclare un Servlet
    Declares,
    /// Filter filtre un Servlet
    Filters,
    /// portlet.xml configure un Portlet
    Configures,
    /// Portlet implémenté par une classe Java
    ImplementedBy,
    /// Service envoie une notification
    Notifies,
    /// JavaScript dépend d'un autre JavaScript
    DependsOn,
    /// JSP et JavaScript partagent des données
    BindsData,
    /// JavaScript cible un élément DOM
    TargetsElement,
}

/// Graphe sémantique unifié pour l'analyse multi-langage
#[derive(Debug)]
pub struct UnifiedGraph {
    pub nodes: HashMap<String, SemanticNode>,
    pub edges: Vec<SemanticEdge>,
}

impl UnifiedGraph {
    /// Crée un nouveau graphe vide
    pub fn new() -> Self {
        UnifiedGraph {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }

    /// Ajoute un nœud au graphe
    pub fn add_node(&mut self, node: SemanticNode) {
        // Si un nœud existe déjà avec cet ID, privilégier le nœud le plus complet
        if let Some(existing) = self.nodes.get(&node.id) {
            // Cas 1: Le nœud existant est local et le nouveau est externe → ignorer le nouveau
            if !existing.file_path.is_empty() && node.file_path.is_empty() {
                tracing::trace!(
                    id = %node.id,
                    existing_file = %existing.file_path,
                    "Ignoring external node (local already exists)"
                );
                return;
            }

            // Cas 2: Le nœud existant a un file_path absolu et le nouveau a un chemin web relatif → ignorer le nouveau
            // Un chemin absolu contient généralement le répertoire du projet (ex: "examples/" ou "/workspaces/")
            // Un chemin web commence par "/" mais n'a pas de répertoire (ex: "/resources/js/app.js")
            let existing_is_absolute = existing.file_path.contains('/')
                && (existing.file_path.contains("examples/")
                    || existing.file_path.contains("workspaces/")
                    || existing.file_path.contains("src/")
                    || !existing.file_path.starts_with('/'));

            let new_is_web_path = node.file_path.starts_with('/')
                && !node.file_path.contains("workspaces/")
                && !node.file_path.contains("examples/");

            if existing_is_absolute && new_is_web_path {
                tracing::trace!(
                    id = %node.id,
                    existing_file = %existing.file_path,
                    new_file = %node.file_path,
                    "Ignoring web path node (absolute path already exists)"
                );
                return;
            }

            // Cas 3: Privilégier le nœud avec le Location le plus précis (non-default)
            let existing_has_location = existing.location.start_line > 0;
            let new_has_location = node.location.start_line > 0;

            if existing_has_location && !new_has_location {
                tracing::trace!(
                    id = %node.id,
                    existing_file = %existing.file_path,
                    "Ignoring node without location (node with location already exists)"
                );
                return;
            }

            // Sinon, écraser avec le nouveau nœud (comportement par défaut)
        }

        self.nodes.insert(node.id.clone(), node);
    }

    /// Ajoute une arête au graphe
    pub fn add_edge(&mut self, edge: SemanticEdge) {
        self.edges.push(edge);
    }

    /// Retourne tous les nœuds de type fonction
    pub fn find_functions(&self) -> Vec<&SemanticNode> {
        self.nodes
            .values()
            .filter(|n| n.kind == NodeKind::Function)
            .collect()
    }

    /// Retourne tous les appels vers une fonction donnée
    #[allow(dead_code)]
    pub fn find_calls_to(&self, function_name: &str) -> Vec<&SemanticEdge> {
        self.edges
            .iter()
            .filter(|e| {
                e.relation == EdgeRelation::Calls
                    && self
                        .nodes
                        .get(&e.to)
                        .is_some_and(|n| n.name == function_name)
            })
            .collect()
    }

    /// Affiche un résumé du graphe via tracing
    pub fn print_summary(&self) {
        let functions = self
            .nodes
            .values()
            .filter(|n| n.kind == NodeKind::Function)
            .count();
        let classes = self
            .nodes
            .values()
            .filter(|n| n.kind == NodeKind::Class)
            .count();
        let variables = self
            .nodes
            .values()
            .filter(|n| n.kind == NodeKind::Variable)
            .count();
        let interfaces = self
            .nodes
            .values()
            .filter(|n| n.kind == NodeKind::Interface)
            .count();
        let modules = self
            .nodes
            .values()
            .filter(|n| n.kind == NodeKind::Module)
            .count();
        let imports = self
            .nodes
            .values()
            .filter(|n| n.kind == NodeKind::Import)
            .count();

        // Compter les relations par type
        let calls = self
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Calls)
            .count();
        let extends = self
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Extends)
            .count();
        let implements = self
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Implements)
            .count();
        let defines = self
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Defines)
            .count();
        let imports_edges = self
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::Imports)
            .count();

        info!("═══════════════════════════════════════════════════════════");
        info!("📊 GRAPH SUMMARY");
        info!("═══════════════════════════════════════════════════════════");
        info!(
            "📈 Nodes:   {} total | Functions: {} | Classes: {} | Interfaces: {}",
            self.nodes.len(),
            functions,
            classes,
            interfaces
        );
        info!(
            "   Modules: {} | Variables: {} | Imports: {}",
            modules, variables, imports
        );
        info!(
            "🔗 Edges:   {} total | CALLS: {} | EXTENDS: {} | IMPLEMENTS: {}",
            self.edges.len(),
            calls,
            extends,
            implements
        );
        info!("   DEFINES: {} | IMPORTS: {}", defines, imports_edges);
        info!("═══════════════════════════════════════════════════════════");
    }

    /// Résout les métadonnées EXTENDS/IMPLEMENTS en créant les relations correspondantes
    /// Utilisé principalement pour les tests et constructions simples sans DependencyResolver
    /// Crée des nœuds fantômes si les classes/interfaces ne sont pas trouvées localement
    #[allow(dead_code)]
    pub fn resolve_extends_implements_local(&mut self) {
        let class_nodes: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, node)| node.kind == NodeKind::Class)
            .map(|(id, node)| (id.clone(), node.clone()))
            .collect();

        for (class_id, class_node) in class_nodes {
            // Résoudre EXTENDS - essayer d'abord le FQN complet, puis le nom simple
            if let Some(superclass_fqn) = class_node.metadata.get("superclass") {
                // Essayer le FQN d'abord
                if self.nodes.contains_key(superclass_fqn) {
                    self.add_edge(SemanticEdge {
                        from: class_id.clone(),
                        to: superclass_fqn.clone(),
                        relation: EdgeRelation::Extends,
                        metadata: HashMap::new(),
                    });
                } else if let Some(superclass_simple) = class_node.metadata.get("superclass_simple")
                {
                    // Fallback: chercher par nom simple
                    if let Some((target_id, _)) = self
                        .nodes
                        .iter()
                        .find(|(_, n)| n.name == *superclass_simple && n.kind == NodeKind::Class)
                    {
                        self.add_edge(SemanticEdge {
                            from: class_id.clone(),
                            to: target_id.clone(),
                            relation: EdgeRelation::Extends,
                            metadata: HashMap::new(),
                        });
                    } else {
                        // Créer un nœud fantôme pour la superclasse externe
                        let phantom_id = superclass_fqn.clone();
                        if !self.nodes.contains_key(&phantom_id) {
                            let mut phantom_metadata = HashMap::new();
                            phantom_metadata.insert("is_external".to_string(), "true".to_string());
                            phantom_metadata.insert("language".to_string(), "java".to_string());

                            // Extraire le package du FQN
                            if let Some(last_dot) = superclass_fqn.rfind('.') {
                                let package = &superclass_fqn[..last_dot];
                                phantom_metadata.insert("package".to_string(), package.to_string());
                            }

                            self.add_node(SemanticNode {
                                id: phantom_id.clone(),
                                kind: NodeKind::Class,
                                name: superclass_simple.clone(),
                                file_path: "".to_string(),
                                location: Location::default(),
                                metadata: phantom_metadata,
                            });
                        }

                        self.add_edge(SemanticEdge {
                            from: class_id.clone(),
                            to: phantom_id,
                            relation: EdgeRelation::Extends,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }

            // Résoudre IMPLEMENTS
            if let Some(interfaces_fqn_str) = class_node.metadata.get("interfaces_fqn") {
                let interfaces_fqn: Vec<&str> =
                    interfaces_fqn_str.split(',').map(|s| s.trim()).collect();
                let interfaces_simple: Vec<&str> = class_node
                    .metadata
                    .get("interfaces")
                    .map(|s| s.split(',').map(|x| x.trim()).collect())
                    .unwrap_or_default();

                for (idx, interface_fqn) in interfaces_fqn.iter().enumerate() {
                    // Essayer le FQN d'abord
                    if self.nodes.contains_key(*interface_fqn) {
                        self.add_edge(SemanticEdge {
                            from: class_id.clone(),
                            to: interface_fqn.to_string(),
                            relation: EdgeRelation::Implements,
                            metadata: HashMap::new(),
                        });
                    } else if let Some(interface_simple) = interfaces_simple.get(idx) {
                        // Fallback: chercher par nom simple
                        if let Some((target_id, _)) = self.nodes.iter().find(|(_, n)| {
                            n.name == *interface_simple
                                && (n.kind == NodeKind::Interface || n.kind == NodeKind::Class)
                        }) {
                            self.add_edge(SemanticEdge {
                                from: class_id.clone(),
                                to: target_id.clone(),
                                relation: EdgeRelation::Implements,
                                metadata: HashMap::new(),
                            });
                        } else {
                            // Créer un nœud fantôme pour l'interface externe
                            let phantom_id = interface_fqn.to_string();
                            if !self.nodes.contains_key(&phantom_id) {
                                let mut phantom_metadata = HashMap::new();
                                phantom_metadata
                                    .insert("is_external".to_string(), "true".to_string());
                                phantom_metadata.insert("language".to_string(), "java".to_string());

                                // Extraire le package du FQN
                                if let Some(last_dot) = interface_fqn.rfind('.') {
                                    let package = &interface_fqn[..last_dot];
                                    phantom_metadata
                                        .insert("package".to_string(), package.to_string());
                                }

                                self.add_node(SemanticNode {
                                    id: phantom_id.clone(),
                                    kind: NodeKind::Interface,
                                    name: interface_simple.to_string(),
                                    file_path: "".to_string(),
                                    location: Location::default(),
                                    metadata: phantom_metadata,
                                });
                            }

                            self.add_edge(SemanticEdge {
                                from: class_id.clone(),
                                to: phantom_id,
                                relation: EdgeRelation::Implements,
                                metadata: HashMap::new(),
                            });
                        }
                    }
                }
            }
        }
    }
}

impl Default for UnifiedGraph {
    fn default() -> Self {
        Self::new()
    }
}
