package com.example.services;

import com.example.models.Document;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.io.File;

/**
 * Service for document management operations
 */
public class DocumentService {
    
    private Map<String, Document> documentCache;
    private DatabaseConnector dbConnector;
    private FileStorageManager storageManager;
    
    public DocumentService() {
        this.documentCache = new HashMap<>();
        this.dbConnector = new DatabaseConnector();
        this.storageManager = new FileStorageManager();
    }
    
    /**
     * Get documents in a folder
     */
    public List<Document> getDocuments(String folderId) {
        List<Document> documents = new ArrayList<>();
        for (Document doc : documentCache.values()) {
            if (folderId == null || folderId.equals(doc.getFolderId())) {
                documents.add(doc);
            }
        }
        return documents;
    }
    
    /**
     * Upload document to repository
     */
    public void uploadDocument(Document document) {
        // Store in file system
        String filePath = storageManager.storeFile(
            document.getRepositoryPath(), 
            document.getFileName()
        );
        
        document.setCreatedDate(System.currentTimeMillis());
        documentCache.put(document.getId(), document);
        
        // Save metadata to database
        dbConnector.saveDocument(document);
    }
    
    /**
     * Delete document
     */
    public void deleteDocument(String documentId) {
        Document document = documentCache.remove(documentId);
        if (document != null) {
            storageManager.deleteFile(
                document.getRepositoryPath(), 
                document.getFileName()
            );
            dbConnector.deleteDocument(documentId);
        }
    }
    
    /**
     * Get document metadata
     */
    public Document getDocument(String documentId) {
        return documentCache.getOrDefault(documentId, null);
    }
    
    /**
     * Search documents by name
     */
    public List<Document> searchDocuments(String searchTerm) {
        List<Document> results = new ArrayList<>();
        for (Document doc : documentCache.values()) {
            if (doc.getFileName().contains(searchTerm)) {
                results.add(doc);
            }
        }
        return results;
    }
}
