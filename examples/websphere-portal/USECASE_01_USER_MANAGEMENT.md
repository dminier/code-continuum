# Usecase 1: User Management Portal

## Vue d'Ensemble
Gestion des utilisateurs avec interface de liste, édition et suppression.

## Architecture Générale

```
┌─────────────────────────────────────────────────────────┐
│                  FRONTEND (Browser)                     │
├─────────────────────────────────────────────────────────┤
│  HTML/JSP                  │ JavaScript                │
├──────────────────────────────────────────────────────────┤
│ user-list.jsp             │ user-management.js        │
│ user-edit.jsp             │ ajax-service.js           │
│ user-detail.jsp           │ form-validator.js         │
└──────────┬──────────────────────────┬──────────────────┘
           │                          │
           │ HTTP POST/GET            │ AJAX Calls
           │                          │
┌──────────▼──────────────────────────▼──────────────────┐
│             PORTLET (Server-side)                      │
├───────────────────────────────────────────────────────┤
│ UserManagementPortlet.java                           │
│  - doView(): Affiche liste/formulaire                │
│  - doEdit(): Édite profil                            │
│  - processAction(): Traite form submit               │
│  - serveResource(): AJAX endpoints                   │
└──────────┬───────────────────────────────────────────┘
           │
           │ Appel direct
           │
┌──────────▼───────────────────────────────────────────┐
│             SERVICE LAYER                             │
├───────────────────────────────────────────────────────┤
│ UserService.java                                    │
│  - getAllUsers(pageSize)                            │
│  - getUserById(userId)                              │
│  - saveUser(user)                                   │
│  - deleteUser(userId)                               │
│  - searchUsers(criteria)                            │
└──────────┬───────────────────────────────────────────┘
           │
           │ Appel direct
           │
┌──────────▼───────────────────────────────────────────┐
│             DATA LAYER                                │
├───────────────────────────────────────────────────────┤
│ DatabaseConnector.java                              │
│  - saveUser()                                       │
│  - deleteUser()                                     │
│  - query()                                          │
└───────────────────────────────────────────────────────┘
```

## Flux Détaillés

### Flux 1: Affichage Liste Utilisateurs

```
1. Portlet.doView() [GET /portal/page]
   ↓
2. Charge paramètres du portlet.xml
   - pageSize = 10
   - template-view = /WEB-INF/portlets/user-list.jsp
   ↓
3. UserService.getAllUsers(10)
   ↓
4. DatabaseConnector.query()
   ↓
5. Retour List<User> → request.setAttribute("users", users)
   ↓
6. DispatcherServlet dispatch("/WEB-INF/portlets/user-list.jsp")
   ↓
7. user-list.jsp rendu
   ├─ Affiche tableau HTML avec users
   ├─ Inclut user-management.js
   └─ Boutons: [Edit], [Delete], [New]
   ↓
8. Browser charge user-management.js
   ├─ Attache event listeners
   ├─ Setup AJAX calls
   └─ Form validation
```

### Flux 2: Édition Utilisateur (AJAX)

```
1. User clique sur [Edit] button dans le tableau
   ↓
2. user-management.js: onEditClick(userId)
   ├─ Valide userId via form-validator.js
   ├─ Crée AJAX request
   └─ Envoie GET vers serveResource()
   ↓
3. UserManagementPortlet.serveResource()
   - resourceID = "getUser"
   ↓
4. request.getParameter("userId")
   ↓
5. UserService.getUserById(userId)
   ├─ DatabaseConnector.query()
   └─ Retour User object
   ↓
6. Portlet.serveResource() [Content-Type: application/json]
   ├─ Convert User → JSON
   └─ Write to response.getWriter()
   ↓
7. Browser reçoit JSON response
   ├─ ajax-service.js: parseResponse()
   └─ user-management.js: populateForm(userData)
   ↓
8. user-edit.jsp rendu dans modal/lightbox
   ├─ Champs pré-remplis
   ├─ Boutons: [Save], [Cancel]
   └─ Form validation en client-side
```

