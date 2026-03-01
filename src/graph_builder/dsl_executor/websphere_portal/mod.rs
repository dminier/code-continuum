// Module WebSphere Portal - Extraction des relations sémantiques pour JSP et XML
//
// Extracteurs actifs :
// - JspExtractor : Parse JSP/JSPX/JSPF pour INCLUDES relations (JS, CSS, JSP)
// - XmlExtractor : Parse web.xml et portlet.xml pour configuration mappings

// pub mod ajax_extractor;  // DISABLED: CALLS_AJAX handled by LLM instructions
pub mod jsp_extractor;
// pub mod portlet_extractor;  // REMOVED: Never used, orchestration moved to dsl_executor
pub mod xml_extractor;

pub use jsp_extractor::JspExtractor;
pub use xml_extractor::XmlExtractor;
