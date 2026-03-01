# Usecase 4: Workflow Management Portal

## Vue d'Ensemble
Gestion des workflows avec approbation/rejet/délégation de tâches et notifications.

## Architecture

```
Portlet (WorkflowPortlet.java)
  ├─ doView(): Affiche list tâches
  └─ processAction(): Approve/Reject/Delegate
       ↓
Service (WorkflowService.java)
  ├─ getPendingTasks(userId)
  ├─ approveTask(taskId, userId, comment)
  ├─ rejectTask(taskId, userId, reason)
  └─ delegateTask(taskId, fromUser, toUser)
       ↓
Notifications (NotificationService.java)
  └─ sendNotification(message)
```

## Flux Principaux

### 1. Affichage Tâches Pendantes
```
1. WorkflowPortlet.doView()
   ↓
2. WorkflowService.getPendingTasks(userId)
   ├─ DatabaseConnector.query()
   └─ Retour List<WorkflowTask>
   ↓
3. dispatch("/WEB-INF/portlets/workflow-list.jsp")
```

### 2. Approbation Tâche
```
1. User clique [Approve] button
   ↓
2. workflow-manager.js: onApproveClick(taskId)
   ├─ Affiche dialog pour commentaire
   └─ AJAX POST /portal/page?action=approve&taskId={id}&comment={text}
   ↓
3. WorkflowPortlet.processAction()
   ├─ action = "approve"
   └─ getUserId() via request
   ↓
4. WorkflowService.approveTask(taskId, userId, comment)
   ├─ DatabaseConnector.updateTask() → status = "APPROVED"
   ├─ DatabaseConnector.saveComment()
   └─ sendNotification("Task approved by {user}")
   ↓
5. NotificationService.sendNotification()
   └─ Notifie l'initiateur du workflow
```

### 3. Rejet Tâche
```
Similaire à approbation:
- action = "reject"
- WorkflowService.rejectTask(taskId, userId, reason)
- Status = "REJECTED"
- Notification à l'initiateur
```

### 4. Délégation Tâche
```
1. User sélectionne [Delegate]
   ↓
2. workflow-manager.js: onDelegateClick(taskId)
   ├─ Affiche user picker
   └─ AJAX POST ?action=delegate&taskId={id}&delegateTo={newUserId}
   ↓
3. WorkflowService.delegateTask(taskId, fromUser, toUser)
   ├─ DatabaseConnector.updateTask() → assignedTo = newUser
   ├─ DatabaseConnector.logDelegation()
   └─ sendNotification("Task delegated to {newUser}")
```

## Fichiers Impliqués

### Java
- **WorkflowPortlet.java**: doView(), processAction()
- **WorkflowService.java**: getPendingTasks(), approveTask(), rejectTask(), delegateTask()
- **WorkflowTask.java**: Modèle task
- **WorkflowProcess.java**: Modèle process
- **NotificationService.java**: sendNotification()

### JSP/JSPF
- **workflow-list.jsp**: Liste tâches
- **workflow-process.jsp**: Détails processus
- **task-actions.jspf**: Fragment actions

### JavaScript
- **workflow-manager.js**: Logique portlet
  - `onApproveClick(taskId)`
  - `onRejectClick(taskId)`
  - `onDelegateClick(taskId)`
  - `refreshTaskList()`

- **task-form.js**: Gestion formulaires
  - `showApprovalDialog(taskId)`
  - `showRejectionDialog(taskId)`
  - `showDelegationDialog(taskId)`

## Relations Sémantiques

```
PORTLET_USES_JSP:
  WorkflowPortlet --renders--> workflow-list.jsp
  WorkflowPortlet --renders--> workflow-process.jsp

JSP_INCLUDES_JS:
  workflow-list.jsp --includes--> workflow-manager.js
  workflow-list.jsp --includes--> task-form.js

JS_CALLS_PORTLET:
  workflow-manager.js --ajax-post--> WorkflowPortlet.processAction()
  task-form.js --submit--> WorkflowPortlet.processAction()

PORTLET_CALLS_SERVICE:
  WorkflowPortlet.doView() --calls--> WorkflowService.getPendingTasks()
  WorkflowPortlet.processAction() --calls--> WorkflowService.approveTask()
  WorkflowPortlet.processAction() --calls--> WorkflowService.rejectTask()
  WorkflowPortlet.processAction() --calls--> WorkflowService.delegateTask()

SERVICE_CALLS_NOTIFICATION:
  WorkflowService --calls--> NotificationService.sendNotification()
```

## Métadonnées Attendues

```
Nœuds: 9 (1 Portlet + 1 Service + 2 Models + 1 Notification + 2 JSPs + 2 JS)
Arêtes: ~12 (Portlet→JSP, JSP→JS, JS→Portlet, Portlet→Service, Service→Notification)
```

---

**Status**: Draft
**Complexity**: Medium
**Key Feature**: Workflow orchestration avec notifications
