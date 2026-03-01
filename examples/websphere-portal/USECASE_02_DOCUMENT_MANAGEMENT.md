# Usecase 2: Document Management Portal

## Vue d'Ensemble
Gestion de documents avec upload, affichage et suppression avec gestion du stockage fichier.

## Architecture Générale

```
┌─────────────────────────────────────────────────────────┐
│                  FRONTEND (Browser)                     │
├─────────────────────────────────────────────────────────┤
│  HTML/JSP                 │ JavaScript                  │
├──────────────────────────────────────────────────────────┤
│ document-list.jsp        │ document-manager.js         │
│ document-upload.jsp      │ file-uploader.js            │
│ document-detail.jsp      │ ajax-service.js             │
│ document-search.jsp      │ file-validator.js           │
└──────────┬──────────────────────────┬──────────────────┘
           │                          │
           │ HTTP POST/GET            │ AJAX Calls
           │ (multipart/form-data)    │ (file chunks)
           │                          │
┌──────────▼──────────────────────────▼──────────────────┐
│             PORTLET (Server-side)                      │
├───────────────────────────────────────────────────────┤
│ DocumentPortlet.java                                 │
│  - doView(): Affiche liste documents                 │
│  - doEdit(): Config upload                           │
│  - processAction(): Traite upload/delete             │
│  - serveResource(): Download/AJAX endpoints          │
└──────────┬───────────────────────────────────────────┘
           │
           │ Appel direct
           │
┌──────────▼───────────────────────────────────────────┐
│             SERVICE LAYER                             │
├───────────────────────────────────────────────────────┤
│ DocumentService.java                                │
│  - getDocuments(folderId)                           │
│  - uploadDocument(document)                         │
│  - deleteDocument(documentId)                       │
│  - getDocument(documentId)                          │
│  - searchDocuments(searchTerm)                      │
└──────────┬───────────────────────────────────────────┘
           │
           ├─ Appel 1: CRUD métadonnées
           │  │
           │  └→ DatabaseConnector
           │
           └─ Appel 2: Gestion fichiers
              │
              └→ FileStorageManager
```

## Flux Détaillés

### Flux 1: Affichage Liste Documents

```
1. DocumentPortlet.doView() [GET /portal/page]
   ↓
2. Récupère paramètres du portlet.xml
   - repository-path = /var/portal/documents
   - max-file-size = 52428800 (50MB)
   ↓
3. Parse request.getParameter("folderId")
   ↓
4. DocumentService.getDocuments(folderId)
   ├─ DatabaseConnector.query()
   └─ Retour List<Document>
   ↓
5. request.setAttribute("documents", documents)
   request.setAttribute("repositoryPath", path)
   ↓
6. dispatch("/WEB-INF/portlets/document-list.jsp")
   ↓
7. document-list.jsp rendu
   ├─ Affiche tableau documents
   ├─ Colonnes: [Name], [Size], [Date], [Actions]
   ├─ Inclut document-manager.js
   ├─ Inclut file-uploader.js
   └─ Boutons: [Upload], [Delete], [Download]
   ↓
8. Browser charge JS files
   ├─ document-manager.js setup
   ├─ file-uploader.js initialization
   └─ Event listeners attachés
```

### Flux 2: Upload Document (Multipart)

```
1. User clique [Upload] button
   ↓
2. Browser affiche file picker (HTML5 <input type="file">)
   ↓
3. User sélectionne fichier (ex: report.pdf)
   ↓
4. file-uploader.js: onFileSelected(file)
   ├─ Valide via file-validator.js:
   │  ├─ validateFileType() → .pdf, .doc, .xls autorisés
   │  ├─ validateFileSize() → < 50MB
   │  └─ validateFileName()
   ├─ Crée FormData
   └─ Décide: upload direct ou chunked
   ↓
5. Si taille < 10MB: Direct upload
   ├─ AJAX POST /portal/page?action=upload
   ├─ Content-Type: multipart/form-data
   └─ Payload: {file, folderId, metadata}
   ↓
6. DocumentPortlet.processAction()
   ├─ action = "upload"
   ├─ Parse request.getParameter("folderId")
   └─ Récupère file upload
   ↓
7. Crée Document object
   ├─ fileName = file.getFileName()
   ├─ fileSize = file.getSize()
   ├─ mimeType = file.getContentType()
   └─ createdDate = System.currentTimeMillis()
   ↓
8. DocumentService.uploadDocument(document)
   ├─ FileStorageManager.storeFile()
   │  ├─ Crée repertoire si nécessaire
   │  ├─ Écrit fichier sur disque
   │  └─ Retour filePath
   ├─ DatabaseConnector.saveDocument()
   │  ├─ INSERT INTO documents (...)
   │  └─ Retour documentId
   └─ document.setStoragePath(filePath)
   ↓
9. response.setRenderParameter("folderId", folderId)
   ↓
10. Browser reçoit success response
    ├─ file-uploader.js: onUploadSuccess()
    ├─ Refresh document list via AJAX
    ├─ Affiche confirmation message
    └─ Clear file input
```

