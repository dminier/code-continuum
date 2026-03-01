<%@ page contentType="text/html;charset=UTF-8" language="java" %>
<%@ taglib uri="http://java.sun.com/jsp/jstl/core" prefix="c" %>

<!DOCTYPE html>
<html>
<head>
    <title>Main Page - JSP Transitive Include Test</title>
    <link rel="stylesheet" href="/resources/css/main.css">
</head>
<body>
    <h1>Test des inclusions transitives JSP</h1>
    
    <!-- Inclusion dynamique de page1.jspx via jsp:include -->
    <div class="content">
        <jsp:include page="page1.jspx"/>
    </div>
    
    <script src="/resources/js/main.js"></script>
</body>
</html>
