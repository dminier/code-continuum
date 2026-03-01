# Usecase 3: Dashboard Portal with AJAX Real-time Updates

## Vue d'Ensemble
Tableau de bord avec statistiques en temps réel, rafraîchissement AJAX et métriques système.

## Architecture Générale

```
┌─────────────────────────────────────────────────────────┐
│                  FRONTEND (Browser)                     │
├─────────────────────────────────────────────────────────┤
│  HTML/JSP                 │ JavaScript                  │
├──────────────────────────────────────────────────────────┤
│ dashboard.jsp            │ dashboard-widgets.js        │
│ dashboard-widget.jspf    │ metrics-service.js          │
│ chart-template.jspf      │ chart-renderer.js           │
│                          │ ajax-poller.js              │
└──────────┬──────────────────────────┬──────────────────┘
           │                          │
           │ HTTP GET (init)          │ AJAX polling
           │                          │ every 30-60 seconds
           │                          │
┌──────────▼──────────────────────────▼──────────────────┐
│             PORTLET (Server-side)                      │
├───────────────────────────────────────────────────────┤
│ DashboardPortlet.java                                │
│  - doView(): Affiche dashboard skeleton              │
│  - serveResource(): AJAX data endpoints              │
│    * resourceID=refreshStats: JSON metrics           │
│    * resourceID=chartData: JSON chart data           │
│    * resourceID=activities: JSON recent activities   │
└──────────┬───────────────────────────────────────────┘
           │
           │ Appel direct
           │
┌──────────▼───────────────────────────────────────────┐
│             SERVICE LAYER                             │
├───────────────────────────────────────────────────────┤
│ DashboardService.java                               │
│  - getStatistics(): Map<String, Object>             │
│  - getRecentActivities(): Map<String, Object>       │
│  - getSystemHealth(): Map<String, Object>           │
│                                                     │
│ MetricsCollector.java                              │
│  - getTotalUsers()                                  │
│  - getActiveUsers()                                 │
│  - getPendingTasks()                                │
│  - getCpuUsage()                                    │
│  - getMemoryUsage()                                 │
└──────────┬───────────────────────────────────────────┘
           │
           │ Appel direct
           │
┌──────────▼───────────────────────────────────────────┐
│             DATA LAYER                                │
├───────────────────────────────────────────────────────┤
│ DatabaseConnector.java                              │
│  - getRecentUploads(limit)                          │
│  - getRecentApprovals(limit)                        │
│  - getRecentLogins(limit)                           │
└───────────────────────────────────────────────────────┘
```

## Flux Détaillés

### Flux 1: Chargement Initial Dashboard

```
1. Browser GET /portal/page?portletName=DashboardPortlet
   ↓
2. DashboardPortlet.doView()
   ├─ getInitParameter("refresh-interval") → 60 secondes
   ├─ Retourne dashboard.jsp skeleton
   └─ Pas d'appels service initiaux (lazy load)
   ↓
3. dashboard.jsp rendu au Browser
   ├─ HTML structure des widgets vides
   ├─ Placeholder spinner/loading states
   ├─ Inclut dashboard-widgets.js
   ├─ Inclut metrics-service.js
   ├─ Inclut chart-renderer.js
   ├─ Inclut ajax-poller.js
   └─ Script initialisation
   ↓
4. Browser exécute dashboard-widgets.js
   ├─ initDashboard()
   ├─ setupRefreshInterval(60000) // 60 secondes
   ├─ triggerInitialLoad()
   └─ attachEventListeners()
```

### Flux 2: Rafraîchissement Statistiques (AJAX Polling)

```
1. dashboard-widgets.js timer tick (every 60 sec)
   ↓
2. metrics-service.js: fetchStatistics()
   ├─ AJAX GET /portal/page?resourceID=refreshStats
   ├─ Content-Type: application/json
   └─ No-cache header
   ↓
3. DashboardPortlet.serveResource()
   ├─ resourceID = "refreshStats"
   ├─ response.setContentType("application/json")
   └─ response.setHeader("Cache-Control", "no-cache")
   ↓
4. DashboardService.getStatistics()
   ├─ MetricsCollector.getTotalUsers() → 150
   ├─ MetricsCollector.getActiveUsers() → 85
   ├─ MetricsCollector.getPendingTasks() → 23
   ├─ MetricsCollector.getCompletedProcesses() → 542
   ├─ MetricsCollector.getDocumentsCount() → 1203
   ├─ MetricsCollector.getStorageUsed() → 2560
   └─ Retour Map<String, Object>
   ↓
5. Portlet convertit Map → JSON
   ```json
   {
     "totalUsers": 150,
     "activeUsers": 85,
     "pendingTasks": 23,
     "completedProcesses": 542,
     "documentsCount": 1203,
     "storageUsed": 2560,
     "timestamp": 1705165422000
   }
   ```
   ↓
6. Browser reçoit JSON response
   ├─ metrics-service.js: parseResponse(json)
   ├─ dashboard-widgets.js: updateMetrics(data)
   ├─ Update DOM elements
   └─ Affiche animations/transitions
```

