# Modèle Sémantique Enrichi - WebSphere Portal

## Vue d'Ensemble

Ce document propose un schéma enrichi des relations sémantiques entre les composants d'une portale WebSphere et les solutions de parsing automatique pour les identifier.

---

## 1. RELATION SÉMANTIQUE: Portlet → JSP (Dispatch/Render)

### Description
Un portlet rend son interface en envoyant la requête vers une JSP.

### Exemple Code
```java
// UserManagementPortlet.doView()
dispatch("/WEB-INF/portlets/user-list.jsp")
```

### Solution de Parsing

**Regex (Generic, rapide)**
```regex
dispatch\s*\(\s*["\']([^"\']+\.jsp[f]?)["\']
```

**Tree-Sitter (Précis, complet)**
```typescript
{
  type: "call_expression",
  function: { name: "dispatch" },
  arguments: [
    { type: "string_literal", value: "/WEB-INF/portlets/*.jsp" }
  ]
}
```

### Instruction pour LLM
```
TÂCHE: Identifier dispatch() appels vers JSP
STRATÉGIE: 
  1. Cherchez appels de méthode "dispatch"
  2. Extrayez le chemin JSP du premier argument (string literal)
  3. Créez nœud JSP avec ce chemin
  4. Créez relation RENDERS: Portlet -[RENDERS]-> JSP
  5. Annotez: {method: "doView|doEdit|doHelp", mode: "view|edit|help"}

VALIDATION:
  - Chemin commence par /WEB-INF/portlets/ ✓
  - Fichier finit par .jsp ou .jspf ✓
  - Portlet et JSP existent ✓
```

### Node Type Neo4j
```cypher
MATCH (portlet:Portlet {name: "UserManagementPortlet"})
CREATE (jsp:JSP {path: "/WEB-INF/portlets/user-list.jsp", name: "user-list"})
CREATE (portlet)-[:RENDERS {mode: "view"}]->(jsp)
```

---

## 2. RELATION SÉMANTIQUE: JSP → JavaScript (Script Include)

### Description
Une JSP inclut ou appelle des fichiers JavaScript pour la logique client.

### Exemple Code
```jsp
<script src="/resources/js/user-form-validator.js"></script>
<%@ include file="/WEB-INF/portlets/common/utils.jspf" %>
```

### Solution de Parsing

**Regex (HTML/JSP)**
```regex
<script\s+(?:type="[^"]*"\s+)?src=["\']([^"\']+\.js)["\']
<link\s+rel="stylesheet"\s+href=["\']([^"\']+\.css)["\']
<%\s*@\s*include\s+file=["\']([^"\']+\.jsp[f]?)["\']
<jsp:include\s+page=["\']([^"\']+)["\']
```

**Tree-Sitter (HTML/JSP)**
```typescript
{
  type: "script_element" | "link_element",
  attributes: [{ name: "src" | "href", value: "*.js|*.css" }]
}
```

### Instruction pour LLM
```
TÂCHE: Identifier includes (JS, CSS, JSP)
STRATÉGIE:
  1. Parsingez chaque ligne JSP
  2. Cherchez: <script src="...">
  3. Cherchez: <link rel="stylesheet" href="...">
  4. Cherchez: <%@ include file="..."%>
  5. Cherchez: <jsp:include page="..."/>
  6. Pour chaque trouvé:
     - Créez nœud File (JavaScript/CSS/JSP)
     - Créez relation INCLUDES_JS/INCLUDES_CSS/INCLUDES_JSP
     - Annotez: {type: "script|stylesheet|fragment", inline: false}

VALIDATION:
  - Paths sont valides ✓
  - Fichiers existent (ou seront créés) ✓
  - Pas de doublons d'includes ✓
```

### Node Type Neo4j
```cypher
MATCH (jsp:JSP {path: "/WEB-INF/portlets/user-list.jsp"})
CREATE (js:JavaScript {path: "/resources/js/user-form-validator.js"})
CREATE (css:CSS {path: "/resources/css/user-form.css"})
CREATE (jsp)-[:INCLUDES_JS {position: 1, async: false}]->(js)
CREATE (jsp)-[:INCLUDES_CSS {position: 0, media: "screen"}]->(css)
```

