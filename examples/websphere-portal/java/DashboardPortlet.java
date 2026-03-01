package com.example.portlets;

import javax.portlet.*;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.Map;
import com.example.services.DashboardService;

/**
 * Dashboard Portlet
 * Displays statistics and analytics
 */
public class DashboardPortlet extends BasePortlet {
    
    private DashboardService dashboardService;
    private int refreshInterval;
    
    @Override
    public void init(PortletConfig config) throws PortletException {
        super.init(config);
        this.dashboardService = new DashboardService();
        this.refreshInterval = Integer.parseInt(getInitParameter("refresh-interval"));
    }
    
    /**
     * Render dashboard with statistics
     */
    @Override
    protected void doView(RenderRequest request, RenderResponse response) 
            throws PortletException, IOException {
        
        response.setContentType("text/html;charset=UTF-8");
        
        // Gather dashboard data
        Map<String, Object> stats = dashboardService.getStatistics();
        Map<String, Object> recentActivities = dashboardService.getRecentActivities();
        
        request.setAttribute("statistics", stats);
        request.setAttribute("activities", recentActivities);
        request.setAttribute("refreshInterval", refreshInterval);
        
        dispatch("/WEB-INF/portlets/dashboard.jsp", request, response);
    }
    
    /**
     * Server-side resource handling for AJAX requests
     */
    @Override
    public void serveResource(ResourceRequest request, ResourceResponse response) 
            throws PortletException, IOException {
        
        String resourceId = request.getResourceID();
        response.setContentType("application/json");
        PrintWriter writer = response.getWriter();
        
        if ("refreshStats".equals(resourceId)) {
            Map<String, Object> stats = dashboardService.getStatistics();
            writer.write(convertToJson(stats));
        }
    }
    
    private String convertToJson(Map<String, Object> data) {
        // Simple JSON conversion
        StringBuilder json = new StringBuilder("{");
        for (Map.Entry<String, Object> entry : data.entrySet()) {
            json.append("\"").append(entry.getKey()).append("\":\"")
                .append(entry.getValue()).append("\",");
        }
        if (json.length() > 1) {
            json.setLength(json.length() - 1);
        }
        json.append("}");
        return json.toString();
    }
}