### Flux 3: Chargement Chart Data (AJAX)

```
1. User clique sur [View Details] pour un chart
   ↓
2. dashboard-widgets.js: onChartClick(chartType)
   ├─ chartType = "users_over_time" | "tasks_distribution" | etc.
   └─ metrics-service.js: fetchChartData(chartType)
   ↓
3. AJAX GET /portal/page?resourceID=chartData&chartType=users_over_time
   ↓
4. DashboardPortlet.serveResource()
   ├─ resourceID = "chartData"
   ├─ chartType from request parameter
   ├─ response.setContentType("application/json")
   └─ response.setHeader("Cache-Control", "max-age=300") // 5 min cache
   ↓
5. DashboardService.getChartData(chartType)
   ├─ DatabaseConnector.getChartData(chartType)
   ├─ Retour List<{label, value}> ou time-series
   └─ Exemple:
      [{label: "Jan", value: 120}, {label: "Feb", value: 135}, ...]
   ↓
6. Portlet convertit → JSON array
   ↓
7. Browser reçoit JSON
   ├─ chart-renderer.js: renderChart(data, chartType)
   ├─ Utilise Chart.js ou D3.js pour rendu
   ├─ Affiche chart modal
   └─ Animate transitions
```

### Flux 4: Activités Récentes (Lazy Load)

```
1. User ouvre section "Recent Activities"
   ↓
2. dashboard-widgets.js: onActivitiesExpand()
   ├─ Check si déjà chargé
   └─ Si non: metrics-service.js: fetchActivities()
   ↓
3. AJAX GET /portal/page?resourceID=activities&limit=10
   ↓
4. DashboardPortlet.serveResource()
   ├─ resourceID = "activities"
   ├─ limit = 10 (from request)
   ├─ response.setContentType("application/json")
   └─ response.setHeader("Cache-Control", "no-cache")
   ↓
5. DashboardService.getRecentActivities()
   ├─ DatabaseConnector.getRecentUploads(10)
   │  └─ Retour List<{user, action, file, timestamp}>
   ├─ DatabaseConnector.getRecentApprovals(10)
   │  └─ Retour List<{user, action, task, timestamp}>
   ├─ DatabaseConnector.getRecentLogins(10)
   │  └─ Retour List<{user, loginTime, ipAddress}>
   └─ Merge et sort by timestamp
   ↓
6. Portlet convertit → JSON
   ```json
   [
     {"timestamp": 1705165400, "user": "john.doe", "action": "upload", 
      "resource": "report.pdf"},
     {"timestamp": 1705165350, "user": "jane.smith", "action": "approve", 
      "resource": "Task #123"},
     ...
   ]
   ```
   ↓
7. Browser reçoit JSON
   ├─ dashboard-widgets.js: populateActivities(data)
   ├─ Utilise dashboard-widget.jspf template
   ├─ Rend liste HTML
   └─ Affiche avec animations
```

### Flux 5: Santé Système (Optional)

```
1. User clique [System Health] badge
   ↓
2. metrics-service.js: fetchSystemHealth()
   ├─ AJAX GET /portal/page?resourceID=systemHealth
   └─ Content-Type: application/json
   ↓
3. DashboardPortlet.serveResource()
   ├─ resourceID = "systemHealth"
   └─ Cache-Control: max-age=60 (1 min cache)
   ↓
4. DashboardService.getSystemHealth()
   ├─ MetricsCollector.getDatabaseStatus() → "HEALTHY"
   ├─ MetricsCollector.getFileSystemStatus() → "HEALTHY"
   ├─ MetricsCollector.getCpuUsage() → 45.3 %
   ├─ MetricsCollector.getMemoryUsage() → 62.1 %
   └─ Retour Map<String, Object>
   ↓
5. Browser reçoit JSON
   ├─ dashboard-widgets.js: updateSystemHealth(data)
   ├─ Update health indicators
   ├─ Change colors based on thresholds
   └─ Show warnings if needed
```

## Fichiers Impliqués

### Java
- **DashboardPortlet.java**
  - Classe: `com.example.portlets.DashboardPortlet extends BasePortlet`
  - Méthodes: `doView()`, `serveResource()`
  - serveResource resourceIDs: refreshStats, chartData, activities, systemHealth

- **DashboardService.java**
  - Classe: `com.example.services.DashboardService`
  - Méthodes: `getStatistics()`, `getRecentActivities()`, `getSystemHealth()`, `getChartData()`

- **MetricsCollector.java**
  - Classe: `com.example.services.MetricsCollector`
  - Méthodes: ~8 getters pour différentes métriques

### JSP/JSPF
- **dashboard.jsp** - Layout principal avec widgets
- **dashboard-widget.jspf** - Fragment pour un widget
- **chart-template.jspf** - Template pour chart rendering

