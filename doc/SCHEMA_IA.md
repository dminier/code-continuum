# SCHEMA_IA.md - Cypher Reference for AI Agents

**Essential Neo4j schema for building Cypher queries.**

---

## Node Labels

```
:Function     - Functions/Methods (object_method: true if JS object literal method)
:Class        - Classes / Interfaces / Types
:Interface    - Interfaces (also carry :Class label)
:Variable     - Variables / Parameters
:Parameter    - Function parameters
:Import       - Imports / requires
:Package      - Java packages
:Module       - Modules / Namespaces (object_literal: true if JS object literal)
:Type         - Custom types
:Trait        - Traits / Mixins (Rust)
:Expression   - Expressions
:Operator     - Operators
:JS           - JavaScript files (file/module anchor)
:Jsp          - JSP / JSPX / JSPF files
:WebXml       - web.xml file
:PortletXml   - portlet.xml file
:Servlet      - Declared servlets (web.xml) ⚠️ Represents the NAME, not the class
:Portlet      - Declared portlets (portlet.xml) ⚠️ Represents the NAME, not the class
:Filter       - Declared filters (web.xml) ⚠️ Represents the NAME, not the class
```

### ⚠️ Name vs Class for backend components

`:Portlet`, `:Servlet` and `:Filter` nodes represent the **name** of the component as declared in configuration files (`portlet.xml`, `web.xml`), **not the implementation class**.

| Node | `name` property | Example | Link to class |
|------|-----------------|---------|---------------|
| `:Portlet` | `<portlet-name>` | `GestionEP` | `IMPLEMENTED_BY` → `:Class` |
| `:Servlet` | `<servlet-name>` | `SearchServlet` | `IMPLEMENTED_BY` → `:Class` |
| `:Filter` | `<filter-name>` | `AuthFilter` | `IMPLEMENTED_BY` → `:Class` |

---

## Main Relations

```
# Call relations
CALLS          Function → Function      (function call)
               Metadata: call_type = "standard|ajax|service|dao"
                         method, async, operation, etc.
TRIGGERS       Function → Node          (trigger)

# Structure relations
CONTAINS       Class/JS/Module → Function/Class  (containment)
DEFINES        Class/Module → Function            (method definition, incl. JS object literals)
HAS_PARAM      Function → Parameter              (parameters)
RETURNS        Function → Type                    (return type)

# Dependency relations
USES           Function → Variable      (variable usage)
IMPORTS        Function/Class → Import  (import/require)
REFERENCES     Node → Definition        (reference)
DEPENDS_ON     JS → JS                  (JS dependency)

# Type relations
HAS_TYPE       Node → Type              (typing)
DECLARE_TYPE   Node → Type              (type declaration)
EXTENDS        Class → Class            (inheritance)
IMPLEMENTS     Class → Interface        (implementation)
IMPLEMENTED_BY Interface → Class        (inverse of IMPLEMENTS)

# Assignment and visibility
ASSIGNED_BY    Variable → Expression    (assignment)
EXPORTS        Module → Symbol          (export)

# WebSphere relations
CONFIGURES     WebXml/PortletXml → Servlet/Portlet/Filter
IMPLEMENTED_BY Servlet/Portlet/Filter → Class    (Java implementation)
RENDERS        Portlet → JSP            (JSP rendering)
NOTIFIES       Service → Service        (notification)

# JSP relations
INCLUDES_JS    Jsp → Js or Jsp          (JS inclusion: .js file OR .jsp/.jspx generating dynamic JS)
INCLUDES_CSS   Jsp → (CSS file)         (CSS inclusion)
INCLUDES_JSP   Jsp → Jsp                (JSP inclusion)
IMPORTS        Jsp → Class              (Java imports: <%@page import="..."%>)
BINDS_DATA     JSP ↔ JS                 (data sharing)

# DOM relations
TARGETS_ELEMENT JS → DOM               (DOM targeting)
```

---

## Key Node Properties

**Always present:**
- `id` : Unique qualified identifier
- `name` : Short name
- `node_type` : Type (e.g. "Function", "Class", "JS")
- `language` : "java", "javascript", "jsp", "xml", etc.
- `file_path` : Source file path
- `start_line`, `end_line` : Position in the file

