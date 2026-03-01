use crate::semantic_graph::dsl::DslRegistry;
use std::path::{Path, PathBuf};

pub fn collect_source_files(path: &Path, files: &mut Vec<PathBuf>, unsupported: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let file_path = entry.path();

                // Ignorer les répertoires cachés (.git, .vscode, etc.)
                if let Some(file_name) = file_path.file_name() {
                    if let Some(name_str) = file_name.to_str() {
                        if name_str.starts_with('.') {
                            continue;
                        }
                    }
                }

                if file_path.is_dir() {
                    // Parcourir récursivement les sous-répertoires
                    collect_source_files(&file_path, files, unsupported);
                } else {
                    // Ajouter les fichiers source supportés par le registre DSL
                    if DslRegistry::detect_language_from_path(&file_path).is_some() {
                        files.push(file_path);
                    } else {
                        unsupported.push(file_path);
                    }
                }
            }
        }
    }
}
