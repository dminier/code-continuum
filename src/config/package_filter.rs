// Package Filter - Filtrage intelligent des packages

use regex::Regex;

/// Filtre pour inclure/exclure les packages lors de la résolution des dépendances
#[derive(Debug, Clone)]
pub struct PackageFilter {
    /// Patterns à inclure (whitelist) - exemple: "java.*", "javax.*", "com.example.*"
    pub include_patterns: Vec<String>,

    /// Patterns à exclure (blacklist) - exemple: "java.lang.*", "sun.*"
    #[allow(dead_code)]
    pub exclude_patterns: Vec<String>,

    /// Créer des nœuds fantômes pour les classes non trouvées
    pub create_phantom_nodes: bool,

    /// Profondeur minimale des packages (java vs java.util vs java.util.stream)
    #[allow(dead_code)]
    pub min_package_depth: usize,

    // Cache des regex compilées (pour performance)
    #[doc(hidden)]
    pub include_regexes: Vec<Regex>,
    #[doc(hidden)]
    pub exclude_regexes: Vec<Regex>,
}

impl PackageFilter {
    /// Crée un filtre par défaut
    /// Par défaut: inclut java.* et javax.*, crée des nœuds fantômes
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Crée un filtre personnalisé
    pub fn with_patterns(
        include_patterns: Vec<String>,
        exclude_patterns: Vec<String>,
        create_phantom_nodes: bool,
    ) -> Self {
        let include_regexes = Self::compile_patterns(&include_patterns);
        let exclude_regexes = Self::compile_patterns(&exclude_patterns);

        Self {
            include_patterns,
            exclude_patterns,
            create_phantom_nodes,
            min_package_depth: 1,
            include_regexes,
            exclude_regexes,
        }
    }

    /// Compile les patterns wildcard en regex
    fn compile_patterns(patterns: &[String]) -> Vec<Regex> {
        patterns
            .iter()
            .map(|p| Self::wildcard_to_regex(p))
            .collect()
    }

    /// Convertit un pattern wildcard (*.java.util.*) en Regex
    fn wildcard_to_regex(pattern: &str) -> Regex {
        let regex_str = pattern.replace('.', r"\.").replace('*', ".*");
        Regex::new(&format!("^{}$", regex_str)).unwrap_or_else(|_| Regex::new("^$").unwrap())
        // Fallback vide si regex invalide
    }

    /// Vérifie si un package doit être traité (include ET NOT exclude)
    pub fn should_process(&self, package_name: &str) -> bool {
        // Si include_patterns est vide: comportement "local-only" (rejeter tous les packages)
        if self.include_patterns.is_empty() {
            return false;
        }

        // Vérifier include patterns
        let matches_include = self
            .include_regexes
            .iter()
            .any(|re| re.is_match(package_name));

        // Vérifier exclude patterns
        let matches_exclude = self
            .exclude_regexes
            .iter()
            .any(|re| re.is_match(package_name));

        matches_include && !matches_exclude
    }

    /// Décide si créer un nœud fantôme pour une classe non trouvée
    pub fn should_create_phantom(&self, _fqn: &str) -> bool {
        // Créer un nœud fantôme basé uniquement sur create_phantom_nodes
        // (indépendant de should_process qui gère le traitement des sources)
        self.create_phantom_nodes
    }

    /// Extrait le package d'un FQN (Fully Qualified Name)
    /// Exemple: "java.util.List" -> "java.util"
    pub fn extract_package(fqn: &str) -> Option<String> {
        fqn.rfind('.').map(|idx| fqn[..idx].to_string())
    }

    /// Extrait le nom de classe d'un FQN
    /// Exemple: "java.util.List" -> "List"
    pub fn extract_class_name(fqn: &str) -> String {
        fqn.rfind('.')
            .map(|idx| fqn[idx + 1..].to_string())
            .unwrap_or_else(|| fqn.to_string())
    }
}

impl Default for PackageFilter {
    fn default() -> Self {
        // Configuration par défaut: aucune inclusion, mais créer nœuds fantômes
        // Les dépendances externes sont créées en tant que nœuds fantômes
        Self::with_patterns(
            vec![], // Aucune whitelist: exclure tous les libs
            vec![], // Aucune blacklist (inutile si aucune whitelist)
            true,   // Créer nœuds fantômes pour les dépendances externes
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_filter() {
        let filter = PackageFilter::default();
        // Par défaut: aucune inclusion → tous les packages externes ignorés (local-only)
        // Mais create_phantom_nodes = true (nœuds fantômes activés)
        assert!(!filter.should_process("java.util"));
        assert!(!filter.should_process("javax.servlet"));
        assert!(!filter.should_process("sun.misc"));
        assert!(!filter.should_process("com.sun.internal"));
        assert!(!filter.should_process("com.example.app"));
        // should_create_phantom avec include_patterns vide accepte tout
        assert!(filter.should_create_phantom("java.util.List"));
        assert!(filter.should_create_phantom("sun.misc.Unsafe"));
    }

    #[test]
    fn test_custom_patterns() {
        let filter = PackageFilter::with_patterns(
            vec!["com.example.*".to_string()],
            vec!["com.example.internal.*".to_string()],
            true,
        );

        assert!(filter.should_process("com.example.app"));
        assert!(!filter.should_process("com.example.internal.secret"));
        assert!(!filter.should_process("java.util"));
    }

    #[test]
    fn test_should_create_phantom() {
        let filter_with_phantom = PackageFilter::with_patterns(vec![], vec![], true);

        // Doit créer phantom si create_phantom_nodes=true
        assert!(filter_with_phantom.should_create_phantom("java.util.List"));
        assert!(filter_with_phantom.should_create_phantom("sun.misc.Unsafe"));

        // Ne doit pas créer phantom si create_phantom_nodes=false
        let filter_no_phantom = PackageFilter::with_patterns(vec![], vec![], false);
        assert!(!filter_no_phantom.should_create_phantom("java.util.List"));
        assert!(!filter_no_phantom.should_create_phantom("sun.misc.Unsafe"));
    }

    #[test]
    fn test_extract_package() {
        assert_eq!(
            PackageFilter::extract_package("java.util.List"),
            Some("java.util".to_string())
        );
        assert_eq!(PackageFilter::extract_package("List"), None);
    }

    #[test]
    fn test_extract_class_name() {
        assert_eq!(PackageFilter::extract_class_name("java.util.List"), "List");
        assert_eq!(PackageFilter::extract_class_name("List"), "List");
    }

    #[test]
    fn test_wildcard_patterns() {
        let filter = PackageFilter::with_patterns(
            vec!["com.example.*".to_string(), "org.test.*.*".to_string()],
            vec![],
            false,
        );

        assert!(filter.should_process("com.example.foo"));
        assert!(filter.should_process("com.example.foo.bar"));
        assert!(filter.should_process("org.test.a.b"));
        assert!(!filter.should_process("com.other.foo"));
    }

    #[test]
    fn test_exclude_overrides_include() {
        let filter = PackageFilter::with_patterns(
            vec!["java.*".to_string()],
            vec!["java.lang.*".to_string()],
            false,
        );

        assert!(filter.should_process("java.util.List"));
        assert!(!filter.should_process("java.lang.String"));
    }
}
