package com.example.portlets;

import javax.portlet.*;
import java.io.IOException;
import java.io.PrintWriter;
import java.util.List;
import com.example.services.UserService;
import com.example.models.User;

/**
 * User Management Portlet
 * Displays list of users and allows editing user profiles
 */
public class UserManagementPortlet extends BasePortlet {
    
    private UserService userService;
    
    @Override
    public void init(PortletConfig config) throws PortletException {
        super.init(config);
        this.userService = new UserService();
    }
    
    /**
     * Render view mode (user list)
     */
    @Override
    protected void doView(RenderRequest request, RenderResponse response) 
            throws PortletException, IOException {
        
        response.setContentType("text/html;charset=UTF-8");
        
        String view = request.getParameter("view");
        String templatePath = getInitParameter("template-view");
        
        if ("edit".equals(view)) {
            templatePath = getInitParameter("template-edit");
            String userId = request.getParameter("userId");
            User user = userService.getUserById(userId);
            request.setAttribute("user", user);
        } else {
            // Display list of users
            int pageSize = Integer.parseInt(getInitParameter("pageSize"));
            List<User> users = userService.getAllUsers(pageSize);
            request.setAttribute("users", users);
        }
        
        dispatch(templatePath, request, response);
    }
    
    /**
     * Render edit mode (user profile editing)
     */
    @Override
    protected void doEdit(RenderRequest request, RenderResponse response) 
            throws PortletException, IOException {
        
        response.setContentType("text/html;charset=UTF-8");
        
        String templatePath = getInitParameter("template-edit");
        String userId = request.getParameter("userId");
        
        if (userId != null) {
            User user = userService.getUserById(userId);
            request.setAttribute("user", user);
        }
        
        dispatch(templatePath, request, response);
    }
    
    /**
     * Process action (form submission)
     */
    @Override
    public void processAction(ActionRequest request, ActionResponse response) 
            throws PortletException, IOException {
        
        String action = request.getParameter("action");
        
        if ("save".equals(action)) {
            String userId = request.getParameter("userId");
            String name = request.getParameter("name");
            String email = request.getParameter("email");
            
            User user = new User();
            user.setId(userId);
            user.setName(name);
            user.setEmail(email);
            
            userService.saveUser(user);
            
            response.setRenderParameter("view", "list");
        } else if ("delete".equals(action)) {
            String userId = request.getParameter("userId");
            userService.deleteUser(userId);
            response.setRenderParameter("view", "list");
        }
    }
}