### Flux 3: Download Document

```
1. User clique [Download] link dans tableau
   ↓
2. document-manager.js: onDownloadClick(documentId)
   ├─ Crée hidden <a> tag
   └─ Navigate vers serveResource
   ↓
3. Browser GET /portal/page?resourceID=downloadDocument&documentId={id}
   ↓
4. DocumentPortlet.serveResource()
   ├─ resourceID = "downloadDocument"
   ├─ documentId from request
   └─ Valide accès utilisateur
   ↓
5. DocumentService.getDocument(documentId)
   ├─ DatabaseConnector.query()
   └─ Retour Document metadata
   ↓
6. FileStorageManager.readFile(path, fileName)
   ├─ Lit fichier depuis disque
   └─ Retour byte[]
   ↓
7. response.setContentType(mimeType)
   response.setProperty("Content-Disposition", "attachment; filename=...")
   ↓
8. response.getPortletOutputStream().write(fileBytes)
   ↓
9. Browser reçoit fichier binaire
   ├─ Démarre téléchargement
   └─ Sauvegarde fichier localement
```

### Flux 4: Suppression Document

```
1. User clique [Delete] dans le tableau
   ↓
2. document-manager.js: onDeleteClick(documentId)
   ├─ Show confirmation dialog
   └─ Si confirmé: POST request
   ↓
3. AJAX POST /portal/page?action=delete&documentId={id}
   ↓
4. DocumentPortlet.processAction()
   ├─ action = "delete"
   └─ documentId from request
   ↓
5. DocumentService.deleteDocument(documentId)
   ├─ Récupère Document metadata
   ├─ FileStorageManager.deleteFile(path, fileName)
   │  ├─ Supprime fichier du disque
   │  └─ Nettoie repertoires vides
   └─ DatabaseConnector.deleteDocument(documentId)
      ├─ DELETE FROM documents WHERE id = ?
      └─ Exécute query
   ↓
6. response.setRenderParameter("folderId", folderId)
   ↓
7. Browser reçoit confirmation
   ├─ Retire row du tableau (DOM manipulation)
   ├─ Mise à jour espace disque utilisé
   └─ Montre notification success
```

### Flux 5: Recherche Documents (AJAX)

```
1. User tape dans search box: "invoice"
   ↓
2. document-manager.js: onSearchInput(searchTerm)
   ├─ Attend 300ms (debounce)
   ├─ Valide searchTerm
   └─ AJAX GET /portal/page?resourceID=search&q=invoice
   ↓
3. DocumentPortlet.serveResource()
   ├─ resourceID = "search"
   ├─ q = "invoice"
   └─ Content-Type: application/json
   ↓
4. DocumentService.searchDocuments("invoice")
   ├─ DatabaseConnector.query()
   │  ├─ SELECT * FROM documents WHERE fileName LIKE '%invoice%'
   │  └─ LIMIT 20
   └─ Retour List<Document>
   ↓
5. Convert List<Document> → JSON array
   ├─ [{id, fileName, fileSize, createdDate}, ...]
   └─ Write to response
   ↓
6. Browser reçoit JSON response
   ├─ document-manager.js: onSearchResults(results)
   ├─ Clear tableau actuel
   ├─ Ajoute nouvelles rows
   └─ Affiche "X documents trouvés"
```

## Fichiers Impliqués

### Java
- **DocumentPortlet.java**
  - Classe: `com.example.portlets.DocumentPortlet extends BasePortlet`
  - Méthodes: `doView()`, `doEdit()`, `processAction()`, `serveResource()`

- **DocumentService.java**
  - Classe: `com.example.services.DocumentService`
  - Méthodes: `getDocuments()`, `uploadDocument()`, `deleteDocument()`, `searchDocuments()`

- **FileStorageManager.java**
  - Classe: `com.example.services.FileStorageManager`
  - Méthodes: `storeFile()`, `deleteFile()`, `readFile()`

- **Document.java**
  - Modèle: `com.example.models.Document`
  - Propriétés: id, fileName, folderId, fileSize, mimeType, createdDate, createdBy

### JSP/JSPF
- **document-list.jsp** - Tableau documents avec actions
- **document-upload.jsp** - Formulaire upload
- **document-detail.jsp** - Détails document
- **document-search.jsp** - Formulaire recherche
- **document-fragment.jspf** - Fragment réutilisable

### JavaScript
- **document-manager.js** - Logique portlet
  - `onDownloadClick(documentId)`
  - `onDeleteClick(documentId)`
  - `onSearchInput(searchTerm)`
  - `refreshDocumentList()`
  - `updateStorageInfo()`

