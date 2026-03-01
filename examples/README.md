# 📁 Examples - Fixtures de Test

Ce dossier contient les **fixtures de test** du projet. Tous les tests d'intégration doivent s'appuyer sur ces fichiers.

> ⚠️ **RÈGLE ABSOLUE** : Ne jamais créer de fixtures de test ailleurs que dans ce dossier.

## Structure

```
examples/
├── backend/                    # Code serveur (Java, JS backend)
│   ├── java/                   # Classes, services, héritage
│   └── javascript/             # Services Node.js, pipelines
├── frontend/                   # Code client (HTML, JS frontend)
│   ├── html/                   # Templates HTML statiques
│   └── javascript/             # Composants, appels API
├── web_templates/              # Templates web dynamiques
│   ├── *.jsp, *.jspx          # Templates JSP/JSPX
│   ├── WEB-INF/               # Configuration web Java EE
│   ├── common/                # Includes JSP partagés
│   └── resources/             # Ressources statiques
├── config/                     # Fichiers de configuration
│   ├── app.xml                # Configuration applicative
│   └── portlet.xml            # Configuration portlet
└── websphere-portal/          # Cas d'usage WebSphere Portal
    ├── java/                  # Code Java Portal
    └── *.xml                  # Configurations Portal
```

## Cas d'Usage Couverts

### 🔷 Java (`backend/java/`)

| Fichier | Cas d'Usage | Concepts Testés |
|---------|-------------|-----------------|
| `User.java` | Entité basique | Classe simple, champs, getters/setters |
| `UserService.java` | Service métier | Injection, appels de méthodes |
| `ServiceA.java`, `ServiceB.java`, `ServiceC.java` | Chaîne d'appels | Dépendances inter-services |
| `BaseClass.java`, `DerivedClass.java` | Héritage | Extension de classe, override |
| `BaseInterface.java` | Interface | Contrats, implémentation |
| `TransitiveChain.java` | Appels transitifs | A→B→C, profondeur d'appels |
| `StaticCallsExample.java` | Appels statiques | Méthodes static, factory |
| `ServiceCSingleton.java` | Singleton | Pattern singleton |
| `DatabaseManager.java` | Accès données | DAO, connexion DB |
| `Level0Class.java`, `Level1Class.java`, `Level2Class.java` | Hiérarchie | Multi-niveaux d'héritage |

### 🟨 JavaScript (`backend/javascript/` & `frontend/javascript/`)

| Fichier | Cas d'Usage | Concepts Testés |
|---------|-------------|-----------------|
| `BaseService.js` | Service de base | Classe ES6, export |
| `DerivedService.js` | Héritage JS | Extension, super() |
| `DataService.js` | Service données | Async/await, fetch |
| `transitive-pipeline.js` | Pipeline | Chaîne de transformations |
| `api.js` | Client API | Appels HTTP, endpoints |
| `app.js` | Point d'entrée | Bootstrap, initialisation |
| `components.tsx` | React/TSX | Composants, JSX |

### 🌐 Web Templates (`web_templates/`)

| Fichier | Cas d'Usage | Concepts Testés |
|---------|-------------|-----------------|
| `index.jsp` | Page principale | Directives JSP, includes |
| `user_profile.jspx` | JSPX | Format XML strict |
| `common/` | Fragments | Réutilisation, includes |
| `WEB-INF/web.xml` | Config web | Servlets, filtres, mappings |

### ⚙️ Configuration (`config/` & `websphere-portal/`)

| Fichier | Cas d'Usage | Concepts Testés |
|---------|-------------|-----------------|
| `app.xml` | Config app | Paramètres applicatifs |
| `portlet.xml` | Portlets | Définition portlets JSR-286 |
| `web.xml` | Web descriptor | Servlets, security |
| `application-context.xml` | Spring | Beans, injection |

## Comment Ajouter une Nouvelle Fixture

### 1. Identifier le cas d'usage

Avant de créer un fichier, vérifier :
- [ ] Le cas d'usage n'est pas déjà couvert par un fichier existant
- [ ] Le concept est suffisamment distinct pour justifier un nouveau fichier

### 2. Choisir l'emplacement

```
backend/java/          → Classes Java serveur
backend/javascript/    → Services Node.js
frontend/javascript/   → Code client JS/TS
web_templates/         → JSP, JSPX, HTML dynamique
config/                → XML de configuration
websphere-portal/      → Spécifique WebSphere
```

### 3. Nommer explicitement

Le nom du fichier doit indiquer clairement le concept testé :

| ❌ Mauvais | ✅ Bon |
|-----------|--------|
| `Test1.java` | `TransitiveChain.java` |
| `Example.js` | `StaticCallsExample.js` |
| `data.java` | `DatabaseManager.java` |

### 4. Documenter dans ce README

Après création, ajouter une ligne dans le tableau correspondant.

### 5. Créer le test associé

```rust
#[test]
fn test_nouveau_concept() {
    let fixture = PathBuf::from("examples/backend/java/NouveauConcept.java");
    assert!(fixture.exists());
    // ...
}
```

## Validation

Avant de commit, vérifier que toutes les fixtures sont utilisées :

```bash
# Lister les fichiers examples non référencés dans les tests
grep -rL "examples/" tests/ --include="*.rs"
```

## Conventions

1. **Code valide** : Chaque fichier doit être syntaxiquement correct
2. **Auto-documenté** : Commentaires expliquant le concept illustré
3. **Minimal** : Le strict nécessaire pour tester le concept
4. **Réaliste** : Proche du code de production (pas de `foo`, `bar`)
