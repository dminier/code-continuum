# WebSphere Portal Test Suite

Ce répertoire contient un jeu de tests complet pour un portail WebSphere applicatif avec support des Portlets.

## Structure

### Configuration Files
- **`web.xml`**: Web application descriptor
  - Servlets: DispatcherServlet, ResourceServlet
  - Filters: AuthenticationFilter, LoggingFilter
  - Listeners: PortalStartupListener, SessionListener
  - Error pages et session configuration

- **`portlet.xml`**: Portlet application descriptor (JSR 168/286)
  - UserManagementPortlet: Gestion des utilisateurs
  - DocumentPortlet: Gestion des documents
  - DashboardPortlet: Tableau de bord avec statistiques
  - WorkflowPortlet: Gestion des workflows
  - ReportPortlet: Génération et export de rapports

### Portlets Java

#### BasePortlet
Classe de base pour tous les portlets fournissant:
- Initialisation du portlet
- Récupération des paramètres
- Dispatch vers JSPs
- Gestion des paramètres de render

#### UserManagementPortlet
Fonctionnalités:
- Affichage liste des utilisateurs
- Édition de profil utilisateur
- Suppression d'utilisateurs
- Traitement d'actions (save, delete)

#### DocumentPortlet
Fonctionnalités:
- Affichage liste documents
- Upload de fichiers
- Suppression de documents
- Gestion du stockage (max file size)

#### DashboardPortlet
Fonctionnalités:
- Statistiques en temps réel
- Activités récentes
- Ressources AJAX (serveResource)
- Rafraîchissement automatique

#### WorkflowPortlet
Fonctionnalités:
- Liste des tâches en attente
- Affichage des processus
- Actions: Approuver, Rejeter, Déléguer
- Notifications

#### ReportPortlet
Fonctionnalités:
- Génération de rapports
- Export multi-formats (PDF, Excel, CSV)
- Téléchargement de rapports
- Filtres de rapport

### Services

#### UserService
- CRUD utilisateurs
- Pagination
- Recherche d'utilisateurs

#### DocumentService
- Gestion fichiers
- Métadonnées documents
- Recherche par nom

#### DashboardService
- Collecte de statistiques
- Activités récentes
- État système

#### WorkflowService
- Gestion processus
- Gestion tâches
- Approbation/Rejet
- Délégation

#### ReportService
- Génération rapports
- Export formats multiples
- Filtres de rapport

### Classes Support

#### Modèles
- `User`: Utilisateur avec département
- `Document`: Document avec métadonnées
- `WorkflowTask`: Tâche workflow
- `WorkflowProcess`: Processus workflow
- `Report`: Rapport avec contenu

#### Serveurs Web
- `DispatcherServlet`: Routage des requêtes
- `ResourceServlet`: Ressources statiques

#### Filtres
- `AuthenticationFilter`: Authentification
- `LoggingFilter`: Logging requête/réponse

#### Listeners
- `PortalStartupListener`: Initialisation
- `SessionListener`: Gestion sessions

#### Utilitaires
- `DatabaseConnector`: Connexion BD
- `FileStorageManager`: Gestion fichiers
- `MetricsCollector`: Collecte métriques
- `NotificationService`: Notifications
- `ReportGenerator`: Génération rapports
- `ExportManager`: Export formats

## Cas d'Usage Représentatifs

### 1. Gestion des Utilisateurs
```
Portlet → Service → Database → JSP
- Liste paginée
- Édition
- Suppression
```

### 2. Gestion des Documents
```
Portlet → Service → FileSystem + Database
- Upload
- Suppression
- Métadonnées
```

### 3. Workflows
```
Portlet → Service → Database + Notification
- Tâches en attente
- Approbation/Rejet
- Délégation
```

### 4. Rapports
```
Portlet → Service → Generator → Exporter
- Génération
- Filtres
- Export PDF/Excel/CSV
```

### 5. Tableau de Bord
```
Portlet → Dashboard Service → Metrics + Database
- Statistiques
- Activités
- Refresh AJAX
```

## Caractéristiques Clés pour le Parseur

Le jeu de tests inclut:

1. **Déclaration d'interfaces**: GenericPortlet, Filter, ServletContextListener
2. **Héritage**: BasePortlet extends GenericPortlet
3. **Imports multiples**: javax.portlet.*, javax.servlet.*
4. **Annotations pratiques**: @Override
5. **Collections**: List, Map, HashMap, ArrayList
6. **Gestion d'exceptions**: try/catch, throw
7. **Appels entre classes**: Portlet → Service → Utility
8. **Paramètres complexes**: Map<String, Object>
9. **Ressources AJAX**: serveResource()
10. **Configuration XML**: web.xml, portlet.xml

## À Faire pour Compléter

- [ ] JSP templates (JSP files)
- [ ] Tests unitaires
- [ ] Configuration application-context.xml (Spring)
- [ ] Scripts de base de données
- [ ] Configuration des ressources AJAX

## Notes

Ce jeu de tests est conçu pour être:
- **Simple** mais fonctionnel
- **Représentatif** des applications portales réelles
- **Suffisant** pour valider un parseur Java
- **Extensible** pour de futures améliorations
