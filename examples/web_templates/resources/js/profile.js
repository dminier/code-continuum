/**
 * Profile - Main profile page logic
 * Depends on: profile-api.js, profile-ui.js
 */

$(document).ready(async function() {
    console.log('Loading profile page...');
    
    try {
        // Load and render profile
        const profile = await ProfileApi.getProfile();
        await ProfileUI.renderProfile(profile);
        
        // Load activity
        const activity = await ProfileApi.getActivity();
        await ProfileUI.renderActivity(activity);
        
        // Load preferences
        const prefs = await ProfileApi.getPreferences();
        await ProfileUI.renderPreferences(prefs);
        
        // Event handlers
        $('#edit-profile-btn').on('click', function() {
            $('#profile-modal').show();
        });
        
        $('#profile-form').on('submit', async function(e) {
            e.preventDefault();
            const data = {
                name: $('#profile-name').val(),
                email: $('#profile-email').val(),
                phone: $('#profile-phone').val()
            };
            await ProfileApi.updateProfile(data);
            $('#profile-modal').hide();
        });
        
    } catch (error) {
        console.error('Failed to load profile:', error);
    }
});