### JavaScript
- **dashboard-widgets.js** - Orchestration widgets
  - `initDashboard()`
  - `setupRefreshInterval(interval)`
  - `updateMetrics(data)`
  - `onChartClick(chartType)`
  - `onActivitiesExpand()`

- **metrics-service.js** - Service data fetching
  - `fetchStatistics()`
  - `fetchChartData(chartType)`
  - `fetchActivities(limit)`
  - `fetchSystemHealth()`
  - `parseResponse(json)`

- **chart-renderer.js** - Chart rendering
  - `renderChart(data, type)`
  - `renderActivityList(data)`
  - `updateChart(data, selector)`

- **ajax-poller.js** - Polling gestion
  - `startPolling(interval, callback)`
  - `stopPolling()`
  - `resetPollingInterval()`

### Configuration
- **portlet.xml**
  ```xml
  <portlet>
    <portlet-name>DashboardPortlet</portlet-name>
    <init-param>
      <name>refresh-interval</name>
      <value>60</value>
    </init-param>
    <expiration-cache>0</expiration-cache>
  </portlet>
  ```

## Relations Sémantiques Clés

```
PORTLET_USES_JSP:
  DashboardPortlet --renders--> dashboard.jsp
  dashboard.jsp --includes--> dashboard-widget.jspf
  dashboard.jsp --includes--> chart-template.jspf

JSP_INCLUDES_JS:
  dashboard.jsp --includes--> dashboard-widgets.js
  dashboard.jsp --includes--> metrics-service.js
  dashboard.jsp --includes--> chart-renderer.js
  dashboard.jsp --includes--> ajax-poller.js

JS_CALLS_PORTLET:
  dashboard-widgets.js --ajax-get--> DashboardPortlet.serveResource()
  metrics-service.js --ajax-get--> DashboardPortlet.serveResource()
  ajax-poller.js --ajax-get--> DashboardPortlet.serveResource()

AJAX_ENDPOINTS (serveResource):
  resourceID=refreshStats --calls--> DashboardService.getStatistics()
  resourceID=chartData --calls--> DashboardService.getChartData()
  resourceID=activities --calls--> DashboardService.getRecentActivities()
  resourceID=systemHealth --calls--> DashboardService.getSystemHealth()

SERVICE_CALLS_METRICS:
  DashboardService --calls--> MetricsCollector (multiple methods)
  MetricsCollector --calls--> DatabaseConnector (for DB metrics)

PORTLET_READS_CONFIG:
  DashboardPortlet --reads--> portlet.xml [refresh-interval]

JS_POLLING_PATTERN:
  ajax-poller.js --starts--> dashboard-widgets.js
  dashboard-widgets.js --periodic-calls--> metrics-service.js
  metrics-service.js --ajax-requests--> DashboardPortlet.serveResource()
```

## Paramètres Critiques

| Paramètre | Source | Valeur | Portée |
|-----------|--------|--------|--------|
| refresh-interval | portlet.xml | 60 sec | dashboard-widgets.js polling |
| chartType | HTTP Query | "users_over_time" | DashboardService.getChartData() |
| limit | HTTP Query | 10 | DatabaseConnector.getRecentActivities() |
| resourceID | HTTP Query | "refreshStats" | Routing in serveResource() |

## Points de Test Critiques

1. **Multiple AJAX endpoints**
   - serveResource avec différents resourceIDs
   - Routing en fonction du resourceID
   - JSON serialization

2. **Polling pattern**
   - Timer callback
   - Interval management
   - Cache-Control headers

3. **Lazy loading**
   - Données chargées on-demand
   - Cache côté client (JavaScript)
   - DOM updates sans refresh

4. **Real-time metrics**
   - MetricsCollector appels
   - System resource monitoring
   - Health checks

5. **Chart data handling**
   - Data transformation
   - Different chart types
   - Caching strategies

## Métadonnées Attendues

```
Nœuds (Nodes):
- 1 Portlet (DashboardPortlet)
- 1 Service (DashboardService)
- 1 Collector (MetricsCollector)
- 1 DAO (DatabaseConnector)
- 1 JSP (dashboard.jsp)
- 2 Fragments (dashboard-widget.jspf, chart-template.jspf)
- 4 JS files (dashboard-widgets, metrics-service, chart-renderer, ajax-poller)
Total: ~13 nœuds

Relations (Edges):
- Portlet USES JSP: 1
- Portlet CALLS Service: 1
- JSP INCLUDES Fragment: 2
- JSP INCLUDES JS: 4
- JS CALLS Portlet: 3 (multiple serveResource calls)
- Service CALLS Metrics: 1
- Service CALLS DAO: 1
- AJAX endpoints: 4
- JS POLLING: 1
Total: ~18 arêtes
```

---

**Status**: Draft
**Complexity**: Medium-High (polling, multiple AJAX endpoints, lazy loading)
**Key Feature**: Real-time updates sans page refresh
**Prochaine étape**: Créer JSP, JS et implémenter polling logic
