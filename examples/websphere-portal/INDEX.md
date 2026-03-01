# WebSphere Portal - Index des Fichiers

## Structure Rapide

```
websphere-portal/
├── README.md              # Vue d'ensemble
├── ANALYSIS.md            # Analyse détaillée dépendances & appels
├── TESTING_GUIDE.md       # Guide complet de test
├── INDEX.md               # Ce fichier
├── web.xml                # Descripteur application web
├── portlet.xml            # Descripteur portlets
├── application-context.xml # Configuration Spring
├── application.properties  # Propriétés application
└── java/                  # 25 fichiers Java
```

## Fichiers Java par Catégorie

### Portlets (5)
- [BasePortlet.java](java/BasePortlet.java) - Classe de base abstraite
- [UserManagementPortlet.java](java/UserManagementPortlet.java) - Gestion utilisateurs
- [DocumentPortlet.java](java/DocumentPortlet.java) - Gestion documents
- [DashboardPortlet.java](java/DashboardPortlet.java) - Tableau de bord
- [WorkflowPortlet.java](java/WorkflowPortlet.java) - Workflows
- [ReportPortlet.java](java/ReportPortlet.java) - Rapports

### Services Métier (5)
- [UserService.java](java/UserService.java) - Logique utilisateurs
- [DocumentService.java](java/DocumentService.java) - Logique documents
- [DashboardService.java](java/DashboardService.java) - Logique dashboard
- [WorkflowService.java](java/WorkflowService.java) - Logique workflows
- [ReportService.java](java/ReportService.java) - Logique rapports

### Modèles de Données (5)
- [User.java](java/User.java) - Modèle User
- [Document.java](java/Document.java) - Modèle Document
- [WorkflowTask.java](java/WorkflowTask.java) - Modèle WorkflowTask
- [WorkflowProcess.java](java/WorkflowProcess.java) - Modèle WorkflowProcess
- [Report.java](java/Report.java) - Modèle Report

### Composants Web (2)
- [DispatcherServlet.java](java/DispatcherServlet.java) - Routage principal
- [ResourceServlet.java](java/ResourceServlet.java) - Ressources statiques

### Filtres (2)
- [AuthenticationFilter.java](java/AuthenticationFilter.java) - Authentification
- [LoggingFilter.java](java/LoggingFilter.java) - Logging requêtes

### Listeners (2)
- [PortalStartupListener.java](java/PortalStartupListener.java) - Initialisation
- [SessionListener.java](java/SessionListener.java) - Gestion sessions

### Utilitaires (6)
- [DatabaseConnector.java](java/DatabaseConnector.java) - Accès BD
- [FileStorageManager.java](java/FileStorageManager.java) - Gestion fichiers
- [MetricsCollector.java](java/MetricsCollector.java) - Collecte statistiques
- [NotificationService.java](java/NotificationService.java) - Notifications
- [ReportGenerator.java](java/ReportGenerator.java) - Génération rapports
- [ExportManager.java](java/ExportManager.java) - Export formats

### Configuration XML (2)
- [web.xml](web.xml) - Configuration web application
- [portlet.xml](portlet.xml) - Configuration portlets

### Configuration Spring (2)
- [application-context.xml](application-context.xml) - Beans Spring
- [application.properties](application.properties) - Propriétés

## Par Cas d'Usage

### Gestion Utilisateurs
- **Portlet**: UserManagementPortlet
- **Service**: UserService
- **Modèle**: User
- **Config**: portlet.xml (UserManagementPortlet)

Flux: UserManagementPortlet → UserService → DatabaseConnector

### Gestion Documents
- **Portlet**: DocumentPortlet
- **Service**: DocumentService
- **Modèle**: Document
- **Utilitaire**: FileStorageManager
- **Config**: portlet.xml (DocumentPortlet)

Flux: DocumentPortlet → DocumentService → FileStorageManager + DatabaseConnector

### Tableau de Bord
- **Portlet**: DashboardPortlet
- **Service**: DashboardService
- **Utilitaire**: MetricsCollector
- **Config**: portlet.xml (DashboardPortlet)

Flux: DashboardPortlet → DashboardService → MetricsCollector + DatabaseConnector

### Workflows
- **Portlet**: WorkflowPortlet
- **Service**: WorkflowService
- **Modèles**: WorkflowTask, WorkflowProcess
- **Utilitaire**: NotificationService
- **Config**: portlet.xml (WorkflowPortlet)

Flux: WorkflowPortlet → WorkflowService → DatabaseConnector + NotificationService

### Rapports
- **Portlet**: ReportPortlet
- **Service**: ReportService
- **Modèle**: Report
- **Utilitaires**: ReportGenerator, ExportManager
- **Config**: portlet.xml (ReportPortlet)

Flux: ReportPortlet → ReportService → ReportGenerator + ExportManager

## Patterns et Constructs Clés

### Héritage
```
GenericPortlet (javax.portlet)
  └── BasePortlet
       ├── UserManagementPortlet
       ├── DocumentPortlet
       ├── DashboardPortlet
       ├── WorkflowPortlet
       └── ReportPortlet

HttpServlet (javax.servlet)
  ├── DispatcherServlet
  └── ResourceServlet
```

