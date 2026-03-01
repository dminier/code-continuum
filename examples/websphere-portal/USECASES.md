# WebSphere Portal - Cas d'Usage Complets (USECASES)

## Vue d'Ensemble

Ce document détaille chaque cas d'usage complet avec la traversée complète:
**Portlet Java → Service → DAO → JSP → JavaScript → AJAX → Servlet**

---

## 1. USECASE: User Management (Gestion des Utilisateurs)

### 1.1 Affichage de la Liste des Utilisateurs

```
Flux: HTTP GET /portal/user?action=list
│
├── DispatcherServlet (web.xml)
│   └── routes vers: /portals/user
│
├── UserManagementPortlet.doView()
│   ├── getInitParameter("pageSize") ← portlet.xml
│   ├── getInitParameter("template-view") ← portlet.xml
│   └── userService.getAllUsers(10)
│       ├── DatabaseConnector.getUsers()
│       └── returns: List<User>
│
├── request.setAttribute("users", userList)
│
└── dispatch("/WEB-INF/portlets/user-list.jsp")
    ├── JSP Template Rendering
    ├── <c:forEach items="${users}"> (JSTL)
    └── Inclut: /WEB-INF/portlets/common/user-row.jspf
        └── Affiche: nom, email, département
```

**Fichiers Impliqués:**
- Java: UserManagementPortlet.java, UserService.java, DatabaseConnector.java
- XML: web.xml (DispatcherServlet), portlet.xml (UserManagementPortlet)
- JSP: user-list.jsp, user-row.jspf (À CRÉER)
- JS: aucun pour cette vue statique

**Relations:**
- `UserManagementPortlet` → `UserService` (call)
- `UserManagementPortlet` → `/WEB-INF/portlets/user-list.jsp` (dispatch)
- `user-list.jsp` → `user-row.jspf` (include)
- `DispatcherServlet` → `UserManagementPortlet` (route from web.xml)

---

### 1.2 Édition d'un Utilisateur (avec Validation AJAX)

```
Flux: HTTP GET /portal/user?action=edit&userId=USR001
│
├── UserManagementPortlet.doEdit()
│   ├── getInitParameter("template-edit") ← portlet.xml
│   ├── userId = request.getParameter("userId")
│   ├── userService.getUserById(userId)
│   └── request.setAttribute("user", userDetails)
│
└── dispatch("/WEB-INF/portlets/user-edit.jsp")
    └── Contient formulaire avec id="userForm"
        └── <script src="/resources/js/user-form-validator.js"></script>
            └── JavaScript validation côté client
```

**Validation AJAX (formulaire):**

```
Utilisateur remplit le formulaire
│
└── JavaScript: user-form-validator.js
    └── on("input", "#userEmail", function() {
        ├── $.ajax({
        │   ├── url: "/portal/api/users/validate-email",
        │   ├── type: "POST",
        │   ├── data: { email: $(this).val() }
        │   └── success: function(response) {
        │       └── Affiche checkmark ou erreur
        │   }
        └── })
```

**Qui appelle?**

```
userForm validation request
│
└── ResourceServlet (web.xml) ou nouveau: ValidationServlet
    ├── resourceId = "validate-email"
    ├── ValidationService.validateEmail(email)
    └── returns: {valid: true/false, message: "..."}
        └── Response JSON to JavaScript
```

**Fichiers Impliqués:**
- Java: UserManagementPortlet.java, UserService.java, ValidationService.java (À CRÉER)
- XML: web.xml (servlet mapping), portlet.xml (UserManagementPortlet)
- JSP: user-edit.jsp (À CRÉER)
- JS: user-form-validator.js (À CRÉER)

**Relations:**
- `UserManagementPortlet` → `UserService` (call)
- `UserManagementPortlet` → `/WEB-INF/portlets/user-edit.jsp` (dispatch)
- `user-edit.jsp` → `user-form-validator.js` (include script)
- `user-form-validator.js` → `/portal/api/users/validate-email` (AJAX call)
- `ValidationServlet` (from web.xml) → `ValidationService` (call)

---

### 1.3 Sauvegarde d'un Utilisateur (Form Submission)

```
Flux: HTTP POST /portal/user?action=save
│
├── AuthenticationFilter (web.xml)
│   └── vérifie session user
│
├── UserManagementPortlet.processAction()
│   ├── action = request.getParameter("action")
│   ├── userId = request.getParameter("userId")
│   ├── User user = new User()
│   ├── user.setId(userId)
│   ├── userService.saveUser(user)
│   │   └── DatabaseConnector.saveUser(user)
│   │
│   └── response.setRenderParameter("view", "list")
│
└── Redirect to doView() for list display
```

