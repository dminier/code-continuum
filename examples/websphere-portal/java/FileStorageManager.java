package com.example.services;

import java.io.File;
import java.nio.file.Files;
import java.nio.file.Paths;

/**
 * File storage management
 */
public class FileStorageManager {
    
    private String baseStoragePath;
    
    public FileStorageManager() {
        this.baseStoragePath = System.getenv("STORAGE_PATH");
    }
    
    public String storeFile(String repositoryPath, String fileName) {
        try {
            String fullPath = repositoryPath + File.separator + fileName;
            File file = new File(fullPath);
            file.getParentFile().mkdirs();
            return fullPath;
        } catch (Exception e) {
            throw new RuntimeException("Failed to store file: " + fileName, e);
        }
    }
    
    public void deleteFile(String repositoryPath, String fileName) {
        try {
            String fullPath = repositoryPath + File.separator + fileName;
            Files.deleteIfExists(Paths.get(fullPath));
        } catch (Exception e) {
            throw new RuntimeException("Failed to delete file: " + fileName, e);
        }
    }
    
    public byte[] readFile(String repositoryPath, String fileName) {
        try {
            String fullPath = repositoryPath + File.separator + fileName;
            return Files.readAllBytes(Paths.get(fullPath));
        } catch (Exception e) {
            throw new RuntimeException("Failed to read file: " + fileName, e);
        }
    }
}
