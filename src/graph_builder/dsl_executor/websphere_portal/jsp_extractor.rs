/// Extracteur pour les relations JSP → JavaScript/CSS (INCLUDES)
///
/// Parse les fichiers JSP pour identifier:
/// - <script src="..."> → INCLUDES_JS
/// - <link rel="stylesheet" href="..."> → INCLUDES_CSS
/// - <%@ include file="..."%> et <jsp:include page="..."/> → INCLUDES_JSP
///
/// Utilise Regex car JSP = HTML + Java, Tree-Sitter HTML insuffisant
use crate::semantic_graph::semantic_graph::{
    EdgeRelation, Location, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};
use regex::Regex;
use tracing::debug;

pub struct JspExtractor {
    script_regex: Regex,
    script_c_url_regex: Regex,
    link_regex: Regex,
    link_c_url_regex: Regex,
    jsp_include_directive_regex: Regex,
    jsp_include_tag_regex: Regex,
    java_import_regex: Regex,
}

impl JspExtractor {
    pub fn new() -> Self {
        Self {
            // <script src="/resources/js/jquery.min.js"></script>
            // <script async src="/resources/js/async.js"></script>
            // <script src="/common/config.jsp"></script> (JSP générant du JS dynamique)
            // Approche 1: Capture directe (format standard HTML)
            // Supporte .js, .jsp, .jspx, .jspf
            script_regex: Regex::new(r#"<script[^>]*src\s*=\s*["']([^"'<]+\.(?:js|jspx?|jspf))\s*["']"#)
                .unwrap(),

            // <script src="<c:url value="/common/js/controles.js"/>"></script>
            // <script src="<c:url value="/dynamic/config.jsp"/>"></script>
            // Approche 2: Capture via <c:url> (format WebSphere Portal - complémentaire)
            // Supporte .js, .jsp, .jspx, .jspf
            script_c_url_regex: Regex::new(
                r#"<script[^>]*src\s*=\s*["']<c:url\s+value\s*=\s*["']([^"']+\.(?:js|jspx?|jspf))["']\s*/\s*>["']"#,
            )
            .unwrap(),

            // <link rel="stylesheet" href="/resources/css/style.css">
            // Approche 1: Capture directe (format standard HTML)
            link_regex: Regex::new(
                r#"<link[^>]*rel\s*=\s*["']stylesheet["'][^>]*href\s*=\s*["']([^"'<]*\.css)[^"'<]*["']"#,
            )
            .unwrap(),

            // <link href="<c:url value="/common/style/master.css"/>" type="text/css" rel="stylesheet" />
            // Approche 2: Capture via <c:url> (format WebSphere Portal - complémentaire)
            // Support multilignes (type/rel peuvent être sur d'autres lignes)
            link_c_url_regex: Regex::new(
                r#"<link[^>]*href\s*=\s*["']<c:url\s+value\s*=\s*["']([^"']+\.css)["'][\s\S]*?/\s*>["']"#,
            )
            .unwrap(),

            // <%@ include file="/WEB-INF/common/header.jspf" %>
            jsp_include_directive_regex: Regex::new(
                r#"<%@\s*include\s+file=["']([^"']+)["']\s*%>"#,
            )
            .unwrap(),

            // <jsp:include page="/WEB-INF/fragments/menu.jsp"/>
            jsp_include_tag_regex: Regex::new(r#"<jsp:include\s+page=["']([^"']+)["']\s*/>"#)
                .unwrap(),

            // <%@page import="com.example.portal.fo.web.portlets.GeneriquePortlet"%>
            // <%@ page import="java.util.List" %>
            java_import_regex: Regex::new(
                r#"<%@?\s*page\s+import\s*=\s*["']([^"']+)["']\s*%>"#,
            )
            .unwrap(),
        }
    }

    /// Extrait les relations INCLUDES depuis un fichier JSP
    pub fn extract_jsp_relations(
        &self,
        file_path: &str,
        content: &str,
        graph: &mut UnifiedGraph,
    ) -> Result<(), String> {
        debug!(
            file = file_path,
            "Extraction JSP → JavaScript/CSS/JSP (INCLUDES)"
        );

        // Créer le nœud JSP source
        let jsp_id = format!("jsp::{}", file_path);
        let jsp_name = file_path
            .split('/')
            .next_back()
            .unwrap_or(file_path)
            .trim_end_matches(".jsp")
            .trim_end_matches(".jspf")
            .to_string();

        graph.add_node(SemanticNode {
            id: jsp_id.clone(),
            kind: NodeKind::Jsp,
            name: jsp_name,
            file_path: file_path.to_string(),
            location: Location::default(),
            metadata: [
                ("type".to_string(), "JSP".to_string()),
                ("path".to_string(), file_path.to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        });

        // Extraire les includes JavaScript
        let mut position = 0;
        // Approche 1: Format direct
        for cap in self.script_regex.captures_iter(content) {
            if let Some(js_path) = cap.get(1) {
                self.create_includes_js_relation(
                    &jsp_id,
                    file_path,
                    js_path.as_str(),
                    position,
                    false,
                    graph,
                );
                position += 1;
            }
        }
        // Approche 2: Format <c:url> (complémentaire)
        for cap in self.script_c_url_regex.captures_iter(content) {
            if let Some(js_path) = cap.get(1) {
                self.create_includes_js_relation(
                    &jsp_id,
                    file_path,
                    js_path.as_str(),
                    position,
                    false,
                    graph,
                );
                position += 1;
            }
        }

        // Extraire les includes CSS
        position = 0;
        // Approche 1: Format direct
        for cap in self.link_regex.captures_iter(content) {
            if let Some(css_path) = cap.get(1) {
                self.create_includes_css_relation(
                    &jsp_id,
                    file_path,
                    css_path.as_str(),
                    position,
                    "screen",
                    graph,
                );
                position += 1;
            }
        }
        // Approche 2: Format <c:url> (complémentaire)
        for cap in self.link_c_url_regex.captures_iter(content) {
            if let Some(css_path) = cap.get(1) {
                self.create_includes_css_relation(
                    &jsp_id,
                    file_path,
                    css_path.as_str(),
                    position,
                    "screen",
                    graph,
                );
                position += 1;
            }
        }

        // Extraire les includes JSP (directives)
        for cap in self.jsp_include_directive_regex.captures_iter(content) {
            if let Some(included_jsp) = cap.get(1) {
                self.create_includes_jsp_relation(
                    &jsp_id,
                    file_path,
                    included_jsp.as_str(),
                    "static",
                    graph,
                );
            }
        }

        // Extraire les includes JSP (tags)
        for cap in self.jsp_include_tag_regex.captures_iter(content) {
            if let Some(included_jsp) = cap.get(1) {
                self.create_includes_jsp_relation(
                    &jsp_id,
                    file_path,
                    included_jsp.as_str(),
                    "dynamic",
                    graph,
                );
            }
        }

        // Extraire les imports Java
        for cap in self.java_import_regex.captures_iter(content) {
            if let Some(import_path) = cap.get(1) {
                self.create_imports_relation(&jsp_id, import_path.as_str(), graph);
            }
        }

        Ok(())
    }

    /// Crée une relation INCLUDES_JS
    ///
    /// Si le chemin pointe vers un fichier .jsp/.jspx/.jspf, le nœud cible sera de type Jsp
    /// Sinon, le nœud cible sera de type Js
    ///
    /// Résolution de chemin:
    /// - `jsp_file_path`: chemin absolu du JSP parent (ex: "examples/web_templates/index.jsp")
    /// - `script_path`: chemin web relatif (ex: "/resources/js/app.js")
    /// - Résultat: "examples/web_templates/resources/js/app.js" (ou chemin web si fichier introuvable)
    fn create_includes_js_relation(
        &self,
        jsp_id: &str,
        jsp_file_path: &str,
        script_path: &str,
        position: usize,
        async_flag: bool,
        graph: &mut UnifiedGraph,
    ) {
        use std::path::Path;

        // Détecter si le script pointe vers un fichier JSP ou JavaScript
        let is_jsp_file = script_path.ends_with(".jsp")
            || script_path.ends_with(".jspx")
            || script_path.ends_with(".jspf");

        // Résoudre le chemin web relatif en chemin absolu du système de fichiers
        let resolved_file_path = {
            // Stratégie 1: Chercher depuis la racine webapp (typique WebSphere)
            // Ex: JSP dans .../webapp/Contractualisation/jsp/file.jsp
            //     Script web: /Contractualisation/js/app.js
            //     → Chercher .../webapp/Contractualisation/js/app.js
            let webapp_root = Path::new(jsp_file_path)
                .ancestors()
                .find(|p| p.file_name().is_some_and(|n| n == "webapp"))
                .or_else(|| {
                    // Fallback: chercher WEB-INF parent
                    Path::new(jsp_file_path)
                        .ancestors()
                        .find(|p| p.join("WEB-INF").exists())
                });

            let clean_script_path = script_path.trim_start_matches('/');

            // Essayer d'abord depuis webapp root
            let candidate_path = if let Some(webapp) = webapp_root {
                webapp.join(clean_script_path)
            } else {
                // Fallback: chercher depuis le répertoire parent du JSP
                let jsp_dir = Path::new(jsp_file_path)
                    .parent()
                    .unwrap_or_else(|| Path::new("."));
                jsp_dir.join(clean_script_path)
            };

            // Vérifier si le fichier existe
            if candidate_path.exists() {
                // Normaliser le chemin (canonicaliser pour correspondre aux autres nœuds)
                candidate_path
                    .canonicalize()
                    .unwrap_or_else(|_| candidate_path.clone())
                    .to_string_lossy()
                    .to_string()
            } else {
                // Fichier externe (CDN, non présent localement)
                // Garder le chemin web relatif
                script_path.to_string()
            }
        };

        // Construire l'ID et le type de nœud en fonction de l'extension
        let (target_id, target_kind, target_type, name, is_external) = if is_jsp_file {
            // Fichier JSP: utiliser le format d'ID JSP
            let id = format!("jsp::{}", resolved_file_path);
            let name = resolved_file_path
                .split('/')
                .next_back()
                .unwrap_or(&resolved_file_path)
                .trim_end_matches(".jsp")
                .trim_end_matches(".jspx")
                .trim_end_matches(".jspf")
                .to_string();
            let external = !Path::new(&resolved_file_path).exists();
            (id, NodeKind::Jsp, "JSP", name, external)
        } else {
            // Fichier JavaScript: utiliser le format d'ID JS
            let id = format!("{}::module", resolved_file_path);
            let name = resolved_file_path
                .split('/')
                .next_back()
                .unwrap_or(&resolved_file_path)
                .trim_end_matches(".js")
                .to_string();
            let external = !Path::new(&resolved_file_path).exists();
            (id, NodeKind::Js, "JavaScript", name, external)
        };

        // Construire les métadonnées
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), target_type.to_string());
        metadata.insert("path".to_string(), script_path.to_string());
        if is_external {
            metadata.insert("external".to_string(), "true".to_string());
            metadata.insert("web_path".to_string(), script_path.to_string());
        }

        // Créer le nœud cible (sera fusionné si déjà existant)
        graph.add_node(SemanticNode {
            id: target_id.clone(),
            kind: target_kind,
            name,
            file_path: resolved_file_path.clone(),
            location: Location::default(),
            metadata,
        });

        // Créer la relation INCLUDES_JS (même si la cible est un fichier JSP)
        graph.add_edge(SemanticEdge {
            from: jsp_id.to_string(),
            to: target_id,
            relation: EdgeRelation::IncludesJs,
            metadata: [
                ("position".to_string(), position.to_string()),
                ("async".to_string(), async_flag.to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        });

        debug!(
            jsp = jsp_id,
            script = script_path,
            target_type = target_type,
            "✅ Relation INCLUDES_JS créée"
        );
    }

    /// Crée une relation INCLUDES_CSS
    ///
    /// Résolution de chemin:
    /// - `jsp_file_path`: chemin absolu du JSP parent
    /// - `css_path`: chemin web relatif (ex: "/resources/css/style.css")
    /// - Résultat: chemin absolu canonicalisé ou chemin web si fichier introuvable
    fn create_includes_css_relation(
        &self,
        jsp_id: &str,
        jsp_file_path: &str,
        css_path: &str,
        position: usize,
        media: &str,
        graph: &mut UnifiedGraph,
    ) {
        use std::path::Path;

        // Résoudre le chemin web relatif en chemin absolu du système de fichiers
        let resolved_file_path = {
            // Stratégie: Chercher depuis la racine webapp (typique WebSphere)
            let webapp_root = Path::new(jsp_file_path)
                .ancestors()
                .find(|p| p.file_name().is_some_and(|n| n == "webapp"))
                .or_else(|| {
                    Path::new(jsp_file_path)
                        .ancestors()
                        .find(|p| p.join("WEB-INF").exists())
                });

            let clean_css_path = css_path.trim_start_matches('/');

            let candidate_path = if let Some(webapp) = webapp_root {
                webapp.join(clean_css_path)
            } else {
                let jsp_dir = Path::new(jsp_file_path)
                    .parent()
                    .unwrap_or_else(|| Path::new("."));
                jsp_dir.join(clean_css_path)
            };

            if candidate_path.exists() {
                candidate_path
                    .canonicalize()
                    .unwrap_or_else(|_| candidate_path.clone())
                    .to_string_lossy()
                    .to_string()
            } else {
                css_path.to_string()
            }
        };

        let css_id = format!("css::{}", resolved_file_path);
        let css_name = resolved_file_path
            .split('/')
            .next_back()
            .unwrap_or(&resolved_file_path)
            .trim_end_matches(".css")
            .to_string();

        let is_external = !Path::new(&resolved_file_path).exists();
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("type".to_string(), "CSS".to_string());
        metadata.insert("path".to_string(), css_path.to_string());
        if is_external {
            metadata.insert("external".to_string(), "true".to_string());
            metadata.insert("web_path".to_string(), css_path.to_string());
        }

        // Créer le nœud CSS
        graph.add_node(SemanticNode {
            id: css_id.clone(),
            kind: NodeKind::Module,
            name: css_name,
            file_path: resolved_file_path,
            location: Location::default(),
            metadata,
        });

        // Créer la relation
        graph.add_edge(SemanticEdge {
            from: jsp_id.to_string(),
            to: css_id,
            relation: EdgeRelation::IncludesCss,
            metadata: [
                ("position".to_string(), position.to_string()),
                ("media".to_string(), media.to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        });

        debug!(
            jsp = jsp_id,
            css = css_path,
            "✅ Relation INCLUDES_CSS créée"
        );
    }

    /// Crée une relation INCLUDES_JSP
    fn create_includes_jsp_relation(
        &self,
        jsp_id: &str,
        jsp_file_path: &str,
        included_jsp_path: &str,
        include_type: &str,
        graph: &mut UnifiedGraph,
    ) {
        use std::path::Path;

        // Nettoyer le chemin (trim whitespace qui peut apparaître dans les JSP)
        let included_jsp_path = included_jsp_path.trim();

        // 🔧 Résolution du chemin JSP avec webapp root (comme JS/CSS)
        let resolved_jsp_path = {
            let candidate_path = if included_jsp_path.starts_with('/') {
                // Chemin absolu web → résoudre depuis webapp root
                let webapp_root = Path::new(jsp_file_path)
                    .ancestors()
                    .find(|p| p.file_name().is_some_and(|n| n == "webapp"))
                    .or_else(|| {
                        // Fallback: chercher WEB-INF parent
                        Path::new(jsp_file_path)
                            .ancestors()
                            .find(|p| p.join("WEB-INF").exists())
                    });

                let clean_jsp_path = included_jsp_path.trim_start_matches('/');

                if let Some(webapp) = webapp_root {
                    webapp.join(clean_jsp_path)
                } else {
                    // Pas de webapp root trouvé, utiliser le chemin tel quel
                    Path::new(jsp_file_path)
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(clean_jsp_path)
                }
            } else {
                // Chemin relatif → résoudre depuis le répertoire du JSP parent
                let jsp_dir = Path::new(jsp_file_path)
                    .parent()
                    .unwrap_or_else(|| Path::new("."));
                jsp_dir.join(included_jsp_path)
            };

            // Vérifier si le fichier existe
            if candidate_path.exists() {
                // Normaliser le chemin (canonicaliser)
                candidate_path
                    .canonicalize()
                    .unwrap_or_else(|_| candidate_path.clone())
                    .to_string_lossy()
                    .to_string()
            } else {
                use tracing::warn;
                warn!(
                    jsp_path = jsp_file_path,
                    included_path = included_jsp_path,
                    candidate = %candidate_path.display(),
                    "⚠️ JSP inclus introuvable"
                );
                // Fallback: garder le chemin original
                included_jsp_path.to_string()
            }
        };

        // Créer l'ID et le nom depuis le chemin résolu
        let included_id = format!("jsp::{}", resolved_jsp_path);
        let included_name = resolved_jsp_path
            .split('/')
            .next_back()
            .unwrap_or(&resolved_jsp_path)
            .trim_end_matches(".jsp")
            .trim_end_matches(".jspx")
            .trim_end_matches(".jspf")
            .to_string();

        // Créer le nœud JSP inclus
        graph.add_node(SemanticNode {
            id: included_id.clone(),
            kind: NodeKind::Jsp,
            name: included_name,
            file_path: resolved_jsp_path,
            location: Location::default(),
            metadata: [
                ("type".to_string(), "JSP".to_string()),
                ("fragment".to_string(), "true".to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        });

        // Créer la relation
        graph.add_edge(SemanticEdge {
            from: jsp_id.to_string(),
            to: included_id,
            relation: EdgeRelation::IncludesJsp,
            metadata: [("type".to_string(), include_type.to_string())]
                .iter()
                .cloned()
                .collect(),
        });

        debug!(
            jsp = jsp_id,
            included = included_jsp_path,
            include_type = include_type,
            "✅ Relation INCLUDES_JSP créée"
        );
    }

    /// Crée une relation IMPORTS entre JSP et Class Java
    fn create_imports_relation(&self, jsp_id: &str, import_path: &str, graph: &mut UnifiedGraph) {
        // Extraire le nom de la classe depuis le chemin complet
        // Exemple: "com.example.portal.fo.web.portlets.GeneriquePortlet" → "GeneriquePortlet"
        let class_name = import_path
            .split('.')
            .next_back()
            .unwrap_or(import_path)
            .to_string();

        // ⚡ IMPORTANT: Utiliser le qualified_name directement comme ID
        // pour matcher le format utilisé par l'extracteur Java
        // L'extracteur Java crée: "com.example.portal.fo.util.PortalProperties"
        // JSP doit créer le même ID pour éviter les doublons
        let class_id = import_path.to_string();

        // Créer le nœud Class si pas déjà existant
        if !graph.nodes.contains_key(&class_id) {
            graph.add_node(SemanticNode {
                id: class_id.clone(),
                kind: NodeKind::Class,
                name: class_name.clone(),
                file_path: String::new(), // Classe externe, pas de fichier local
                location: Location::default(),
                metadata: [
                    ("qualified_name".to_string(), import_path.to_string()),
                    ("external".to_string(), "true".to_string()),
                    ("imported_by_jsp".to_string(), "true".to_string()),
                ]
                .iter()
                .cloned()
                .collect(),
            });
        }

        // Créer la relation IMPORTS
        graph.add_edge(SemanticEdge {
            from: jsp_id.to_string(),
            to: class_id,
            relation: EdgeRelation::Imports,
            metadata: [
                ("import_type".to_string(), "java".to_string()),
                ("qualified_name".to_string(), import_path.to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        });

        debug!(
            jsp = jsp_id,
            class = import_path,
            "✅ Relation IMPORTS créée (JSP → Class)"
        );
    }
}

impl Default for JspExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_regex() {
        let extractor = JspExtractor::new();
        let content = r#"<script src="/resources/js/app.js"></script>"#;

        let caps = extractor.script_regex.captures(content).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "/resources/js/app.js");
    }

    #[test]
    fn test_link_regex() {
        let extractor = JspExtractor::new();
        let content = r#"<link rel="stylesheet" href="/resources/css/style.css">"#;

        let caps = extractor.link_regex.captures(content).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "/resources/css/style.css");
    }

    #[test]
    fn test_jsp_include_directive() {
        let extractor = JspExtractor::new();
        let content = r#"<%@ include file="/WEB-INF/common/header.jspf" %>"#;

        let caps = extractor
            .jsp_include_directive_regex
            .captures(content)
            .unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "/WEB-INF/common/header.jspf");
    }

    #[test]
    fn test_jsp_include_tag() {
        let extractor = JspExtractor::new();
        let content = r#"<jsp:include page="/WEB-INF/fragments/menu.jsp"/>"#;

        let caps = extractor.jsp_include_tag_regex.captures(content).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str(), "/WEB-INF/fragments/menu.jsp");
    }

    #[test]
    fn test_java_import_regex() {
        let extractor = JspExtractor::new();

        // Test format avec espace
        let content1 = r#"<%@page import="com.example.portal.fo.web.portlets.GeneriquePortlet"%>"#;
        let caps1 = extractor.java_import_regex.captures(content1).unwrap();
        assert_eq!(
            caps1.get(1).unwrap().as_str(),
            "com.example.portal.fo.web.portlets.GeneriquePortlet"
        );

        // Test format sans @
        let content2 = r#"<% page import="java.util.List" %>"#;
        let caps2 = extractor.java_import_regex.captures(content2).unwrap();
        assert_eq!(caps2.get(1).unwrap().as_str(), "java.util.List");
    }

    /// Test pour les scripts <script src="..."> qui pointent vers des fichiers JSP
    /// Le regex doit capturer .js ET .jsp/.jspx
    #[test]
    fn test_script_regex_captures_jsp_files() {
        let extractor = JspExtractor::new();

        // Test script classique .js (doit toujours fonctionner)
        let js_content = r#"<script src="/resources/js/app.js"></script>"#;
        let caps = extractor.script_regex.captures(js_content);
        assert!(caps.is_some(), "Should capture .js files");

        // Test script pointant vers .jsp (NOUVEAU)
        let jsp_content = r#"<script src="/common/config.jsp"></script>"#;
        let caps_jsp = extractor.script_regex.captures(jsp_content);
        assert!(
            caps_jsp.is_some(),
            "Should capture .jsp files in script src"
        );
        if let Some(c) = caps_jsp {
            assert_eq!(c.get(1).unwrap().as_str(), "/common/config.jsp");
        }

        // Test script pointant vers .jspx
        let jspx_content = r#"<script src="/fragments/translations.jspx"></script>"#;
        let caps_jspx = extractor.script_regex.captures(jspx_content);
        assert!(
            caps_jspx.is_some(),
            "Should capture .jspx files in script src"
        );
    }

    /// Test que INCLUDES_JS peut pointer vers un nœud JSP
    #[test]
    fn test_includes_js_pointing_to_jsp_node() {
        let extractor = JspExtractor::new();
        let mut graph = UnifiedGraph::new();

        let jsp_content = r#"
<%@ page contentType="text/html;charset=UTF-8" %>
<html>
<head>
    <script src="/resources/js/main.js"></script>
    <script src="/common/config.jsp"></script>
    <script src="/fragments/translations.jspx"></script>
</head>
</html>
"#;

        extractor
            .extract_jsp_relations("/test/page.jsp", jsp_content, &mut graph)
            .unwrap();

        // Vérifier qu'on a 3 relations INCLUDES_JS
        let js_edges: Vec<_> = graph
            .edges
            .iter()
            .filter(|e| e.relation == EdgeRelation::IncludesJs)
            .collect();
        assert_eq!(js_edges.len(), 3, "Should have 3 INCLUDES_JS relations");

        // Vérifier les types de nœuds cibles
        // main.js → Js
        let main_target = graph.nodes.get("/resources/js/main.js::module");
        assert!(main_target.is_some(), "Should have main.js node");
        assert_eq!(
            main_target.unwrap().kind,
            NodeKind::Js,
            "main.js should be Js node"
        );

        // config.jsp → Jsp
        let config_target = graph.nodes.get("jsp::/common/config.jsp");
        assert!(config_target.is_some(), "Should have config.jsp node");
        assert_eq!(
            config_target.unwrap().kind,
            NodeKind::Jsp,
            "config.jsp should be Jsp node"
        );

        // translations.jspx → Jsp
        let trans_target = graph.nodes.get("jsp::/fragments/translations.jspx");
        assert!(trans_target.is_some(), "Should have translations.jspx node");
        assert_eq!(
            trans_target.unwrap().kind,
            NodeKind::Jsp,
            "translations.jspx should be Jsp node"
        );
    }
}
