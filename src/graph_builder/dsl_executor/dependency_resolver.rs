// DependencyResolver - Résolution intelligente des dépendances

use crate::config::PackageFilter;
use std::collections::HashMap;

/// Résultat de la résolution d'une dépendance
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencyTarget {
    /// Classe trouvée localement (dans le projet analysé)
    Local(String), // node_id

    /// Classe externe (library/framework)
    External(String), // fqn (java.util.List, etc.)

    /// Classe non résolue (ne sait pas si locale ou externe)
    Unresolved(String), // reference name
}

/// Résolveur de dépendances avec support du filtrage intelligent
#[derive(Debug)]
pub struct DependencyResolver {
    /// Index: FQN -> node_id pour les classes locales
    local_classes: HashMap<String, String>,

    /// Index: classe simple -> [FQN] pour recherche rapide
    simple_name_index: HashMap<String, Vec<String>>,

    /// Filtre de packages
    filter: PackageFilter,
}

impl DependencyResolver {
    /// Crée un nouveau resolver avec le filtre par défaut
    pub fn new() -> Self {
        Self::with_filter(PackageFilter::default())
    }

    /// Crée un resolver avec un filtre personnalisé
    pub fn with_filter(filter: PackageFilter) -> Self {
        Self {
            local_classes: HashMap::new(),
            simple_name_index: HashMap::new(),
            filter,
        }
    }

    /// Enregistre une classe locale
    /// Exemple: register_local("java.util.List", "java.util::List")
    pub fn register_local(&mut self, fqn: &str, node_id: &str) {
        self.local_classes
            .insert(fqn.to_string(), node_id.to_string());

        // Indexer par nom simple aussi pour recherche rapide
        let simple_name = PackageFilter::extract_class_name(fqn);
        self.simple_name_index
            .entry(simple_name)
            .or_default()
            .push(fqn.to_string());
    }

    /// Résout une référence à une classe
    /// 1. Chercher dans les classes locales
    /// 2. Si trouvé: retourner Local
    /// 3. Si non trouvé et matches filtre: retourner External
    /// 4. Sinon: retourner Unresolved
    pub fn resolve(&self, reference: &str) -> DependencyTarget {
        // Cas 1: Reference est un FQN complet (java.util.List)
        if let Some(node_id) = self.local_classes.get(reference) {
            return DependencyTarget::Local(node_id.clone());
        }

        // Cas 2: Reference est un nom simple (List) - chercher localement
        if !reference.contains('.') {
            // Chercher dans l'index par nom simple
            if let Some(fqns) = self.simple_name_index.get(reference) {
                if !fqns.is_empty() {
                    if fqns.len() > 1 {
                        tracing::warn!(
                            name = %reference,
                            candidates = ?fqns,
                            "Résolution ambiguë par nom simple, utilise le premier enregistré"
                        );
                    }
                    if let Some(node_id) = self.local_classes.get(&fqns[0]) {
                        return DependencyTarget::Local(node_id.clone());
                    }
                }
            }
        }

        // Cas 3: Vérifier si c'est une classe externe acceptable
        if self.should_create_node(reference) {
            return DependencyTarget::External(reference.to_string());
        }

        // Cas 4: Non résolu (probablement filtrée ou inconnue)
        DependencyTarget::Unresolved(reference.to_string())
    }

    /// Décide si créer un nœud pour cette référence
    pub fn should_create_node(&self, reference: &str) -> bool {
        self.filter.should_create_phantom(reference)
    }

    /// Récupère le filtre courant
    pub fn filter(&self) -> &PackageFilter {
        &self.filter
    }

    /// Retourne le nombre de classes locales enregistrées (utile pour diagnostics)
    #[allow(dead_code)]
    pub fn local_class_count(&self) -> usize {
        self.local_classes.len()
    }

    /// Retourne le nombre de classes simples indexées (utile pour diagnostics)
    #[allow(dead_code)]
    pub fn simple_name_count(&self) -> usize {
        self.simple_name_index.len()
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_resolve_local() {
        let mut resolver = DependencyResolver::new();

        resolver.register_local("java.util.List", "java.util::List");
        resolver.register_local("java.util.HashMap", "java.util::HashMap");

        assert_eq!(
            resolver.resolve("java.util.List"),
            DependencyTarget::Local("java.util::List".to_string())
        );
    }

    #[test]
    fn test_resolve_simple_name_local() {
        let mut resolver = DependencyResolver::new();

        resolver.register_local("com.example.MyClass", "com.example::MyClass");

        // Résoudre par nom simple
        assert_eq!(
            resolver.resolve("MyClass"),
            DependencyTarget::Local("com.example::MyClass".to_string())
        );
    }

    #[test]
    fn test_resolve_external() {
        let resolver = DependencyResolver::new(); // Filtre par défaut: java.*, javax.*

        // java.util.List doit être external (pas enregistré localement)
        match resolver.resolve("java.util.List") {
            DependencyTarget::External(fqn) => assert_eq!(fqn, "java.util.List"),
            _ => panic!("Expected External"),
        }
    }

    #[test]
    fn test_resolve_filtered_out() {
        let filter = PackageFilter::with_patterns(vec![], vec![], false); // No phantoms
        let resolver = DependencyResolver::with_filter(filter);

        // sun.misc.Unsafe doit être unresolved (create_phantom_nodes=false)
        assert_eq!(
            resolver.resolve("sun.misc.Unsafe"),
            DependencyTarget::Unresolved("sun.misc.Unsafe".to_string())
        );
    }

    #[test]
    fn test_custom_filter() {
        let filter = PackageFilter::with_patterns(vec!["com.example.*".to_string()], vec![], true);
        let resolver = DependencyResolver::with_filter(filter);

        // java.util.List doit être external (create_phantom_nodes=true, même si pas dans include list)
        match resolver.resolve("java.util.List") {
            DependencyTarget::External(_) => {}
            _ => panic!("Expected External for phantom creation"),
        }

        // com.example.Foo doit aussi être external
        match resolver.resolve("com.example.Foo") {
            DependencyTarget::External(_) => {}
            _ => panic!("Expected External"),
        }
    }

    #[test]
    fn test_local_class_count() {
        let mut resolver = DependencyResolver::new();

        assert_eq!(resolver.local_class_count(), 0);

        resolver.register_local("java.util.List", "java.util::List");
        resolver.register_local("java.util.HashMap", "java.util::HashMap");

        assert_eq!(resolver.local_class_count(), 2);
    }

    #[test]
    fn test_ambiguous_simple_name_prefers_first() {
        let mut resolver = DependencyResolver::new();

        resolver.register_local("com.example.Foo", "com.example::Foo");
        resolver.register_local("com.other.Foo", "com.other::Foo");

        // En cas d'ambiguïté, préfère le premier enregistré
        let result = resolver.resolve("Foo");
        match result {
            DependencyTarget::Local(id) => {
                assert_eq!(id, "com.example::Foo"); // Premier enregistré
            }
            _ => panic!("Expected Local"),
        }
    }
}
