# Usecase 5: Report Generation and Export Portal

## Vue d'Ensemble
Génération de rapports avec export multi-formats et téléchargement.

## Architecture

```
Portlet (ReportPortlet.java)
  ├─ doView(): Liste rapports
  ├─ processAction(): Generate/Export
  └─ serveResource(): Download
       ↓
Service (ReportService.java)
  ├─ getAvailableReports()
  ├─ generateReport(reportId, filters)
  └─ exportReport(reportId, format)
       ↓
Generators (ReportGenerator, ExportManager)
  ├─ ReportGenerator.generate(report, filters)
  └─ ExportManager.export(report, format)
```

## Flux Principaux

### 1. Affichage Rapports Disponibles
```
1. ReportPortlet.doView()
   ↓
2. ReportService.getAvailableReports()
   └─ Retour List<Report>
   ↓
3. dispatch("/WEB-INF/portlets/report-list.jsp")
```

### 2. Génération Rapport avec Filtres
```
1. User sélectionne rapport + filtres dans report-list.jsp
   ↓
2. report-manager.js: onGenerateClick(reportId, filters)
   ├─ Valide filtres
   └─ AJAX POST ?action=generate&reportId={id}&filters={filterStr}
   ↓
3. ReportPortlet.processAction()
   ├─ action = "generate"
   ├─ reportId from request
   └─ filters = parseFilters(request.getParameter("filters"))
   ↓
4. ReportService.generateReport(reportId, filters)
   ├─ ReportGenerator.generate(report, filters)
   │  └─ Construit rapport avec filtres
   └─ DatabaseConnector.saveReport()
   ↓
5. response.setRenderParameter("reportId", reportId)
```

### 3. Export Rapport
```
1. User clique [Export as PDF/Excel/CSV]
   ↓
2. report-manager.js: onExportClick(reportId, format)
   ├─ Valide format (isValidFormat)
   └─ AJAX POST ?action=export&reportId={id}&format={fmt}
   ↓
3. ReportPortlet.processAction()
   ├─ action = "export"
   ├─ format from request
   └─ isValidFormat(format) validation
   ↓
4. ReportService.exportReport(reportId, format)
   ├─ ExportManager.export(report, format)
   │  ├─ switch(format):
   │  │  ├─ case "PDF": generatePdf()
   │  │  ├─ case "Excel": generateExcel()
   │  │  └─ case "CSV": generateCsv()
   │  └─ Retour byte[]
   └─ Cache export pour download
   ↓
5. response.setRenderParameter("reportId", reportId)
   response.setRenderParameter("export", format)
```

### 4. Download Rapport
```
1. User clique [Download]
   ↓
2. Browser GET /portal/page?resourceID=downloadReport&reportId={id}&format={fmt}
   ↓
3. ReportPortlet.serveResource()
   ├─ resourceID = "downloadReport"
   ├─ isValidFormat(format) check
   └─ ReportService.exportReport(reportId, format)
   ↓
4. response.setContentType(getContentType(format))
   response.setProperty("Content-Disposition", "attachment; filename=...")
   ↓
5. response.getPortletOutputStream().write(fileBytes)
   ↓
6. Browser démarre téléchargement
```

## Fichiers Impliqués

### Java
- **ReportPortlet.java**: doView(), processAction(), serveResource()
- **ReportService.java**: generateReport(), exportReport(), getAvailableReports()
- **ReportGenerator.java**: generate(report, filters)
- **ExportManager.java**: export(report, format)
  - generatePdf(content)
  - generateExcel(content)
  - generateCsv(content)
- **Report.java**: Modèle report

### JSP/JSPF
- **report-list.jsp**: Liste rapports
- **report-detail.jsp**: Détails rapport généré
- **report-filters.jspf**: Fragment filtres

### JavaScript
- **report-manager.js**: Logique portlet
  - `onGenerateClick(reportId, filters)`
  - `onExportClick(reportId, format)`
  - `onDownloadClick(reportId, format)`
  - `validateFormat(format)`

- **filter-builder.js**: Construction filtres
  - `buildFilterString(formData)`
  - `validateFilters(filters)`

## Relations Sémantiques

```
PORTLET_USES_JSP:
  ReportPortlet --renders--> report-list.jsp
  ReportPortlet --renders--> report-detail.jsp

JSP_INCLUDES_JS:
  report-list.jsp --includes--> report-manager.js
  report-list.jsp --includes--> filter-builder.js

JS_CALLS_PORTLET:
  report-manager.js --ajax-post--> ReportPortlet.processAction()
  report-manager.js --http-get--> ReportPortlet.serveResource()

PORTLET_CALLS_SERVICE:
  ReportPortlet.doView() --calls--> ReportService.getAvailableReports()
  ReportPortlet.processAction() --calls--> ReportService.generateReport()
  ReportPortlet.processAction() --calls--> ReportService.exportReport()
  ReportPortlet.serveResource() --calls--> ReportService.exportReport()

SERVICE_CALLS_GENERATORS:
  ReportService --calls--> ReportGenerator.generate()
  ReportService --calls--> ExportManager.export()

EXPORT_MANAGER_PATTERNS:
  ExportManager.export() --switch-format-->
    ├─ generatePdf()
    ├─ generateExcel()
    └─ generateCsv()
```

## Export Format Handling

```java
// Exemple de dispatch dans ExportManager:
switch (format.toUpperCase()) {
    case "PDF":
        content = generatePdf(report);
        break;
    case "EXCEL":
        content = generateExcel(report);
        break;
    case "CSV":
        content = generateCsv(report);
        break;
    default:
        throw new IllegalArgumentException("Invalid format");
}
```

## Métadonnées Attendues

```
Nœuds: 10 (1 Portlet + 1 Service + 2 Generators + 1 Model + 2 JSPs + 3 JS)
Arêtes: ~14 (Portlet→JSP, JSP→JS, JS→Portlet, Portlet→Service, 
             Service→Generators, Export switch branches)
```

---

**Status**: Draft
**Complexity**: Medium (export formats, binary streaming)
**Key Feature**: Multi-format export avec Content-Disposition