**Fichiers Impliqués:**
- Java: UserManagementPortlet.java, UserService.java, DatabaseConnector.java
- Java: AuthenticationFilter.java (from web.xml)
- XML: web.xml (AuthenticationFilter mapping), portlet.xml

**Relations:**
- `AuthenticationFilter` → `UserManagementPortlet` (filter chain)
- `UserManagementPortlet` → `UserService` (call)
- `UserService` → `DatabaseConnector` (call)

---

## 2. USECASE: Document Management (Gestion des Documents)

### 2.1 Affichage de la Liste des Documents

```
Flux: HTTP GET /portal/document?action=list
│
├── DispatcherServlet (web.xml)
│
├── DocumentPortlet.doView()
│   ├── folderId = request.getParameter("folderId")
│   ├── documentService.getDocuments(folderId)
│   │   └── returns: List<Document>
│   │
│   └── dispatch("/WEB-INF/portlets/document-list.jsp")
│       └── Affiche avec colonnes: nom, taille, date, actions
│
└── JavaScript pour actions:
    └── document-list.js
        ├── on("click", ".btn-delete", function() {
        │   ├── $.post("/portal/api/documents/delete",
        │   │   └── {documentId: $(this).data("id")}
        │   └── )
        └── on("click", ".btn-download", function() {
            └── window.location = "/portal/document/download?id=..."
```

**Fichiers Impliqués:**
- Java: DocumentPortlet.java, DocumentService.java, DatabaseConnector.java
- XML: web.xml (DispatcherServlet), portlet.xml (DocumentPortlet)
- JSP: document-list.jsp (À CRÉER)
- JS: document-list.js (À CRÉER)

**Relations:**
- `DocumentPortlet` → `DocumentService` (call)
- `DocumentPortlet` → `/WEB-INF/portlets/document-list.jsp` (dispatch)
- `document-list.jsp` → `document-list.js` (include script)
- `document-list.js` → `/portal/api/documents/delete` (AJAX POST)
- `document-list.js` → `/portal/document/download` (form action)

---

### 2.2 Upload de Fichier (avec Progress Bar)

```
Flux: HTTP POST /portal/document?action=upload
│
├── DocumentPortlet.processAction()
│   ├── action = "upload"
│   ├── Document document = new Document()
│   ├── document.setFileName(request.getParameter("fileName"))
│   ├── documentService.uploadDocument(document)
│   │   ├── FileStorageManager.storeFile(path, filename)
│   │   └── DatabaseConnector.saveDocument(document)
│   │
│   └── response.setRenderParameter("status", "success")
│
└── AJAX Response to document-upload.js
```

**Upload avec Progress AJAX:**

```
document-upload.js
├── on("change", "#fileInput", function() {
│   ├── FormData formData = new FormData()
│   ├── formData.append("file", file)
│   ├── $.ajax({
│   │   ├── url: "/portal/document?action=upload",
│   │   ├── type: "POST",
│   │   ├── data: formData,
│   │   ├── xhr: function() {
│   │   │   └── track upload progress
│   │   └── },
│   │   └── success: function() {
│   │       ├── Update progress bar to 100%
│   │       └── Reload document list
│   │   }
│   └── })
└── Affiche barre de progression
```

**Fichiers Impliqués:**
- Java: DocumentPortlet.java, DocumentService.java, FileStorageManager.java
- Java: DatabaseConnector.java
- XML: web.xml, portlet.xml
- JSP: document-upload.jsp (À CRÉER)
- JS: document-upload.js (À CRÉER)

**Relations:**
- `DocumentPortlet` → `DocumentService` (call)
- `DocumentPortlet` → `FileStorageManager` (call)
- `DocumentService` → `DatabaseConnector` (call)
- `document-upload.jsp` → `document-upload.js` (include script)
- `document-upload.js` → `/portal/document?action=upload` (AJAX POST)

---

## 3. USECASE: Dashboard (Tableau de Bord)

### 3.1 Affichage du Dashboard (avec Refresh AJAX)