**Optional by type:**
- `package` : For Java nodes
- `class` : For methods
- `module` : For JavaScript nodes
- `metadata` : Dictionary for extra properties
  - `metadata.class` : For Servlet/Portlet/Filter (FQN of the Java class)
  - `metadata.url-pattern` : For Servlet/Filter (URL pattern, e.g. `/user/*`, `*.srv`)
  - `metadata.qualified_name` : For Class (fully qualified name)
  - `metadata.external` : Boolean (true if external/unanalyzed class)
  - `metadata.import_type` : For IMPORTS relation (e.g. "java")

---

## Relation Properties

### IMPORTS (JSP → Class or Class → Class)

```
IMPORTS {
  import_type: "java"              # Import type
  qualified_name: String           # Fully qualified name of the imported class
}
```

```cypher
MATCH (jsp:Jsp)-[r:IMPORTS]->(cls:Class)
RETURN jsp.file_path, r.qualified_name, cls.id
```

### INCLUDES_JS (JSP → Js or Jsp)

`INCLUDES_JS` can point to:
- A `:Js` node for a standard JavaScript file (.js)
- A `:Jsp` node for a dynamic JSP file (.jsp/.jspx/.jspf) that generates JavaScript

```cypher
MATCH (jsp:Jsp)-[:INCLUDES_JS]->(target)
WHERE jsp.file_path CONTAINS 'index.jsp'
RETURN labels(target), target.file_path
```

### INCLUDES_JSP (JSP → JSP)

```
INCLUDES_JSP {
  type: "static|dynamic"   # <%@ include file > (static) or <jsp:include> (dynamic)
}
```

---

## Essential Query Patterns

### Find a function/class by name

```cypher
MATCH (f:Function {name: "getUser"})
RETURN f.id, f.file_path, f.start_line

MATCH (c:Class {name: "UserService"})
RETURN c.id, c.file_path
```

### Analyze a function signature

```cypher
MATCH (f:Function)-[:HAS_PARAM]->(p:Parameter)
WHERE f.name = "getUser"
RETURN p.name, p.node_type

MATCH (f:Function)-[:RETURNS]->(t:Type)
WHERE f.name = "getUser"
RETURN t.name

MATCH (f:Function)-[:USES]->(v:Variable)
WHERE f.name = "processData"
RETURN v.name, v.node_type
```

### Direct calls

```cypher
MATCH (caller:Function)-[:CALLS]->(callee:Function)
WHERE caller.name = "processData"
RETURN DISTINCT callee.name
```

### Transitive calls (up to N levels)

```cypher
MATCH (start:Function)-[:CALLS*1..5]->(end:Function)
WHERE start.name = "getUser"
RETURN DISTINCT end.name, length(path) AS depth
```

### Methods of a class

```cypher
MATCH (c:Class)-[:DEFINES|CONTAINS]->(f:Function)
WHERE c.name = "UserService"
RETURN f.name, f.start_line
```

### Class hierarchy

```cypher
MATCH (child:Class)-[:EXTENDS]->(parent:Class)
WHERE child.name = "AdminService"
RETURN child.name, parent.name

MATCH (impl:Class)-[:IMPLEMENTS]->(iface:Interface)
WHERE iface.name = "IUserRepository"
RETURN DISTINCT impl.name
```

### JavaScript - Module contents

```cypher
MATCH (jsfile:JS)-[:CONTAINS]->(content)
WHERE jsfile.file_path CONTAINS "app.js"
RETURN content.name, labels(content) AS type

MATCH (jsfile:JS)-[:CONTAINS]->(f:Function)
WHERE jsfile.file_path CONTAINS "app.js"
AND NOT (()-[:CONTAINS]->(f))
RETURN f.name AS top_level_function
```

### JavaScript - Object Literals

```cypher
MATCH (obj:Module {object_literal: true})-[:DEFINES]->(method:Function)
RETURN obj.name AS object_name, method.name AS method_name

MATCH (obj:Module)-[:DEFINES]->(m:Function {object_method: true})
WHERE obj.name = "compasNotification"
RETURN m.name, m.file_path
```

