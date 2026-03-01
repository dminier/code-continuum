# Guide de Test - WebSphere Portal Parser

Ce guide décrit comment utiliser le jeu de tests WebSphere Portal pour valider votre parseur.

## Vue d'ensemble

Le jeu de tests contient **32 fichiers** représentant une application portale WebSphere complète:

- **5 Portlets** (UserManagement, Document, Dashboard, Workflow, Report)
- **5 Services** métier
- **5 Modèles de données** (User, Document, Task, Process, Report)
- **2 Servlets** (Dispatcher, Resource)
- **2 Filtres** (Authentication, Logging)
- **2 Listeners** (Startup, Session)
- **6 Utilitaires** (Database, FileStorage, Metrics, Notification, ReportGen, Export)
- **2 XML descriptors** (web.xml, portlet.xml)
- **2 Configuration** (application-context.xml, application.properties)

## Cas de Test Recommandés

### Test 1: Extraction des Portlets

**Objectif**: Vérifier que le parseur extrait correctement les portlets

```bash
# Fichiers concernés:
examples/websphere-portal/java/BasePortlet.java
examples/websphere-portal/java/UserManagementPortlet.java
examples/websphere-portal/java/DocumentPortlet.java
examples/websphere-portal/java/DashboardPortlet.java
examples/websphere-portal/java/WorkflowPortlet.java
examples/websphere-portal/java/ReportPortlet.java

# Attendus:
- 5 classes portlet identifiées
- BasePortlet comme classe parente commune
- Méthodes: doView(), doEdit(), processAction(), serveResource()
- Paramètres de portlet.xml
```

### Test 2: Analyse des Appels de Service

**Objectif**: Vérifier la détection des appels de service

```bash
# Flux exemple:
UserManagementPortlet.processAction()
  → UserService.saveUser(user)
    → DatabaseConnector.saveUser(user)

# Points à vérifier:
- Appel Portlet → Service
- Appel Service → DatabaseConnector
- Passage de paramètres (User object)
- Chaîne d'appels complète
```

### Test 3: Hiérarchie d'Héritage

**Objectif**: Vérifier l'extraction de l'héritage

```bash
# Hiérarchie:
GenericPortlet (javax.portlet)
  → BasePortlet (abstract)
    → UserManagementPortlet
    → DocumentPortlet
    → DashboardPortlet
    → WorkflowPortlet
    → ReportPortlet

HttpServlet (javax.servlet)
  → DispatcherServlet
  → ResourceServlet

# Points à vérifier:
- Classes parentes identifiées
- Différenciation: extends vs implements
```

### Test 4: Interfaces et Implémentations

**Objectif**: Vérifier la détection des interfaces

```bash
# Interfaces:
Filter
  → AuthenticationFilter
  → LoggingFilter

ServletContextListener
  → PortalStartupListener

HttpSessionListener
  → SessionListener

# Points à vérifier:
- implements correctement identifiés
- Méthodes implémentées
- Signature des méthodes
```

### Test 5: Modèles de Données

**Objectif**: Vérifier l'extraction des POJO

```bash
# Modèles:
User (id, name, email, department, active)
Document (id, fileName, folderId, repositoryPath, fileSize, mimeType, createdDate, createdBy)
WorkflowTask (id, processId, taskName, assignedTo, status, createdDate, dueDate, priority)
WorkflowProcess (id, name, status, initiatedBy, initiatedDate, tasks)
Report (id, title, description, reportType, createdBy, createdDate, content)

# Points à vérifier:
- Propriétés privées
- Getters/Setters
- Constructeurs (default et paramétrisés)
```

### Test 6: Analyse de Configurations XML

**Objectif**: Vérifier la lecture de web.xml et portlet.xml

```bash
# web.xml:
- Servlets déclarés
- Filters et mappings
- Listeners
- Session configuration
- Error pages

# portlet.xml:
- 5 Portlets déclarés
- Modes supportés (view, edit, help)
- États de fenêtre (normal, maximized)
- Paramètres d'initialisation
- Cache expiration
```

## Cas d'Usage Complexes

### Scénario 1: Flux Utilisateur Complet

```
1. UserManagementPortlet.doView()
   - Récupère pageSize du portlet.xml
   - Appelle UserService.getAllUsers(pageSize)
   - AttributeDans service:
     * Récupère List<User> de Map
     * Appelle DatabaseConnector.saveUser()
   - Dispatch vers JSP template

2. UserManagementPortlet.processAction()
   - Parse paramètres (userId, name, email)
   - Crée nouvel objet User
   - Appelle UserService.saveUser(user)
   - Set render parameter pour transition

3. Vérifications attendues:
   ✓ Appels between portlet ↔ service
   ✓ Passage de type complexe (User object)
   ✓ Utilisation de configuration (pageSize)
   ✓ Collections (List<User>)
```

### Scénario 2: Gestion de Fichiers