---

## 3. RELATION SÉMANTIQUE: JavaScript → Servlet (AJAX Call)

### Description
Un fichier JavaScript effectue des appels AJAX vers des Servlets/Portlets Java.

### Exemple Code
```javascript
$.ajax({
  url: "/portal/api/users/validate-email",
  type: "POST",
  data: { email: emailValue }
})
```

### Solution de Parsing

**Regex (JavaScript - Phase 1)**
```regex
\$\.ajax\s*\(\s*\{[^}]*url\s*:\s*["\']([^"\']+)["\']
\$\.(get|post)\s*\(\s*["\']([^"\']+)["\']
fetch\s*\(\s*["\']([^"\']+)["\']
window\.location\s*=\s*["\']([^"\']+)["\']
```

**Tree-Sitter (JavaScript - Phase 2, précis)**
```typescript
{
  type: "call_expression",
  function: {
    type: "member_expression",
    object: { name: "$|axios|fetch" },
    property: { name: "ajax|get|post" }
  },
  arguments: [{
    type: "object",
    properties: [{
      key: { name: "url" },
      value: { type: "string", value: "^/portal/.*" }
    }]
  }]
}
```

### Instruction pour LLM
```
TÂCHE: Identifier AJAX calls vers serveurs
STRATÉGIE:
  1. PHASE 1 - Regex rapide: Trouvez tous les $.ajax/$.get/$.post/fetch
  2. PHASE 2 - Tree-Sitter: Extrayez précisément l'URL, méthode, données
  3. PHASE 3 - Résolution:
     - Parsez l'URL cible
     - Résolvez vers Servlet ou Portlet (basé sur routing pattern)
       * "/portal/api/users/validate" → ValidationServlet
       * "/portal/document?action=upload" → DocumentPortlet
       * "/portal/workflow?action=approve" → WorkflowPortlet.processAction()
     - Si ambigüité: recherchez dans web.xml pour servlet-mapping
  4. Créez relation CALLS_AJAX avec annotations:
     - {method: "GET|POST|PUT|DELETE", async: true, dataType: "json|html"}
     - {line: linenumber, column: columnnumber}

VALIDATION:
  - URL comporte /portal ou /api ✓
  - Méthode HTTP valide ✓
  - Endpoint existe dans web.xml ou portlet.xml ✓

DIFFICULTÉS:
  - URLs dynamiques: try regex patterns
  - URLs avec paramètres: extrayez base URL
  - URLs en variables: suivez la variable
```

### Node Type Neo4j
```cypher
MATCH (js:JavaScript {path: "/resources/js/user-form-validator.js"})
CREATE (servlet:Servlet {name: "ValidationServlet", path: "/portal/api/users/validate-email"})
CREATE (js)-[:CALLS_AJAX {method: "POST", async: true, line: 42, content_type: "application/json"}]->(servlet)
```

---

## 4. RELATION SÉMANTIQUE: Portlet → Service (Service Call)

### Description
Un Portlet appelle des services métier pour la logique applicative.

### Exemple Code
```java
private UserService userService;

public void init(PortletConfig config) {
  this.userService = new UserService();
}

protected void doView(RenderRequest request, RenderResponse response) {
  List<User> users = userService.getAllUsers(10);
}
```

### Solution de Parsing

**Regex (Java - Field Declaration)**
```regex
private\s+(\w+Service)\s+(\w+)\s*[=;]
@Autowired\s+(?:.*\n)*?\s*(\w+Service)\s+(\w+)
```

**Regex (Java - Method Call)**
```regex
(\w+Service)\.\s*(\w+)\s*\(\s*([^)]*)\s*\)
this\.(\w+Service)\.\s*(\w+)\s*\(\s*([^)]*)\s*\)
```