### Implémentations
```
Filter
  ├── AuthenticationFilter
  └── LoggingFilter

ServletContextListener
  └── PortalStartupListener

HttpSessionListener
  └── SessionListener
```

### Collections
- `List<User>` dans UserService
- `List<Document>` dans DocumentService
- `List<WorkflowTask>` dans WorkflowService
- `List<Report>` dans ReportService
- `Map<String, User>` dans UserService
- `Map<String, Document>` dans DocumentService
- `Map<String, WorkflowTask>` dans WorkflowService
- `Map<String, WorkflowProcess>` dans WorkflowService
- `Map<String, Object>` pour résultats

### Exceptions
- `throws PortletException, IOException`
- `throws ServletException, IOException`
- `throws Exception`
- `try/catch (Exception e)`
- `throw new RuntimeException()`

### Annotations
- `@Override` sur méthodes surchargées

### Signatures Complexes
- `List<User> getAllUsers(int pageSize)`
- `void processAction(ActionRequest request, ActionResponse response)`
- `void doFilter(ServletRequest request, ServletResponse response, FilterChain chain)`
- `byte[] export(Object report, String format)`
- `Map<String, Object> getStatistics()`

## Statistiques du Jeu de Tests

| Métrique | Valeur |
|----------|--------|
| Fichiers Java | 25 |
| Fichiers XML | 2 |
| Fichiers Config | 2 |
| Fichiers Doc | 4 |
| **Total** | **33** |
| Classes | 21 |
| Interfaces | 3 |
| Abstract Classes | 1 |
| Methods | ~220 |
| Fields | ~150 |
| Lines of Code | ~2000 |

## Recommandations de Lecture

1. **Pour comprendre la structure**:
   - Commencez par [README.md](README.md)
   - Puis [ANALYSIS.md](ANALYSIS.md)

2. **Pour tester votre parseur**:
   - Lisez [TESTING_GUIDE.md](TESTING_GUIDE.md)
   - Validez les cas d'usage un par un

3. **Pour implémenter le parseur**:
   - Analysez [BasePortlet.java](java/BasePortlet.java) (classe parente)
   - Puis [UserManagementPortlet.java](java/UserManagementPortlet.java) (dépendances simples)
   - Puis [DashboardPortlet.java](java/DashboardPortlet.java) (plus complexe)
   - Enfin [WorkflowService.java](java/WorkflowService.java) (logique complexe)

4. **Pour les configurations**:
   - [web.xml](web.xml) pour servlets et filtres
   - [portlet.xml](portlet.xml) pour portlets
   - [application-context.xml](application-context.xml) pour beans Spring

## Fichiers Clés pour Chaque Cas

### Test 1: Extraction Basique
```
java/BasePortlet.java
java/UserManagementPortlet.java
java/User.java
```

### Test 2: Héritage & Interfaces
```
java/BasePortlet.java
java/AuthenticationFilter.java
java/PortalStartupListener.java
```

### Test 3: Services & Dépendances
```
java/UserManagementPortlet.java
java/UserService.java
java/DatabaseConnector.java
```

### Test 4: Collections & Génériques
```
java/UserService.java
java/DocumentService.java
java/DashboardService.java
java/WorkflowService.java
```

### Test 5: Ressources & AJAX
```
java/DashboardPortlet.java
```

### Test 6: Appels en Chaîne
```
java/WorkflowPortlet.java
java/WorkflowService.java
java/NotificationService.java
java/DatabaseConnector.java
```

### Test 7: Export & Format
```
java/ReportPortlet.java
java/ReportService.java
java/ExportManager.java
```

## Validation Rapide

Pour vérifier que votre parseur fonctionne correctement:

```bash
# 1. Compter les classes
cargo run -- examples/websphere-portal/java | grep "Classes: 25"

# 2. Vérifier héritage
cargo run -- examples/websphere-portal/java | grep "BasePortlet"

# 3. Vérifier appels
cargo run -- examples/websphere-portal/java | grep "UserService.saveUser"

# 4. Vérifier interfaces
cargo run -- examples/websphere-portal/java | grep "implements Filter"
```

## Prochaines Étapes

Après validation avec ce jeu de tests:

1. **Étendre le test**:
   - Ajouter JSPs
   - Ajouter scripts SQL
   - Ajouter DTDs

2. **Tester d'autres patterns**:
   - Annotations (Spring, Portlet)
   - Reflection
   - Dynamic proxies

3. **Valider la qualité**:
   - Vérifier les qualified IDs
   - Analyser les call chains
   - Mesurer la couverture

## Contact & Support

Pour des questions sur ce jeu de tests, consultez:
- [ANALYSIS.md](ANALYSIS.md) - Analyse détaillée
- [TESTING_GUIDE.md](TESTING_GUIDE.md) - Guide de test
- [README.md](README.md) - Vue d'ensemble

---

**Dernière mise à jour**: 2026-01-13  
**Version**: 1.0  
**Cas d'usage couverts**: 5 (Users, Documents, Dashboard, Workflows, Reports)
