package com.example.portlets;

import javax.portlet.*;
import java.io.IOException;
import java.util.List;
import com.example.services.DocumentService;
import com.example.models.Document;

/**
 * Document Management Portlet
 * Handles file upload, download, and management
 */
public class DocumentPortlet extends BasePortlet {
    
    private DocumentService documentService;
    private String repositoryPath;
    private long maxFileSize;
    
    @Override
    public void init(PortletConfig config) throws PortletException {
        super.init(config);
        this.documentService = new DocumentService();
        this.repositoryPath = getInitParameter("repository-path");
        this.maxFileSize = Long.parseLong(getInitParameter("max-file-size"));
    }
    
    /**
     * Display document list
     */
    @Override
    protected void doView(RenderRequest request, RenderResponse response) 
            throws PortletException, IOException {
        
        response.setContentType("text/html;charset=UTF-8");
        
        String folderId = request.getParameter("folderId");
        List<Document> documents = documentService.getDocuments(folderId);
        
        request.setAttribute("documents", documents);
        request.setAttribute("repositoryPath", repositoryPath);
        request.setAttribute("maxFileSize", maxFileSize);
        
        dispatch("/WEB-INF/portlets/document-list.jsp", request, response);
    }
    
    /**
     * Process file upload and management actions
     */
    @Override
    public void processAction(ActionRequest request, ActionResponse response) 
            throws PortletException, IOException {
        
        String action = request.getParameter("action");
        
        if ("upload".equals(action)) {
            String fileName = request.getParameter("fileName");
            String folderId = request.getParameter("folderId");
            
            Document document = new Document();
            document.setFileName(fileName);
            document.setFolderId(folderId);
            document.setRepositoryPath(repositoryPath);
            
            documentService.uploadDocument(document);
            
        } else if ("delete".equals(action)) {
            String documentId = request.getParameter("documentId");
            documentService.deleteDocument(documentId);
        }
        
        response.setRenderParameter("folderId", request.getParameter("folderId"));
    }
}