```
1. DocumentPortlet.processAction()
   - Crée Document avec repositoryPath
   - Appelle DocumentService.uploadDocument(doc)
   - Service appelle FileStorageManager.storeFile()
   - Service appelle DatabaseConnector.saveDocument()

2. Vérifications:
   ✓ Appels à FileStorageManager
   ✓ Gestion d'exceptions (try/catch)
   ✓ Métadonnées de fichier
   ✓ Appels en chaîne (portlet → service → util)
```

### Scénario 3: Workflows avec Notifications

```
1. WorkflowPortlet.processAction() avec action="approve"
   - Récupère userId via getRemoteUser()
   - Appelle WorkflowService.approveTask(taskId, userId, comment)
   - Service appelle:
     * DatabaseConnector.updateTask(task)
     * DatabaseConnector.saveComment(...)
     * sendNotification(message)
       → NotificationService.sendNotification()

2. Vérifications:
   ✓ Appels multiples dans une action
   ✓ Gestion de l'utilisateur courant
   ✓ Chaîne d'appels profonde
   ✓ Notifications asynchrones
```

### Scénario 4: Dashboard AJAX

```
1. DashboardPortlet.serveResource()
   - Récupère resourceID
   - Appelle DashboardService.getStatistics()
   - Service appelle MetricsCollector.getTotalUsers()
   - Service appelle DatabaseConnector.getRecentUploads(limit)
   - Convertit Map en JSON
   - Écrit dans OutputStream

2. Vérifications:
   ✓ Ressources côté serveur (serveResource)
   ✓ Appels à MetricsCollector
   ✓ Conversion d'objets complexes
   ✓ I/O avec PrintWriter
```

### Scénario 5: Générations de Rapports

```
1. ReportPortlet.processAction() avec action="export"
   - Valide format (PDF/Excel/CSV)
   - Appelle ReportService.exportReport(reportId, format)
   - Service appelle ExportManager.export(report, format)
   - ExportManager switch sur format et appelle generatePdf()/generateExcel()/generateCsv()
   - Retourne byte[]

2. ReportPortlet.serveResource() pour download
   - Récupère reportId et format
   - Appelle reportService.exportReport()
   - Set Content-Type et Content-Disposition
   - Écrit bytes dans OutputStream

3. Vérifications:
   ✓ Validations de paramètres
   ✓ Patterns Switch/Case
   ✓ Retours binaires
   ✓ Téléchargement de fichiers
```

## Indices de Qualité du Parseur

Votre parseur devrait être capable de:

1. **Extraire tous les éléments**:
   - Classes (25 fichiers Java)
   - Interfaces et implémentations
   - Méthodes et signatures
   - Champs et propriétés
   - ✓ Attendus: ~250+ noeuds de graphe

2. **Identifier les relations**:
   - Héritage (5 portlets → BasePortlet)
   - Implémentation (2 filters, 2 listeners)
   - Dépendances (portlet → service → util)
   - Collections (List<T>, Map<K,V>)
   - ✓ Attendus: ~150+ arêtes de relation

3. **Supporter les constructs Java**:
   - Packages (com.example.portlets, com.example.services, etc.)
   - Access modifiers (public, private)
   - Static fields (activeSessions)
   - Exceptions (throws, catch)
   - Annotations (@Override)
   - Génériques (List<User>, Map<String, Object>)

4. **Analyser les XMLs**:
   - Lire et indexer web.xml
   - Lire et indexer portlet.xml
   - Lier configurations aux classes Java
   - Extraire paramètres d'initialisation

## Métriques de Validation

```
Classes: 25
Interfaces: 3 (GenericPortlet, Filter, ServletContextListener, HttpSessionListener)
Méthodes: ~200
Champs: ~150
Packages: 6 (portlets, services, models, web, filters, listeners)
Relations:
  - Héritage: 7
  - Implémentation: 4
  - Dépendance: ~40
  - Appels de méthode: ~80
```

## Conseils de Parsage

1. **Commencer simple**: Parser d'abord les portlets simples (Dashboard, Report)
2. **Puis complexe**: Services avec collections et appels en chaîne
3. **Valider incrementalement**: Comparer avec ANALYSIS.md
4. **Tester patterns**:
   - Héritage profond
   - Interfaces multiples
   - Appels en cascade
   - Gestion d'exceptions

## Exécution du Test

```bash
# Avec votre parser
cargo run -- examples/websphere-portal/java

# Vérifier la sortie Neo4j
curl -X POST http://localhost:7687 \
  -H "Content-Type: application/json" \
  -d 'MATCH (c:Class) WHERE c.language = "java" RETURN count(c)'
# Attendu: ~25 classes

# Vérifier les appels
curl -X POST http://localhost:7687 \
  -H "Content-Type: application/json" \
  -d 'MATCH (f:Function)-[:CALLS]->(g:Function) RETURN count(*)'
# Attendu: ~80+ appels
```

## Notes

- Le code est intentionnellement simple mais représentatif
- Les imports incluent javax.portlet et javax.servlet
- Les méthodes sont complètes (ne sont pas des stubs)
- Les patterns reflètent des applications réelles
- Les configurations XML sont valides