**Tree-Sitter (Java)**
```typescript
{
  type: "method_declaration",
  name: "doView|processAction|doEdit",
  body: {
    contains: {
      type: "method_invocation",
      object: { type: "identifier", name: "*Service" },
      method: { name: "*" }
    }
  }
}
```

### Instruction pour LLM
```
TÂCHE: Identifier Service calls depuis Portlets
STRATÉGIE:
  1. Cherchez déclarations de champs Service dans Portlet
  2. Cherchez instantiation: new UserService()
  3. Cherchez appels de méthode sur ces services
  4. Pour chaque appel:
     - Extrayez: ServiceName.methodName(params)
     - Créez/trouvez le nœud Service
     - Créez relation CALLS_SERVICE
     - Annotez: {method: "methodName", paramCount: N, returnType: "..."}

EXEMPLE complet:
  Source: UserManagementPortlet.doView()
  Appel: userService.getAllUsers(10)
  Relation: UserManagementPortlet -[CALLS_SERVICE]-> UserService
  Annotation: {method: "getAllUsers", params: ["int"], returns: "List<User>"}

VALIDATION:
  - Service existe (classe ou sera créée) ✓
  - Méthode est publique ✓
  - Paramètres correspondent ✓
```

### Node Type Neo4j
```cypher
MATCH (portlet:Portlet {name: "UserManagementPortlet"})
MATCH (service:Service {name: "UserService"})
CREATE (portlet)-[:CALLS_SERVICE {method: "getAllUsers", paramTypes: ["int"], returns: "List<User>", line: 45}]->(service)
```

---

## 5. RELATION SÉMANTIQUE: Service → DAO (Data Access)

### Description
Un Service appelle une couche d'accès aux données (DatabaseConnector, FileStorageManager).

### Exemple Code
```java
public void saveUser(User user) {
  userDatabase.put(user.getId(), user);
  dbConnector.saveUser(user);  // ← DAO call
}
```

### Solution de Parsing

**Regex (Java)**
```regex
(\w+(?:Connector|Manager|Repository|DAO))\s*\.\s*(\w+)\s*\(\s*([^)]*)\s*\)
```

**Tree-Sitter (Java)**
```typescript
{
  type: "method_invocation",
  object: { name: "*Connector|*Manager|*Repository|*DAO" },
  method: { name: "save|update|delete|query|get|find" }
}
```

### Instruction pour LLM
```
TÂCHE: Identifier DAO/Connector calls depuis Services
STRATÉGIE:
  1. Cherchez classes terminant par: Connector, Manager, Repository, DAO
  2. Cherchez appels vers ces classes
  3. Catégorisez l'opération:
     - save*/create*/insert* → SAVE
     - update* → UPDATE
     - delete*/remove* → DELETE
     - get*/find*/query/select* → QUERY
  4. Créez relation CALLS_DAO avec annotation d'opération
  5. Si possible, extrayez l'entité (User, Document, etc.)

EXEMPLE:
  Service: UserService
  Appel: dbConnector.saveUser(user)
  Relation: UserService -[CALLS_DAO]-> DatabaseConnector
  Annotation: {operation: "SAVE", entity: "User", method: "saveUser"}

VALIDATION:
  - DAO classe identifiée ✓
  - Opération correctement catégorisée ✓
  - Entité déterminée si possible ✓
```

### Node Type Neo4j
```cypher
MATCH (service:Service {name: "UserService"})
CREATE (dao:DAO {name: "DatabaseConnector"})
CREATE (service)-[:CALLS_DAO {operation: "SAVE", entity: "User", method: "saveUser", line: 52}]->(dao)
```

---

## 6. RELATION SÉMANTIQUE: web.xml → Servlet (Declaration)

### Description
Le fichier web.xml déclare les Servlets et mappe les URLs.

### Exemple Code
```xml
<servlet>
  <servlet-name>DispatcherServlet</servlet-name>
  <servlet-class>com.example.web.DispatcherServlet</servlet-class>
</servlet>
<servlet-mapping>
  <servlet-name>DispatcherServlet</servlet-name>
  <url-pattern>*.do</url-pattern>
</servlet-mapping>
```

