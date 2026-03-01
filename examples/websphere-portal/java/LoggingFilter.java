package com.example.filters;

import javax.servlet.*;
import javax.servlet.http.HttpServletRequest;
import java.io.IOException;

/**
 * Logging filter for request/response logging
 */
public class LoggingFilter implements Filter {
    
    @Override
    public void init(FilterConfig config) throws ServletException {
        // Initialize filter
    }
    
    @Override
    public void doFilter(ServletRequest request, ServletResponse response, FilterChain chain)
            throws IOException, ServletException {
        
        HttpServletRequest httpRequest = (HttpServletRequest) request;
        
        long startTime = System.currentTimeMillis();
        String method = httpRequest.getMethod();
        String path = httpRequest.getRequestURI();
        
        // Log request
        System.out.println("Request: " + method + " " + path);
        
        chain.doFilter(request, response);
        
        long duration = System.currentTimeMillis() - startTime;
        System.out.println("Response time: " + duration + "ms");
    }
    
    @Override
    public void destroy() {
        // Cleanup
    }
}
