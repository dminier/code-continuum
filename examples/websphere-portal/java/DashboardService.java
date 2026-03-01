package com.example.services;

import java.util.HashMap;
import java.util.Map;

/**
 * Service for dashboard statistics and metrics
 */
public class DashboardService {
    
    private MetricsCollector metricsCollector;
    private DatabaseConnector dbConnector;
    
    public DashboardService() {
        this.metricsCollector = new MetricsCollector();
        this.dbConnector = new DatabaseConnector();
    }
    
    /**
     * Get dashboard statistics
     */
    public Map<String, Object> getStatistics() {
        Map<String, Object> stats = new HashMap<>();
        
        stats.put("totalUsers", metricsCollector.getTotalUsers());
        stats.put("activeUsers", metricsCollector.getActiveUsers());
        stats.put("pendingTasks", metricsCollector.getPendingTasks());
        stats.put("completedProcesses", metricsCollector.getCompletedProcesses());
        stats.put("documentsCount", metricsCollector.getDocumentsCount());
        stats.put("storageUsed", metricsCollector.getStorageUsed());
        
        return stats;
    }
    
    /**
     * Get recent activities
     */
    public Map<String, Object> getRecentActivities() {
        Map<String, Object> activities = new HashMap<>();
        
        activities.put("recentUploads", dbConnector.getRecentUploads(10));
        activities.put("recentApprovals", dbConnector.getRecentApprovals(10));
        activities.put("recentLogins", dbConnector.getRecentLogins(10));
        
        return activities;
    }
    
    /**
     * Get system health status
     */
    public Map<String, Object> getSystemHealth() {
        Map<String, Object> health = new HashMap<>();
        
        health.put("databaseStatus", metricsCollector.getDatabaseStatus());
        health.put("fileSystemStatus", metricsCollector.getFileSystemStatus());
        health.put("cpuUsage", metricsCollector.getCpuUsage());
        health.put("memoryUsage", metricsCollector.getMemoryUsage());
        
        return health;
    }
}