```
Flux: HTTP GET /portal/dashboard
│
├── DispatcherServlet (web.xml)
│
├── DashboardPortlet.doView()
│   ├── getInitParameter("refresh-interval") ← portlet.xml
│   ├── dashboardService.getStatistics()
│   │   ├── metricsCollector.getTotalUsers()
│   │   ├── metricsCollector.getActiveUsers()
│   │   ├── metricsCollector.getPendingTasks()
│   │   └── returns: Map<String, Object>
│   │
│   ├── request.setAttribute("statistics", stats)
│   │
│   └── dispatch("/WEB-INF/portlets/dashboard.jsp")
│       ├── Affiche widgets avec statistiques
│       └── <script src="/resources/js/dashboard-refresh.js"></script>
│           ├── Initialise avec refreshInterval = 60s
│           └── Lance refresh automatique toutes les 60s
```

**Refresh AJAX Périodique:**

```
dashboard-refresh.js
├── setInterval(function() {
│   ├── $.ajax({
│   │   ├── url: "/portal/dashboard?resourceId=refreshStats",
│   │   ├── type: "GET",
│   │   ├── dataType: "json",
│   │   └── success: function(newStats) {
│   │       ├── Update widget 1: totalUsers
│   │       ├── Update widget 2: activeUsers
│   │       ├── Update widget 3: pendingTasks
│   │       └── Redraw avec animation
│   │   }
│   └── })
└── , refreshInterval) // 60000ms = 60s
```

**Qui appelle?**

```
ResourceServlet (web.xml) ou DashboardPortlet.serveResource()
│
├── resourceId = "refreshStats"
├── DashboardService.getStatistics()
│   ├── MetricsCollector.getTotalUsers()
│   ├── MetricsCollector.getActiveUsers()
│   └── returns: Map<String, Object>
│
└── Response JSON to JavaScript
```

**Fichiers Impliqués:**
- Java: DashboardPortlet.java, DashboardService.java, MetricsCollector.java
- XML: web.xml (ResourceServlet), portlet.xml (DashboardPortlet with refresh-interval)
- JSP: dashboard.jsp (À CRÉER)
- JS: dashboard-refresh.js (À CRÉER)

**Relations:**
- `DashboardPortlet` → `DashboardService` (call in doView)
- `DashboardPortlet` → `/WEB-INF/portlets/dashboard.jsp` (dispatch)
- `DashboardPortlet.serveResource()` → `DashboardService` (call for AJAX refresh)
- `dashboard.jsp` → `dashboard-refresh.js` (include script)
- `dashboard-refresh.js` → `DashboardPortlet.serveResource()` (AJAX call)

---

## 4. USECASE: Workflow Management (Gestion des Workflows)

### 4.1 Affichage des Tâches En Attente

```
Flux: HTTP GET /portal/workflow
│
├── LoggingFilter (web.xml)
│   └── log request
│
├── WorkflowPortlet.doView()
│   ├── userId = request.getRemoteUser()
│   ├── workflowService.getPendingTasks(userId)
│   │   └── returns: List<WorkflowTask>
│   │
│   └── dispatch("/WEB-INF/portlets/workflow-list.jsp")
│       └── Affiche tableau des tâches avec actions
│
└── JavaScript pour actions:
    └── workflow-actions.js
        ├── on("click", ".btn-approve", function() {
        │   └── showApprovalForm(taskId)
        └── on("click", ".btn-reject", function() {
            └── showRejectionForm(taskId)
```

**Fichiers Impliqués:**
- Java: WorkflowPortlet.java, WorkflowService.java, DatabaseConnector.java
- XML: web.xml (LoggingFilter), portlet.xml (WorkflowPortlet)
- JSP: workflow-list.jsp (À CRÉER)
- JS: workflow-actions.js (À CRÉER)

**Relations:**
- `LoggingFilter` → `WorkflowPortlet` (filter chain)
- `WorkflowPortlet` → `WorkflowService` (call)
- `WorkflowPortlet` → `/WEB-INF/portlets/workflow-list.jsp` (dispatch)
- `workflow-list.jsp` → `workflow-actions.js` (include script)
- `workflow-actions.js` → dialog forms (DOM manipulation)

---

### 4.2 Approbation de Tâche (avec Modale et AJAX)

