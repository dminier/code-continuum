package com.example.portlets;

import javax.portlet.*;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.List;
import com.example.services.ReportService;
import com.example.models.Report;

/**
 * Report Portlet
 * Generates and exports reports in multiple formats
 */
public class ReportPortlet extends BasePortlet {
    
    private ReportService reportService;
    private String[] exportFormats;
    
    @Override
    public void init(PortletConfig config) throws PortletException {
        super.init(config);
        this.reportService = new ReportService();
        String formats = getInitParameter("export-formats");
        this.exportFormats = formats != null ? formats.split(",") : new String[]{"PDF"};
    }
    
    /**
     * Display available reports
     */
    @Override
    protected void doView(RenderRequest request, RenderResponse response) 
            throws PortletException, IOException {
        
        response.setContentType("text/html;charset=UTF-8");
        
        String reportId = request.getParameter("reportId");
        
        if (reportId != null) {
            // View specific report
            Report report = reportService.getReport(reportId);
            request.setAttribute("report", report);
            request.setAttribute("exportFormats", exportFormats);
            dispatch("/WEB-INF/portlets/report-detail.jsp", request, response);
        } else {
            // List available reports
            List<Report> reports = reportService.getAvailableReports();
            request.setAttribute("reports", reports);
            dispatch("/WEB-INF/portlets/report-list.jsp", request, response);
        }
    }
    
    /**
     * Process report actions
     */
    @Override
    public void processAction(ActionRequest request, ActionResponse response) 
            throws PortletException, IOException {
        
        String action = request.getParameter("action");
        String reportId = request.getParameter("reportId");
        
        if ("generate".equals(action)) {
            String filters = request.getParameter("filters");
            Report report = reportService.generateReport(reportId, filters);
            response.setRenderParameter("reportId", report.getId());
            
        } else if ("export".equals(action)) {
            String format = request.getParameter("format");
            if (isValidFormat(format)) {
                // Handle export
                response.setRenderParameter("reportId", reportId);
                response.setRenderParameter("export", format);
            }
        }
    }
    
    /**
     * Serve resource for report downloads
     */
    @Override
    public void serveResource(ResourceRequest request, ResourceResponse response) 
            throws PortletException, IOException {
        
        String resourceId = request.getResourceID();
        
        if ("downloadReport".equals(resourceId)) {
            String reportId = request.getParameter("reportId");
            String format = request.getParameter("format");
            
            if (isValidFormat(format)) {
                byte[] reportData = reportService.exportReport(reportId, format);
                response.setContentType(getContentType(format));
                response.setProperty("Content-Disposition", 
                    "attachment; filename=\"report-" + reportId + "." + getFileExtension(format) + "\"");
                response.getPortletOutputStream().write(reportData);
            }
        }
    }
    
    /**
     * Validate export format
     */
    private boolean isValidFormat(String format) {
        if (format == null) return false;
        for (String f : exportFormats) {
            if (f.trim().equalsIgnoreCase(format)) {
                return true;
            }
        }
        return false;
    }
    
    /**
     * Get content type for format
     */
    private String getContentType(String format) {
        switch (format.toUpperCase()) {
            case "PDF":
                return "application/pdf";
            case "EXCEL":
                return "application/vnd.ms-excel";
            case "CSV":
                return "text/csv";
            default:
                return "application/octet-stream";
        }
    }
    
    /**
     * Get file extension for format
     */
    private String getFileExtension(String format) {
        switch (format.toUpperCase()) {
            case "PDF":
                return "pdf";
            case "EXCEL":
                return "xlsx";
            case "CSV":
                return "csv";
            default:
                return "bin";
        }
    }
}
