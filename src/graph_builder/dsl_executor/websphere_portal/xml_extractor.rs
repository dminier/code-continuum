/// Extracteur pour les fichiers de configuration XML
///
/// Parse:
/// - web.xml → Servlet declarations, Filters, Servlet mappings
/// - portlet.xml → Portlet configurations, modes, init-params
///
/// Utilise Regex car structures XML bien définies
use crate::semantic_graph::semantic_graph::{
    EdgeRelation, Location, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};
use regex::Regex;
use std::collections::HashMap;
use tracing::debug;

pub struct XmlExtractor {
    // web.xml patterns
    servlet_regex: Regex,
    servlet_class_regex: Regex,
    servlet_mapping_regex: Regex,
    url_pattern_regex: Regex,
    filter_regex: Regex,
    filter_mapping_regex: Regex,

    // portlet.xml patterns
    portlet_regex: Regex,
    portlet_class_regex: Regex,
    portlet_mode_regex: Regex,
    window_state_regex: Regex,
    init_param_regex: Regex,
    cache_regex: Regex,

    // Inline patterns used in loops (stored to avoid regex-in-loop)
    servlet_name_regex: Regex,
    filter_name_regex: Regex,
    filter_class_regex: Regex,
    portlet_name_regex: Regex,
}

impl XmlExtractor {
    pub fn new() -> Self {
        Self {
            // <servlet><servlet-name>...</servlet-name>...</servlet>
            // Use (?s) flag to make . match newlines (dotall mode)
            servlet_regex: Regex::new(r"(?s)<servlet>(.*?)</servlet>").unwrap(),
            servlet_class_regex: Regex::new(r"<servlet-class>\s*([^<]+)\s*</servlet-class>").unwrap(),

            // <servlet-mapping>...</servlet-mapping>
            servlet_mapping_regex: Regex::new(r"(?s)<servlet-mapping>(.*?)</servlet-mapping>").unwrap(),
            url_pattern_regex: Regex::new(r"<url-pattern>\s*([^<]+)\s*</url-pattern>").unwrap(),

            // <filter>...</filter>
            filter_regex: Regex::new(r"(?s)<filter>(.*?)</filter>").unwrap(),
            filter_mapping_regex: Regex::new(r"(?s)<filter-mapping>(.*?)</filter-mapping>").unwrap(),

            // portlet.xml
            portlet_regex: Regex::new(r"(?s)<portlet>(.*?)</portlet>").unwrap(),
            portlet_class_regex: Regex::new(r"<portlet-class>\s*([^<]+)\s*</portlet-class>").unwrap(),
            portlet_mode_regex: Regex::new(r"<portlet-mode>\s*([^<]+)\s*</portlet-mode>").unwrap(),
            window_state_regex: Regex::new(r"<window-state>\s*([^<]+)\s*</window-state>").unwrap(),
            init_param_regex: Regex::new(r"(?s)<init-param>\s*<name>\s*([^<]+)\s*</name>\s*<value>\s*([^<]+)\s*</value>\s*</init-param>").unwrap(),
            cache_regex: Regex::new(r"<expiration-cache>\s*([^<]+)\s*</expiration-cache>").unwrap(),

            // Inline patterns used in loops
            servlet_name_regex: Regex::new(r"<servlet-name>\s*([^<]+)\s*</servlet-name>").unwrap(),
            filter_name_regex: Regex::new(r"<filter-name>\s*([^<]+)\s*</filter-name>").unwrap(),
            filter_class_regex: Regex::new(r"<filter-class>\s*([^<]+)\s*</filter-class>").unwrap(),
            portlet_name_regex: Regex::new(r"<portlet-name>\s*([^<]+)\s*</portlet-name>").unwrap(),
        }
    }

