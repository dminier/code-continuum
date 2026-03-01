package com.example.web;

import javax.servlet.ServletException;
import javax.servlet.http.HttpServlet;
import javax.servlet.http.HttpServletRequest;
import javax.servlet.http.HttpServletResponse;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Paths;

/**
 * Resource servlet for serving static resources
 */
public class ResourceServlet extends HttpServlet {
    
    private String resourcePath;
    
    @Override
    public void init(ServletException config) throws ServletException {
        super.init();
        this.resourcePath = getServletConfig().getServletContext().getRealPath("/");
    }
    
    @Override
    protected void doGet(HttpServletRequest request, HttpServletResponse response)
            throws ServletException, IOException {
        
        String path = request.getPathInfo();
        
        if (path == null || path.isEmpty()) {
            response.sendError(HttpServletResponse.SC_NOT_FOUND);
            return;
        }
        
        String fullPath = resourcePath + path;
        
        try {
            byte[] content = Files.readAllBytes(Paths.get(fullPath));
            response.setContentLength(content.length);
            response.getOutputStream().write(content);
        } catch (IOException e) {
            response.sendError(HttpServletResponse.SC_NOT_FOUND);
        }
    }
}
