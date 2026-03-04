// ============================================================
// Integration Tests - Extraction
// ============================================================

#[path = "common/mod.rs"]
mod common;

#[path = "extraction/field_extraction.rs"]
mod extraction_field;

#[path = "extraction/javascript.rs"]
mod extraction_javascript;

#[path = "extraction/java_imports.rs"]
mod extraction_java_imports;

#[path = "extraction/external_class_file_path.rs"]
mod extraction_external_class_file_path;

#[path = "extraction/batch_analysis_file_path.rs"]
mod extraction_batch_analysis;

#[path = "extraction/real_examples_analysis.rs"]
mod extraction_real_examples;

#[path = "extraction/servlet_url_pattern.rs"]
mod extraction_servlet_url_pattern;

#[path = "extraction/jsp_transitive_includes.rs"]
mod extraction_jsp_transitive_includes;

#[path = "extraction/jsp_imports.rs"]
mod extraction_jsp_imports;

#[path = "extraction/rust.rs"]
mod extraction_rust;

#[path = "extraction/cobol.rs"]
mod extraction_cobol;
