/**
 * Profile API - Handles profile-specific API calls
 * Depends on: api-client.js
 * Used by: profile.js
 */

const ProfileApi = {
    async getProfile() {
        return await ApiClient.get('/profile');
    },
    
    async updateProfile(profileData) {
        return await ApiClient.put('/profile', profileData);
    },
    
    async getActivity() {
        return await ApiClient.get('/profile/activity');
    },
    
    async getPreferences() {
        return await ApiClient.get('/profile/preferences');
    },
    
    async updatePreferences(prefs) {
        return await ApiClient.put('/profile/preferences', prefs);
    },
    
    async uploadAvatar(file) {
        const formData = new FormData();
        formData.append('file', file);
        return fetch('/api/v1/profile/avatar', {
            method: 'POST',
            body: formData
        }).then(r => r.json());
    }
};
