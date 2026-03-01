/**
 * User Manager - Manages user operations via AJAX
 * Depends on: api-client.js
 * Used by: main.js
 */

const UserManager = {
    container: '#users-list',
    
    async loadUsers() {
        try {
            const users = await ApiClient.get('/users');
            this.renderUsers(users);
        } catch (error) {
            console.error('Failed to load users:', error);
        }
    },
    
    renderUsers(users) {
        const html = users.map(user => `
            <div class="user-item" data-user-id="${user.id}">
                <h4>${user.name}</h4>
                <p>Email: ${user.email}</p>
                <button onclick="UserManager.editUser(${user.id})">Edit</button>
                <button onclick="UserManager.deleteUser(${user.id})">Delete</button>
            </div>
        `).join('');
        
        $(this.container).html(html);
    },
    
    async addUser(userData) {
        try {
            await ApiClient.post('/users', userData);
            this.loadUsers();
        } catch (error) {
            console.error('Failed to add user:', error);
        }
    },
    
    async editUser(userId) {
        try {
            const user = await ApiClient.get(`/users/${userId}`);
            console.log('Editing user:', user);
        } catch (error) {
            console.error('Failed to edit user:', error);
        }
    },
    
    async deleteUser(userId) {
        try {
            await ApiClient.delete(`/users/${userId}`);
            this.loadUsers();
        } catch (error) {
            console.error('Failed to delete user:', error);
        }
    }
};
