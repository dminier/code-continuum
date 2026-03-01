import java.sql.*;

public class DatabaseManager {
    private Connection connection;
    private String url = "jdbc:sqlite:app.db";
    
    public DatabaseManager() {
        this.connection = null;
    }
    
    public void connect() throws SQLException {
        connection = DriverManager.getConnection(url);
    }
    
    public User findUserById(int id) {
        User user = new User();
        try {
            String sql = "SELECT * FROM users WHERE id = ?";
            PreparedStatement pstmt = connection.prepareStatement(sql);
            pstmt.setInt(1, id);
            ResultSet rs = pstmt.executeQuery();
            
            if (rs.next()) {
                user.setId(rs.getInt("id"));
                user.setName(rs.getString("name"));
            }
        } catch (SQLException e) {
            e.printStackTrace();
        }
        return user;
    }
    
    public void save(User user) {
        String sql = "INSERT INTO users(name) VALUES(?)";
        executeUpdate(sql, user.getName());
    }
    
    public void update(User user) {
        String sql = "UPDATE users SET name = ? WHERE id = ?";
        executeUpdate(sql, user.getName());
    }
    
    public void delete(int id) {
        String sql = "DELETE FROM users WHERE id = ?";
        executeUpdate(sql);
    }
    
    private void executeUpdate(String sql, Object... params) {
        try {
            PreparedStatement pstmt = connection.prepareStatement(sql);
            for (int i = 0; i < params.length; i++) {
                pstmt.setObject(i + 1, params[i]);
            }
            pstmt.executeUpdate();
        } catch (SQLException e) {
            e.printStackTrace();
        }
    }
    
    public void close() throws SQLException {
        if (connection != null) {
            connection.close();
        }
    }
}