### JSP - Inclusions and Java imports

```cypher
MATCH (jsp:Jsp)-[:INCLUDES_JS|INCLUDES_CSS|INCLUDES_JSP]->(target)
WHERE jsp.file_path CONTAINS "index"
RETURN jsp.file_path, type(last(relationships(path))), target

MATCH path = (jsp:Jsp)-[:INCLUDES_JSP*]->(child:Jsp)
WHERE jsp.file_path CONTAINS "layout"
RETURN path

MATCH (jsp:Jsp)-[:IMPORTS]->(cls:Class)
WHERE jsp.file_path CONTAINS "gestion_ep"
RETURN jsp.file_path, cls.id, cls.metadata.qualified_name

MATCH path = (main:Jsp)-[:INCLUDES_JSP*1..10]->(jspf:Jsp)
WHERE main.file_path ENDS WITH 'main.jsp'
  AND jspf.file_path ENDS WITH '.jspf'
WITH DISTINCT jspf, min(length(path)) AS min_depth
RETURN jspf.file_path AS jspf_path, min_depth AS depth
ORDER BY min_depth, jspf_path
```

### WebSphere - Configuration

```cypher
MATCH (xml:WebXml)-[:CONFIGURES]->(servlet:Servlet)-[:IMPLEMENTED_BY]->(cls:Class)
RETURN servlet.name, servlet.metadata.`url-pattern`, cls.id, cls.file_path

MATCH (s:Servlet)-[:IMPLEMENTED_BY]->(c:Class)
WHERE s.metadata.`url-pattern` IN ['/ErreurSigma.srv', '/TarifPjServlet.srv']
RETURN s.metadata.`url-pattern` AS url, c.id AS implementing_class

MATCH (portlet:Portlet)-[:IMPLEMENTED_BY]->(cls:Class)
MATCH (cls)-[:CONTAINS]->(method:Function)-[:CALLS]->(service:Function)-[:CALLS]->(dao:Function)
RETURN portlet.name, service.name, dao.name

MATCH (portlet:Portlet)-[:RENDERS]->(jsp:Jsp)
RETURN portlet.name, jsp.file_path

MATCH (filter:Filter)-[:IMPLEMENTED_BY]->(cls:Class)
WHERE filter.metadata.`url-pattern` = "/*"
RETURN filter.name, cls.id
```

### All nodes for a given language

```cypher
MATCH (n)
WHERE n.language = "java"
RETURN DISTINCT labels(n) AS type, COUNT(*) AS count

MATCH (f:Function)
WHERE f.language = "javascript"
RETURN COUNT(*)
```

### Full call graph

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

### Locate code

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

### Rust — Modules, structs, enums, traits

```cypher
// Modules Rust (fichiers analysés)
MATCH (m:Module) WHERE m.language = 'rust' RETURN m.name, m.file_path

// Structs (node_type='Class', metadata.kind='struct')
MATCH (c:Class) WHERE c.language = 'rust' RETURN c.name, c.file_path

// Enums (node_type='Type', metadata.kind='enum')
MATCH (t:Type) WHERE t.language = 'rust' RETURN t.name, t.file_path

// Traits
MATCH (tr:Trait) WHERE tr.language = 'rust' RETURN tr.name, tr.file_path

// Méthodes d'un impl block
MATCH (f:Function)
WHERE f.language = 'rust' AND f.metadata.struct IS NOT NULL
RETURN f.name, f.metadata.struct AS struct_name, f.file_path

// Call graph Rust
MATCH (caller:Function)-[:CALLS]->(callee:Function)
WHERE caller.language = 'rust'
RETURN caller.name, callee.name, caller.file_path

// Contenu d'un module (fichier)
MATCH (m:Module)-[:CONTAINS]->(child)
WHERE m.language = 'rust'
RETURN m.name, labels(child) AS kind, child.name
```

---

## Important Rules

