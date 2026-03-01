// Module DSL pour les définitions Tree-sitter Graph
// Centralise les considérations par langage (extensions, parser, kinds)
//
//! # Module DSL (Domain-Specific Language)
//!
//! Ce module centralise toutes les spécifications par langage pour l'analyse de code:
//! - Extensions de fichiers supportées
//! - Parsers tree-sitter associés
//! - Types de nœuds AST pour fonctions/méthodes
//! - Détection d'appels de fonction
//!
//! ## Ajout d'un nouveau langage
//!
//! Pour supporter un nouveau langage, il suffit de modifier ce fichier:
//! 1. Ajouter la dépendance tree-sitter dans `Cargo.toml`
//! 2. Ajouter l'import du parser (ex: `tree_sitter_go::language`)
//! 3. Ajouter une constante DSL (ex: `GO_TSG`)
//! 4. Ajouter une entrée dans `all_specs()`
//! 5. Ajouter un case dans `get_dsl()`
//! 6. Optionnel: ajouter la détection d'appels dans `extract_callee_name()`
//!
//! ## Exemple d'utilisation
//!
//! ```rust
//! use std::path::Path;
//! use code_continuum:code_continuum::dsl::DslRegistry;
//!
//! // Détecter le langage d'un fichier
//! let path = Path::new("src/main.rs");
//! if let Some(lang) = DslRegistry::detect_language_from_path(path) {
//!     println!("Langage détecté: {}", lang);
//!     
//!     // Obtenir le parser tree-sitter
//!     if let Some(parser) = DslRegistry::get_tree_sitter_language(lang) {
//!         // Utiliser le parser pour analyser le code
//!     }
//! }
//! ```

use std::path::Path;
use tree_sitter::Language;

// Fournisseurs de Language tree-sitter
use tree_sitter_html::language as ts_html;
use tree_sitter_java::language as ts_java;
use tree_sitter_javascript::language as ts_javascript;
use tree_sitter_python::language as ts_python;
use tree_sitter_rust::language as ts_rust;

/// Type d'extracteur à utiliser pour un langage
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractorType {
    /// Pas d'extraction spécifique
    None,
    /// Extracteur Tree-Sitter générique (DSL)
    TreeSitter,
    /// Extracteur Java spécialisé (AST hiérarchique)
    Java,
    /// Extracteur JavaScript spécialisé
    JavaScript,
    /// Extracteur XML spécialisé (portlet.xml, web.xml)
    Xml,
    /// Extracteur JSP spécialisé (INCLUDES relations)
    Jsp,
}

/// Spécification complète d'un langage de programmation supporté
///
/// Contient toutes les métadonnées nécessaires pour parser et analyser un langage:
/// - Nom canonique du langage (utilisé comme identifiant)
/// - Extensions de fichiers reconnues (sans le point)
/// - Constructeur du parser tree-sitter
/// - Type de nœud AST représentant une fonction/méthode
/// - Nom du champ AST contenant le nom de la fonction
/// - Noms de fichiers spécifiques (pour web.xml, portlet.xml, etc.)
/// - Type d'extracteur spécialisé à utiliser
///
/// # Exemple
///
/// ```rust,ignore
/// use code_continuum:code_continuum::dsl::LanguageSpec;
/// LanguageSpec {
///     name: "python",
///     extensions: &["py", "pyw"],
///     tree_sitter: tree_sitter_python::language,
///     function_node_kind: "function_definition",
///     function_name_field: "name",
///     file_name_patterns: &[],
///     extractor: ExtractorType::TreeSitter,
/// }
/// ```
#[allow(dead_code)]
pub struct LanguageSpec {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub tree_sitter: fn() -> Language,
    pub function_node_kind: &'static str,
    pub function_name_field: &'static str,
    /// Noms de fichiers spécifiques à détecter (ex: "portlet.xml", "web.xml")
    pub file_name_patterns: &'static [&'static str],
    /// Type d'extracteur spécialisé à utiliser
    pub extractor: ExtractorType,
}

/// DSL Tree-sitter pour Python
/// Extraction des fonctions et appels
#[allow(dead_code)]
pub const PYTHON_TSG: &str = r#"
(module) @root {
    node @root.module
}

(function_definition
    name: (identifier) @func_name
) @func {
    node func
    attr (func) type = "function"
    attr (func) name = (source-text @func_name)
    attr (func) language = "python"
}

