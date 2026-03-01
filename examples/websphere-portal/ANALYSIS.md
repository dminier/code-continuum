# WebSphere Portal - Analyse des Dépendances et Appels

## Graphe des Dépendances

### Portlets (extends BasePortlet)
```
BasePortlet (abstract)
├── UserManagementPortlet
├── DocumentPortlet
├── DashboardPortlet
├── WorkflowPortlet
└── ReportPortlet
```

### Services
```
UserService
├── calls: DatabaseConnector.saveUser()
├── calls: DatabaseConnector.deleteUser()
└── uses: Map<String, User>

DocumentService
├── calls: DatabaseConnector.saveDocument()
├── calls: DatabaseConnector.deleteDocument()
├── calls: FileStorageManager.storeFile()
├── calls: FileStorageManager.deleteFile()
└── uses: Map<String, Document>

DashboardService
├── calls: MetricsCollector.getTotalUsers()
├── calls: MetricsCollector.getActiveUsers()
├── calls: MetricsCollector.getPendingTasks()
├── calls: MetricsCollector.getCompletedProcesses()
├── calls: MetricsCollector.getDocumentsCount()
├── calls: MetricsCollector.getStorageUsed()
├── calls: DatabaseConnector.getRecentUploads()
├── calls: DatabaseConnector.getRecentApprovals()
└── calls: DatabaseConnector.getRecentLogins()

WorkflowService
├── calls: DatabaseConnector.saveProcess()
├── calls: DatabaseConnector.updateTask()
├── calls: DatabaseConnector.saveComment()
├── calls: DatabaseConnector.logDelegation()
├── calls: NotificationService.sendNotification()
└── uses: Map<String, WorkflowTask>, Map<String, WorkflowProcess>

ReportService
├── calls: DatabaseConnector.saveReport()
├── calls: ReportGenerator.generate()
├── calls: ExportManager.export()
└── uses: List<Report>
```

### Web Components
```
DispatcherServlet (extends HttpServlet)
└── calls: forward() / redirect()

ResourceServlet (extends HttpServlet)
├── calls: Files.readAllBytes()
└── uses: Path

AuthenticationFilter (implements Filter)
├── calls: httpRequest.getSession()
├── calls: requestDispatcher.forward()
└── calls: chain.doFilter()

LoggingFilter (implements Filter)
├── calls: System.currentTimeMillis()
├── calls: chain.doFilter()
└── calls: System.out.println()
```

### Listeners
```
PortalStartupListener (implements ServletContextListener)
├── calls: initializeDatabase()
├── calls: initializeCache()
├── calls: initializeScheduledTasks()
├── calls: cleanupDatabase()
├── calls: cleanupCache()
└── calls: cleanupScheduledTasks()

SessionListener (implements HttpSessionListener)
├── calls: getActiveSessions()
└── uses: static int activeSessions
```

## Appels Détaillés par Portlet

### UserManagementPortlet
```
doView():
  ├── getInitParameter("pageSize")
  ├── getInitParameter("template-view" | "template-edit")
  ├── userService.getUserById(userId)
  ├── userService.getAllUsers(pageSize)
  └── dispatch(templatePath, request, response)

doEdit():
  ├── getInitParameter("template-edit")
  ├── userService.getUserById(userId)
  └── dispatch(templatePath, request, response)

processAction():
  ├── userService.saveUser(user)
  ├── userService.deleteUser(userId)
  └── setRenderParameter(name, value)
```

### DocumentPortlet
```
doView():
  ├── getInitParameter("repository-path")
  ├── getInitParameter("max-file-size")
  ├── documentService.getDocuments(folderId)
  └── dispatch("/WEB-INF/portlets/document-list.jsp", request, response)

processAction():
  ├── documentService.uploadDocument(document)
  └── documentService.deleteDocument(documentId)
```

### DashboardPortlet
```
doView():
  ├── getInitParameter("refresh-interval")
  ├── dashboardService.getStatistics()
  ├── dashboardService.getRecentActivities()
  └── dispatch("/WEB-INF/portlets/dashboard.jsp", request, response)

serveResource():
  ├── getResourceID()
  ├── dashboardService.getStatistics()
  └── convertToJson(stats)
```

### WorkflowPortlet
```
doView():
  ├── getRemoteUser()
  ├── workflowService.getPendingTasks(userId)
  ├── workflowService.getProcess(processId)
  └── dispatch(templatePath, request, response)

processAction():
  ├── getRemoteUser()
  ├── workflowService.approveTask(taskId, userId, comment)
  ├── workflowService.rejectTask(taskId, userId, reason)
  ├── workflowService.delegateTask(taskId, fromUser, toUser)
  ├── sendNotification(message)
  └── setRenderParameter("view", "list")
```