1. **All nodes inherit `:Node`** — use `MATCH (n:Node)` for universal queries
2. **IDs are qualified** — `com.app.UserService.getUser` for Java, `controllers::getUser` for JS
3. **JS files = `:JS` nodes** — JSP inclusions point to `:JS` nodes, not functions/classes
4. **Metadata on relations** — use direct property access or `apoc.get()`
5. **CONTAINS for hierarchy** — Class/JS/Module CONTAINS Function/Class
6. **CALLS for calls** — always Function CALLS Function (no intermediate nodes)

---

## Cypher Optimizations

```cypher
-- Use WHERE with index instead of inline properties
MATCH (n:Function) WHERE n.name = "getUser"

-- Limit results
MATCH (...) RETURN ... LIMIT 100

-- Count before returning
MATCH (c:Class)-[:CONTAINS]->(f:Function)
WHERE c.name = "UserService"
RETURN COUNT(f) AS method_count

-- DISTINCT to avoid duplicates
MATCH (f:Function)-[:CALLS*]->(g:Function)
RETURN DISTINCT g.name

-- EXISTS for fast checks
MATCH (c:Class)
WHERE EXISTS((c)-[:CONTAINS]->())
RETURN c.name
```

---

## Common Patterns for AI Agents

| Goal | Pattern |
|------|---------|
| Trace function calls | `MATCH (start:Function)-[:CALLS*1..N]->(end:Function) WHERE start.name = "X"` |
| Find callers of a function | `MATCH (caller:Function)-[:CALLS*]->(target:Function) WHERE target.name = "X"` |
| Analyze a class | `MATCH (c:Class {name: "X"})-[:CONTAINS\|DEFINES]->(f:Function)` |
| Function signature | `MATCH (f:Function)-[:HAS_PARAM\|RETURNS]->(p) WHERE f.name = "X"` |
| Analyze a JS file | `MATCH (js:JS)-[:CONTAINS]->(content) WHERE js.file_path CONTAINS "X"` |
| Analyze a JSP | `MATCH (jsp:Jsp)-[:INCLUDES_JS\|INCLUDES_JSP\|INCLUDES_CSS]->(target) RETURN labels(target), target.file_path` |
| Java classes imported by JSP | `MATCH (jsp:Jsp)-[:IMPORTS]->(cls:Class) WHERE jsp.file_path CONTAINS "X"` |
| Transitive JSPF includes | `MATCH path = (jsp:Jsp)-[:INCLUDES_JSP*]->(jspf:Jsp) WHERE jspf.file_path ENDS WITH '.jspf'` |
| Analyze a Servlet | `MATCH (xml:WebXml)-[:CONFIGURES]->(servlet:Servlet)-[:IMPLEMENTED_BY]->(cls:Class)` |
| Portlet → Service → DAO chain | `MATCH (p:Portlet)-[:IMPLEMENTED_BY]->(c:Class)-[:CONTAINS]->(m)-[:CALLS]->(s)-[:CALLS]->(d)` |
| AJAX calls | `MATCH ()-[c:CALLS]->() WHERE c.call_type = "ajax"` |
| Analyze a Portlet | `MATCH (portlet:Portlet)-[:IMPLEMENTED_BY]->(cls:Class)` |
| Servlet by url-pattern | `MATCH (s:Servlet)-[:IMPLEMENTED_BY]->(c:Class) WHERE s.metadata.\`url-pattern\` = '/user/*'` |
| Impact analysis | `MATCH (n:Function)-[:CALLS*1..N]->(target) WHERE n.name = "X" RETURN COUNT(*)` |
| Coverage | `MATCH (f:Function) WHERE f.language = "java" RETURN COUNT(*)` |
| Analyser un module Rust | `MATCH (m:Module)-[:CONTAINS]->(child) WHERE m.language='rust' AND m.file_path CONTAINS "X" RETURN labels(child), child.name` |
| Méthodes d'une struct Rust | `MATCH (f:Function) WHERE f.language='rust' AND f.metadata.struct = "StructName" RETURN f.name, f.file_path` |
| Call graph Rust depuis entrée | `MATCH path=(start:Function {name:"X"})-[:CALLS*1..5]->(end:Function) WHERE start.language='rust' RETURN DISTINCT end.name, length(path) AS depth` |
| Tous les enums Rust | `MATCH (t:Type) WHERE t.language='rust' RETURN t.name, t.file_path` |
| Imports Rust d'un fichier | `MATCH (i:Import) WHERE i.language='rust' AND i.file_path CONTAINS "X" RETURN i.name` |

