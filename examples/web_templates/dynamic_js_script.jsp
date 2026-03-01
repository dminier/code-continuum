<%@ page contentType="text/html;charset=UTF-8" language="java" %>
<%@ taglib uri="http://java.sun.com/jsp/jstl/core" prefix="c" %>
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Page avec script JSP dynamique</title>
    
    <!-- Script classique .js -->
    <script src="/resources/js/main.js"></script>
    
    <!-- Script qui pointe vers un fichier JSP (génère du JS dynamique) -->
    <script src="/common/config.jsp"></script>
    
    <!-- Script via c:url pointant vers JSP -->
    <script src="<c:url value="/dynamic/settings.jsp"/>"></script>
    
    <!-- Script JSPX -->
    <script src="/fragments/translations.jspx"></script>
</head>
<body>
    <h1>Page de test pour scripts JSP dynamiques</h1>
    <p>Cette page inclut des scripts JavaScript qui sont en fait générés par des JSP.</p>
</body>
</html>
