package com.example.services;

/**
 * Metrics collection for dashboard
 */
public class MetricsCollector {
    
    public long getTotalUsers() {
        return 150;
    }
    
    public long getActiveUsers() {
        return 85;
    }
    
    public long getPendingTasks() {
        return 23;
    }
    
    public long getCompletedProcesses() {
        return 542;
    }
    
    public long getDocumentsCount() {
        return 1203;
    }
    
    public long getStorageUsed() {
        return 2560; // in MB
    }
    
    public String getDatabaseStatus() {
        return "HEALTHY";
    }
    
    public String getFileSystemStatus() {
        return "HEALTHY";
    }
    
    public double getCpuUsage() {
        return 45.3;
    }
    
    public double getMemoryUsage() {
        return 62.1;
    }
}
