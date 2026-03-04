use neo4rs::{query, Graph};
use std::env;
use tracing::{debug, info, warn};

use crate::semantic_graph::{NodeKind, UnifiedGraph};

#[allow(dead_code)]
pub struct Neo4jExporter {
    graph: Graph,
}

#[allow(dead_code)]
impl Neo4jExporter {
    /// Crée une nouvelle connexion à Neo4j
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let uri = env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://neo4j:7687".to_string());
        let user = env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
        let password = env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());

        debug!(uri = %uri, "Connecting to Neo4j");

        let graph = Graph::new(&uri, &user, &password).await?;

        Ok(Neo4jExporter { graph })
    }

    /// Exporte le graphe unifié vers Neo4j (version optimisée avec batch)
    pub async fn export_graph(
        &self,
        unified_graph: &UnifiedGraph,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Starting optimized graph export");
        crate::ui::phase_start("Neo4j Export");

        info!("Step 1/4: Clearing database");
        // Supprimer les anciennes données
        self.clear_database().await?;
        debug!("Database cleared");

        // Créer les index
        info!("Step 2/4: Creating indexes");
        self.create_indexes().await?;
        debug!("Indexes created");

        // Batch insert des nœuds
        const BATCH_SIZE: usize = 500;
        let nodes_vec: Vec<_> = unified_graph.nodes.values().collect();

        info!("Step 3/4: Exporting nodes");
        debug!(
            total = nodes_vec.len(),
            batch_size = BATCH_SIZE,
            "Starting batch node export"
        );

        // Mode d'étiquetage: "both" (défaut), "typed-only", "property-only"
        let label_mode = env::var("NEO4J_LABEL_MODE").unwrap_or_else(|_| "both".to_string());

        for (batch_idx, chunk) in nodes_vec.chunks(BATCH_SIZE).enumerate() {
            debug!(
                batch = batch_idx,
                count = chunk.len(),
                "Processing node batch"
            );
            // Construire une seule requête avec paramètres
            let mut ids = Vec::new();
            let mut names = Vec::new();
            let mut node_types = Vec::new();
            let mut languages = Vec::new();
            let mut file_paths = Vec::new();
            let mut modules = Vec::new();
            let mut packages = Vec::new();
            let mut classes = Vec::new();
            let mut caller_ids = Vec::new();
            let mut object_types = Vec::new();
            let mut url_patterns = Vec::new();
            let mut start_lines = Vec::new();
            let mut start_cols = Vec::new();
            let mut end_lines = Vec::new();
            let mut end_cols = Vec::new();

            for node in chunk {
                ids.push(node.id.clone());
                names.push(node.name.clone());
                let node_type = match &node.kind {
                    NodeKind::Js => "JS".to_string(),
                    other => format!("{:?}", other),
                };
                node_types.push(node_type);
                languages.push(
                    node.metadata
                        .get("language")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                );
                file_paths.push(node.file_path.to_string());
                modules.push(
                    node.metadata
                        .get("module")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                packages.push(
                    node.metadata
                        .get("package")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                classes.push(
                    node.metadata
                        .get("class")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                caller_ids.push(
                    node.metadata
                        .get("caller_id")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                object_types.push(
                    node.metadata
                        .get("object_type")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                url_patterns.push(
                    node.metadata
                        .get("url-pattern")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                start_lines.push(node.location.start_line as i64);
                start_cols.push(node.location.start_col as i64);
                end_lines.push(node.location.end_line as i64);
                end_cols.push(node.location.end_col as i64);
            }

            // Construire la requête selon le mode d'étiquetage
            let cypher = match label_mode.as_str() {
                // Mode étiquettes typées uniquement (pas de propriété node_type)
                "typed-only" => {
                    query(
                        "UNWIND range(0, size($ids)-1) AS i
                         CREATE (n:Node {
                            id: $ids[i],
                            name: $names[i],
                            language: $languages[i],
                            file_path: $file_paths[i],
                            module: $modules[i],
                            package: $packages[i],
                            class: $classes[i],
                            caller_id: $caller_ids[i],
                            object_type: $object_types[i],
                            url_pattern: $url_patterns[i],
                            start_line: $start_lines[i],
                            start_col: $start_cols[i],
                            end_line: $end_lines[i],
                            end_col: $end_cols[i]
                         })
                         WITH n, $node_types[i] AS node_type
                         FOREACH (_ IN CASE WHEN node_type = 'Function' THEN [1] ELSE [] END | SET n:Function)
                         FOREACH (_ IN CASE WHEN node_type = 'Class' THEN [1] ELSE [] END | SET n:Class)
                         FOREACH (_ IN CASE WHEN node_type = 'Interface' THEN [1] ELSE [] END | SET n:Interface)
                         FOREACH (_ IN CASE WHEN node_type = 'Module' THEN [1] ELSE [] END | SET n:Module)
                         FOREACH (_ IN CASE WHEN node_type = 'JS' THEN [1] ELSE [] END | SET n:JS)
                         FOREACH (_ IN CASE WHEN node_type = 'Type' THEN [1] ELSE [] END | SET n:Type)
                         FOREACH (_ IN CASE WHEN node_type = 'Trait' THEN [1] ELSE [] END | SET n:Trait)
                         FOREACH (_ IN CASE WHEN node_type = 'Parameter' THEN [1] ELSE [] END | SET n:Parameter)
                         FOREACH (_ IN CASE WHEN node_type = 'Variable' THEN [1] ELSE [] END | SET n:Variable)
                         FOREACH (_ IN CASE WHEN node_type = 'Import' THEN [1] ELSE [] END | SET n:Import)
                         FOREACH (_ IN CASE WHEN node_type = 'Package' THEN [1] ELSE [] END | SET n:Package)
                         FOREACH (_ IN CASE WHEN node_type = 'Expression' THEN [1] ELSE [] END | SET n:Expression)
                         FOREACH (_ IN CASE WHEN node_type = 'Operator' THEN [1] ELSE [] END | SET n:Operator)
                         FOREACH (_ IN CASE WHEN node_type = 'PortletXml' THEN [1] ELSE [] END | SET n:PortletXml)
                         FOREACH (_ IN CASE WHEN node_type = 'WebXml' THEN [1] ELSE [] END | SET n:WebXml)
                         FOREACH (_ IN CASE WHEN node_type = 'Portlet' THEN [1] ELSE [] END | SET n:Portlet)
                         FOREACH (_ IN CASE WHEN node_type = 'Servlet' THEN [1] ELSE [] END | SET n:Servlet)
                         FOREACH (_ IN CASE WHEN node_type = 'Filter' THEN [1] ELSE [] END | SET n:Filter)
                         FOREACH (_ IN CASE WHEN node_type = 'Jsp' THEN [1] ELSE [] END | SET n:Jsp)",
                    )
                }
                // Mode hybride: propriété node_type + étiquettes typées
                "both" => {
                    query(
                        "UNWIND range(0, size($ids)-1) AS i
                         CREATE (n:Node {
                            id: $ids[i],
                            name: $names[i],
                            node_type: $node_types[i],
                            language: $languages[i],
                            file_path: $file_paths[i],
                            module: $modules[i],
                            package: $packages[i],
                            class: $classes[i],
                            caller_id: $caller_ids[i],
                            object_type: $object_types[i],
                            url_pattern: $url_patterns[i],
                            start_line: $start_lines[i],
                            start_col: $start_cols[i],
                            end_line: $end_lines[i],
                            end_col: $end_cols[i]
                         })
                         WITH n, $node_types[i] AS node_type
                         FOREACH (_ IN CASE WHEN node_type = 'Function' THEN [1] ELSE [] END | SET n:Function)
                         FOREACH (_ IN CASE WHEN node_type = 'Class' THEN [1] ELSE [] END | SET n:Class)
                         FOREACH (_ IN CASE WHEN node_type = 'Interface' THEN [1] ELSE [] END | SET n:Interface)
                         FOREACH (_ IN CASE WHEN node_type = 'Module' THEN [1] ELSE [] END | SET n:Module)
                         FOREACH (_ IN CASE WHEN node_type = 'JS' THEN [1] ELSE [] END | SET n:JS)
                         FOREACH (_ IN CASE WHEN node_type = 'Type' THEN [1] ELSE [] END | SET n:Type)
                         FOREACH (_ IN CASE WHEN node_type = 'Trait' THEN [1] ELSE [] END | SET n:Trait)
                         FOREACH (_ IN CASE WHEN node_type = 'Parameter' THEN [1] ELSE [] END | SET n:Parameter)
                         FOREACH (_ IN CASE WHEN node_type = 'Variable' THEN [1] ELSE [] END | SET n:Variable)
                         FOREACH (_ IN CASE WHEN node_type = 'Import' THEN [1] ELSE [] END | SET n:Import)
                         FOREACH (_ IN CASE WHEN node_type = 'Package' THEN [1] ELSE [] END | SET n:Package)
                         FOREACH (_ IN CASE WHEN node_type = 'Expression' THEN [1] ELSE [] END | SET n:Expression)
                         FOREACH (_ IN CASE WHEN node_type = 'Operator' THEN [1] ELSE [] END | SET n:Operator)
                         FOREACH (_ IN CASE WHEN node_type = 'PortletXml' THEN [1] ELSE [] END | SET n:PortletXml)
                         FOREACH (_ IN CASE WHEN node_type = 'WebXml' THEN [1] ELSE [] END | SET n:WebXml)
                         FOREACH (_ IN CASE WHEN node_type = 'Portlet' THEN [1] ELSE [] END | SET n:Portlet)
                         FOREACH (_ IN CASE WHEN node_type = 'Servlet' THEN [1] ELSE [] END | SET n:Servlet)
                         FOREACH (_ IN CASE WHEN node_type = 'Filter' THEN [1] ELSE [] END | SET n:Filter)
                         FOREACH (_ IN CASE WHEN node_type = 'Jsp' THEN [1] ELSE [] END | SET n:Jsp)",
                    )
                }
                // Mode propriété uniquement (comportement historique)
                _ => {
                    query(
                        "UNWIND range(0, size($ids)-1) AS i
                         CREATE (n:Node {
                            id: $ids[i],
                            name: $names[i],
                            node_type: $node_types[i],
                            language: $languages[i],
                            file_path: $file_paths[i],
                            module: $modules[i],
                            package: $packages[i],
                            class: $classes[i],
                            caller_id: $caller_ids[i],
                            object_type: $object_types[i],
                            url_pattern: $url_patterns[i],
                            start_line: $start_lines[i],
                            start_col: $start_cols[i],
                            end_line: $end_lines[i],
                            end_col: $end_cols[i]
                         })",
                    )
                }
            }
            .param("ids", ids)
            .param("names", names)
            .param("node_types", node_types)
            .param("languages", languages)
            .param("file_paths", file_paths)
            .param("modules", modules)
            .param("packages", packages)
            .param("classes", classes)
            .param("caller_ids", caller_ids)
            .param("object_types", object_types)
            .param("url_patterns", url_patterns)
            .param("start_lines", start_lines)
            .param("start_cols", start_cols)
            .param("end_lines", end_lines)
            .param("end_cols", end_cols);

            debug!(batch = batch_idx, "Executing node batch query");
            self.graph.run(cypher).await?;
            debug!(batch = batch_idx, "Node batch inserted");

            // Afficher la progression des nœuds
            let current = (batch_idx + 1) * BATCH_SIZE;
            let current_clamped = current.min(nodes_vec.len());
            crate::ui::show_progress_stepped(
                current_clamped,
                nodes_vec.len(),
                "Exporting nodes",
                BATCH_SIZE,
            );

            if (batch_idx + 1) % 10 == 0 {
                debug!(batch = batch_idx + 1, "Batches de nœuds exportés");
            }
        }

        debug!(count = unified_graph.nodes.len(), "All nodes exported");

        // Batch insert des relations
        let mut created_count = 0;
        let edges_vec: Vec<_> = unified_graph.edges.iter().collect();

        info!("Step 4/4: Exporting edges");
        debug!(
            total = edges_vec.len(),
            batch_size = BATCH_SIZE,
            "Starting batch edge export"
        );

        for (batch_idx, chunk) in edges_vec.chunks(BATCH_SIZE).enumerate() {
            debug!(
                batch = batch_idx,
                count = chunk.len(),
                "Processing edge batch"
            );
            let mut from_ids = Vec::new();
            let mut to_ids = Vec::new();
            let mut rel_types = Vec::new();

            for edge in chunk {
                // Résolution du 'to' si nécessaire
                let resolved_to = if unified_graph.nodes.contains_key(&edge.to) {
                    edge.to.clone()
                } else {
                    let to_name = edge.to.split("::").last().unwrap_or("");
                    unified_graph
                        .nodes
                        .values()
                        .find(|n| n.kind == NodeKind::Function && n.name == to_name)
                        .map(|n| n.id.clone())
                        .unwrap_or_else(|| edge.to.clone())
                };

                from_ids.push(edge.from.clone());
                to_ids.push(resolved_to);
                rel_types.push(format!("{:?}", edge.relation).to_uppercase());
            }

            // Créer les relations avec UNWIND (7 types principaux)
            let cypher = query(
                "UNWIND range(0, size($from_ids)-1) AS i
                 MATCH (from:Node {id: $from_ids[i]})
                 MATCH (to:Node {id: $to_ids[i]})
                 WITH from, to, $rel_types[i] AS rel_type
                 FOREACH (_ IN CASE WHEN rel_type = 'CALLS' THEN [1] ELSE [] END |
                   CREATE (from)-[:CALLS]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'DEFINES' THEN [1] ELSE [] END |
                   CREATE (from)-[:DEFINES]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'IMPORTS' THEN [1] ELSE [] END |
                   CREATE (from)-[:IMPORTS]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'EXTENDS' THEN [1] ELSE [] END |
                   CREATE (from)-[:EXTENDS]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'IMPLEMENTS' THEN [1] ELSE [] END |
                   CREATE (from)-[:IMPLEMENTS]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'USES' THEN [1] ELSE [] END |
                   CREATE (from)-[:USES]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'CONTAINS' THEN [1] ELSE [] END |
                   CREATE (from)-[:CONTAINS]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'CONFIGURES' THEN [1] ELSE [] END |
                   CREATE (from)-[:CONFIGURES]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'IMPLEMENTEDBY' THEN [1] ELSE [] END |
                   CREATE (from)-[:IMPLEMENTED_BY]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'INCLUDESJS' THEN [1] ELSE [] END |
                   CREATE (from)-[:INCLUDES_JS]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'INCLUDESCSS' THEN [1] ELSE [] END |
                   CREATE (from)-[:INCLUDES_CSS]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'INCLUDESJSP' THEN [1] ELSE [] END |
                   CREATE (from)-[:INCLUDES_JSP]->(to)
                 )",
            )
            .param("from_ids", from_ids.clone())
            .param("to_ids", to_ids.clone())
            .param("rel_types", rel_types);

            tracing::debug!(batch = batch_idx, "Executing edge batch query");
            match self.graph.run(cypher).await {
                Ok(_) => {
                    created_count += chunk.len();
                    debug!(
                        batch = batch_idx,
                        created = chunk.len(),
                        "Edge batch inserted"
                    );

                    // Afficher la progression des edges
                    let current = (batch_idx + 1) * BATCH_SIZE;
                    let current_clamped = current.min(edges_vec.len());
                    crate::ui::show_progress_stepped(
                        current_clamped,
                        edges_vec.len(),
                        "Exporting edges",
                        BATCH_SIZE,
                    );

                    if (batch_idx + 1) % 10 == 0 {
                        debug!(
                            batch = batch_idx + 1,
                            created = created_count,
                            "Batches de relations exportés"
                        );
                    }
                }
                Err(e) => {
                    debug!(error = %e, batch = batch_idx, "Edge batch error");
                    warn!(error = %e, batch = batch_idx, "Failed to insert edge batch");
                }
            }
        }

        debug!(
            total = unified_graph.edges.len(),
            created = created_count,
            "All edges exported"
        );

        crate::ui::phase_complete("Neo4j Export");
        info!(
            "Export completed: {} nodes, {} edges",
            unified_graph.nodes.len(),
            created_count
        );

        Ok(())
    }

    /// Crée les index pour optimiser les requêtes
    async fn create_indexes(&self) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Creating indexes");

        // Index sur Node.id (le plus important)
        debug!("Creating node_id_index");
        let _ = self
            .graph
            .run(query(
                "CREATE INDEX node_id_index IF NOT EXISTS FOR (n:Node) ON (n.id)",
            ))
            .await;

        // Index sur Node.name
        debug!("Creating node_name_index");
        let _ = self
            .graph
            .run(query(
                "CREATE INDEX node_name_index IF NOT EXISTS FOR (n:Node) ON (n.name)",
            ))
            .await;

        // Index sur Node.file_path
        debug!("Creating node_file_index");
        let _ = self
            .graph
            .run(query(
                "CREATE INDEX node_file_index IF NOT EXISTS FOR (n:Node) ON (n.file_path)",
            ))
            .await;

        // Index sur Node.project_path (pour le mode MCP multi-projets)
        debug!("Creating node_project_index");
        let _ = self
            .graph
            .run(query(
                "CREATE INDEX node_project_index IF NOT EXISTS FOR (n:Node) ON (n.project_path)",
            ))
            .await;

        debug!("Indexes created");
        Ok(())
    }

    /// Supprime toutes les données de la base Neo4j
    pub async fn clear_database(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Suppression des anciennes données...");
        debug!("Clearing database in batches");

        // Supprimer par lots de 10000 pour éviter les timeouts
        loop {
            debug!("Deleting batch of 10000 nodes");
            let mut result = self
                .graph
                .execute(query(
                    "MATCH (n)
                 WITH n LIMIT 10000
                 DETACH DELETE n
                 RETURN count(n) as deleted",
                ))
                .await?;

            if let Some(row) = result.next().await? {
                let deleted: i64 = row.get("deleted").unwrap_or(0);
                debug!(deleted = deleted, "Batch deleted");

                if deleted == 0 {
                    debug!("Database clear complete");
                    break;
                }
            } else {
                break;
            }
        }

        Ok(())
    }

    /// Vérifie la connexion à Neo4j
    pub async fn test_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut result = self.graph.execute(query("RETURN 1 as num")).await?;

        if let Some(row) = result.next().await? {
            let num: i64 = row.get("num")?;
            info!(result = num, "Connexion Neo4j OK");
        }

        Ok(())
    }

    /// Supprime tous les nœuds (et leurs relations) appartenant à un projet
    pub async fn delete_project(
        &self,
        project_path: &str,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        info!("Suppression des données du projet: {}", project_path);
        let mut total_deleted = 0usize;

        loop {
            let mut result = self
                .graph
                .execute(
                    query(
                        "MATCH (n:Node {project_path: $project_path})
                         WITH n LIMIT 10000
                         DETACH DELETE n
                         RETURN count(n) as deleted",
                    )
                    .param("project_path", project_path.to_string()),
                )
                .await?;

            if let Some(row) = result.next().await? {
                let deleted: i64 = row.get("deleted").unwrap_or(0);
                total_deleted += deleted as usize;
                if deleted == 0 {
                    break;
                }
            } else {
                break;
            }
        }

        info!("Projet supprimé: {} nœuds", total_deleted);
        Ok(total_deleted)
    }

    /// Exporte le graphe vers Neo4j en mode projet (sans vider la base entière)
    ///
    /// Contrairement à `export_graph`, cette méthode:
    /// - N'efface PAS toute la base de données
    /// - Identifie chaque nœud et relation avec `project_path` et `project_name`
    /// - Peut optionnellement supprimer les données existantes du projet (`clear_project`)
    pub async fn export_graph_for_project(
        &self,
        unified_graph: &UnifiedGraph,
        project_path: &str,
        project_name: &str,
        clear_project: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Starting project graph export for: {}", project_path);
        crate::ui::phase_start("Neo4j Export (projet)");

        if clear_project {
            info!("Step 1/4: Clearing existing project data");
            self.delete_project(project_path).await?;
        } else {
            info!("Step 1/4: Skipping project clear (clear_project=false)");
        }

        info!("Step 2/4: Creating indexes");
        self.create_indexes().await?;

        const BATCH_SIZE: usize = 500;
        let nodes_vec: Vec<_> = unified_graph.nodes.values().collect();
        let label_mode = env::var("NEO4J_LABEL_MODE").unwrap_or_else(|_| "both".to_string());

        info!("Step 3/4: Exporting nodes");
        debug!(
            total = nodes_vec.len(),
            "Starting batch node export (project)"
        );

        for (batch_idx, chunk) in nodes_vec.chunks(BATCH_SIZE).enumerate() {
            let mut ids = Vec::new();
            let mut names = Vec::new();
            let mut node_types = Vec::new();
            let mut languages = Vec::new();
            let mut file_paths = Vec::new();
            let mut modules = Vec::new();
            let mut packages = Vec::new();
            let mut classes = Vec::new();
            let mut caller_ids = Vec::new();
            let mut object_types = Vec::new();
            let mut url_patterns = Vec::new();
            let mut start_lines = Vec::new();
            let mut start_cols = Vec::new();
            let mut end_lines = Vec::new();
            let mut end_cols = Vec::new();

            for node in chunk {
                ids.push(node.id.clone());
                names.push(node.name.clone());
                let node_type = match &node.kind {
                    NodeKind::Js => "JS".to_string(),
                    other => format!("{:?}", other),
                };
                node_types.push(node_type);
                languages.push(
                    node.metadata
                        .get("language")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "unknown".to_string()),
                );
                file_paths.push(node.file_path.to_string());
                modules.push(
                    node.metadata
                        .get("module")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                packages.push(
                    node.metadata
                        .get("package")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                classes.push(
                    node.metadata
                        .get("class")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                caller_ids.push(
                    node.metadata
                        .get("caller_id")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                object_types.push(
                    node.metadata
                        .get("object_type")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                url_patterns.push(
                    node.metadata
                        .get("url-pattern")
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                );
                start_lines.push(node.location.start_line as i64);
                start_cols.push(node.location.start_col as i64);
                end_lines.push(node.location.end_line as i64);
                end_cols.push(node.location.end_col as i64);
            }

            let cypher = match label_mode.as_str() {
                "typed-only" => {
                    query(
                        "UNWIND range(0, size($ids)-1) AS i
                         CREATE (n:Node {
                            id: $ids[i],
                            name: $names[i],
                            language: $languages[i],
                            file_path: $file_paths[i],
                            module: $modules[i],
                            package: $packages[i],
                            class: $classes[i],
                            caller_id: $caller_ids[i],
                            object_type: $object_types[i],
                            url_pattern: $url_patterns[i],
                            start_line: $start_lines[i],
                            start_col: $start_cols[i],
                            end_line: $end_lines[i],
                            end_col: $end_cols[i],
                            project_path: $project_path,
                            project_name: $project_name
                         })
                         WITH n, $node_types[i] AS node_type
                         FOREACH (_ IN CASE WHEN node_type = 'Function' THEN [1] ELSE [] END | SET n:Function)
                         FOREACH (_ IN CASE WHEN node_type = 'Class' THEN [1] ELSE [] END | SET n:Class)
                         FOREACH (_ IN CASE WHEN node_type = 'Interface' THEN [1] ELSE [] END | SET n:Interface)
                         FOREACH (_ IN CASE WHEN node_type = 'Module' THEN [1] ELSE [] END | SET n:Module)
                         FOREACH (_ IN CASE WHEN node_type = 'JS' THEN [1] ELSE [] END | SET n:JS)
                         FOREACH (_ IN CASE WHEN node_type = 'Type' THEN [1] ELSE [] END | SET n:Type)
                         FOREACH (_ IN CASE WHEN node_type = 'Trait' THEN [1] ELSE [] END | SET n:Trait)
                         FOREACH (_ IN CASE WHEN node_type = 'Parameter' THEN [1] ELSE [] END | SET n:Parameter)
                         FOREACH (_ IN CASE WHEN node_type = 'Variable' THEN [1] ELSE [] END | SET n:Variable)
                         FOREACH (_ IN CASE WHEN node_type = 'Import' THEN [1] ELSE [] END | SET n:Import)
                         FOREACH (_ IN CASE WHEN node_type = 'Package' THEN [1] ELSE [] END | SET n:Package)
                         FOREACH (_ IN CASE WHEN node_type = 'Expression' THEN [1] ELSE [] END | SET n:Expression)
                         FOREACH (_ IN CASE WHEN node_type = 'Operator' THEN [1] ELSE [] END | SET n:Operator)
                         FOREACH (_ IN CASE WHEN node_type = 'PortletXml' THEN [1] ELSE [] END | SET n:PortletXml)
                         FOREACH (_ IN CASE WHEN node_type = 'WebXml' THEN [1] ELSE [] END | SET n:WebXml)
                         FOREACH (_ IN CASE WHEN node_type = 'Portlet' THEN [1] ELSE [] END | SET n:Portlet)
                         FOREACH (_ IN CASE WHEN node_type = 'Servlet' THEN [1] ELSE [] END | SET n:Servlet)
                         FOREACH (_ IN CASE WHEN node_type = 'Filter' THEN [1] ELSE [] END | SET n:Filter)
                         FOREACH (_ IN CASE WHEN node_type = 'Jsp' THEN [1] ELSE [] END | SET n:Jsp)",
                    )
                }
                "both" => {
                    query(
                        "UNWIND range(0, size($ids)-1) AS i
                         CREATE (n:Node {
                            id: $ids[i],
                            name: $names[i],
                            node_type: $node_types[i],
                            language: $languages[i],
                            file_path: $file_paths[i],
                            module: $modules[i],
                            package: $packages[i],
                            class: $classes[i],
                            caller_id: $caller_ids[i],
                            object_type: $object_types[i],
                            url_pattern: $url_patterns[i],
                            start_line: $start_lines[i],
                            start_col: $start_cols[i],
                            end_line: $end_lines[i],
                            end_col: $end_cols[i],
                            project_path: $project_path,
                            project_name: $project_name
                         })
                         WITH n, $node_types[i] AS node_type
                         FOREACH (_ IN CASE WHEN node_type = 'Function' THEN [1] ELSE [] END | SET n:Function)
                         FOREACH (_ IN CASE WHEN node_type = 'Class' THEN [1] ELSE [] END | SET n:Class)
                         FOREACH (_ IN CASE WHEN node_type = 'Interface' THEN [1] ELSE [] END | SET n:Interface)
                         FOREACH (_ IN CASE WHEN node_type = 'Module' THEN [1] ELSE [] END | SET n:Module)
                         FOREACH (_ IN CASE WHEN node_type = 'JS' THEN [1] ELSE [] END | SET n:JS)
                         FOREACH (_ IN CASE WHEN node_type = 'Type' THEN [1] ELSE [] END | SET n:Type)
                         FOREACH (_ IN CASE WHEN node_type = 'Trait' THEN [1] ELSE [] END | SET n:Trait)
                         FOREACH (_ IN CASE WHEN node_type = 'Parameter' THEN [1] ELSE [] END | SET n:Parameter)
                         FOREACH (_ IN CASE WHEN node_type = 'Variable' THEN [1] ELSE [] END | SET n:Variable)
                         FOREACH (_ IN CASE WHEN node_type = 'Import' THEN [1] ELSE [] END | SET n:Import)
                         FOREACH (_ IN CASE WHEN node_type = 'Package' THEN [1] ELSE [] END | SET n:Package)
                         FOREACH (_ IN CASE WHEN node_type = 'Expression' THEN [1] ELSE [] END | SET n:Expression)
                         FOREACH (_ IN CASE WHEN node_type = 'Operator' THEN [1] ELSE [] END | SET n:Operator)
                         FOREACH (_ IN CASE WHEN node_type = 'PortletXml' THEN [1] ELSE [] END | SET n:PortletXml)
                         FOREACH (_ IN CASE WHEN node_type = 'WebXml' THEN [1] ELSE [] END | SET n:WebXml)
                         FOREACH (_ IN CASE WHEN node_type = 'Portlet' THEN [1] ELSE [] END | SET n:Portlet)
                         FOREACH (_ IN CASE WHEN node_type = 'Servlet' THEN [1] ELSE [] END | SET n:Servlet)
                         FOREACH (_ IN CASE WHEN node_type = 'Filter' THEN [1] ELSE [] END | SET n:Filter)
                         FOREACH (_ IN CASE WHEN node_type = 'Jsp' THEN [1] ELSE [] END | SET n:Jsp)",
                    )
                }
                _ => {
                    query(
                        "UNWIND range(0, size($ids)-1) AS i
                         CREATE (n:Node {
                            id: $ids[i],
                            name: $names[i],
                            node_type: $node_types[i],
                            language: $languages[i],
                            file_path: $file_paths[i],
                            module: $modules[i],
                            package: $packages[i],
                            class: $classes[i],
                            caller_id: $caller_ids[i],
                            object_type: $object_types[i],
                            url_pattern: $url_patterns[i],
                            start_line: $start_lines[i],
                            start_col: $start_cols[i],
                            end_line: $end_lines[i],
                            end_col: $end_cols[i],
                            project_path: $project_path,
                            project_name: $project_name
                         })",
                    )
                }
            }
            .param("ids", ids)
            .param("names", names)
            .param("node_types", node_types)
            .param("languages", languages)
            .param("file_paths", file_paths)
            .param("modules", modules)
            .param("packages", packages)
            .param("classes", classes)
            .param("caller_ids", caller_ids)
            .param("object_types", object_types)
            .param("url_patterns", url_patterns)
            .param("start_lines", start_lines)
            .param("start_cols", start_cols)
            .param("end_lines", end_lines)
            .param("end_cols", end_cols)
            .param("project_path", project_path.to_string())
            .param("project_name", project_name.to_string());

            self.graph.run(cypher).await?;

            let current = (batch_idx + 1) * BATCH_SIZE;
            let current_clamped = current.min(nodes_vec.len());
            crate::ui::show_progress_stepped(
                current_clamped,
                nodes_vec.len(),
                "Exporting nodes",
                BATCH_SIZE,
            );
        }

        // Batch insert des relations avec propriétés projet
        let mut created_count = 0;
        let edges_vec: Vec<_> = unified_graph.edges.iter().collect();

        info!("Step 4/4: Exporting edges");

        for (batch_idx, chunk) in edges_vec.chunks(BATCH_SIZE).enumerate() {
            let mut from_ids = Vec::new();
            let mut to_ids = Vec::new();
            let mut rel_types = Vec::new();

            for edge in chunk {
                let resolved_to = if unified_graph.nodes.contains_key(&edge.to) {
                    edge.to.clone()
                } else {
                    let to_name = edge.to.split("::").last().unwrap_or("");
                    unified_graph
                        .nodes
                        .values()
                        .find(|n| n.kind == NodeKind::Function && n.name == to_name)
                        .map(|n| n.id.clone())
                        .unwrap_or_else(|| edge.to.clone())
                };

                from_ids.push(edge.from.clone());
                to_ids.push(resolved_to);
                rel_types.push(format!("{:?}", edge.relation).to_uppercase());
            }

            let cypher = query(
                "UNWIND range(0, size($from_ids)-1) AS i
                 MATCH (from:Node {id: $from_ids[i]})
                 MATCH (to:Node {id: $to_ids[i]})
                 WITH from, to, $rel_types[i] AS rel_type
                 FOREACH (_ IN CASE WHEN rel_type = 'CALLS' THEN [1] ELSE [] END |
                   CREATE (from)-[:CALLS {project_path: $project_path, project_name: $project_name}]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'DEFINES' THEN [1] ELSE [] END |
                   CREATE (from)-[:DEFINES {project_path: $project_path, project_name: $project_name}]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'IMPORTS' THEN [1] ELSE [] END |
                   CREATE (from)-[:IMPORTS {project_path: $project_path, project_name: $project_name}]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'EXTENDS' THEN [1] ELSE [] END |
                   CREATE (from)-[:EXTENDS {project_path: $project_path, project_name: $project_name}]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'IMPLEMENTS' THEN [1] ELSE [] END |
                   CREATE (from)-[:IMPLEMENTS {project_path: $project_path, project_name: $project_name}]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'USES' THEN [1] ELSE [] END |
                   CREATE (from)-[:USES {project_path: $project_path, project_name: $project_name}]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'CONTAINS' THEN [1] ELSE [] END |
                   CREATE (from)-[:CONTAINS {project_path: $project_path, project_name: $project_name}]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'CONFIGURES' THEN [1] ELSE [] END |
                   CREATE (from)-[:CONFIGURES {project_path: $project_path, project_name: $project_name}]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'IMPLEMENTEDBY' THEN [1] ELSE [] END |
                   CREATE (from)-[:IMPLEMENTED_BY {project_path: $project_path, project_name: $project_name}]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'INCLUDESJS' THEN [1] ELSE [] END |
                   CREATE (from)-[:INCLUDES_JS {project_path: $project_path, project_name: $project_name}]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'INCLUDESCSS' THEN [1] ELSE [] END |
                   CREATE (from)-[:INCLUDES_CSS {project_path: $project_path, project_name: $project_name}]->(to)
                 )
                 FOREACH (_ IN CASE WHEN rel_type = 'INCLUDESJSP' THEN [1] ELSE [] END |
                   CREATE (from)-[:INCLUDES_JSP {project_path: $project_path, project_name: $project_name}]->(to)
                 )",
            )
            .param("from_ids", from_ids)
            .param("to_ids", to_ids)
            .param("rel_types", rel_types)
            .param("project_path", project_path.to_string())
            .param("project_name", project_name.to_string());

            match self.graph.run(cypher).await {
                Ok(_) => {
                    created_count += chunk.len();
                    let current = (batch_idx + 1) * BATCH_SIZE;
                    let current_clamped = current.min(edges_vec.len());
                    crate::ui::show_progress_stepped(
                        current_clamped,
                        edges_vec.len(),
                        "Exporting edges",
                        BATCH_SIZE,
                    );
                }
                Err(e) => {
                    warn!(error = %e, batch = batch_idx, "Failed to insert edge batch");
                }
            }
        }

        crate::ui::phase_complete("Neo4j Export (projet)");
        info!(
            "Export projet terminé: {} nœuds, {} relations (project: {})",
            unified_graph.nodes.len(),
            created_count,
            project_name
        );

        Ok(())
    }
}
