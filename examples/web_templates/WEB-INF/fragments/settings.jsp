<%-- Settings page fragment --%>
<%@ page contentType="text/html;charset=UTF-8" language="java" %>

<div id="settings-panel" class="settings-form">
    <h3>Application Settings</h3>
    
    <div class="settings-group">
        <label for="theme-select">Theme:</label>
        <select id="theme-select">
            <option value="light">Light</option>
            <option value="dark">Dark</option>
        </select>
    </div>
    
    <div class="settings-group">
        <label for="language-select">Language:</label>
        <select id="language-select">
            <option value="en">English</option>
            <option value="fr">Français</option>
        </select>
    </div>
    
    <button id="save-settings-btn" class="btn btn-primary">Save Settings</button>
</div>

<script src="/resources/js/settings-manager.js"></script>
