public class UserService {
    private DatabaseManager dbManager;
    
    public UserService() {
        this.dbManager = new DatabaseManager();
    }
    
    public User getUserById(int id) {
        return dbManager.findUserById(id);
    }
    
    public void createUser(User user) {
        validateUser(user);
        dbManager.save(user);
    }
    
    public void updateUser(User user) {
        if (user.getId() <= 0) {
            throw new IllegalArgumentException("Invalid user ID");
        }
        dbManager.update(user);
    }
    
    public void deleteUser(int id) {
        dbManager.delete(id);
    }
    
    private void validateUser(User user) {
        if (user.getName() == null || user.getName().isEmpty()) {
            throw new IllegalArgumentException("User name cannot be empty");
        }
    }
}
