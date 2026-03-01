use std::path::Path;

pub struct CliArgs {
    pub source_directory: String,
}

pub fn parse_args() -> Result<CliArgs, String> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return Err("Veuillez spécifier le répertoire source à analyser".to_string());
    }

    Ok(CliArgs {
        source_directory: args[1].clone(),
    })
}

pub fn validate_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Le chemin n'existe pas: {}", path.display()));
    }

    if !path.is_dir() {
        return Err(format!("N'est pas un répertoire: {}", path.display()));
    }

    Ok(())
}