### Solution de Parsing

**Regex (XML)**
```regex
<servlet>\s*<servlet-name>\s*([^<]+)\s*</servlet-name>\s*.*?\s*<servlet-class>\s*([^<]+)\s*</servlet-class>
<servlet-mapping>\s*<servlet-name>\s*([^<]+)\s*</servlet-name>\s*.*?\s*<url-pattern>\s*([^<]+)\s*</url-pattern>
<filter>\s*<filter-name>\s*([^<]+)\s*</filter-name>\s*.*?\s*<filter-class>\s*([^<]+)\s*</filter-class>
```

**Tree-Sitter (XML)**
```typescript
{
  type: "element",
  name: "servlet" | "filter" | "servlet-mapping",
  children: [
    { type: "element", name: "servlet-class|filter-class|url-pattern" }
  ]
}
```

### Instruction pour LLM
```
TÂCHE: Parser web.xml pour Servlets, Filters, Mappings
STRATÉGIE:
  1. Extrayez chaque <servlet> avec:
     - servlet-name
     - servlet-class
  2. Créez nœud Servlet pour chaque servlet-class
  3. Extrayez chaque <servlet-mapping>:
     - Mappez servlet-name à servlet-class
     - Collectez url-pattern(s)
  4. Créez relations:
     - web.xml -[DECLARES]-> Servlet
     - Servlet -[MAPPED_TO_URL]-> Pattern
  5. Pour chaque <filter>:
     - Créez nœud Filter
     - Extrayez <filter-mapping> pour ordre et url-pattern
     - Créez relations -[FILTERS]-> avec order
  6. Construisez chaîne de filtres en fonction de l'ordre

EXEMPLE:
  Déclaration: DispatcherServlet → com.example.web.DispatcherServlet
  Mapping: /portal/user.do → DispatcherServlet
  Relation: web.xml -[DECLARES {urlPattern: "*.do"}]-> DispatcherServlet

VALIDATION:
  - Servlet classe existe ✓
  - URL patterns valides ✓
  - Filter order cohérent ✓
  - Pas de conflits de mapping ✓
```

### Node Type Neo4j
```cypher
CREATE (webXml:Config {name: "web.xml", type: "deployment-descriptor"})
CREATE (servlet:Servlet {name: "DispatcherServlet", class: "com.example.web.DispatcherServlet"})
CREATE (filter1:Filter {name: "AuthenticationFilter", class: "com.example.filters.AuthenticationFilter"})
CREATE (filter2:Filter {name: "LoggingFilter", class: "com.example.filters.LoggingFilter"})
CREATE (webXml)-[:DECLARES {urlPattern: "*.do"}]->(servlet)
CREATE (filter1)-[:FILTERS {order: 1, urlPattern: "/*"}]->(servlet)
CREATE (filter2)-[:FILTERS {order: 2, urlPattern: "/*"}]->(servlet)
```

---

## 7. RELATION SÉMANTIQUE: portlet.xml → Portlet (Configuration)

### Description
Le fichier portlet.xml configure les Portlets avec des paramètres et modes.

### Exemple Code
```xml
<portlet>
  <portlet-name>UserManagementPortlet</portlet-name>
  <portlet-class>com.example.portlets.UserManagementPortlet</portlet-class>
  <supports>
    <mime-type>text/html</mime-type>
    <portlet-mode>view</portlet-mode>
    <portlet-mode>edit</portlet-mode>
  </supports>
  <init-param>
    <name>template-view</name>
    <value>/WEB-INF/portlets/user-list.jsp</value>
  </init-param>
  <expiration-cache>300</expiration-cache>
</portlet>
```

### Solution de Parsing