(call
    function: (identifier) @call_target
) @call_node {
    node call
    attr (call) type = "call"
    attr (call) target = (source-text @call_target)
}
"#;

/// DSL Tree-sitter pour JavaScript/TypeScript
/// Extraction des déclarations de fonctions et appels
#[allow(dead_code)]
pub const JAVASCRIPT_TSG: &str = r#"
(program) @root {
    node @root.module
}

(function_declaration
    name: (identifier) @func_name
) @func {
    node func
    attr (func) type = "function"
    attr (func) name = (source-text @func_name)
    attr (func) language = "javascript"
}

(call_expression
    function: (identifier) @call_target
) @call_node {
    node call
    attr (call) type = "call"
    attr (call) target = (source-text @call_target)
}
"#;

/// DSL Tree-sitter pour Rust
/// Extraction des items de fonction et appels
#[allow(dead_code)]
pub const RUST_TSG: &str = r#"
(source_file) @root {
    node @root.module
}

(function_item
    name: (identifier) @func_name
) @func {
    node func
    attr (func) type = "function"
    attr (func) name = (source-text @func_name)
    attr (func) language = "rust"
}

(call_expression
    function: (identifier) @call_target
) @call_node {
    node call
    attr (call) type = "call"
    attr (call) target = (source-text @call_target)
}
"#;

/// DSL Tree-sitter pour Java
/// Extraction des déclarations de méthodes
#[allow(dead_code)]
pub const JAVA_TSG: &str = r#"
(program) @root {
    node @root.module
}

(method_declaration
    name: (identifier) @method_name
) @method {
    node func
    attr (func) type = "function"
    attr (func) name = (source-text @method_name)
    attr (func) language = "java"
}
"#;

/// DSL Tree-sitter pour HTML (stub minimal)
#[allow(dead_code)]
pub const HTML_TSG: &str = r#"
(document) @root {
    node @root.module
}
"#;

/// DSL Tree-sitter pour JSP (utilise la grammaire HTML)
#[allow(dead_code)]
pub const JSP_TSG: &str = HTML_TSG;

/// DSL Tree-sitter pour JSPX (utilise la grammaire HTML/XML)
#[allow(dead_code)]
pub const JSPX_TSG: &str = HTML_TSG;

/// DSL Tree-sitter pour XML (stub minimal)
#[allow(dead_code)]
pub const XML_TSG: &str = r#"
(document) @root {
    node @root.module
}
"#;

/// Registre des grammaires DSL disponibles
/// Fournit un accès centralisé à tous les DSL supportés
pub struct DslRegistry;

