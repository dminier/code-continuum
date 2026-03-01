# Roadmap: Enrichissement du Jeu de Tests WebSphere Portal

## Phase 1: COMPLETED ✅ (Usecase Documentation)

### 5 Usecase Files Créés:
- ✅ USECASE_01_USER_MANAGEMENT.md (400+ lignes)
- ✅ USECASE_02_DOCUMENT_MANAGEMENT.md (400+ lignes)
- ✅ USECASE_03_DASHBOARD.md (350+ lignes)
- ✅ USECASE_04_WORKFLOW.md (200+ lignes)
- ✅ USECASE_05_REPORTS.md (250+ lignes)

**Total**: ~1600 lignes de documentation d'architecture

### Contenus documentés:
- Architecture générale (diagrammes ASCII)
- Flux détaillés par scénario
- Fichiers impliqués (Java, JSP, JS)
- Relations sémantiques
- Configuration XML
- Métadonnées attendues

---

## Phase 2: PLANNED 🔄 (JSP/Fragment Templates)

### À Créer:

#### User Management
```
WEB-INF/portlets/
├── user-list.jsp          # 150 lignes
├── user-edit.jsp          # 150 lignes
├── user-detail.jsp        # 100 lignes
└── fragments/
    └── user-row.jspf      # 50 lignes
```

#### Document Management
```
WEB-INF/portlets/
├── document-list.jsp      # 150 lignes
├── document-upload.jsp    # 200 lignes (form multipart)
├── document-detail.jsp    # 100 lignes
├── document-search.jsp    # 100 lignes
└── fragments/
    └── document-item.jspf # 50 lignes
```

#### Dashboard
```
WEB-INF/portlets/
├── dashboard.jsp          # 200 lignes
├── dashboard-stats.jsp    # 150 lignes
└── fragments/
    ├── dashboard-widget.jspf     # 80 lignes
    └── chart-template.jspf       # 100 lignes
```

#### Workflow
```
WEB-INF/portlets/
├── workflow-list.jsp      # 150 lignes
├── workflow-process.jsp   # 150 lignes
└── fragments/
    └── task-actions.jspf  # 80 lignes
```

#### Reports
```
WEB-INF/portlets/
├── report-list.jsp        # 150 lignes
├── report-detail.jsp      # 150 lignes
└── fragments/
    └── report-filters.jspf # 100 lignes
```

**Sous-total Phase 2**: ~2000 lignes JSP/JSPF

---

## Phase 3: PLANNED 🔄 (JavaScript Files)

### Fichiers Génériques:

```
js/
├── ajax-service.js        # Service AJAX réutilisable
│   - get(url, callback)
│   - post(url, data, callback)
│   - put(url, data, callback)
│   - delete(url, callback)
│   - upload(url, formData, callback)
│   (150 lignes)
│
├── form-validator.js      # Validation côté client
│   - validate(formData)
│   - validateEmail(email)
│   - validateRequired(field)
│   - validateFileSize(size)
│   - validateFileType(type)
│   (200 lignes)
│
└── ui-utils.js            # Utilitaires UI
    - showDialog(title, content)
    - showNotification(message, type)
    - updateDOM(selector, content)
    - debounce(func, delay)
    (150 lignes)
```

### Fichiers Portlet-Spécifiques:

