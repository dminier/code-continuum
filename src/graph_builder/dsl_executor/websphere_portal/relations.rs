/// Types et constantes pour les relations sémantiques WebSphere Portal
///
/// Ce module définit les types de relations spécifiques à WebSphere Portal
/// et les métadonnées associées.
///
/// Note: Les appels de fonctions (AJAX, Service→DAO, etc.) utilisent la relation
/// générique CALLS avec des métadonnées pour distinguer le type d'appel.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Relations sémantiques spécifiques WebSphere Portal
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WebSphereRelation {
    /// Portlet → JSP (dispatch/render)
    /// Annotations: {mode: "view|edit|help"}
    Renders,

    /// JSP → JavaScript
    /// Annotations: {position: int, async: bool}
    IncludesJs,

    /// JSP → CSS
    /// Annotations: {position: int, media: "screen|print|all"}
    IncludesCss,

    /// JSP → JSP (fragment include)
    /// Annotations: {type: "static|dynamic"}
    IncludesJsp,

    /// web.xml → Servlet (declaration)
    /// Annotations: {urlPattern: string, loadOnStartup: int}
    Declares,

    /// Filter → Servlet (filter chain)
    /// Annotations: {order: int, urlPattern: string}
    Filters,

    /// portlet.xml → Portlet (configuration)
    /// Annotations: {modes: [string], states: [string], cacheSeconds: int}
    Configures,

    /// Service → NotificationService (side effect)
    /// Annotations: {type: "EMAIL|PUSH|IN_APP|SMS", trigger: string, async: bool}
    Notifies,

    /// JavaScript → JavaScript (dependency)
    /// Annotations: {type: "import|require|scriptOrder"}
    DependsOn,

    /// JSP ↔ JavaScript (data binding)
    /// Annotations: {variables: [string], direction: "JSP_TO_JS|JS_TO_JSP|BIDIRECTIONAL"}
    BindsData,

    /// JavaScript → DOM Element (manipulation)
    /// Annotations: {selector: string, action: "show|hide|click|change"}
    TargetsElement,
}

impl WebSphereRelation {
    /// Convertit en string pour Neo4j
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Renders => "RENDERS",
            Self::IncludesJs => "INCLUDES_JS",
            Self::IncludesCss => "INCLUDES_CSS",
            Self::IncludesJsp => "INCLUDES_JSP",
            Self::Declares => "DECLARES",
            Self::Filters => "FILTERS",
            Self::Configures => "CONFIGURES",
            Self::Notifies => "NOTIFIES",
            Self::DependsOn => "DEPENDS_ON",
            Self::BindsData => "BINDS_DATA",
            Self::TargetsElement => "TARGETS_ELEMENT",
        }
    }

    /// Retourne la priorité de la relation (pour l'ordre d'extraction)
    #[allow(dead_code)]
    pub fn priority(&self) -> RelationPriority {
        match self {
            Self::Renders | Self::Declares => RelationPriority::Critical,
            Self::IncludesJs | Self::IncludesCss | Self::Configures | Self::Filters => {
                RelationPriority::High
            }
            Self::Notifies | Self::IncludesJsp | Self::DependsOn | Self::BindsData => {
                RelationPriority::Medium
            }
            Self::TargetsElement => RelationPriority::Low,
        }
    }
}

/// Priorité d'extraction des relations
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RelationPriority {
    /// Relations critiques (parsées en priorité)
    Critical = 0,
    /// Relations hautes priorités
    High = 1,
    /// Relations moyennes priorités
    Medium = 2,
    /// Relations basses priorités
    Low = 3,
}

/// Métadonnées pour une relation RENDERS (Portlet → JSP)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RendersMetadata {
    /// Mode du portlet: view, edit, help, custom
    pub mode: String,
    /// Ligne dans le code source
    pub line: usize,
    /// Méthode source: doView, doEdit, processAction, etc.
    pub method: Option<String>,
}

