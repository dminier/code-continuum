package com.example.models;

import java.util.List;

/**
 * Workflow Process model class
 */
public class WorkflowProcess {
    private String id;
    private String name;
    private String status; // INITIATED, IN_PROGRESS, COMPLETED, REJECTED
    private String initiatedBy;
    private long initiatedDate;
    private List<WorkflowTask> tasks;
    
    public WorkflowProcess() {
    }
    
    public WorkflowProcess(String id, String name) {
        this.id = id;
        this.name = name;
        this.status = "INITIATED";
    }
    
    // Getters and Setters
    public String getId() {
        return id;
    }
    
    public void setId(String id) {
        this.id = id;
    }
    
    public String getName() {
        return name;
    }
    
    public void setName(String name) {
        this.name = name;
    }
    
    public String getStatus() {
        return status;
    }
    
    public void setStatus(String status) {
        this.status = status;
    }
    
    public String getInitiatedBy() {
        return initiatedBy;
    }
    
    public void setInitiatedBy(String initiatedBy) {
        this.initiatedBy = initiatedBy;
    }
    
    public long getInitiatedDate() {
        return initiatedDate;
    }
    
    public void setInitiatedDate(long initiatedDate) {
        this.initiatedDate = initiatedDate;
    }
    
    public List<WorkflowTask> getTasks() {
        return tasks;
    }
    
    public void setTasks(List<WorkflowTask> tasks) {
        this.tasks = tasks;
    }
}
