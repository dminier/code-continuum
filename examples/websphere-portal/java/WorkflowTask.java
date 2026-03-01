package com.example.models;

/**
 * Workflow Task model class
 */
public class WorkflowTask {
    private String id;
    private String processId;
    private String taskName;
    private String assignedTo;
    private String status; // PENDING, APPROVED, REJECTED
    private long createdDate;
    private long dueDate;
    private int priority;
    
    public WorkflowTask() {
    }
    
    public WorkflowTask(String id, String taskName, String assignedTo) {
        this.id = id;
        this.taskName = taskName;
        this.assignedTo = assignedTo;
        this.status = "PENDING";
    }
    
    // Getters and Setters
    public String getId() {
        return id;
    }
    
    public void setId(String id) {
        this.id = id;
    }
    
    public String getProcessId() {
        return processId;
    }
    
    public void setProcessId(String processId) {
        this.processId = processId;
    }
    
    public String getTaskName() {
        return taskName;
    }
    
    public void setTaskName(String taskName) {
        this.taskName = taskName;
    }
    
    public String getAssignedTo() {
        return assignedTo;
    }
    
    public void setAssignedTo(String assignedTo) {
        this.assignedTo = assignedTo;
    }
    
    public String getStatus() {
        return status;
    }
    
    public void setStatus(String status) {
        this.status = status;
    }
    
    public long getCreatedDate() {
        return createdDate;
    }
    
    public void setCreatedDate(long createdDate) {
        this.createdDate = createdDate;
    }
    
    public long getDueDate() {
        return dueDate;
    }
    
    public void setDueDate(long dueDate) {
        this.dueDate = dueDate;
    }
    
    public int getPriority() {
        return priority;
    }
    
    public void setPriority(int priority) {
        this.priority = priority;
    }
}