### Flux 3: Sauvegarde Utilisateur

```
1. User remplit formulaire dans user-edit.jsp
   ↓
2. User clique [Save]
   ↓
3. user-management.js: onSaveClick()
   ├─ form-validator.js: validate()
   ├─ Crée FormData
   └─ Appelle ajax-service.js: post()
   ↓
4. AJAX POST /portal/page?action=saveUser
   ├─ Content-Type: application/x-www-form-urlencoded
   └─ Payload: {userId, name, email, department}
   ↓
5. Portlet.processAction(request, response)
   ├─ action = "save"
   ├─ Parse parameters
   └─ Crée User object
   ↓
6. UserService.saveUser(user)
   ↓
7. DatabaseConnector.saveUser(user)
   ├─ Prépare SQL INSERT/UPDATE
   └─ Exécute query
   ↓
8. response.setRenderParameter("view", "list")
   ↓
9. Browser reçoit redirect
   ├─ Ferme modal
   ├─ Refresh liste via AJAX
   └─ Affiche confirmation message
```

### Flux 4: Suppression Utilisateur

```
1. User clique [Delete] dans le tableau
   ↓
2. user-management.js: onDeleteClick(userId)
   ├─ Show confirmation dialog
   └─ Si confirmé: POST request
   ↓
3. AJAX POST /portal/page?action=deleteUser&userId={id}
   ↓
4. Portlet.processAction()
   ├─ action = "delete"
   └─ userId from request
   ↓
5. UserService.deleteUser(userId)
   ↓
6. DatabaseConnector.deleteUser(userId)
   ├─ DELETE FROM users WHERE id = ?
   └─ Exécute query
   ↓
7. response.setRenderParameter("view", "list")
   ↓
8. Browser reçoit confirmation
   ├─ Retire row du tableau (DOM manipulation)
   └─ Montre notification success
```

## Fichiers Impliqués

### Java
- **UserManagementPortlet.java**
  - Classe: `com.example.portlets.UserManagementPortlet extends BasePortlet`
  - Méthodes: `doView()`, `doEdit()`, `processAction()`, `serveResource()`
  
- **UserService.java**
  - Classe: `com.example.services.UserService`
  - Méthodes: `getAllUsers()`, `getUserById()`, `saveUser()`, `deleteUser()`

- **User.java**
  - Modèle: `com.example.models.User`
  - Propriétés: id, name, email, department, active

### JSP/JSPF
- **user-list.jsp** - Affiche tableau des utilisateurs
- **user-edit.jsp** - Formulaire édition utilisateur
- **user-detail.jsp** - Détails utilisateur (optionnel)
- **user-fragment.jspf** - Fragment réutilisable

### JavaScript
- **user-management.js** - Logique spécifique portlet
  - `onEditClick(userId)`
  - `onDeleteClick(userId)`
  - `onSaveClick()`
  - `populateForm(userData)`
  - `refreshUserList()`

- **ajax-service.js** - Service AJAX générique
  - `get(url, callback)`
  - `post(url, data, callback)`
  - `put(url, data, callback)`
  - `delete(url, callback)`

- **form-validator.js** - Validation côté client
  - `validate(formData)`
  - `validateEmail(email)`
  - `validateRequired(field)`

### Configuration
- **portlet.xml**
  ```xml
  <portlet>
    <portlet-name>UserManagementPortlet</portlet-name>
    <portlet-class>com.example.portlets.UserManagementPortlet</portlet-class>
    <init-param>
      <name>template-view</name>
      <value>/WEB-INF/portlets/user-list.jsp</value>
    </init-param>
    <init-param>
      <name>template-edit</name>
      <value>/WEB-INF/portlets/user-edit.jsp</value>
    </init-param>
    <init-param>
      <name>pageSize</name>
      <value>10</value>
    </init-param>
  </portlet>
  ```