**Regex (XML)**
```regex
<portlet-class>\s*([^<]+)\s*</portlet-class>
<portlet-mode>\s*([^<]+)\s*</portlet-mode>
<window-state>\s*([^<]+)\s*</window-state>
<name>\s*([^<]+)\s*</name>\s*<value>\s*([^<]+)\s*</value>
<expiration-cache>\s*([^<]+)\s*</expiration-cache>
```

**Tree-Sitter (XML)**
```typescript
{
  type: "element",
  name: "portlet",
  children: [
    { name: "portlet-class", text: "*.Portlet" },
    { name: "init-param" },
    { name: "supports" }
  ]
}
```

### Instruction pour LLM
```
TÂCHE: Parser portlet.xml pour configurer Portlets
STRATÉGIE:
  1. Pour chaque <portlet>:
     - Extrayez portlet-name
     - Extrayez portlet-class
     - Créez/trouvez nœud Portlet
  2. Collectez supports:
     - portlet-mode (view, edit, help, etc.)
     - window-state (normal, maximized, etc.)
     - mime-type
  3. Collectez init-param:
     - name → value
     - Cas spécial: nom contenant "template" → mappe vers JSP
  4. Collectez expiration-cache → cache TTL
  5. Créez relation:
     - portlet.xml -[CONFIGURES]-> Portlet
     - Annotez avec modes, states, params, caching

EXEMPLE:
  Configuration: UserManagementPortlet
  Modes: [view, edit]
  Params: {
    "template-view": "/WEB-INF/portlets/user-list.jsp",
    "template-edit": "/WEB-INF/portlets/user-edit.jsp",
    "pageSize": "10"
  }
  Cache: 300 seconds

VALIDATION:
  - Portlet classe existe ✓
  - Modes supportés cohérents avec code ✓
  - Templates JSP existent (ou seront créés) ✓
```

### Node Type Neo4j
```cypher
CREATE (portletXml:Config {name: "portlet.xml", type: "portlet-descriptor"})
CREATE (portlet:Portlet {name: "UserManagementPortlet", class: "com.example.portlets.UserManagementPortlet"})
CREATE (portletXml)-[:CONFIGURES {
  modes: ["view", "edit"],
  states: ["normal", "maximized"],
  cacheSeconds: 300,
  params: {
    "template-view": "/WEB-INF/portlets/user-list.jsp",
    "pageSize": "10"
  }
}]->(portlet)
```

---

## 8. RELATION SÉMANTIQUE: Service → Notification (Side Effect)

### Description
Un Service déclenche une notification ou action asynchrone.

### Exemple Code
```java
public void approveTask(String taskId, String userId, String comment) {
  databaseConnector.updateTask(task);
  databaseConnector.saveComment(...);
  notificationService.sendNotification(message);  // ← Side effect
}
```

### Solution de Parsing

**Regex (Java)**
```regex
(\w+NotificationService|EventBus|Observable)\s*\.\s*(send|post|publish)\s*\(
```

**Tree-Sitter (Java)**
```typescript
{
  type: "method_invocation",
  object: { name: "*NotificationService|EventBus|Observable|*Listener" },
  method: { name: "send*|post*|publish*|dispatch*|trigger*" }
}
```

### Instruction pour LLM
```
TÂCHE: Identifier Notifications et Side Effects
STRATÉGIE:
  1. Cherchez appels à NotificationService, EventBus, etc.
  2. Catégorisez le type:
     - Email: sendEmail, sendMail
     - Push: sendPush, pushNotification
     - In-app: sendNotification, postNotification
     - SMS: sendSMS
  3. Extrayez le contexte (ce qui déclenche):
     - User action (approve, reject, submit)
     - Task completion
     - Threshold reached
  4. Créez relation:
     - Service -[NOTIFIES]-> NotificationService
     - Annotez: {type: "EMAIL|PUSH|IN_APP|SMS", trigger: "..."}

EXEMPLE:
  Service: WorkflowService
  Appel: notificationService.sendNotification("Task approved")
  Relation: WorkflowService -[NOTIFIES]-> NotificationService
  Annotation: {type: "IN_APP", trigger: "TASK_APPROVED", async: true}

VALIDATION:
  - NotificationService existe ✓
  - Type de notification identifié ✓
  - Trigger event identifié ✓
```

