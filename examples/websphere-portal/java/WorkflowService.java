package com.example.services;

import com.example.models.WorkflowTask;
import com.example.models.WorkflowProcess;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Service for workflow management operations
 */
public class WorkflowService {
    
    private Map<String, WorkflowProcess> processCache;
    private Map<String, WorkflowTask> taskCache;
    private DatabaseConnector dbConnector;
    private NotificationService notificationService;
    
    public WorkflowService() {
        this.processCache = new HashMap<>();
        this.taskCache = new HashMap<>();
        this.dbConnector = new DatabaseConnector();
        this.notificationService = new NotificationService();
    }
    
    /**
     * Get pending tasks for user
     */
    public List<WorkflowTask> getPendingTasks(String userId) {
        List<WorkflowTask> tasks = new ArrayList<>();
        for (WorkflowTask task : taskCache.values()) {
            if (userId.equals(task.getAssignedTo()) && "PENDING".equals(task.getStatus())) {
                tasks.add(task);
            }
        }
        return tasks;
    }
    
    /**
     * Get workflow process
     */
    public WorkflowProcess getProcess(String processId) {
        return processCache.getOrDefault(processId, null);
    }
    
    /**
     * Create new workflow process
     */
    public WorkflowProcess createProcess(String name, String initiatedBy) {
        WorkflowProcess process = new WorkflowProcess(generateId(), name);
        process.setInitiatedBy(initiatedBy);
        process.setInitiatedDate(System.currentTimeMillis());
        
        processCache.put(process.getId(), process);
        dbConnector.saveProcess(process);
        
        return process;
    }
    
    /**
     * Approve task
     */
    public void approveTask(String taskId, String userId, String comment) {
        WorkflowTask task = taskCache.get(taskId);
        if (task != null) {
            task.setStatus("APPROVED");
            dbConnector.updateTask(task);
            dbConnector.saveComment(taskId, userId, comment, "APPROVAL");
        }
    }
    
    /**
     * Reject task
     */
    public void rejectTask(String taskId, String userId, String reason) {
        WorkflowTask task = taskCache.get(taskId);
        if (task != null) {
            task.setStatus("REJECTED");
            dbConnector.updateTask(task);
            dbConnector.saveComment(taskId, userId, reason, "REJECTION");
        }
    }
    
    /**
     * Delegate task to another user
     */
    public void delegateTask(String taskId, String fromUser, String toUser) {
        WorkflowTask task = taskCache.get(taskId);
        if (task != null) {
            task.setAssignedTo(toUser);
            dbConnector.updateTask(task);
            dbConnector.logDelegation(taskId, fromUser, toUser);
        }
    }
    
    /**
     * Send notification
     */
    public void sendNotification(String message) {
        notificationService.sendNotification(message);
    }
    
    /**
     * Generate unique ID
     */
    private String generateId() {
        return "WF" + System.currentTimeMillis();
    }
}
