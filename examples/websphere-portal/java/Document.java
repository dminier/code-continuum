package com.example.models;

/**
 * Document model class
 */
public class Document {
    private String id;
    private String fileName;
    private String folderId;
    private String repositoryPath;
    private long fileSize;
    private String mimeType;
    private long createdDate;
    private String createdBy;
    
    public Document() {
    }
    
    public Document(String id, String fileName, String folderId) {
        this.id = id;
        this.fileName = fileName;
        this.folderId = folderId;
    }
    
    // Getters and Setters
    public String getId() {
        return id;
    }
    
    public void setId(String id) {
        this.id = id;
    }
    
    public String getFileName() {
        return fileName;
    }
    
    public void setFileName(String fileName) {
        this.fileName = fileName;
    }
    
    public String getFolderId() {
        return folderId;
    }
    
    public void setFolderId(String folderId) {
        this.folderId = folderId;
    }
    
    public String getRepositoryPath() {
        return repositoryPath;
    }
    
    public void setRepositoryPath(String repositoryPath) {
        this.repositoryPath = repositoryPath;
    }
    
    public long getFileSize() {
        return fileSize;
    }
    
    public void setFileSize(long fileSize) {
        this.fileSize = fileSize;
    }
    
    public String getMimeType() {
        return mimeType;
    }
    
    public void setMimeType(String mimeType) {
        this.mimeType = mimeType;
    }
    
    public long getCreatedDate() {
        return createdDate;
    }
    
    public void setCreatedDate(long createdDate) {
        this.createdDate = createdDate;
    }
    
    public String getCreatedBy() {
        return createdBy;
    }
    
    public void setCreatedBy(String createdBy) {
        this.createdBy = createdBy;
    }
}
