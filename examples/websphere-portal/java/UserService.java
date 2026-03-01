package com.example.services;

import com.example.models.User;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/**
 * Service for user management operations
 */
public class UserService {
    
    private Map<String, User> userDatabase;
    private DatabaseConnector dbConnector;
    
    public UserService() {
        this.userDatabase = new HashMap<>();
        this.dbConnector = new DatabaseConnector();
        initializeUsers();
    }
    
    /**
     * Get all users with pagination
     */
    public List<User> getAllUsers(int pageSize) {
        List<User> users = new ArrayList<>(userDatabase.values());
        return users.subList(0, Math.min(pageSize, users.size()));
    }
    
    /**
     * Get user by ID
     */
    public User getUserById(String userId) {
        return userDatabase.getOrDefault(userId, null);
    }
    
    /**
     * Save or update user
     */
    public void saveUser(User user) {
        userDatabase.put(user.getId(), user);
        dbConnector.saveUser(user);
    }
    
    /**
     * Delete user
     */
    public void deleteUser(String userId) {
        userDatabase.remove(userId);
        dbConnector.deleteUser(userId);
    }
    
    /**
     * Search users by criteria
     */
    public List<User> searchUsers(String criteria) {
        List<User> results = new ArrayList<>();
        for (User user : userDatabase.values()) {
            if (user.getName().contains(criteria) || user.getEmail().contains(criteria)) {
                results.add(user);
            }
        }
        return results;
    }
    
    /**
     * Initialize sample users
     */
    private void initializeUsers() {
        User user1 = new User("USR001", "John Doe", "john.doe@example.com");
        user1.setDepartment("IT");
        user1.setActive(true);
        
        User user2 = new User("USR002", "Jane Smith", "jane.smith@example.com");
        user2.setDepartment("HR");
        user2.setActive(true);
        
        userDatabase.put(user1.getId(), user1);
        userDatabase.put(user2.getId(), user2);
    }
}
