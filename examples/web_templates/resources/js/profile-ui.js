/**
 * Profile UI - Renders profile UI components
 * Depends on: profile-api.js
 * Used by: profile.js
 */

const ProfileUI = {
    async renderProfile(profile) {
        const html = `
            <div class="profile-details">
                <h2>${profile.name}</h2>
                <p><strong>Email:</strong> ${profile.email}</p>
                <p><strong>Phone:</strong> ${profile.phone || 'N/A'}</p>
                <p><strong>Member since:</strong> ${profile.joinDate}</p>
            </div>
        `;
        $('#profile-info').html(html);
    },
    
    async renderActivity(activities) {
        const html = activities.map(activity => `
            <div class="activity-item">
                <p><strong>${activity.action}</strong> on ${activity.timestamp}</p>
                <p>${activity.description}</p>
            </div>
        `).join('');
        
        $('#activity-tab').html(html);
    },
    
    async renderPreferences(prefs) {
        const html = `
            <form id="preferences-form">
                <label>Notifications: <input type="checkbox" ${prefs.notifications ? 'checked' : ''} /></label>
                <label>Email: <input type="checkbox" ${prefs.emailNotifications ? 'checked' : ''} /></label>
                <button type="submit" class="btn btn-primary">Save</button>
            </form>
        `;
        
        $('#preferences-tab').html(html);
    }
};