impl DslRegistry {
    /// Retourne les spécifications de tous les langages supportés
    ///
    /// # Retour
    /// Slice statique de `LanguageSpec` contenant les métadonnées pour chaque langage:
    /// - nom du langage
    /// - extensions de fichier supportées
    /// - fonction de création du parser tree-sitter
    /// - type de nœud pour les fonctions/méthodes
    /// - nom du champ contenant le nom de fonction
    ///
    /// # Exemple
    /// ```
    /// use code_continuum:code_continuum::dsl::DslRegistry;
    /// let specs = DslRegistry::all_specs();
    /// for spec in specs {
    ///     println!("Langage: {}, Extensions: {:?}", spec.name, spec.extensions);
    /// }
    /// ```
    pub fn all_specs() -> &'static [LanguageSpec] {
        &[
            LanguageSpec {
                name: "python",
                extensions: &["py"],
                tree_sitter: || ts_python(),
                function_node_kind: "function_definition",
                function_name_field: "name",
                file_name_patterns: &[],
                extractor: ExtractorType::TreeSitter,
            },
            LanguageSpec {
                name: "javascript",
                extensions: &["js", "jsx", "ts", "tsx"],
                tree_sitter: || ts_javascript(),
                function_node_kind: "function_declaration",
                function_name_field: "name",
                file_name_patterns: &[],
                extractor: ExtractorType::JavaScript,
            },
            LanguageSpec {
                name: "html",
                extensions: &["html", "htm"],
                tree_sitter: || ts_html(),
                function_node_kind: "",
                function_name_field: "name",
                file_name_patterns: &[],
                extractor: ExtractorType::None,
            },
            LanguageSpec {
                name: "xml",
                extensions: &["xml"],
                tree_sitter: || ts_html(),
                function_node_kind: "",
                function_name_field: "name",
                file_name_patterns: &["portlet.xml", "web.xml"],
                extractor: ExtractorType::Xml,
            },
            LanguageSpec {
                name: "jsp",
                extensions: &["jsp"],
                tree_sitter: || ts_html(),
                function_node_kind: "",
                function_name_field: "name",
                file_name_patterns: &[],
                extractor: ExtractorType::Jsp,
            },
            LanguageSpec {
                name: "jspx",
                extensions: &["jspx"],
                tree_sitter: || ts_html(),
                function_node_kind: "",
                function_name_field: "name",
                file_name_patterns: &[],
                extractor: ExtractorType::Jsp,
            },
            LanguageSpec {
                name: "jspf",
                extensions: &["jspf"],
                tree_sitter: || ts_html(),
                function_node_kind: "",
                function_name_field: "name",
                file_name_patterns: &[],
                extractor: ExtractorType::Jsp,
            },
            LanguageSpec {
                name: "java",
                extensions: &["java"],
                tree_sitter: || ts_java(),
                function_node_kind: "method_declaration",
                function_name_field: "name",
                file_name_patterns: &[],
                extractor: ExtractorType::Java,
            },
            LanguageSpec {
                name: "rust",
                extensions: &["rs"],
                tree_sitter: || ts_rust(),
                function_node_kind: "function_item",
                function_name_field: "name",
                file_name_patterns: &[],
                extractor: ExtractorType::TreeSitter,
            },
        ]
    }

    /// Retourne la spécification d'un langage donné
    ///
    /// # Arguments
    /// * `language` - Nom du langage (ex: "python", "javascript", "java")
    ///
    /// # Retour
    /// `Some(LanguageSpec)` si le langage est supporté, `None` sinon
    ///
    /// # Exemple
    /// ```
    /// use code_continuum:code_continuum::dsl::DslRegistry;
    /// if let Some(spec) = DslRegistry::get_spec("python") {
    ///     println!("Extensions Python: {:?}", spec.extensions);
    /// }
    /// ```
    pub fn get_spec(language: &str) -> Option<&'static LanguageSpec> {
        Self::all_specs().iter().find(|s| s.name == language)
    }

    /// Retourne le DSL Tree-sitter Graph pour un langage donné
    ///
    /// Le DSL est une chaîne contenant la grammaire tree-sitter-graph
    /// utilisée pour extraire les nœuds et relations du code source.
    ///
    /// # Arguments
    /// * `language` - Nom du langage ("python", "javascript", "rust", etc.)
    ///
    /// # Retour
    /// `Some(&str)` contenant le DSL si le langage est supporté, `None` sinon
    ///
    /// # Exemple
    /// ```
    /// use code_continuum:code_continuum::dsl::DslRegistry;
    /// if let Some(dsl) = DslRegistry::get_dsl("python") {
    ///     println!("DSL Python: {}", dsl);
    /// }
    /// ```
    #[allow(dead_code)]
    pub fn get_dsl(language: &str) -> Option<&'static str> {
        match language {
            "python" => Some(PYTHON_TSG),
            "javascript" => Some(JAVASCRIPT_TSG),
            "html" => Some(HTML_TSG),
            "xml" => Some(XML_TSG),
            "jsp" => Some(JSP_TSG),
            "jspx" => Some(JSPX_TSG),
            "jspf" => Some(JSP_TSG),
            "java" => Some(JAVA_TSG),
            "rust" => Some(RUST_TSG),
            _ => None,
        }
    }

    /// Retourne la liste des noms de tous les langages supportés
    ///
    /// # Retour
    /// Vecteur contenant les noms des langages ("python", "javascript", "java", etc.)
    ///
    /// # Exemple
    /// ```
    /// use code_continuum:code_continuum::dsl::DslRegistry;
    /// let languages = DslRegistry::supported_languages();
    /// println!("Langages supportés: {}", languages.join(", "));
    /// // Affiche: "Langages supportés: python, javascript, html, xml, ..."
    /// ```
    #[allow(dead_code)]
    pub fn supported_languages() -> Vec<&'static str> {
        Self::all_specs().iter().map(|s| s.name).collect::<Vec<_>>()
    }

    /// Retourne l'objet tree-sitter Language pour un langage donné
    ///
    /// Construit et retourne le parser tree-sitter spécifique au langage.
    /// Nécessaire pour analyser le code source avec tree-sitter.
    ///
    /// # Arguments
    /// * `language` - Nom du langage
    ///
    /// # Retour
    /// `Some(Language)` si le langage est supporté, `None` sinon
    ///
    /// # Exemple
    /// ```
    /// use tree_sitter::Parser;
    /// use code_continuum:code_continuum::dsl::DslRegistry;
    /// if let Some(lang) = DslRegistry::get_tree_sitter_language("python") {
    ///     let mut parser = Parser::new();
    ///     parser.set_language(lang).unwrap();
    /// }
    /// ```
    pub fn get_tree_sitter_language(language: &str) -> Option<Language> {
        Self::get_spec(language).map(|s| (s.tree_sitter)())
    }

    /// Détecte le langage à partir d'une extension de fichier
    ///
    /// Comparaison insensible à la casse. Supporte plusieurs extensions
    /// pour un même langage (ex: js, jsx, ts, tsx → javascript).
    ///
    /// # Arguments
    /// * `ext` - Extension de fichier sans le point (ex: "py", "java", "rs")
    ///
    /// # Retour
    /// `Some(&str)` nom du langage si reconnu, `None` sinon
    ///
    /// # Exemple
    /// ```
    /// use code_continuum:code_continuum::dsl::DslRegistry;
    /// assert_eq!(DslRegistry::detect_language_from_extension("py"), Some("python"));
    /// assert_eq!(DslRegistry::detect_language_from_extension("tsx"), Some("javascript"));
    /// assert_eq!(DslRegistry::detect_language_from_extension("unknown"), None);
    /// ```
    pub fn detect_language_from_extension(ext: &str) -> Option<&'static str> {
        let ext = ext.to_ascii_lowercase();
        Self::all_specs()
            .iter()
            .find(|s| s.extensions.iter().any(|e| e.eq_ignore_ascii_case(&ext)))
            .map(|s| s.name)
    }

    /// Détecte le langage à partir d'un chemin de fichier complet
    ///
    /// Vérifie d'abord les noms de fichiers spécifiques (portlet.xml, web.xml)
    /// puis l'extension du fichier.
    /// Utile pour scanner des répertoires et identifier automatiquement les fichiers supportés.
    ///
    /// # Arguments
    /// * `path` - Chemin complet du fichier (absolu ou relatif)
    ///
    /// # Retour
    /// `Some(&str)` nom du langage si reconnu, `None` si l'extension n'est pas supportée
    ///
    /// # Exemple
    /// ```
    /// use std::path::Path;
    /// use code_continuum:code_continuum::dsl::DslRegistry;
    /// let path = Path::new("/src/main.rs");
    /// assert_eq!(DslRegistry::detect_language_from_path(path), Some("rust"));
    /// let path = Path::new("/WEB-INF/portlet.xml");
    /// assert_eq!(DslRegistry::detect_language_from_path(path), Some("xml"));
    /// ```
    pub fn detect_language_from_path(path: &Path) -> Option<&'static str> {
        // Vérifier d'abord les noms de fichiers spécifiques
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            for spec in Self::all_specs() {
                if spec
                    .file_name_patterns
                    .iter()
                    .any(|pattern| file_name.eq_ignore_ascii_case(pattern))
                {
                    return Some(spec.name);
                }
            }
        }

        // Sinon, détecter par extension
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(|ext| Self::detect_language_from_extension(ext))
    }

    /// Vérifie si un langage est supporté par le registre
    ///
    /// # Arguments
    /// * `language` - Nom du langage à vérifier
    ///
    /// # Retour
    /// `true` si le langage est supporté, `false` sinon
    ///
    /// # Exemple
    /// ```
    /// use code_continuum:code_continuum::dsl::DslRegistry;
    /// assert!(DslRegistry::is_supported("python"));
    /// assert!(DslRegistry::is_supported("java"));
    /// assert!(!DslRegistry::is_supported("cobol"));
    /// ```
    #[allow(dead_code)]
    pub fn is_supported(language: &str) -> bool {
        Self::get_spec(language).is_some()
    }

    /// Vérifie si un langage possède un extracteur spécialisé
    ///
    /// Certains langages (Java, JavaScript, XML avec portlet.xml/web.xml)
    /// ont des extracteurs spécialisés qui vont au-delà du simple DSL tree-sitter.
    ///
    /// # Arguments
    /// * `language` - Nom du langage à vérifier
    ///
    /// # Retour
    /// `true` si un extracteur spécialisé existe, `false` sinon
    ///
    /// # Exemple
    /// ```
    /// use code_continuum:code_continuum::dsl::DslRegistry;
    /// assert!(DslRegistry::has_specialized_extractor("java"));
    /// assert!(DslRegistry::has_specialized_extractor("xml"));
    /// ```
    #[allow(dead_code)]
    pub fn has_specialized_extractor(language: &str) -> bool {
        Self::get_extractor_type(language) != ExtractorType::None
            && Self::get_extractor_type(language) != ExtractorType::TreeSitter
    }

    /// Retourne le type d'extracteur pour un langage
    ///
    /// # Arguments
    /// * `language` - Nom du langage
    ///
    /// # Retour
    /// Type d'extracteur à utiliser
    ///
    /// # Exemple
    /// ```
    /// use code_continuum:code_continuum::dsl::{DslRegistry, ExtractorType};
    /// assert_eq!(DslRegistry::get_extractor_type("java"), ExtractorType::Java);
    /// assert_eq!(DslRegistry::get_extractor_type("xml"), ExtractorType::Xml);
    /// ```
    pub fn get_extractor_type(language: &str) -> ExtractorType {
        Self::get_spec(language)
            .map(|spec| spec.extractor)
            .unwrap_or(ExtractorType::None)
    }

    /// Extrait le nom de la fonction appelée depuis un nœud tree-sitter
    ///
    /// Détecte les appels de fonction selon les conventions du langage:
    /// - **Python**: nœud `call` avec champ `function`
    /// - **JavaScript/TypeScript**: nœud `call_expression` avec champ `function`
    /// - **Rust**: nœud `call_expression` avec champ `function`
    /// - **Java**: nœud `method_invocation` avec enfant `identifier`
    ///
    /// # Arguments
    /// * `language` - Nom du langage source
    /// * `node` - Nœud tree-sitter à analyser
    /// * `source` - Code source complet (pour extraction de texte)
    ///
    /// # Retour
    /// `Some(String)` nom de la fonction appelée, `None` si le nœud n'est pas un appel
    ///
    /// # Exemple
    /// ```
    /// // En Python, pour le code: "print('hello')"
    /// // Retourne: Some("print")
    /// ```
    #[allow(dead_code)]
    pub fn extract_callee_name(
        language: &str,
        node: tree_sitter::Node,
        source: &str,
    ) -> Option<String> {
        match language {
            // Python: node kind `call`, field `function`
            "python" => {
                if node.kind() == "call" {
                    if let Some(fnode) = node.child_by_field_name("function") {
                        return Self::identifier_text(&fnode, source);
                    }
                }
                None
            }
            // JavaScript/TypeScript: `call_expression`, field `function`
            "javascript" | "typescript" => {
                if node.kind() == "call_expression" {
                    if let Some(fnode) = node.child_by_field_name("function") {
                        return Self::identifier_text(&fnode, source);
                    }
                }
                None
            }
            // Rust: `call_expression`, field `function`
            "rust" => {
                if node.kind() == "call_expression" {
                    if let Some(fnode) = node.child_by_field_name("function") {
                        return Self::identifier_text(&fnode, source);
                    }
                }
                None
            }
            // Java: `method_invocation` with a `name` field
            "java" => {
                if node.kind() == "method_invocation" {
                    // Try to get the method name from the 'name' field
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &source[name_node.start_byte()..name_node.end_byte()];
                        return Some(name.to_string());
                    }

                    // Fallback: find first identifier child
                    let mut cursor = node.walk();
                    for child in node.children(&mut cursor) {
                        if child.kind() == "identifier" {
                            let name = &source[child.start_byte()..child.end_byte()];
                            return Some(name.to_string());
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Extrait le texte d'un identifiant depuis un nœud tree-sitter
    ///
    /// Gère récursivement les expressions membre/champ pour extraire
    /// le dernier identifiant trouvé. Utile pour les appels chaînés
    /// comme `obj.method()` où on veut extraire "method".
    fn identifier_text(node: &tree_sitter::Node, source: &str) -> Option<String> {
        if node.kind() == "identifier" {
            let name = &source[node.start_byte()..node.end_byte()];
            return Some(name.to_string());
        }
        // Some languages wrap identifiers in member/field expressions; pick the last identifier
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(name) = Self::identifier_text(&child, source) {
                return Some(name);
            }
        }
        None
    }
}