### ReportPortlet
```
doView():
  ├── reportService.getReport(reportId)
  ├── reportService.getAvailableReports()
  └── dispatch(templatePath, request, response)

processAction():
  ├── reportService.generateReport(reportId, filters)
  ├── reportService.exportReport(reportId, format)
  └── setRenderParameter(name, value)

serveResource():
  ├── reportService.exportReport(reportId, format)
  ├── isValidFormat(format)
  ├── getContentType(format)
  └── getFileExtension(format)
```

## Flux de Données

### Flux Utilisateur (UserManagementPortlet)
```
User List View:
  UserManagementPortlet.doView()
    → UserService.getAllUsers(pageSize)
      → DatabaseConnector (simulated)
        → List<User>
          → request.setAttribute("users", users)
            → dispatch("/WEB-INF/portlets/user-list.jsp")

User Update:
  UserManagementPortlet.processAction()
    → new User()
    → UserService.saveUser(user)
      → DatabaseConnector.saveUser(user)
        → response.setRenderParameter("view", "list")
```

### Flux Document (DocumentPortlet)
```
Document Upload:
  DocumentPortlet.processAction()
    → new Document()
    → DocumentService.uploadDocument(document)
      → FileStorageManager.storeFile()
        → File system
      → DatabaseConnector.saveDocument(document)
        → Database

Document List:
  DocumentPortlet.doView()
    → DocumentService.getDocuments(folderId)
      → List<Document>
        → request.setAttribute("documents", documents)
          → dispatch("/WEB-INF/portlets/document-list.jsp")
```

### Flux Workflow (WorkflowPortlet)
```
Task Approval:
  WorkflowPortlet.processAction()
    → WorkflowService.approveTask(taskId, userId, comment)
      → DatabaseConnector.updateTask(task)
      → DatabaseConnector.saveComment()
      → sendNotification(message)
        → NotificationService.sendNotification()

Task List:
  WorkflowPortlet.doView()
    → WorkflowService.getPendingTasks(userId)
      → List<WorkflowTask>
        → request.setAttribute("tasks", tasks)
          → dispatch("/WEB-INF/portlets/workflow-list.jsp")
```

### Flux Dashboard (DashboardPortlet)
```
Dashboard View:
  DashboardPortlet.doView()
    → DashboardService.getStatistics()
      → MetricsCollector.getTotalUsers()
      → MetricsCollector.getActiveUsers()
      → MetricsCollector.getPendingTasks()
      → ... (autres métriques)
      → Map<String, Object>
        → request.setAttribute("statistics", stats)
          → dispatch("/WEB-INF/portlets/dashboard.jsp")

AJAX Refresh:
  DashboardPortlet.serveResource()
    → dashboardService.getStatistics()
      → Map<String, Object>
        → convertToJson(stats)
          → PrintWriter.write(json)
```

## Hiérarchie des Classes

```
javax.portlet.GenericPortlet
  └── BasePortlet (abstract)
       ├── UserManagementPortlet
       ├── DocumentPortlet
       ├── DashboardPortlet
       ├── WorkflowPortlet
       └── ReportPortlet

javax.servlet.http.HttpServlet
  ├── DispatcherServlet
  └── ResourceServlet

javax.servlet.Filter
  ├── AuthenticationFilter
  └── LoggingFilter

javax.servlet.ServletContextListener
  └── PortalStartupListener

javax.servlet.http.HttpSessionListener
  └── SessionListener
```

## Cas de Collecte pour le Parseur

### 1. Héritage
- BasePortlet extends GenericPortlet
- DispatcherServlet extends HttpServlet
- AuthenticationFilter implements Filter

### 2. Interfaces
- Filter (doFilter, init, destroy)
- ServletContextListener (contextInitialized, contextDestroyed)
- HttpSessionListener (sessionCreated, sessionDestroyed)

### 3. Appels de Méthode
- userService.getUserById()
- documentService.uploadDocument()
- workflowService.approveTask()
- reportService.generateReport()

### 4. Constructeurs
- UserService(), DocumentService(), DashboardService()
- User(), Document(), WorkflowTask(), WorkflowProcess()

### 5. Propriétés
- Private fields: userService, documentService, refreshInterval
- Static fields: activeSessions

### 6. Paramètres
- Simple: String, int, boolean
- Collections: List<User>, Map<String, Object>
- Objets: User, Document, WorkflowTask

### 7. Exceptions
- throw new RuntimeException()
- catch (Exception e)
- throws ServletException, IOException

### 8. Ressources
- getInitParameter(), getRemoteUser()
- request.setAttribute(), response.setRenderParameter()
- request.getRequestDispatcher()

## Statistiques

- **Portlets**: 5
- **Services**: 5
- **Modèles**: 5
- **Servlets**: 2
- **Filtres**: 2
- **Listeners**: 2
- **Utilitaires**: 6
- **Fichiers XML**: 2 (web.xml, portlet.xml)
- **Fichiers Java**: 25
- **Lignes de code**: ~1500
