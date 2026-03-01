package com.example.filters;

import javax.servlet.*;
import javax.servlet.http.HttpServletRequest;
import java.io.IOException;

/**
 * Authentication filter
 */
public class AuthenticationFilter implements Filter {
    
    @Override
    public void init(FilterConfig config) throws ServletException {
        // Initialize filter
    }
    
    @Override
    public void doFilter(ServletRequest request, ServletResponse response, FilterChain chain)
            throws IOException, ServletException {
        
        HttpServletRequest httpRequest = (HttpServletRequest) request;
        String path = httpRequest.getRequestURI();
        
        // Check if user is authenticated
        if (!path.contains("login") && httpRequest.getSession().getAttribute("user") == null) {
            // Redirect to login
            httpRequest.getRequestDispatcher("/login.jsp").forward(request, response);
            return;
        }
        
        chain.doFilter(request, response);
    }
    
    @Override
    public void destroy() {
        // Cleanup
    }
}