```
Flux: Utilisateur clique "Approuver"
│
├── workflow-actions.js
│   ├── showApprovalForm(taskId)
│   │   └── Affiche modale HTML
│   │       ├── <input id="approvalComment">
│   │       └── <button id="submitApproval">
│   │
│   └── on("click", "#submitApproval", function() {
│       ├── comment = $("#approvalComment").val()
│       ├── $.ajax({
│       │   ├── url: "/portal/workflow?action=approve",
│       │   ├── type: "POST",
│       │   ├── data: {
│       │   │   ├── taskId: taskId,
│       │   │   ├── comment: comment
│       │   │ }
│       │   └── success: function(response) {
│       │       ├── showSuccessMessage()
│       │       └── reloadTaskList()
│       │   }
│       └── })
│
└── BackEnd: WorkflowPortlet.processAction()
    ├── action = "approve"
    ├── taskId = request.getParameter("taskId")
    ├── comment = request.getParameter("comment")
    ├── userId = request.getRemoteUser()
    ├── workflowService.approveTask(taskId, userId, comment)
    │   ├── DatabaseConnector.updateTask(task)
    │   ├── DatabaseConnector.saveComment(...)
    │   └── sendNotification("Task approved")
    │       └── NotificationService.sendNotification()
    │
    └── response.setRenderParameter("status", "success")
```

**Notification:**

```
NotificationService.sendNotification()
└── Peut dispatcher vers:
    ├── Email service
    ├── Push notification service
    └── In-app notification system
        └── Peut appeler /portal/api/notifications/send AJAX
```

**Fichiers Impliqués:**
- Java: WorkflowPortlet.java, WorkflowService.java, DatabaseConnector.java, NotificationService.java
- XML: web.xml, portlet.xml
- JSP: workflow-list.jsp (avec modale HTML)
- JS: workflow-actions.js

**Relations:**
- `WorkflowPortlet` → `WorkflowService` (call in processAction)
- `WorkflowService` → `DatabaseConnector` (call)
- `WorkflowService` → `NotificationService` (call)
- `workflow-actions.js` → `/portal/workflow?action=approve` (AJAX POST)
- `WorkflowPortlet.processAction()` → `WorkflowService` (call)

---

## 5. USECASE: Report Generation (Génération de Rapports)

### 5.1 Affichage de la Liste des Rapports

```
Flux: HTTP GET /portal/reports
│
├── ReportPortlet.doView()
│   ├── reportId = request.getParameter("reportId")
│   ├── reportService.getAvailableReports()
│   │   └── returns: List<Report>
│   │
│   └── dispatch("/WEB-INF/portlets/report-list.jsp")
│       ├── Affiche liste des rapports
│       └── Include: /WEB-INF/portlets/common/report-filters.jspf
│           └── Formulaire de filtrage avec:
│               ├── Date range picker (jQuery UI)
│               ├── Department selector
│               └── Report type selector
```

**Fichiers Impliqués:**
- Java: ReportPortlet.java, ReportService.java
- XML: web.xml, portlet.xml
- JSP: report-list.jsp, report-filters.jspf (À CRÉER)
- JS: jquery-ui (date picker externe)

---

### 5.2 Génération de Rapport (avec Filtres)

```
Flux: Utilisateur soumet les filtres
│
├── report-filters.jspf
│   └── <form id="reportForm" method="POST" action="...">
│       ├── <input id="startDate">
│       ├── <select id="department">
│       └── <select id="reportType">
│
├── JavaScript: report-filters.js
│   ├── on("click", "#generateBtn", function() {
│   │   ├── Valide les filtres
│   │   ├── $.ajax({
│   │   │   ├── url: "/portal/api/reports/generate",
│   │   │   ├── type: "POST",
│   │   │   ├── data: {
│   │   │   │   ├── reportId: reportId,
│   │   │   │   ├── startDate: startDate,
│   │   │   │   ├── department: department,
│   │   │   │   ├── reportType: reportType
│   │   │   │ }
│   │   │   └── success: function(response) {
│   │   │       └── Affiche message "Generating..."
│   │   │   }
│   │   └── })
│   │
│   └── Polling: checkReportStatus(reportId) every 2s
│       ├── $.ajax({
│       │   ├── url: "/portal/api/reports/status",
│       │   ├── data: {reportId: reportId}
│       │   └── success: function(response) {
│       │       ├── if status == "COMPLETE"
│       │       │   └── Enable download button
│       │       └── if status == "ERROR"
│       │           └── Show error message
│       └── })
│
└── BackEnd: ReportGenerationServlet (À CRÉER)
    ├── reportId = request.getParameter("reportId")
    ├── filters = parseFilters(request)
    ├── AsyncTask.execute() {
    │   ├── reportService.generateReport(reportId, filters)
    │   │   ├── ReportGenerator.generate(report, filters)
    │   │   └── DatabaseConnector.saveReport()
    │   │
    │   └── Cache result with status = "COMPLETE"
    └── }
```

