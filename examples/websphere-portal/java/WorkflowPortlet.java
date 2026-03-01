package com.example.portlets;

import javax.portlet.*;
import java.io.IOException;
import java.util.List;
import com.example.services.WorkflowService;
import com.example.models.WorkflowTask;
import com.example.models.WorkflowProcess;

/**
 * Workflow Management Portlet
 * Manages workflow processes, approvals, and task assignments
 */
public class WorkflowPortlet extends BasePortlet {
    
    private WorkflowService workflowService;
    private boolean enableNotifications;
    
    @Override
    public void init(PortletConfig config) throws PortletException {
        super.init(config);
        this.workflowService = new WorkflowService();
        this.enableNotifications = Boolean.parseBoolean(getInitParameter("enable-notifications"));
    }
    
    /**
     * Display workflow tasks and processes
     */
    @Override
    protected void doView(RenderRequest request, RenderResponse response) 
            throws PortletException, IOException {
        
        response.setContentType("text/html;charset=UTF-8");
        
        String view = request.getParameter("view");
        String userId = request.getRemoteUser();
        
        if ("process".equals(view)) {
            String processId = request.getParameter("processId");
            WorkflowProcess process = workflowService.getProcess(processId);
            request.setAttribute("process", process);
            dispatch("/WEB-INF/portlets/workflow-process.jsp", request, response);
        } else {
            // List pending tasks
            List<WorkflowTask> tasks = workflowService.getPendingTasks(userId);
            request.setAttribute("tasks", tasks);
            dispatch("/WEB-INF/portlets/workflow-list.jsp", request, response);
        }
    }
    
    /**
     * Process workflow actions (approve, reject, delegate)
     */
    @Override
    public void processAction(ActionRequest request, ActionResponse response) 
            throws PortletException, IOException {
        
        String action = request.getParameter("action");
        String taskId = request.getParameter("taskId");
        String userId = request.getRemoteUser();
        
        if ("approve".equals(action)) {
            String comment = request.getParameter("comment");
            workflowService.approveTask(taskId, userId, comment);
            sendNotification("Task approved: " + taskId);
            
        } else if ("reject".equals(action)) {
            String reason = request.getParameter("reason");
            workflowService.rejectTask(taskId, userId, reason);
            sendNotification("Task rejected: " + taskId);
            
        } else if ("delegate".equals(action)) {
            String delegateTo = request.getParameter("delegateTo");
            workflowService.delegateTask(taskId, userId, delegateTo);
            sendNotification("Task delegated to: " + delegateTo);
        }
        
        response.setRenderParameter("view", "list");
    }
    
    /**
     * Send notification if enabled
     */
    private void sendNotification(String message) {
        if (enableNotifications) {
            workflowService.sendNotification(message);
        }
    }
}
