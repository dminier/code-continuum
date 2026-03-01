# SCHEMA_IA.md - Référence Cypher pour Agents IA

**Schéma Neo4j essentiels et suffisants pour construire des requêtes Cypher.**

---

## 🎯 Labels (Types de nœuds)

```
:Function     - Fonctions/Méthodes (object_method: true si méthode d'object literal JS)
:Class        - Classes/Interfaces/Types
:Interface    - Interfaces (héritent aussi de :Class)
:Variable     - Variables/Paramètres
:Parameter    - Paramètres de fonction
:Import       - Imports/requires
:Package      - Packages Java
:Module       - Modules/Namespaces (object_literal: true si object literal JS)
:Type         - Types personnalisés
:Trait        - Traits/Mixins (Rust)
:Expression   - Expressions
:Operator     - Opérateurs
:JS           - Fichiers JavaScript (file/module anchor)
:Jsp          - Fichiers JSP/JSPX/JSPF
:WebXml       - Fichier web.xml
:PortletXml   - Fichier portlet.xml
:Servlet      - Servlets déclarés (web.xml) ⚠️ Représente le NOM, pas la classe
:Portlet      - Portlets déclarés (portlet.xml) ⚠️ Représente le NOM, pas la classe
:Filter       - Filters déclarés (web.xml) ⚠️ Représente le NOM, pas la classe
```

### ⚠️ Important : NOM vs CLASSE pour les composants backend

Les nœuds `:Portlet`, `:Servlet` et `:Filter` représentent le **NOM** du composant tel que défini dans les fichiers de configuration (`portlet.xml`, `web.xml`), **pas la classe d'implémentation**.

| Nœud | Propriété `name` | Exemple | Lien vers classe |
|------|------------------|---------|------------------|
| `:Portlet` | `<portlet-name>` | `GestionEP` | `IMPLEMENTED_BY` → `:Class` |
| `:Servlet` | `<servlet-name>` | `SearchServlet` | `IMPLEMENTED_BY` → `:Class` |
| `:Filter` | `<filter-name>` | `AuthFilter` | `IMPLEMENTED_BY` → `:Class` |

Voir : [definition-composants-backend.md](../rules/definition-composants-backend.md)

---

## 🔗 Relations principales

```
# Relations d'appel
CALLS          Function → Function      (appel de fonction)
               Métadonnées: call_type = "standard|ajax|service|dao"
                          method, async, operation, etc.
TRIGGERS       Function → Node          (déclenchement)

# Relations de structure
CONTAINS       Class/JS/Module → Function/Class (contenance)
DEFINES        Class/Module → Function  (définition de méthode, y compris object literal JS)
HAS_PARAM      Function → Parameter     (paramètres)
RETURNS        Function → Type          (type de retour)

# Relations de dépendance
USES           Function → Variable      (utilisation)
IMPORTS        Function/Class → Import  (import/require)
REFERENCES     Node → Definition        (référence)
DEPENDS_ON     JS → JS                  (dépendance JS)

# Relations de type
HAS_TYPE       Node → Type              (typage)
DECLARE_TYPE   Node → Type              (déclaration type)
EXTENDS        Class → Class            (héritage)
IMPLEMENTS     Class → Interface        (implémentation)
IMPLEMENTED_BY Interface → Class        (inverse IMPLEMENTS)

# Relations d'assignation et visibilité
ASSIGNED_BY    Variable → Expression    (assignation)
EXPORTS        Module → Symbol          (export)

# Relations WebSphere
CONFIGURES     WebXml/PortletXml → Servlet/Portlet/Filter
IMPLEMENTED_BY Servlet/Portlet/Filter → Class    (implémentation Java)
RENDERS        Portlet → JSP            (rendu JSP)
NOTIFIES       Service → Service        (notification)

# Relations JSP
INCLUDES_JS    Jsp → Js ou Jsp          (inclusion JS: fichier .js OU .jsp/.jspx générant du JS dynamique)
INCLUDES_CSS   Jsp → (CSS file)         (inclusion CSS)
INCLUDES_JSP   Jsp → Jsp                (inclusion JSP)
IMPORTS        Jsp → Class              (imports Java: <%@page import="..."%>)
BINDS_DATA     JSP ↔ JS                 (partage données)

# Relations DOM
TARGETS_ELEMENT JS → DOM                (ciblage DOM)
```