- **file-uploader.js** - Upload spécialisé
  - `onFileSelected(file)`
  - `validateFile(file)`
  - `uploadFile(file, onProgress)`
  - `uploadChunked(file)` (pour gros fichiers)
  - `onUploadProgress(event)`

- **file-validator.js** - Validation fichiers
  - `validateFileType(fileName, allowedTypes)`
  - `validateFileSize(fileSize, maxSize)`
  - `validateFileName(fileName)`

- **ajax-service.js** - Service AJAX générique
  - `get()`, `post()`, `delete()`, `upload()`

### Configuration
- **portlet.xml**
  ```xml
  <portlet>
    <portlet-name>DocumentPortlet</portlet-name>
    <init-param>
      <name>repository-path</name>
      <value>/var/portal/documents</value>
    </init-param>
    <init-param>
      <name>max-file-size</name>
      <value>52428800</value>
    </init-param>
  </portlet>
  ```

## Relations Sémantiques Clés

```
PORTLET_USES_JSP:
  DocumentPortlet --renders--> document-list.jsp
  DocumentPortlet --renders--> document-upload.jsp
  DocumentPortlet --renders--> document-detail.jsp

JSP_INCLUDES_JS:
  document-list.jsp --includes--> document-manager.js
  document-list.jsp --includes--> file-uploader.js
  document-list.jsp --includes--> file-validator.js
  document-upload.jsp --includes--> file-uploader.js
  document-upload.jsp --includes--> file-validator.js

JSP_INCLUDES_FRAGMENT:
  document-list.jsp --includes--> document-fragment.jspf
  document-detail.jsp --includes--> document-fragment.jspf

JS_CALLS_PORTLET:
  document-manager.js --ajax-get--> DocumentPortlet.doView()
  document-manager.js --ajax-post--> DocumentPortlet.processAction()
  document-manager.js --ajax-get--> DocumentPortlet.serveResource()
  file-uploader.js --multipart-post--> DocumentPortlet.processAction()

PORTLET_CALLS_SERVICE:
  DocumentPortlet.doView() --calls--> DocumentService.getDocuments()
  DocumentPortlet.processAction() --calls--> DocumentService.uploadDocument()
  DocumentPortlet.processAction() --calls--> DocumentService.deleteDocument()
  DocumentPortlet.serveResource() --calls--> DocumentService.searchDocuments()

SERVICE_CALLS_STORAGE:
  DocumentService --calls--> FileStorageManager.storeFile()
  DocumentService --calls--> FileStorageManager.deleteFile()
  DocumentService --calls--> FileStorageManager.readFile()

SERVICE_CALLS_DAO:
  DocumentService --calls--> DatabaseConnector

PORTLET_READS_CONFIG:
  DocumentPortlet --reads--> portlet.xml [repository-path, max-file-size]
```

## Paramètres Critiques

| Paramètre | Source | Valeur | Portée |
|-----------|--------|--------|--------|
| repository-path | portlet.xml | /var/portal/documents | FileStorageManager |
| max-file-size | portlet.xml | 52428800 | file-uploader.js validation |
| folderId | HTTP Request | String | getDocuments(), upload path |
| documentId | HTTP Request | String | delete, download operations |
| action | HTTP Request | upload\|delete | Routing |
| resourceID | HTTP Request | search\|downloadDocument | serveResource routing |

## Points de Test Critiques

1. **Upload multipart**
   - RequestDispatcher gère multipart/form-data
   - File parsing depuis request
   - Storage path handling

2. **Binary download**
   - Content-Type setting
   - Content-Disposition header
   - OutputStream.write(bytes)

3. **Espace disque**
   - Suivi fileSize
   - Max file size validation
   - Repository path management

4. **Recherche AJAX**
   - DatabaseConnector.query() avec LIKE
   - JSON serialization
   - Result limiting

5. **Validation client**
   - file-validator.js checks
   - File type whitelist
   - Size restrictions

## Métadonnées Attendues

```
Nœuds (Nodes):
- 1 Portlet (DocumentPortlet)
- 1 Service (DocumentService)
- 1 Manager (FileStorageManager)
- 1 Model (Document)
- 1 DAO (DatabaseConnector)
- 4 JSPs (document-list, document-upload, document-detail, document-search)
- 4 JS files (document-manager, file-uploader, file-validator, ajax-service)
- 1 Fragment (document-fragment.jspf)
Total: ~15 nœuds

Relations (Edges):
- Portlet USES JSP: 4
- Portlet CALLS Service: 4
- Service CALLS Manager: 3
- Service CALLS DAO: 1
- JSP INCLUDES JS: 6
- JS CALLS Portlet: 4
- Portlet READS Config: 1
Total: ~23 arêtes
```

---

**Status**: Draft - À enrichir avec fichiers JSP/JS
**Complexity**: Medium (multipart uploads, binary streaming)
**Prochaine étape**: Créer fichiers JSP, JS et gérer storage mappings
