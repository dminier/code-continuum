<%@ page contentType="text/html;charset=UTF-8" language="java" %>
<%@ taglib uri="http://java.sun.com/jsp/jstl/core" prefix="c" %>
<%@ taglib uri="http://java.sun.com/jsp/jstl/fmt" prefix="fmt" %>
<%@ include file="/WEB-INF/common/header.jspf" %>

<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>User Management Portal</title>
    <link rel="stylesheet" href="/resources/css/bootstrap.css">
    <link rel="stylesheet" href="/resources/css/main.css">
    <link rel="stylesheet" href="/resources/css/theme.css">
</head>
<body>
    <div id="app-header"></div>
    
    <div class="container">
        <h1>User Management System</h1>
        
        <!-- Tab Navigation -->
        <ul class="nav nav-tabs" id="mainTabs">
            <li><a href="#users-tab">Users</a></li>
            <li><a href="#documents-tab">Documents</a></li>
            <li><a href="#settings-tab">Settings</a></li>
        </ul>
        
        <!-- Users Tab -->
        <div id="users-tab" class="tab-content">
            <h2>Manage Users</h2>
            <div id="users-list"></div>
            <button id="add-user-btn" class="btn btn-primary">Add New User</button>
        </div>
        
        <!-- Documents Tab -->
        <div id="documents-tab" class="tab-content">
            <h2>User Documents</h2>
            <div id="documents-list"></div>
        </div>
        
        <!-- Settings Tab -->
        <div id="settings-tab" class="tab-content">
            <jsp:include page="/WEB-INF/fragments/settings.jsp" />
        </div>
    </div>
    
    <div id="modal-container"></div>
    
    <!-- Scripts -->
    <script src="/resources/js/jquery.min.js"></script>
    <script src="/resources/js/bootstrap.js"></script>
    <script src="/resources/js/api-client.js"></script>
    <script src="/resources/js/user-manager.js"></script>
    <script src="/resources/js/document-manager.js"></script>
    <script src="/resources/js/main.js"></script>
</body>
</html>