---

## 📋 Propriétés clés des nœuds

**Toujours disponibles :**
- `id` : Identifiant unique qualifié
- `name` : Nom court
- `node_type` : Type (ex: "Function", "Class", "JS")
- `language` : "java", "javascript", "jsp", "xml", etc.
- `file_path` : Chemin du fichier source
- `start_line`, `end_line` : Position dans le fichier

**Optionnels par type :**
- `package` : Pour les nœuds Java
- `class` : Pour les méthodes
- `module` : Pour les nœuds JavaScript
- `metadata` : Dictionnaire pour propriétés supplémentaires
  - `metadata.class` : Pour Servlet/Portlet/Filter (FQN de la classe Java)
  - `metadata.url-pattern` : Pour Servlet/Filter (pattern d'URL, ex: `/user/*`, `*.srv`)
  - `metadata.qualified_name` : Pour Class (nom complet avec package)
  - `metadata.external` : Boolean (true si classe externe/non analysée)
  - `metadata.import_type` : Pour relation IMPORTS (ex: "java")

---

## 🔗 Propriétés des relations

### IMPORTS (JSP → Class ou Class → Class)

```
IMPORTS {
  import_type: "java"              # Type d'import
  qualified_name: String           # Nom complet de la classe importée
}
```

**Exemple JSP :**
```cypher
MATCH (jsp:Jsp)-[r:IMPORTS]->(cls:Class)
RETURN jsp.file_path, r.qualified_name, cls.id
```

### INCLUDES_JS (JSP → Js ou Jsp)

**⚠️ Important :** La relation `INCLUDES_JS` peut pointer vers :
- Un nœud `:Js` pour un fichier JavaScript standard (.js)
- Un nœud `:Jsp` pour un fichier JSP dynamique (.jsp/.jspx/.jspf) qui génère du JavaScript

```cypher
# Exemple: Trouver tous les scripts inclus (JS ou JSP)
MATCH (jsp:Jsp)-[:INCLUDES_JS]->(target)
WHERE jsp.file_path CONTAINS 'index.jsp'
RETURN labels(target), target.file_path
```

**Cas d'usage JSP dynamique :**
```html
<!-- JSP incluant un fichier JSP comme script -->
<script src="/common/config.jsp"></script>
<script src="/fragments/translations.jspx"></script>
```

Ces fichiers `.jsp`/`.jspx` génèrent du JavaScript dynamiquement (configuration serveur, traductions, données contextuelles).

### INCLUDES_JSP (JSP → JSP)

```
INCLUDES_JSP {
  type: "static|dynamic"           # <%@ include file > (static) ou <jsp:include> (dynamic)
}
```

---

## 🔍 Patterns de requêtes essentiels

### Trouver une fonction/classe par nom

```cypher
MATCH (f:Function {name: "getUser"})
RETURN f.id, f.file_path, f.start_line

MATCH (c:Class {name: "UserService"})
RETURN c.id, c.file_path
```

### Analyser signature de fonction

```cypher
# Paramètres d'une fonction
MATCH (f:Function)-[:HAS_PARAM]->(p:Parameter)
WHERE f.name = "getUser"
RETURN p.name, p.node_type

# Type de retour
MATCH (f:Function)-[:RETURNS]->(t:Type)
WHERE f.name = "getUser"
RETURN t.name

# Variables utilisées
MATCH (f:Function)-[:USES]->(v:Variable)
WHERE f.name = "processData"
RETURN v.name, v.node_type
```

### Appels directs

```cypher
MATCH (caller:Function)-[:CALLS]->(callee:Function)
WHERE caller.name = "processData"
RETURN DISTINCT callee.name
```

### Appels transitifs (jusqu'à N niveaux)

```cypher
MATCH (start:Function)-[:CALLS*1..5]->(end:Function)
WHERE start.name = "getUser"
RETURN DISTINCT end.name, length(path) AS depth
```

### Méthodes d'une classe

```cypher
MATCH (c:Class)-[:DEFINES|CONTAINS]->(f:Function)
WHERE c.name = "UserService"
RETURN f.name, f.start_line
```

### Classe et ses parents/enfants (héritage)

```cypher
MATCH (child:Class)-[:EXTENDS]->(parent:Class)
WHERE child.name = "AdminService"
RETURN child.name, parent.name

MATCH (impl:Class)-[:IMPLEMENTS]->(iface:Interface)
WHERE iface.name = "IUserRepository"
RETURN DISTINCT impl.name
```

### Analyse JavaScript - Module et contenu

```cypher
MATCH (jsfile:JS)-[:CONTAINS]->(content:Class|Function)
WHERE jsfile.file_path CONTAINS "app.js"
RETURN content.name, labels(content) AS type

MATCH (jsfile:JS)-[:CONTAINS]->(f:Function)
WHERE jsfile.file_path CONTAINS "app.js"
AND NOT (()-[:CONTAINS]->(f))
RETURN f.name AS top_level_function
```

### Analyse JavaScript - Object Literals

```cypher
# Trouver les object literals et leurs méthodes
MATCH (obj:Module {object_literal: true})-[:DEFINES]->(method:Function)
RETURN obj.name AS object_name, method.name AS method_name

# Exemple : trouver toutes les méthodes d'un object literal spécifique
MATCH (obj:Module)-[:DEFINES]->(m:Function {object_method: true})
WHERE obj.name = "compasNotification"
RETURN m.name, m.file_path
```

### Analyse JSP - Inclusions et Imports Java

```cypher
# JSP avec inclusions JS/CSS/JSP
MATCH (jsp:Jsp)-[:INCLUDES_JS|INCLUDES_CSS|INCLUDES_JSP]->(target)
WHERE jsp.file_path CONTAINS "index"
RETURN jsp.file_path, type(last(relationships)), target

# Inclusions transitives (JSP → JSP → JSP...)
MATCH path = (jsp:Jsp)-[:INCLUDES_JSP*]->(child:Jsp)
WHERE jsp.file_path CONTAINS "layout"
RETURN path

# ⭐ NOUVEAU: Classes Java importées par une JSP
MATCH (jsp:Jsp)-[:IMPORTS]->(cls:Class)
WHERE jsp.file_path CONTAINS "gestion_ep"
RETURN jsp.file_path, cls.id, cls.metadata.qualified_name

# Tous les JSPF (fragments) utilisés par une page JSP (directs et transitifs)
MATCH path = (main:Jsp)-[:INCLUDES_JSP*1..10]->(jspf:Jsp)
WHERE main.file_path ENDS WITH 'main.jsp'
  AND jspf.file_path ENDS WITH '.jspf'
WITH DISTINCT jspf, min(length(path)) AS min_depth
RETURN jspf.file_path AS jspf_path, min_depth AS depth
ORDER BY min_depth, jspf_path
```

### WebSphere - Configuration

```cypher
# Servlets configurés avec leurs classes d'implémentation
MATCH (xml:WebXml)-[:CONFIGURES]->(servlet:Servlet)-[:IMPLEMENTED_BY]->(cls:Class)
RETURN servlet.name, servlet.metadata.`url-pattern`, cls.id, cls.file_path

# Trouver les servlets par url-pattern
MATCH (s:Servlet)-[:IMPLEMENTED_BY]->(c:Class)
WHERE s.metadata.`url-pattern` IN ['/ErreurSigma.srv', '/TarifPjServlet.srv']
RETURN s.metadata.`url-pattern` AS url, c.id AS implementing_class

# Portlet → Service → DAO chain
MATCH (portlet:Portlet)-[:IMPLEMENTED_BY]->(cls:Class)
MATCH (cls)-[:CONTAINS]->(method:Function)-[:CALLS]->(service:Function)-[:CALLS]->(dao:Function)
RETURN portlet.name, service.name, dao.name

# Portlet rend JSP
MATCH (portlet:Portlet)-[:RENDERS]->(jsp:Jsp)
RETURN portlet.name, jsp.file_path

# Filters par url-pattern
MATCH (filter:Filter)-[:IMPLEMENTED_BY]->(cls:Class)
WHERE filter.metadata.`url-pattern` = "/*"
RETURN filter.name, cls.id
```

### Trouver tous les nœuds d'un langage

```cypher
MATCH (n)
WHERE n.language = "java"
RETURN DISTINCT labels(n) AS type, COUNT(*) AS count

MATCH (f:Function)
WHERE f.language = "javascript"
RETURN COUNT(*)
```

### Dépendances et graphe d'appels complet

```cypher
MATCH (start:Function)
WHERE start.name = "main"
CALL apoc.path.expandConfig(start, {
  relationshipFilter: "CALLS>",
  maxLevel: 10
})
YIELD path
RETURN path

MATCH (caller:Function)-[:CALLS*1..5]->(callee:Function)
WHERE caller.name = "processData"
AND NOT EXISTS((callee)-[:CALLS]->())
RETURN DISTINCT callee.name AS leaf_function
```

### Localiser du code

```cypher
MATCH (n)
WHERE n.file_path CONTAINS "UserService.java"
RETURN n.name, n.node_type, n.start_line, n.end_line
ORDER BY n.start_line

MATCH (f:Function)
WHERE f.start_line >= 100 AND f.start_line <= 200
AND f.file_path CONTAINS "service"
RETURN f.name, f.file_path, f.start_line
```

---

## 📌 Règles importantes

1. **Tous les nœuds héritent de `:Node`** : utilise `MATCH (n:Node)` pour requête universelle
2. **Les IDs sont qualifiés** : `com.app.UserService.getUser` pour Java, `controllers::getUser` pour JS
3. **JS files = `:JS` nodes** : les inclusions JSP pointent sur `:JS` nodes, pas sur les fonctions/classes
4. **Métadonnées dans les relations** : Utilise `apoc.get()` ou accès direct si métadonnées stockées
5. **CONTAINS pour la hiérarchie** : Class/JS/Module CONTAINS Function/Class
6. **CALLS pour les appels** : Toujours Function CALLS Function (pas de nœuds intermédiaires)

---

## 🚀 Optimisations Cypher

```cypher
-- Utiliser WHERE avec index plutôt que MATCH
MATCH (n:Function) WHERE n.name = "getUser"
-- Au lieu de
MATCH (n:Function {name: "getUser"})

-- Limiter les résultats
MATCH (...) RETURN ... LIMIT 100

-- Compter avant de retourner
MATCH (c:Class)-[:CONTAINS]->(f:Function)
WHERE c.name = "UserService"
RETURN COUNT(f) AS method_count

-- Distinct pour éviter doublons
MATCH (f:Function)-[:CALLS*]->(g:Function)
RETURN DISTINCT g.name

-- EXISTS pour vérifications rapides
MATCH (c:Class)
WHERE EXISTS((c)-[:CONTAINS]->())
RETURN c.name
```

---

## 🔑 Cas courants pour agents IA

| Objectif | Pattern |
|----------|---------|
| Tracer l'appel d'une fonction | `MATCH (start:Function)-[:CALLS*1..N]->(end:Function) WHERE start.name = "X"` |
| Trouver qui appelle une fonction | `MATCH (caller:Function)-[:CALLS*]->(target:Function) WHERE target.name = "X"` |
| Analyser une classe | `MATCH (c:Class {name: "X"})-[:CONTAINS\|DEFINES]->(f:Function)` |
| Signature de fonction | `MATCH (f:Function)-[:HAS_PARAM\|RETURNS]->(p) WHERE f.name = "X"` |
| Analyser un fichier JS | `MATCH (js:JS)-[:CONTAINS]->(content) WHERE js.file_path CONTAINS "X"` |
| Analyser une JSP | `MATCH (jsp:Jsp)-[:INCLUDES_JS\|INCLUDES_JSP\|INCLUDES_CSS]->(target) RETURN labels(target), target.file_path` |
| ⭐ Classes importées par JSP | `MATCH (jsp:Jsp)-[:IMPORTS]->(cls:Class) WHERE jsp.file_path CONTAINS "gestion_ep"` |
| ⭐ JSPF inclus transitifs | `MATCH path = (jsp:Jsp)-[:INCLUDES_JSP*]->(jspf:Jsp) WHERE jspf.file_path ENDS WITH '.jspf'` |
| Analyser un Servlet | `MATCH (xml:WebXml)-[:CONFIGURES\|DECLARES]->(servlet:Servlet)` |
| Chain Portlet → Service → DAO | `MATCH (p:Portlet)-[c1:CALLS]->(s)-[c2:CALLS]->(d) WHERE c1.call_type="service" AND c2.call_type="dao"` |
| Appels AJAX | `MATCH ()-[c:CALservlet:Servlet)-[:IMPLEMENTED_BY]->(cls:Class) WHERE servlet.metadata.\`url-pattern\` = '/user/*'` |
| Analyser un Portlet | `MATCH (portlet:Portlet)-[:IMPLEMENTED_BY]->(cls:Class)` |
| Chain Portlet → Service → DAO | `MATCH (p:Portlet)-[:IMPLEMENTED_BY]->(c:Class)-[:CONTAINS]->(m)-[:CALLS]->(s)-[:CALLS]->(d)
| Filtres de Servlet | `MATCH (filter:Filter)-[:FILTERS]->(servlet:Servlet)` |
| Impact analysis | `MATCH (n:Function)-[:CALLS*1..N]->(target) WHERE n.name = "X" RETURN COUNT(*)` |
| Coverage | `MATCH (f:Function) WHERE f.language = "java" RETURN COUNT(*)` |

---

## 📊 Résumé des labels par catégorie

**Code Java :** `:Function`, `:Class`, `:Interface`, `:Package`, `:Variable`, `:Parameter`  
**Code JavaScript :** `:Function`, `:Class`, `:JS`, `:Import`, `:Variable`  
**WebSphere :** `:Servlet`, `:Portlet`, `:Filter`, `:WebXml`, `:PortletXml`  
**Web :** `:Jsp`, `:JS` (fichiers)  
**Types :** `:Type`, `:Trait`  

**Note importante sur les Servlets/Portlets/Filters :**
- `metadata.class` : FQN de la classe Java d'implémentation
- `metadata.url-pattern` : Pattern d'URL (ex: `/user/*`, `*.srv`, `/*`)
- Relation `IMPLEMENTED_BY` : Pointe vers le nœud `:Class` Java correspondant

**Autres :** `:Expression`, `:Operator`, `:Module`

---

## 🔗 Résumé des relations par type

**Appels :** `CALLS` (avec call_type: standard|ajax|service|dao), `TRIGGERS`  
**Structure :** `DEFINES`, `CONIMPLEMENTED_BY, `RETURNS`  
**Héritage :** `EXTENDS`, `IMPLEMENTS`, `IMPLEMENTED_BY`  
**Dépendances :** `IMPORTS`, `USES`, `REFERENCES`, `DEPENDS_ON`  
**Types :** `HAS_TYPE`, `DECLARE_TYPE`  
**Assignation :** `ASSIGNED_BY`, `EXPORTS`  
**WebSphere :** `CONFIGURES`, `DECLARES`, `FILTERS`, `RENDERS`, `NOTIFIES`  
**JSP :** `INCLUDES_JS`, `INCLUDES_CSS`, `INCLUDES_JSP`, `BINDS_DATA`  
**DOM :** `TARGETS_ELEMENT`

---

*Référence rapide pour agents IA - Mise à jour: January 19, 2026*
