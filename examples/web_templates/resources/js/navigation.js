/**
 * Navigation - Handles navigation and menu interactions
 * Included by: common/header.jspf
 */

$(document).ready(function() {
    // Active menu highlighting
    const currentPath = window.location.pathname;
    $('.navbar-menu a').each(function() {
        if ($(this).attr('href') === currentPath) {
            $(this).addClass('active');
        }
    });
});