**Status Polling:**

```
ReportStatusServlet (à ajouter à web.xml)
├── reportId = request.getParameter("reportId")
├── status = cache.getStatus(reportId)
└── returns: {status: "PENDING|GENERATING|COMPLETE|ERROR", progress: 45}
```

**Fichiers Impliqués:**
- Java: ReportPortlet.java, ReportService.java, ReportGenerator.java
- Java: ReportGenerationServlet.java (À CRÉER), ReportStatusServlet.java (À CRÉER)
- XML: web.xml (nouvelles servlets), portlet.xml
- JSP: report-list.jsp, report-filters.jspf
- JS: report-filters.js (À CRÉER), report-polling.js (À CRÉER)

---

### 5.3 Export et Téléchargement de Rapport

```
Flux: Utilisateur clique "Download as PDF"
│
├── report-list.js
│   └── on("click", ".btn-download", function() {
│       ├── format = $(this).data("format") // PDF, Excel, CSV
│       ├── reportId = $(this).data("id")
│       │
│       ├── $.ajax({
│       │   ├── url: "/portal/api/reports/export",
│       │   ├── type: "POST",
│       │   ├── data: {
│       │   │   ├── reportId: reportId,
│       │   │   ├── format: format
│       │   │ }
│       │   └── success: function(response) {
│       │       ├── downloadUrl = response.downloadUrl
│       │       └── window.location = downloadUrl
│       └── })
│
└── BackEnd: ReportExportServlet (À CRÉER) ou ReportPortlet.serveResource()
    ├── reportId = request.getParameter("reportId")
    ├── format = request.getParameter("format")
    ├── ReportService.exportReport(reportId, format)
    │   └── ExportManager.export(report, format)
    │       ├── switch(format) {
    │       │   ├── case "PDF": generatePdf()
    │       │   ├── case "EXCEL": generateExcel()
    │       │   └── case "CSV": generateCsv()
    │       │ }
    │       └── returns: byte[]
    │
    ├── response.setContentType("application/pdf")
    ├── response.setHeader("Content-Disposition", "attachment; filename=...")
    └── response.getOutputStream().write(bytes)
```

**Fichiers Impliqués:**
- Java: ReportPortlet.java, ReportService.java, ReportExportServlet.java (À CRÉER)
- Java: ExportManager.java
- XML: web.xml (ReportExportServlet)
- JSP: report-list.jsp
- JS: report-list.js (À CRÉER)

**Relations:**
- `ReportPortlet` → `ReportService` (call)
- `ReportService` → `ReportGenerator` (call for generation)
- `ReportService` → `ExportManager` (call for export)
- `report-list.jsp` → `report-list.js` (include script)
- `report-list.js` → `/portal/api/reports/generate` (AJAX POST)
- `report-list.js` → `/portal/api/reports/status` (AJAX GET polling)
- `report-list.js` → `/portal/api/reports/export` (AJAX POST)
- `ReportExportServlet` → `ExportManager` (call)

---

## 6. USECASE: Authentication & Security (Authentification & Sécurité)

### 6.1 Login Page

```
Flux: HTTP GET /portal/login
│
├── LoginServlet (À CRÉER - ajouter à web.xml)
│   └── dispatch: /WEB-INF/pages/login.jsp
│
└── login.jsp
    ├── <form id="loginForm" method="POST" action="/portal/login">
    ├── <input id="username">
    ├── <input id="password">
    ├── <button id="loginBtn">
    │
    └── JavaScript: login.js (À CRÉER)
        ├── on("click", "#loginBtn", function() {
        │   ├── $.ajax({
        │   │   ├── url: "/portal/login",
        │   │   ├── type: "POST",
        │   │   ├── data: {
        │   │   │   ├── username: $("#username").val(),
        │   │   │   ├── password: $("#password").val()
        │   │   │ }
        │   │   └── success: function() {
        │   │       └── window.location = "/portal"
        │   │   }
        │   └── })
        └── on("input", ".password-field", function() {
            └── Check password strength (client-side feedback)
```

### 6.2 Session Management (avec AuthenticationFilter)