    /// Extrait les relations depuis web.xml
    pub fn extract_web_xml(
        &self,
        file_path: &str,
        content: &str,
        graph: &mut UnifiedGraph,
    ) -> Result<(), String> {
        debug!(file = file_path, "Extracting web.xml DECLARES relations");

        // Créer le nœud web.xml
        let xml_id = "config::web.xml".to_string();
        graph.add_node(SemanticNode {
            id: xml_id.clone(),
            kind: NodeKind::WebXml,
            name: "web.xml".to_string(),
            file_path: file_path.to_string(),
            location: Location::default(),
            metadata: [("type".to_string(), "deployment-descriptor".to_string())]
                .iter()
                .cloned()
                .collect(),
        });

        // Phase 1: Extraire les servlets
        let mut servlet_names = HashMap::new();
        for cap in self.servlet_regex.captures_iter(content) {
            if let Some(servlet_block) = cap.get(1) {
                let block = servlet_block.as_str();

                // Extraire servlet-name
                if let Some(name_cap) = self.servlet_name_regex.captures(block) {
                    let servlet_name = name_cap.get(1).unwrap().as_str().trim();

                    // Extraire servlet-class
                    if let Some(class_cap) = self.servlet_class_regex.captures(block) {
                        let servlet_class = class_cap.get(1).unwrap().as_str().trim();
                        servlet_names.insert(servlet_name.to_string(), servlet_class.to_string());

                        self.create_servlet_declaration(
                            &xml_id,
                            servlet_name,
                            servlet_class,
                            graph,
                        );
                    }
                }
            }
        }

        // Phase 2: Extraire les servlet-mappings et ajouter url-pattern aux métadonnées
        for cap in self.servlet_mapping_regex.captures_iter(content) {
            if let Some(mapping_block) = cap.get(1) {
                let block = mapping_block.as_str();

                // Extraire servlet-name
                if let Some(name_cap) = self.servlet_name_regex.captures(block) {
                    let servlet_name = name_cap.get(1).unwrap().as_str().trim();
                    let servlet_id = format!("servlet::{}", servlet_name);

                    // Extraire url-pattern(s) et les ajouter aux métadonnées du Servlet
                    let mut url_patterns = Vec::new();
                    for url_cap in self.url_pattern_regex.captures_iter(block) {
                        let url_pattern = url_cap.get(1).unwrap().as_str().trim();
                        url_patterns.push(url_pattern.to_string());

                        if let Some(servlet_class) = servlet_names.get(servlet_name) {
                            debug!(
                                servlet = servlet_name,
                                class = servlet_class,
                                pattern = url_pattern,
                                "✅ Servlet mapping"
                            );
                        }
                    }

                    // Mettre à jour les métadonnées du Servlet avec les url-patterns
                    if !url_patterns.is_empty() {
                        if let Some(servlet_node) = graph.nodes.get_mut(&servlet_id) {
                            servlet_node
                                .metadata
                                .insert("url-pattern".to_string(), url_patterns.join(","));
                        }
                    }
                }
            }
        }

        // Phase 3: Extraire les filters
        let mut filter_names = HashMap::new();
        for cap in self.filter_regex.captures_iter(content) {
            if let Some(filter_block) = cap.get(1) {
                let block = filter_block.as_str();

                if let (Some(name_cap), Some(class_cap)) = (
                    self.filter_name_regex.captures(block),
                    self.filter_class_regex.captures(block),
                ) {
                    let filter_name = name_cap.get(1).unwrap().as_str().trim();
                    let filter_class = class_cap.get(1).unwrap().as_str().trim();
                    filter_names.insert(filter_name.to_string(), filter_class.to_string());

                    self.create_filter_declaration(&xml_id, filter_name, filter_class, graph);
                }
            }
        }

        // Phase 4: Extraire filter-mappings
        let mut filter_order = 0;
        for cap in self.filter_mapping_regex.captures_iter(content) {
            if let Some(mapping_block) = cap.get(1) {
                let block = mapping_block.as_str();

                if let Some(name_cap) = self.filter_name_regex.captures(block) {
                    let filter_name = name_cap.get(1).unwrap().as_str().trim();

                    // Extraire url-pattern ou servlet-name
                    for url_cap in self.url_pattern_regex.captures_iter(block) {
                        let url_pattern = url_cap.get(1).unwrap().as_str().trim();

                        if let Some(filter_class) = filter_names.get(filter_name) {
                            self.create_filter_relation(
                                filter_name,
                                filter_class,
                                url_pattern,
                                filter_order,
                                graph,
                            );
                            filter_order += 1;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extrait les relations depuis portlet.xml
    pub fn extract_portlet_xml(
        &self,
        file_path: &str,
        content: &str,
        graph: &mut UnifiedGraph,
    ) -> Result<(), String> {
        debug!(
            file = file_path,
            content_length = content.len(),
            "Extracting portlet.xml CONFIGURES relations"
        );

        // Créer le nœud portlet.xml
        let xml_id = "config::portlet.xml".to_string();
        graph.add_node(SemanticNode {
            id: xml_id.clone(),
            kind: NodeKind::PortletXml,
            name: "portlet.xml".to_string(),
            file_path: file_path.to_string(),
            location: Location::default(),
            metadata: [("type".to_string(), "portlet-descriptor".to_string())]
                .iter()
                .cloned()
                .collect(),
        });

        // Extraire chaque <portlet>
        let portlet_count = self.portlet_regex.captures_iter(content).count();
        debug!(node_id = %xml_id, count = portlet_count, "Found portlet blocks");

        for cap in self.portlet_regex.captures_iter(content) {
            if let Some(portlet_block) = cap.get(1) {
                let block = portlet_block.as_str();

                // Extraire portlet-name
                if let Some(name_cap) = self.portlet_name_regex.captures(block) {
                    let portlet_name = name_cap.get(1).unwrap().as_str().trim();
                    debug!(portlet_name = portlet_name, "Found portlet-name");

                    // Extraire portlet-class
                    if let Some(class_cap) = self.portlet_class_regex.captures(block) {
                        let portlet_class = class_cap.get(1).unwrap().as_str().trim();
                        debug!(portlet_class = portlet_class, "Found portlet-class");

                        // Extraire modes
                        let modes: Vec<String> = self
                            .portlet_mode_regex
                            .captures_iter(block)
                            .map(|m| m.get(1).unwrap().as_str().trim().to_string())
                            .collect();

                        // Extraire states
                        let states: Vec<String> = self
                            .window_state_regex
                            .captures_iter(block)
                            .map(|m| m.get(1).unwrap().as_str().trim().to_string())
                            .collect();

                        // Extraire init-params
                        let mut params = HashMap::new();
                        for param_cap in self.init_param_regex.captures_iter(block) {
                            let name = param_cap.get(1).unwrap().as_str().trim();
                            let value = param_cap.get(2).unwrap().as_str().trim();
                            params.insert(name.to_string(), value.to_string());
                        }

                        // Extraire cache
                        let cache_seconds = self
                            .cache_regex
                            .captures(block)
                            .and_then(|c| c.get(1))
                            .and_then(|m| m.as_str().trim().parse::<i32>().ok());

                        debug!(
                            portlet_name = portlet_name,
                            portlet_class = portlet_class,
                            modes = ?modes,
                            states = ?states,
                            params_count = params.len(),
                            "Creating portlet configuration"
                        );
                        self.create_portlet_configuration(
                            &xml_id,
                            portlet_name,
                            portlet_class,
                            modes,
                            states,
                            params,
                            cache_seconds,
                            graph,
                        );
                    } else {
                        debug!(portlet_name = portlet_name, "⚠️ No portlet-class found");
                    }
                } else {
                    debug!("⚠️ No portlet-name found in block");
                }
            }
        }

        debug!(
            nodes = graph.nodes.len(),
            edges = graph.edges.len(),
            "Portlet extraction completed"
        );

        Ok(())
    }

    /// Crée une déclaration Servlet
    fn create_servlet_declaration(
        &self,
        xml_id: &str,
        servlet_name: &str,
        servlet_class: &str,
        graph: &mut UnifiedGraph,
    ) {
        let servlet_id = format!("servlet::{}", servlet_name);

        // 1. Créer le nœud Servlet intermédiaire
        graph.add_node(SemanticNode {
            id: servlet_id.clone(),
            kind: NodeKind::Servlet,
            name: servlet_name.to_string(),
            file_path: format!("/servlets/{}.java", servlet_name),
            location: Location::default(),
            metadata: [("class".to_string(), servlet_class.to_string())]
                .iter()
                .cloned()
                .collect(),
        });

        // 2. Créer la relation CONFIGURES : web.xml -> servlet::{name}
        graph.add_edge(SemanticEdge {
            from: xml_id.to_string(),
            to: servlet_id.clone(),
            relation: EdgeRelation::Configures,
            metadata: [("class".to_string(), servlet_class.to_string())]
                .iter()
                .cloned()
                .collect(),
        });

        // 3. Créer la relation IMPLEMENTED_BY : servlet::{name} -> Java class
        // Stocker le FQN dans les métadonnées pour résolution ultérieure
        // La relation sera résolue par resolve_servlet_implementations_global() après indexation des classes
        let mut servlet_metadata = HashMap::new();
        servlet_metadata.insert("implements_class".to_string(), servlet_class.to_string());

        graph.add_edge(SemanticEdge {
            from: servlet_id.clone(),
            to: servlet_class.to_string(),
            relation: EdgeRelation::ImplementedBy,
            metadata: servlet_metadata,
        });

        debug!(
            servlet = servlet_name,
            class = servlet_class,
            "✅ Servlet configuré"
        );
    }

    /// Crée une déclaration Filter
    fn create_filter_declaration(
        &self,
        xml_id: &str,
        filter_name: &str,
        filter_class: &str,
        graph: &mut UnifiedGraph,
    ) {
        let filter_id = format!("filter::{}", filter_name);

        // 1. Créer le nœud Filter intermédiaire
        graph.add_node(SemanticNode {
            id: filter_id.clone(),
            kind: NodeKind::Filter,
            name: filter_name.to_string(),
            file_path: format!("/filters/{}.java", filter_name),
            location: Location::default(),
            metadata: [("class".to_string(), filter_class.to_string())]
                .iter()
                .cloned()
                .collect(),
        });

        // 2. Créer la relation CONFIGURES : web.xml -> filter::{name}
        graph.add_edge(SemanticEdge {
            from: xml_id.to_string(),
            to: filter_id.clone(),
            relation: EdgeRelation::Configures,
            metadata: [("class".to_string(), filter_class.to_string())]
                .iter()
                .cloned()
                .collect(),
        });

        // 3. Créer la relation IMPLEMENTED_BY : filter::{name} -> Java class
        graph.add_edge(SemanticEdge {
            from: filter_id,
            to: filter_class.to_string(),
            relation: EdgeRelation::ImplementedBy,
            metadata: [("fqn".to_string(), filter_class.to_string())]
                .iter()
                .cloned()
                .collect(),
        });

        debug!(
            filter = filter_name,
            class = filter_class,
            "✅ Filter configuré"
        );
    }

    /// Crée une relation FILTERS
    fn create_filter_relation(
        &self,
        filter_name: &str,
        _filter_class: &str,
        url_pattern: &str,
        order: usize,
        graph: &mut UnifiedGraph,
    ) {
        let filter_id = format!("filter::{}", filter_name);

        // Pour simplifier, on crée une relation vers un nœud "FilterChain"
        let chain_id = "servlet::FilterChain".to_string();

        graph.add_edge(SemanticEdge {
            from: filter_id,
            to: chain_id,
            relation: EdgeRelation::Filters,
            metadata: [
                ("order".to_string(), order.to_string()),
                ("urlPattern".to_string(), url_pattern.to_string()),
            ]
            .iter()
            .cloned()
            .collect(),
        });

        debug!(
            filter = filter_name,
            pattern = url_pattern,
            order = order,
            "✅ Filter mapping"
        );
    }

    /// Crée une configuration Portlet
    #[allow(clippy::too_many_arguments)]
    fn create_portlet_configuration(
        &self,
        xml_id: &str,
        portlet_name: &str,
        portlet_class: &str,
        modes: Vec<String>,
        states: Vec<String>,
        params: HashMap<String, String>,
        cache_seconds: Option<i32>,
        graph: &mut UnifiedGraph,
    ) {
        let portlet_id = format!("portlet::{}", portlet_name);

        // 1. Créer le nœud Portlet intermédiaire
        graph.add_node(SemanticNode {
            id: portlet_id.clone(),
            kind: NodeKind::Portlet,
            name: portlet_name.to_string(),
            file_path: format!("/portlets/{}.java", portlet_name),
            location: Location::default(),
            metadata: [("class".to_string(), portlet_class.to_string())]
                .iter()
                .cloned()
                .collect(),
        });

        // 2. Créer la relation CONFIGURES : portlet.xml -> portlet::{name}
        let mut configures_metadata = vec![
            ("modes".to_string(), modes.join(",")),
            ("states".to_string(), states.join(",")),
        ];

        if let Some(cache) = cache_seconds {
            configures_metadata.push(("cacheSeconds".to_string(), cache.to_string()));
        }

        // Ajouter les init-params importants
        for (key, value) in params.iter() {
            if key.starts_with("template") {
                configures_metadata.push((format!("param_{}", key), value.clone()));
            }
        }

        let configures_map: HashMap<String, String> = configures_metadata.into_iter().collect();
        debug!(
            portlet_name = portlet_name,
            portlet_class = portlet_class,
            metadata_count = configures_map.len(),
            "Creating CONFIGURES relation to portlet node"
        );

        graph.add_edge(SemanticEdge {
            from: xml_id.to_string(),
            to: portlet_id.clone(),
            relation: EdgeRelation::Configures,
            metadata: configures_map,
        });

        // 3. Créer la relation IMPLEMENTED_BY : portlet::{name} -> Java class
        debug!(
            portlet = portlet_name,
            class = portlet_class,
            "Creating IMPLEMENTED_BY relation"
        );

        graph.add_edge(SemanticEdge {
            from: portlet_id,
            to: portlet_class.to_string(),
            relation: EdgeRelation::ImplementedBy,
            metadata: [("fqn".to_string(), portlet_class.to_string())]
                .iter()
                .cloned()
                .collect(),
        });

        debug!(
            portlet = portlet_name,
            class = portlet_class,
            modes = ?modes,
            "Portlet fully configured"
        );
    }
}

impl Default for XmlExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_servlet_class_regex() {
        let extractor = XmlExtractor::new();
        let content = "<servlet-class>com.example.web.DispatcherServlet</servlet-class>";

        let caps = extractor.servlet_class_regex.captures(content).unwrap();
        assert_eq!(
            caps.get(1).unwrap().as_str().trim(),
            "com.example.web.DispatcherServlet"
        );
    }

    #[test]
    fn test_url_pattern_regex() {
        let extractor = XmlExtractor::new();
        let content = "<url-pattern>*.do</url-pattern>";

        let caps = extractor.url_pattern_regex.captures(content).unwrap();
        assert_eq!(caps.get(1).unwrap().as_str().trim(), "*.do");
    }
}