- **web.xml** (Servlet filters & listeners)
  ```xml
  <filter>
    <filter-name>AuthenticationFilter</filter-name>
    <filter-class>com.example.filters.AuthenticationFilter</filter-class>
  </filter>
  <filter-mapping>
    <filter-name>AuthenticationFilter</filter-name>
    <url-pattern>/*</url-pattern>
  </filter-mapping>
  ```

## Relations Sémantiques Clés

```
PORTLET_USES_JSP:
  UserManagementPortlet --renders--> user-list.jsp
  UserManagementPortlet --renders--> user-edit.jsp
  UserManagementPortlet --renders--> user-detail.jsp

JSP_INCLUDES_JS:
  user-list.jsp --includes--> user-management.js
  user-list.jsp --includes--> ajax-service.js
  user-list.jsp --includes--> form-validator.js

JSP_INCLUDES_FRAGMENT:
  user-list.jsp --includes--> user-fragment.jspf
  user-edit.jsp --includes--> user-fragment.jspf

JS_CALLS_SERVLET:
  user-management.js --ajax-post--> UserManagementPortlet.processAction()
  user-management.js --ajax-get--> UserManagementPortlet.serveResource()
  ajax-service.js --http-request--> /portal/page

PORTLET_CALLS_SERVICE:
  UserManagementPortlet.doView() --calls--> UserService.getAllUsers()
  UserManagementPortlet.processAction() --calls--> UserService.saveUser()
  UserManagementPortlet.serveResource() --calls--> UserService.getUserById()

SERVICE_CALLS_DAO:
  UserService --calls--> DatabaseConnector

PORTLET_REFERENCES_CONFIG:
  UserManagementPortlet --reads--> portlet.xml [pageSize]
  UserManagementPortlet --uses--> AuthenticationFilter (via web.xml)
```

## Paramètres Critiques

| Paramètre | Source | Valeur | Portée |
|-----------|--------|--------|--------|
| pageSize | portlet.xml | 10 | UserManagementPortlet.doView() |
| template-view | portlet.xml | /WEB-INF/portlets/user-list.jsp | Portlet rendering |
| template-edit | portlet.xml | /WEB-INF/portlets/user-edit.jsp | Edit mode |
| userId | HTTP Request | String | processAction(), serveResource() |
| action | HTTP Request | save\|delete\|search | Routing in processAction() |

## Points de Test Critiques

1. **Extraction Portlet**
   - Classe identifiée
   - Méthodes: doView, doEdit, processAction, serveResource

2. **Extraction Service**
   - UserService identifiée
   - Appels détectés depuis Portlet

3. **Lien JSP**
   - JSP files pointés par portlet.xml
   - JSP includes JS files

4. **Lien AJAX**
   - JavaScript POST/GET calls détectés
   - Cibles: serveResource, processAction

5. **Lien DAO**
   - UserService → DatabaseConnector
   - CRUD operations identifiées

6. **Configuration**
   - portlet.xml parsed
   - Paramètres extraits
   - Filters linked

## Métadonnées Attendues

```
Nœuds (Nodes):
- 1 Portlet (UserManagementPortlet)
- 1 Service (UserService)
- 1 Model (User)
- 1 DAO (DatabaseConnector)
- 3 JSPs (user-list, user-edit, user-detail)
- 3 JS files (user-management, ajax-service, form-validator)
- 1 Fragment (user-fragment.jspf)
- 1 Config (portlet.xml)
Total: ~12 nœuds

Relations (Edges):
- Portlet USES JSP: 3
- Portlet CALLS Service: 4
- Service CALLS DAO: 4
- JSP INCLUDES JS: 3
- JSP INCLUDES Fragment: 2
- JS CALLS Portlet: 2 (AJAX)
- Portlet READS Config: 1
Total: ~19 arêtes
```

---

**Status**: Draft - À enrichir avec JSP/JS
**Prochaine étape**: Créer fichiers JSP, JS et mettre à jour le graphe sémantique