```
js/portlets/
├── user-management.js
│   - onEditClick(userId)
│   - onDeleteClick(userId)
│   - onSaveClick()
│   - populateForm(userData)
│   - refreshUserList()
│   (200 lignes)
│
├── document-manager.js
│   - onDownloadClick(documentId)
│   - onDeleteClick(documentId)
│   - onSearchInput(searchTerm)
│   - refreshDocumentList()
│   - updateStorageInfo()
│   (200 lignes)
│
├── file-uploader.js
│   - onFileSelected(file)
│   - validateFile(file)
│   - uploadFile(file, onProgress)
│   - uploadChunked(file)
│   - onUploadProgress(event)
│   (250 lignes)
│
├── dashboard-widgets.js
│   - initDashboard()
│   - setupRefreshInterval(interval)
│   - updateMetrics(data)
│   - onChartClick(chartType)
│   - onActivitiesExpand()
│   (250 lignes)
│
├── metrics-service.js
│   - fetchStatistics()
│   - fetchChartData(chartType)
│   - fetchActivities(limit)
│   - fetchSystemHealth()
│   - parseResponse(json)
│   (200 lignes)
│
├── chart-renderer.js
│   - renderChart(data, type)
│   - renderActivityList(data)
│   - updateChart(data, selector)
│   (200 lignes)
│
├── ajax-poller.js
│   - startPolling(interval, callback)
│   - stopPolling()
│   - resetPollingInterval()
│   (150 lignes)
│
├── workflow-manager.js
│   - onApproveClick(taskId)
│   - onRejectClick(taskId)
│   - onDelegateClick(taskId)
│   - refreshTaskList()
│   (200 lignes)
│
├── task-form.js
│   - showApprovalDialog(taskId)
│   - showRejectionDialog(taskId)
│   - showDelegationDialog(taskId)
│   (150 lignes)
│
├── report-manager.js
│   - onGenerateClick(reportId, filters)
│   - onExportClick(reportId, format)
│   - onDownloadClick(reportId, format)
│   - validateFormat(format)
│   (200 lignes)
│
└── filter-builder.js
    - buildFilterString(formData)
    - validateFilters(filters)
    (100 lignes)
```

**Sous-total Phase 3**: ~2500 lignes JavaScript

---

## Phase 4: PLANNED 🔄 (Fichiers Support)

### HTML Templates & Static:
```
html/
├── layout.html            # Layout principal
├── modals.html            # Templates modales
└── components.html        # Composants réutilisables
(300 lignes)
```

### Fichiers Existants à Enrichir:
```
├── java/*.java            # Déjà créés (1886 lignes)
├── web.xml                # Déjà créé
├── portlet.xml            # Déjà créé
├── application-context.xml # Déjà créé
└── application.properties  # Déjà créé
```

---

## Phase 5: PLANNED 🔄 (Améliorations Intégration)

### Ajouter au Modèle Sémantique:

1. **Relation JSP → JS**
   ```
   MATCH (jsp:JSPPage {path: "user-list.jsp"})
   -[:INCLUDES]-> (js:JavaScript)
   RETURN jsp, js
   ```

2. **Relation JS → AJAX**
   ```
   MATCH (js:JavaScript {file: "user-manager.js"})
   -[:AJAX_CALLS]-> (endpoint:AjaxEndpoint)
   RETURN js, endpoint
   ```

3. **Relation AJAX → Service**
   ```
   MATCH (endpoint:AjaxEndpoint {resourceID: "getUser"})
   -[:MAPS_TO]-> (method:Method)
   <-[:DEFINED_IN]- (service:Service)
   RETURN endpoint, method, service
   ```

4. **Configuration Mapping**
   ```
   MATCH (config:XMLConfig {file: "portlet.xml"})
   -[:CONFIGURES]-> (portlet:Portlet)
   -[:RENDERED_BY]-> (jsp:JSPPage)
   RETURN config, portlet, jsp
   ```

---

## Statistiques Cumulées

| Phase | Fichiers | Lignes Code | Type |
|-------|----------|-------------|------|
| Phase 1 (Actuel) | 7 docs | ~1600 | Markdown |
| Phase 2 (JSP) | 15 fichiers | ~2000 | JSP/JSPF |
| Phase 3 (JS) | 14 fichiers | ~2500 | JavaScript |
| Phase 4 (HTML) | 3 fichiers | ~300 | HTML |
| Phase 5 (Data) | Config files | varies | XML/Props |
| **TOTAL** | **~40-45** | **~6400-7000** | **Multi-type** |

---

## Architecture Complète Après Enrichissement

