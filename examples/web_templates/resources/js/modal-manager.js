/**
 * Modal Manager - Handles modal dialogs
 * Included by: WEB-INF/fragments/profile-edit-modal.jsp
 */

$(document).ready(function() {
    // Close modal
    $('#close-modal, #cancel-btn').on('click', function() {
        $('#profile-modal').hide();
    });
    
    // Close modal when clicking outside
    $(window).on('click', function(event) {
        const modal = $('#profile-modal');
        if (event.target === modal[0]) {
            modal.hide();
        }
    });
});