### Node Type Neo4j
```cypher
MATCH (service:Service {name: "WorkflowService"})
CREATE (notifService:Service {name: "NotificationService"})
CREATE (service)-[:NOTIFIES {type: "IN_APP", trigger: "TASK_APPROVED", async: true, line: 68}]->(notifService)
```

---

## Matrice Complète des Relations

| De | Vers | Relation | Parsing | Priority |
|:---|:-----|:---------|:--------|:---------|
| Portlet | JSP | RENDERS | Regex/TS Java | CRÍTICA |
| JSP | JavaScript | INCLUDES_JS | Regex HTML | ALTA |
| JavaScript | Servlet | CALLS_AJAX | Regex JS | CRÍTICA |
| Portlet | Service | CALLS_SERVICE | Regex/TS Java | ALTA |
| Service | DAO | CALLS_DAO | Regex/TS Java | ALTA |
| web.xml | Servlet | DECLARES | Regex XML | ALTA |
| portlet.xml | Portlet | CONFIGURES | Regex XML | ALTA |
| Service | Notification | NOTIFIES | Regex Java | MEDIA |
| JSP | JSP | INCLUDES_JSP | Regex HTML | MEDIA |
| JavaScript | JavaScript | DEPENDS_ON | Regex JS | MEDIA |
| JSP | JavaScript | BINDS_DATA | Regex HTML/JS | MEDIA |
| JavaScript | DOM | TARGETS_ELEMENT | Regex JS | BAJA |

---

## Résumé: Instruction Synthétique pour LLM

```
MISSION: Analyser architecture WebSphere Portal

INPUT:
  - Sources Java (.java files)
  - Templates JSP (.jsp, .jspf)
  - Scripts JavaScript (.js)
  - Configuration XML (web.xml, portlet.xml)

PROCESS:

Phase 1 - EXTRACTION:
  1. Java: Extraire classes, méthodes, imports, champs
  2. JSP: Parser HTML, scripts, includes, variables
  3. JS: Identifier AJAX calls, handlers, dependencies
  4. XML: Parser servlets, filters, portlets, params

Phase 2 - IDENTIFICATION RELATIONS (dans cet ordre):
  1. CRÍTICA: Portlet→JSP (dispatch), JS→AJAX ($.ajax), web.xml→Servlet
  2. ALTA: Portlet→Service, Service→DAO, portlet.xml→Portlet
  3. MEDIA: Service→Notification, JSP includes, JS dependencies
  4. BAJA: DOM targeting, logging, error handling

Phase 3 - MAPPING:
  - Créer nœuds Neo4j pour chaque component
  - Créer relations avec types sémantiques
  - Annoter avec metadata (method, params, line numbers, etc.)

Phase 4 - VALIDATION:
  - Vérifier chaînes complètes (Portlet→Service→DAO)
  - Vérifier cohérence des noms
  - Détecter cycles ou anomalies

OUTPUT:
  Neo4j graph avec:
  - ~250+ nœuds (classes, JSPs, JS, servlets)
  - ~150+ relations sémantiques annotées
  - Couverture 100% des usecases

RESTRICTIONS:
  - Pas de code exécution, analyse statique uniquement
  - Résoudre URLs ambigüés via web.xml routing
  - Flags sur relations non validées
```

---

## Checklist Finale

- [ ] Tous les portlets sont identifiés
- [ ] Tous les JSP dispatches sont mappés
- [ ] Tous les $.ajax calls sont tracés
- [ ] web.xml est complètement parsé
- [ ] portlet.xml est complètement parsé
- [ ] Toutes les dépendances Services sont identifiées
- [ ] Toutes les DAO calls sont annotées avec l'opération
- [ ] Les notifications sont identifiées
- [ ] Les filtres et ordre d'exécution sont documentés
- [ ] Au moins 80% de couverture d'usecases