/// Métadonnées pour une relation CALLS_AJAX (JS → Servlet)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallsAjaxMetadata {
    /// Méthode HTTP: GET, POST, PUT, DELETE
    pub method: String,
    /// URL cible (peut être partielle si dynamique)
    pub url: String,
    /// Asynchrone ou synchrone
    pub async_call: bool,
    /// Type de données: json, html, xml, text
    pub data_type: Option<String>,
    /// Content-Type
    pub content_type: Option<String>,
    /// Ligne dans le code source
    pub line: usize,
}

/// Métadonnées pour une relation CALLS_SERVICE (Portlet → Service)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallsServiceMetadata {
    /// Nom de la méthode appelée
    pub method: String,
    /// Types des paramètres
    pub param_types: Vec<String>,
    /// Type de retour
    pub returns: Option<String>,
    /// Ligne dans le code source
    pub line: usize,
}

/// Métadonnées pour une relation CALLS_DAO (Service → DAO)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallsDaoMetadata {
    /// Opération: SAVE, UPDATE, DELETE, QUERY
    pub operation: DaoOperation,
    /// Entité manipulée: User, Document, etc.
    pub entity: Option<String>,
    /// Nom de la méthode
    pub method: String,
    /// Ligne dans le code source
    pub line: usize,
}

/// Type d'opération DAO
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DaoOperation {
    Save,
    Update,
    Delete,
    Query,
    Unknown,
}

impl DaoOperation {
    /// Devine l'opération depuis le nom de méthode
    #[allow(dead_code)]
    pub fn from_method_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower.starts_with("save") || lower.starts_with("insert") || lower.starts_with("create") {
            Self::Save
        } else if lower.starts_with("update") || lower.starts_with("modify") {
            Self::Update
        } else if lower.starts_with("delete") || lower.starts_with("remove") {
            Self::Delete
        } else if lower.starts_with("get")
            || lower.starts_with("find")
            || lower.starts_with("query")
            || lower.starts_with("select")
            || lower.starts_with("load")
        {
            Self::Query
        } else {
            Self::Unknown
        }
    }

    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Save => "SAVE",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
            Self::Query => "QUERY",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// Métadonnées pour une relation CONFIGURES (portlet.xml → Portlet)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfiguresMetadata {
    /// Modes supportés: view, edit, help
    pub modes: Vec<String>,
    /// Window states: normal, maximized, minimized
    pub states: Vec<String>,
    /// Cache expiration en secondes
    pub cache_seconds: Option<i32>,
    /// Init parameters
    pub params: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relation_priority() {
        assert_eq!(
            WebSphereRelation::Renders.priority(),
            RelationPriority::Critical
        );
        assert_eq!(
            WebSphereRelation::CallsService.priority(),
            RelationPriority::High
        );
        assert_eq!(
            WebSphereRelation::Notifies.priority(),
            RelationPriority::Medium
        );
        assert_eq!(
            WebSphereRelation::TargetsElement.priority(),
            RelationPriority::Low
        );
    }

    #[test]
    fn test_dao_operation_from_method() {
        assert_eq!(
            DaoOperation::from_method_name("saveUser"),
            DaoOperation::Save
        );
        assert_eq!(
            DaoOperation::from_method_name("updateDocument"),
            DaoOperation::Update
        );
        assert_eq!(
            DaoOperation::from_method_name("deleteRecord"),
            DaoOperation::Delete
        );
        assert_eq!(
            DaoOperation::from_method_name("getUserById"),
            DaoOperation::Query
        );
        assert_eq!(
            DaoOperation::from_method_name("processData"),
            DaoOperation::Unknown
        );
    }

    #[test]
    fn test_relation_as_str() {
        assert_eq!(WebSphereRelation::Renders.as_str(), "RENDERS");
        assert_eq!(WebSphereRelation::Declares.as_str(), "DECLARES");
        assert_eq!(WebSphereRelation::Configures.as_str(), "CONFIGURES");
    }
}
