<%@ page language="java" contentType="text/html; charset=UTF-8" pageEncoding="UTF-8"%>
<%@page import="com.example.portal.fo.web.portlets.synthese.GestionEPSessionBean"%>
<%@page import="com.example.portal.fo.web.portlets.synthese.GestionEPPortlet"%>
<%@page import="com.example.portal.fo.util.PortalProperties"%>
<%@page import="com.example.portal.fo.util.Constantes"%>
<%@page import="com.example.portal.fo.util.UserProfileUtils"%>
<%@page import="com.example.portal.fo.web.portlets.GeneriquePortlet"%>
<%@page import="java.util.List"%>
<%@page import="java.util.ArrayList"%>

<!DOCTYPE html>
<html>
<head>
    <title>Gestion EP</title>
</head>
<body>
    <h1>Gestion des Épargnes</h1>
    <%
        GestionEPSessionBean sessionBean = new GestionEPSessionBean();
        List<String> items = new ArrayList<>();
    %>
</body>
</html>