---

## MCP Tools API Reference

Available through `code-continuum` MCP server (port 8001).

### `list_projects()`

Lists all available projects (subdirectories) under CODE_PATH (`/app/data`).

**Use this first to discover available projects.**

**Parameters:** None

**Returns:** List of project names relative to CODE_PATH

**Example response:**
```
Available projects:
  backend/java
  backend/javascript
  frontend/javascript
  web_templates
```

### `add_project(project_path, [project_name], [include_packages], [clear_project])`

Analyzes a project and adds its nodes/relations to Neo4j. Does not clear other projects.

**All paths are relative to CODE_PATH (`/app/data`).**

| Parameter | Type | Required | Description |
|---|---|---|---|
| `project_path` | string | ✅ | Path relative to CODE_PATH (e.g., `backend/java`, `my-app`) |
| `project_name` | string | | Friendly name (defaults to last path segment) |
| `include_packages` | string | | CSV filter: only index matching packages (e.g., `com.example,org.myapp`) |
| `clear_project` | boolean | | Delete existing project data before re-indexing (default: false) |

**Example calls:**
```
add_project(project_path = "backend/java")
add_project(project_path = "backend/java", project_name = "my-backend", include_packages = "com.example,org.myapp")
add_project(project_path = "frontend/javascript", clear_project = true)
```

### `remove_project(project_path)`

Deletes all nodes and relations belonging to a project.

**The path must be relative to CODE_PATH and match the `project_path` used when adding.**

| Parameter | Type | Required | Description |
|---|---|---|---|
| `project_path` | string | ✅ | Path relative to CODE_PATH (must match add_project call) |

**Example call:**
```
remove_project(project_path = "backend/java")
```

---

## MCP Tools + Neo4j Cypher Workflow

**Typical agent workflow:**

1. Call `list_projects()` to discover available projects
2. Call `add_project(project_path = "...")` to analyze a project
3. Execute Cypher queries (via `neo4j` MCP tool) to explore the graph
4. Call `remove_project(project_path = "...")` when done

**Important:** All project paths are relative to `/app/data`. The `code-continuum` MCP server automatically resolves them.

---

## Labels by Category

**Java:** `:Function`, `:Class`, `:Interface`, `:Package`, `:Variable`, `:Parameter`
**JavaScript:** `:Function`, `:Class`, `:JS`, `:Import`, `:Variable`
**Rust:** `:Function`, `:Class`, `:Type`, `:Trait`, `:Module`, `:Import`
**WebSphere:** `:Servlet`, `:Portlet`, `:Filter`, `:WebXml`, `:PortletXml`
**Web:** `:Jsp`, `:JS`
**Types:** `:Type`, `:Trait`
**Other:** `:Expression`, `:Operator`, `:Module`

**Note on Servlets/Portlets/Filters:**
- `metadata.class` : FQN of the Java implementation class
- `metadata.url-pattern` : URL pattern (e.g. `/user/*`, `*.srv`, `/*`)
- `IMPLEMENTED_BY` relation: points to the corresponding Java `:Class` node

---

## Relations Summary

**Calls:** `CALLS` (with call_type: standard|ajax|service|dao), `TRIGGERS`
**Structure:** `DEFINES`, `CONTAINS`, `HAS_PARAM`, `RETURNS`
**Inheritance:** `EXTENDS`, `IMPLEMENTS`, `IMPLEMENTED_BY`
**Dependencies:** `IMPORTS`, `USES`, `REFERENCES`, `DEPENDS_ON`
**Types:** `HAS_TYPE`, `DECLARE_TYPE`
**Assignment:** `ASSIGNED_BY`, `EXPORTS`
**WebSphere:** `CONFIGURES`, `DECLARES`, `FILTERS`, `RENDERS`, `NOTIFIES`
**JSP:** `INCLUDES_JS`, `INCLUDES_CSS`, `INCLUDES_JSP`, `BINDS_DATA`
**DOM:** `TARGETS_ELEMENT`

---

*Quick reference for AI agents*
