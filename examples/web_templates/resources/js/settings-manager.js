/**
 * Settings Manager - Handles settings page functionality
 * Included by: WEB-INF/fragments/settings.jsp
 * Depends on: api-client.js
 */

$(document).ready(function() {
    // Load saved settings
    ApiClient.get('/settings').then(settings => {
        $('#theme-select').val(settings.theme);
        $('#language-select').val(settings.language);
    });
    
    // Save settings
    $('#save-settings-btn').on('click', async function() {
        const settings = {
            theme: $('#theme-select').val(),
            language: $('#language-select').val()
        };
        
        try {
            await ApiClient.put('/settings', settings);
            alert('Settings saved successfully!');
        } catch (error) {
            alert('Failed to save settings');
        }
    });
});
