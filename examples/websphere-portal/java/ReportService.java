package com.example.services;

import com.example.models.Report;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Service for report generation and management
 */
public class ReportService {
    
    private List<Report> reports;
    private DatabaseConnector dbConnector;
    private ReportGenerator reportGenerator;
    private ExportManager exportManager;
    
    public ReportService() {
        this.reports = new ArrayList<>();
        this.dbConnector = new DatabaseConnector();
        this.reportGenerator = new ReportGenerator();
        this.exportManager = new ExportManager();
        initializeReports();
    }
    
    /**
     * Get available reports
     */
    public List<Report> getAvailableReports() {
        return new ArrayList<>(reports);
    }
    
    /**
     * Get specific report
     */
    public Report getReport(String reportId) {
        for (Report report : reports) {
            if (reportId.equals(report.getId())) {
                return report;
            }
        }
        return null;
    }
    
    /**
     * Generate report with filters
     */
    public Report generateReport(String reportId, String filters) {
        Report report = getReport(reportId);
        if (report != null) {
            byte[] content = reportGenerator.generate(report, parseFilters(filters));
            report.setContent(content);
            dbConnector.saveReport(report);
        }
        return report;
    }
    
    /**
     * Export report in specified format
     */
    public byte[] exportReport(String reportId, String format) {
        Report report = getReport(reportId);
        if (report != null) {
            return exportManager.export(report, format);
        }
        return new byte[0];
    }
    
    /**
     * Initialize sample reports
     */
    private void initializeReports() {
        Report report1 = new Report("REP001", "Monthly Sales Report", "FINANCIAL");
        report1.setDescription("Monthly sales statistics and trends");
        reports.add(report1);
        
        Report report2 = new Report("REP002", "User Activity Report", "ANALYTICS");
        report2.setDescription("User activities and engagement metrics");
        reports.add(report2);
        
        Report report3 = new Report("REP003", "System Performance Report", "OPERATIONAL");
        report3.setDescription("System performance and resource usage");
        reports.add(report3);
    }
    
    /**
     * Parse filter parameters
     */
    private Map<String, String> parseFilters(String filters) {
        Map<String, String> filterMap = new HashMap<>();
        if (filters != null && !filters.isEmpty()) {
            String[] pairs = filters.split("&");
            for (String pair : pairs) {
                String[] kv = pair.split("=");
                if (kv.length == 2) {
                    filterMap.put(kv[0], kv[1]);
                }
            }
        }
        return filterMap;
    }
}
