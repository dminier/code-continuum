use serde::Serialize;

pub mod executor;

#[derive(Default, Serialize)]
pub struct FileError {
    pub file: String,
    pub error: String,
}

#[derive(Default, Serialize)]
pub struct ParseError {
    pub file: String,
    pub language: String,
    pub error: String,
}

#[derive(Default, Serialize)]
pub struct AnalysisReport {
    pub generated_at: String,
    pub directory: String,
    pub supported_languages: Vec<&'static str>,
    pub processed_files: usize,
    pub unsupported_files: Vec<String>,
    pub read_errors: Vec<FileError>,
    pub parse_errors: Vec<ParseError>,
}