```
AuthenticationFilter (web.xml)
├── doFilter() {
│   ├── if (path.contains("login")) {
│   │   └── chain.doFilter() // Allow
│   │
│   └── if (session.getAttribute("user") == null) {
│       └── redirect to /portal/login
│   }
```

**Fichiers Impliqués:**
- Java: LoginServlet.java (À CRÉER), AuthenticationFilter.java
- Java: SessionListener.java (tracking active sessions)
- XML: web.xml (LoginServlet mapping, AuthenticationFilter)
- JSP: login.jsp (À CRÉER)
- JS: login.js (À CRÉER)

---

## Résumé des Fichiers À Créer

### Fichiers JSP (8)
- [ ] user-list.jsp
- [ ] user-edit.jsp
- [ ] user-row.jspf
- [ ] document-list.jsp
- [ ] document-upload.jsp
- [ ] dashboard.jsp
- [ ] workflow-list.jsp
- [ ] report-list.jsp

### Fichiers JavaScript (10)
- [ ] user-form-validator.js
- [ ] document-list.js
- [ ] document-upload.js
- [ ] dashboard-refresh.js
- [ ] workflow-actions.js
- [ ] report-filters.js
- [ ] report-polling.js
- [ ] report-list.js
- [ ] login.js
- [ ] common/utils.js (utilities partagées)

### Fichiers Java (6)
- [ ] ValidationService.java
- [ ] LoginServlet.java
- [ ] ReportGenerationServlet.java
- [ ] ReportStatusServlet.java
- [ ] ReportExportServlet.java
- [ ] ReportPollingHelper.java (async task management)

### Mise à Jour XML (web.xml)
- [ ] Ajouter LoginServlet
- [ ] Ajouter ValidationServlet
- [ ] Ajouter ReportGenerationServlet
- [ ] Ajouter ReportStatusServlet
- [ ] Ajouter ReportExportServlet
- [ ] Ajouter mappings JavaScript resources (/resources/js/*)

---

## Relations Globales Identifiées

### Relations Portlet ↔ Service
```
UserManagementPortlet ↔ UserService
DocumentPortlet ↔ DocumentService
DashboardPortlet ↔ DashboardService
WorkflowPortlet ↔ WorkflowService
ReportPortlet ↔ ReportService
```

### Relations Service ↔ Storage/External
```
UserService → DatabaseConnector
DocumentService → DatabaseConnector + FileStorageManager
DashboardService → MetricsCollector + DatabaseConnector
WorkflowService → DatabaseConnector + NotificationService
ReportService → ReportGenerator + ExportManager + DatabaseConnector
```

### Relations JSP ↔ JavaScript
```
user-list.jsp → user-form-validator.js
document-list.jsp → document-list.js + document-upload.js
dashboard.jsp → dashboard-refresh.js
workflow-list.jsp → workflow-actions.js
report-list.jsp → report-filters.js + report-polling.js + report-list.js
login.jsp → login.js
```

### Relations JavaScript ↔ Servlets
```
user-form-validator.js → ValidationServlet
document-upload.js → DocumentPortlet.processAction
dashboard-refresh.js → DashboardPortlet.serveResource()
workflow-actions.js → WorkflowPortlet.processAction
report-filters.js → ReportGenerationServlet
report-polling.js → ReportStatusServlet
report-list.js → ReportExportServlet
login.js → LoginServlet
```

### Relations Servlets ↔ Services
```
ValidationServlet → ValidationService
LoginServlet → AuthenticationService (À CRÉER)
ReportGenerationServlet → ReportService
ReportStatusServlet → ReportService
ReportExportServlet → ReportService + ExportManager
```

### Relations Filter ↔ Portlets/Servlets
```
AuthenticationFilter → ALL Portlets & Servlets
LoggingFilter → ALL Portlets & Servlets
```

---

## Statistiques Complètes

| Catégorie | Actuel | À Créer | Total |
|-----------|--------|---------|-------|
| Java Files | 25 | 7 | 32 |
| JSP Files | 0 | 8 | 8 |
| JavaScript Files | 0 | 10 | 10 |
| XML Files | 2 | 0 | 2 |
| Config Files | 2 | 0 | 2 |
| **Total** | **29** | **25** | **54** |

### Lignes de Code (estimé)
- Java existant: 1,886 lignes
- Java à ajouter: ~500 lignes
- JSP: ~600 lignes
- JavaScript: ~1,500 lignes
- **Total**: ~4,500 lignes

