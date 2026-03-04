// Module COBOL — Extraction et traversée AST via tree-sitter-COBOL
//
// Structure AST COBOL (grammar yutaro-sakamoto/tree-sitter-cobol, tree-sitter ~0.20.3):
//
//   start
//   └── program_definition
//       ├── identification_division
//       │   └── program_name          ← texte direct = PROGRAM-ID
//       ├── data_division
//       │   └── working_storage_section
//       │       ├── data_description  ← level_number + entry_name
//       │       └── copy_statement    ← copybook (field "book" ou WORD enfant)
//       └── procedure_division        ← séquence plate de section_header,
//           ├── section_header        ← paragraph_header, statements
//           ├── paragraph_header      ← nœuds frères (pas imbriqués)
//           ├── perform_statement_call_proc
//           └── call_statement
//
// IMPORTANT: section_header et paragraph_header n'ont pas d'enfant nommé pour le nom.
// Le nom est extrait par parsing du texte complet du nœud.

use std::collections::HashMap;
use tracing::debug;
use tree_sitter::Node;

use crate::semantic_graph::semantic_graph::{
    EdgeRelation, NodeKind, SemanticEdge, SemanticNode, UnifiedGraph,
};

use super::{DslExecutor, ScopeContext};

impl DslExecutor {
    /// Extrait le graphe sémantique d'un fichier COBOL.
    ///
    /// Construit :
    /// - Un nœud `Module` pour le PROGRAM-ID
    /// - Des nœuds `Function` (kind=section) pour chaque SECTION de la PROCEDURE DIVISION
    /// - Des nœuds `Function` (kind=paragraph) pour chaque PARAGRAPH
    /// - Des nœuds `Variable` pour les items de données niveau 01
    /// - Des nœuds `Import` + arêtes `Imports` pour les directives COPY
    /// - Des arêtes `Calls` pour les CALL et PERFORM
    pub(super) fn extract_cobol(&mut self, node: Node, source: &str, graph: &mut UnifiedGraph) {
        // Première passe: localiser le PROGRAM-ID pour nommer le module
        let prog_name = self
            .find_cobol_program_name(node, source)
            .unwrap_or_else(|| {
                // Fallback: utiliser le nom du fichier sans extension
                std::path::Path::new(&self.file_path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("UNKNOWN")
                    .to_uppercase()
            });

        let module_id = format!("{}::module", self.file_path);

        debug!(
            file = %self.file_path,
            program = %prog_name,
            "Extraction COBOL démarrée"
        );

        let mut metadata = self.metadata("cobol");
        metadata.insert("program_id".to_string(), prog_name.clone());

        graph.add_node(SemanticNode {
            id: module_id.clone(),
            kind: NodeKind::Module,
            name: prog_name,
            file_path: self.file_path.clone(),
            location: self.root_location(),
            metadata,
        });

        let ctx = ScopeContext::new().with_module(self.file_path.clone(), module_id);
        self.traverse_cobol_root(node, source, graph, &ctx);
    }

    /// Recherche récursive du premier nœud `program_name` dans l'AST.
    fn find_cobol_program_name(&self, node: Node, source: &str) -> Option<String> {
        if node.kind() == "program_name" {
            let text = self.get_text(node, source).trim().to_uppercase();
            if !text.is_empty() {
                return Some(text);
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(name) = self.find_cobol_program_name(child, source) {
                return Some(name);
            }
        }
        None
    }

    /// Traversée de haut niveau : dispatch vers les sous-extracteurs.
    fn traverse_cobol_root(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        match node.kind() {
            // Divisions de données: chercher les data_description et copy_statement
            "data_division"
            | "working_storage_section"
            | "linkage_section"
            | "local_storage_section"
            | "file_section" => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.traverse_cobol_root(child, source, graph, ctx);
                }
            }

            "data_description" => {
                self.extract_cobol_data_item(node, source, graph, ctx);
            }

            "copy_statement" => {
                self.extract_cobol_copy(node, source, graph, ctx);
            }

            "procedure_division" => {
                self.extract_cobol_procedure_division(node, source, graph, ctx);
            }

            // Pour tous les autres nœuds, descendre récursivement
            _ => {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    self.traverse_cobol_root(child, source, graph, ctx);
                }
            }
        }
    }

    /// Traite la PROCEDURE DIVISION de façon linéaire.
    ///
    /// En COBOL, sections et paragraphes sont des nœuds FRÈRES (pas imbriqués).
    /// On maintient l'état courant (section/paragraph) pour rattacher les appels.
    fn extract_cobol_procedure_division(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        let module_id = ctx.module_id.as_deref().unwrap_or(&self.file_path);
        let mut current_section_id: Option<String> = None;
        let mut current_para_id: Option<String> = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "section_header" => {
                    let name = self.extract_section_name(child, source);
                    if name.is_empty() {
                        continue;
                    }

                    let section_id = format!("{}::section:{}", module_id, name);
                    let mut meta = self.metadata("cobol");
                    meta.insert("kind".to_string(), "section".to_string());

                    debug!(section = %name, "COBOL section extraite");

                    graph.add_node(SemanticNode {
                        id: section_id.clone(),
                        kind: NodeKind::Function,
                        name,
                        file_path: self.file_path.clone(),
                        location: self.node_location(child),
                        metadata: meta,
                    });
                    graph.add_edge(SemanticEdge {
                        from: module_id.to_string(),
                        to: section_id.clone(),
                        relation: EdgeRelation::Contains,
                        metadata: HashMap::new(),
                    });

                    current_section_id = Some(section_id);
                    current_para_id = None;
                }

                "paragraph_header" => {
                    let name = self.extract_paragraph_name(child, source);
                    if name.is_empty() {
                        continue;
                    }

                    let para_id = format!("{}::paragraph:{}", module_id, name);
                    let mut meta = self.metadata("cobol");
                    meta.insert("kind".to_string(), "paragraph".to_string());

                    debug!(paragraph = %name, "COBOL paragraph extrait");

                    graph.add_node(SemanticNode {
                        id: para_id.clone(),
                        kind: NodeKind::Function,
                        name,
                        file_path: self.file_path.clone(),
                        location: self.node_location(child),
                        metadata: meta,
                    });

                    let parent_id = current_section_id.as_deref().unwrap_or(module_id);
                    graph.add_edge(SemanticEdge {
                        from: parent_id.to_string(),
                        to: para_id.clone(),
                        relation: EdgeRelation::Contains,
                        metadata: HashMap::new(),
                    });

                    current_para_id = Some(para_id);
                }

                "perform_statement_call_proc" => {
                    let caller = current_para_id
                        .as_deref()
                        .or(current_section_id.as_deref())
                        .unwrap_or(module_id);

                    if let Some(target) = self.extract_perform_target(child, source) {
                        let mut meta = HashMap::new();
                        meta.insert("perform_target".to_string(), target.clone());
                        meta.insert("cobol_verb".to_string(), "PERFORM".to_string());

                        debug!(caller = %caller, target = %target, "COBOL PERFORM détecté");

                        graph.add_edge(SemanticEdge {
                            from: caller.to_string(),
                            to: format!("temp_call:{}", target),
                            relation: EdgeRelation::Calls,
                            metadata: meta,
                        });
                    }
                }

                "call_statement" => {
                    let caller = current_para_id
                        .as_deref()
                        .or(current_section_id.as_deref())
                        .unwrap_or(module_id);

                    if let Some(target) = self.extract_call_target(child, source) {
                        let mut meta = HashMap::new();
                        meta.insert("call_program".to_string(), target.clone());
                        meta.insert("cobol_verb".to_string(), "CALL".to_string());

                        debug!(caller = %caller, program = %target, "COBOL CALL détecté");

                        graph.add_edge(SemanticEdge {
                            from: caller.to_string(),
                            to: format!("temp_call:{}", target),
                            relation: EdgeRelation::Calls,
                            metadata: meta,
                        });
                    }
                }

                // COPY peut aussi apparaître dans la PROCEDURE DIVISION
                "copy_statement" => {
                    self.extract_cobol_copy(child, source, graph, ctx);
                }

                _ => {}
            }
        }
    }

    /// Extrait le nom d'une SECTION depuis le nœud `section_header`.
    ///
    /// La grammaire tree-sitter-COBOL stocke le nom comme un token anonyme (inline rule).
    /// Le nom est donc extrait par parsing du texte complet : "MAIN-SECTION SECTION." → "MAIN-SECTION"
    fn extract_section_name(&self, node: Node, source: &str) -> String {
        let full = self.get_text(node, source);
        full.split_ascii_whitespace()
            .next()
            .map(|s| {
                let upper = s.trim_end_matches('.').to_uppercase();
                // Filtrer le mot-clé SECTION lui-même (cas dégénéré sans nom)
                if upper == "SECTION" {
                    String::new()
                } else {
                    upper
                }
            })
            .unwrap_or_default()
    }

    /// Extrait le nom d'un PARAGRAPH depuis le nœud `paragraph_header`.
    ///
    /// "INIT-PARAGRAPH." → "INIT-PARAGRAPH"
    fn extract_paragraph_name(&self, node: Node, source: &str) -> String {
        self.get_text(node, source)
            .trim()
            .trim_end_matches('.')
            .trim()
            .to_uppercase()
    }

    /// Extrait la cible d'un PERFORM statement.
    ///
    /// Structure AST: perform_statement_call_proc → perform_procedure → label → qualified_word → WORD
    fn extract_perform_target(&self, node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "perform_procedure" {
                // Naviguer vers le WORD le plus profond
                if let Some(word) = self.find_word_in_node(child, source) {
                    return Some(word.to_uppercase());
                }
                // Fallback: texte brut du perform_procedure (ex: "INIT-PARAGRAPH")
                let text = self.get_text(child, source).trim().to_uppercase();
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        // Fallback final: chercher un WORD directement
        self.find_word_in_node(node, source)
            .map(|w| w.to_uppercase())
    }

    /// Extrait le nom du programme cible d'un CALL statement.
    ///
    /// Structure AST: call_statement → string "'DBACCESS'" → texte sans guillemets
    fn extract_call_target(&self, node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "string" => {
                    let clean = self
                        .get_text(child, source)
                        .trim()
                        .trim_matches('\'')
                        .trim_matches('"')
                        .trim()
                        .to_uppercase();
                    if !clean.is_empty() {
                        return Some(clean);
                    }
                }
                "WORD" => {
                    let text = self.get_text(child, source).trim().to_uppercase();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// Recherche récursive du premier nœud `WORD` dans un sous-arbre.
    fn find_word_in_node(&self, node: Node, source: &str) -> Option<String> {
        if node.kind() == "WORD" {
            let text = self.get_text(node, source).trim().to_string();
            if !text.is_empty() {
                return Some(text);
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(word) = self.find_word_in_node(child, source) {
                return Some(word);
            }
        }
        None
    }

    /// Extrait un item de données WORKING-STORAGE (niveau 01 ou 77 uniquement).
    fn extract_cobol_data_item(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        let module_id = ctx.module_id.as_deref().unwrap_or(&self.file_path);

        let mut level_text = String::new();
        let mut var_name = String::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "level_number" => {
                    level_text = self.get_text(child, source).trim().to_string();
                }
                "entry_name" => {
                    var_name = self.get_text(child, source).trim().to_uppercase();
                }
                _ => {}
            }
        }

        // Extraire seulement les items indépendants (niveau 01 ou 77)
        if !var_name.is_empty() && (level_text == "01" || level_text == "77") {
            let var_id = format!("{}::variable:{}", module_id, var_name);
            let mut meta = self.metadata("cobol");
            meta.insert("level".to_string(), level_text);

            debug!(variable = %var_name, "COBOL variable extraite");

            graph.add_node(SemanticNode {
                id: var_id.clone(),
                kind: NodeKind::Variable,
                name: var_name,
                file_path: self.file_path.clone(),
                location: self.node_location(node),
                metadata: meta,
            });
            graph.add_edge(SemanticEdge {
                from: module_id.to_string(),
                to: var_id,
                relation: EdgeRelation::Contains,
                metadata: HashMap::new(),
            });
        }
    }

    /// Extrait une directive COPY (inclusion de copybook).
    ///
    /// Crée un nœud `Import` et une arête `Imports` vers le copybook.
    fn extract_cobol_copy(
        &self,
        node: Node,
        source: &str,
        graph: &mut UnifiedGraph,
        ctx: &ScopeContext,
    ) {
        let module_id = ctx.module_id.as_deref().unwrap_or(&self.file_path);

        let copybook_name = self
            .extract_copy_book_name(node, source)
            .unwrap_or_default();

        if copybook_name.is_empty() {
            return;
        }

        let import_id = format!("{}::copy:{}", module_id, copybook_name);
        let mut meta = self.metadata("cobol");
        meta.insert("copybook".to_string(), copybook_name.clone());

        debug!(copybook = %copybook_name, "COBOL COPY extrait");

        graph.add_node(SemanticNode {
            id: import_id.clone(),
            kind: NodeKind::Import,
            name: copybook_name,
            file_path: self.file_path.clone(),
            location: self.node_location(node),
            metadata: meta,
        });
        graph.add_edge(SemanticEdge {
            from: module_id.to_string(),
            to: import_id,
            relation: EdgeRelation::Imports,
            metadata: HashMap::new(),
        });
    }

    /// Extrait le nom du copybook depuis un nœud `copy_statement`.
    ///
    /// Essaie d'abord le champ "book", puis cherche un WORD ou string dans les enfants.
    fn extract_copy_book_name(&self, node: Node, source: &str) -> Option<String> {
        // Champ "book" (selon la grammaire)
        if let Some(book_node) = node.child_by_field_name("book") {
            let text = self
                .get_text(book_node, source)
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .trim()
                .to_uppercase();
            if !text.is_empty() {
                return Some(text);
            }
        }

        // Fallback: premier WORD ou string parmi les enfants
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "WORD" => {
                    let text = self.get_text(child, source).trim().to_uppercase();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
                "string" => {
                    let text = self
                        .get_text(child, source)
                        .trim()
                        .trim_matches('\'')
                        .trim_matches('"')
                        .trim()
                        .to_uppercase();
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
                _ => {}
            }
        }
        None
    }
}