```
WebSphere Portal Application
├── Java Layer (25 classes, ~1900 lignes)
│   ├── Portlets (5)
│   ├── Services (5)
│   ├── Models (5)
│   ├── Web Components (2)
│   ├── Filters (2)
│   ├── Listeners (2)
│   └── Utilities (6)
│
├── JSP/JSPF Layer (~15 fichiers, ~2000 lignes)
│   ├── Full Pages (10 JSPs)
│   └── Reusable Fragments (5 JSPFs)
│
├── JavaScript Layer (~14 fichiers, ~2500 lignes)
│   ├── Generic Services (3)
│   ├── Portlet-Specific (11)
│   └── AJAX/Polling/Validation
│
├── Configuration Layer (6 fichiers)
│   ├── Descriptors (2: web.xml, portlet.xml)
│   ├── Spring Config (1: application-context.xml)
│   ├── Properties (1: application.properties)
│   ├── HTML Templates (2: layout.html, components.html)
│
└── Documentation (~2000 lignes)
    ├── 5 Usecases détaillés
    ├── Semantic Model
    ├── Analysis & Testing guides
    └── Architecture docs
```

---

## Graph Nodes & Relationships (Après Phase 5)

### Nodes par Type:
```
- Portlet classes: 5
- Service classes: 5
- Model classes: 5
- Servlet classes: 2
- Filter classes: 2
- Listener classes: 2
- Utility classes: 6
- JSP Pages: 10
- JSP Fragments: 5
- JavaScript files: 14
- Methods: ~200
- Properties/Fields: ~150
- Configuration nodes: 6
- AJAX Endpoints: ~10
- HTML Templates: 2

Total Nodes: ~425 nœuds
```

### Relationships (Estimé):
```
- Inheritance (EXTENDS): 7
- Implementation (IMPLEMENTS): 4
- Method calls (CALLS): ~80
- JSP rendering (RENDERED_BY): 10
- JSP includes (INCLUDES): 15
- JS includes (INCLUDED_BY): 10
- AJAX calls (AJAX_CALLS): ~10
- Configuration mapping: 6
- Service dependencies: 15
- DAO calls: 10
- Data flow: ~30

Total Edges: ~200+ arêtes
```

---

## Points Clés à Implémenter

### Parsing Requirements:
1. ✅ Java class extraction (DONE)
2. ⏳ JSP template parsing (directive, includes, EL)
3. ⏳ JavaScript parsing (function def, AJAX patterns)
4. ⏳ XML descriptor parsing (portlet.xml, web.xml)
5. ⏳ Cross-file reference resolution
6. ⏳ AJAX endpoint discovery
7. ⏳ Semantic relationship creation

### Test Coverage:
1. ✅ Java-to-Java relationships
2. ⏳ Java-to-JSP mappings
3. ⏳ JSP-to-JS includes
4. ⏳ JS-to-AJAX calls
5. ⏳ Configuration-to-Runtime
6. ⏳ Complete request flows

---

## Recommandations Implémentation

### Ordre Suggested:
1. Commencer par les usecases existants (Phase 1) ✅
2. Ajouter JSP files minimales (Phase 2)
3. Ajouter JS files pour un usecase (Phase 3 - partial)
4. Implémenter parseur JSP/JS incrémentalement
5. Enrichir relations sémantiques progressivement

### Focus Initial:
- **Usecase 1 (User Management)**
  - JSP: user-list.jsp, user-edit.jsp
  - JS: user-manager.js, form-validator.js
  - Tests: JSP→JS→AJAX→Service chain

### Puis Étendre à:
- Usecase 2 (Document) - ajouter multipart handling
- Usecase 3 (Dashboard) - ajouter polling patterns
- Usecases 4-5 - patterns plus simples

---

**Status**: Feuille de route complète
**Next Action**: Impl Phase 2 (JSP files) pour 1-2 usecases
**Timeline**: Flexible (par increments)
**Complexity**: Medium (parsing JSP/JS) → High (semantic enrichment)
