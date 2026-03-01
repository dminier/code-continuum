# Organisation des tests d'intégration

Ce projet utilise la structure de tests recommandée par Rust selon [la documentation officielle](https://doc.rust-lang.org/book/ch11-03-test-organization.html).

## Structure

```
code-continuum/
├── src/                    # Code source principal + tests unitaires
│   ├── main.rs
│   ├── lib.rs             # (avec #[cfg(test)] modules si applicable)
│   └── ...
└── tests/                 # Tests d'intégration (répertoire spécial Cargo)
    ├── common/            # Code partagé entre les tests d'intégration
    │   └── mod.rs         # Utilitaires, configurations, helpers
    ├── extraction/        # Tests relatifs à l'extraction de code
    │   ├── field_extraction.rs
    │   └── javascript.rs
    ├── neo4j/            # Tests relatifs à Neo4j
    │   ├── connection.rs
    │   └── calls.rs
    └── debug/            # Tests de débogage et AST
        └── java_ast.rs
```

## Types de tests

### Tests unitaires
- **Emplacement** : Dans les fichiers source (`src/`)
- **Annotation** : `#[cfg(test)]`
- **Scope** : Testent des modules individuels en isolation
- **Accès** : Peuvent tester les APIs privées

### Tests d'intégration
- **Emplacement** : Dans `tests/`
- **Chaque fichier** : Compilé comme une crate binaire séparée
- **Scope** : Testent la configuration globale et l'interaction entre modules
- **Accès** : Utilisent uniquement les APIs publiques

## Module commun

Le module `tests/common/mod.rs` contient des utilitaires partagés :
- Configuration de l'environnement Neo4j
- Fonctions d'assistance pour les tests
- Configurations communes

Pour utiliser le module commun dans un test d'intégration :
```rust
mod common;

#[test]
fn my_test() {
    common::setup_env();
    // ... test code
}
```

## Exécution

```bash
# Tous les tests
cargo test

# Seulement les tests d'intégration
cargo test --test '*'

# Tests d'intégration spécifiques
cargo test --test extraction

# Test spécifique dans un fichier
cargo test --test java_ast test_java_ast_structure
```

## Avantages de cette structure

1. **Séparation claire** : Tests unitaires vs tests d'intégration
2. **Compilation indépendante** : Chaque fichier de test compile seul
3. **Isolation** : Pas de fuite entre les tests
4. **Maintenabilité** : Structure standard reconnue par la communauté Rust
5. **Partage facile** : Code commun via `tests/common/`
