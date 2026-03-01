/**
 * Main App - Main initialization and orchestration
 * Depends on: user-manager.js, document-manager.js, api-client.js
 * Initializes: tabs, event handlers
 */

$(document).ready(function() {
    console.log('Initializing main application...');

    // Initialize tabs
    $('#mainTabs a').on('click', function(e) {
        e.preventDefault();
        const tabId = $(this).attr('href');
        $('.tab-content').hide();
        $(tabId).show();
    });

    // Load initial data
    UserManager.loadUsers();
    DocumentManager.loadDocuments();

    // Event handlers
    $('#add-user-btn').on('click', function() {
        console.log('Opening add user dialog');
    });
});
