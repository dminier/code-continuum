package com.example.web;

import javax.servlet.ServletException;
import javax.servlet.http.HttpServlet;
import javax.servlet.http.HttpServletRequest;
import javax.servlet.http.HttpServletResponse;
import java.io.IOException;

/**
 * Main dispatcher servlet for request routing
 */
public class DispatcherServlet extends HttpServlet {
    
    private String configLocation;
    
    @Override
    public void init(ServletException config) throws ServletException {
        super.init();
        this.configLocation = getServletConfig().getInitParameter("configLocation");
    }
    
    @Override
    protected void doGet(HttpServletRequest request, HttpServletResponse response)
            throws ServletException, IOException {
        dispatch(request, response);
    }
    
    @Override
    protected void doPost(HttpServletRequest request, HttpServletResponse response)
            throws ServletException, IOException {
        dispatch(request, response);
    }
    
    private void dispatch(HttpServletRequest request, HttpServletResponse response)
            throws ServletException, IOException {
        String path = request.getRequestURI();
        String action = request.getParameter("action");
        
        // Route to appropriate handler
        if ("user".equals(action)) {
            request.getRequestDispatcher("/portals/user").forward(request, response);
        } else if ("document".equals(action)) {
            request.getRequestDispatcher("/portals/document").forward(request, response);
        } else {
            response.sendRedirect("/index.jsp");
        }
    }
}
