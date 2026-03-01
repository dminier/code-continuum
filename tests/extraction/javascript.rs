// Test d'intégration : extraction et analyse JavaScript
use tree_sitter::Parser;

#[test]
fn test_javascript_function_extraction() {
    let javascript = tree_sitter_javascript::language();
    let mut parser = Parser::new();
    parser
        .set_language(javascript)
        .expect("Failed to set language");

    let source = r#"
function processData(input) {
    return transformData(input);
}

function transformData(data) {
    return data.toUpperCase();
}

class DataService {
    constructor() {
        this.cache = {};
    }
    
    getData(key) {
        return this.cache[key];
    }
}
"#;

    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();

    // Chercher les déclarations de fonction
    let mut found_functions = Vec::new();

    fn find_functions(node: tree_sitter::Node, source: &str, found: &mut Vec<String>) {
        if node.kind() == "function_declaration" || node.kind() == "function" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(text) = name_node.utf8_text(source.as_bytes()) {
                    found.push(text.to_string());
                }
            }
        }

        // Pour les méthodes dans les classes
        if node.kind() == "method_definition" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(text) = name_node.utf8_text(source.as_bytes()) {
                    found.push(text.to_string());
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_functions(child, source, found);
        }
    }

    find_functions(root, source, &mut found_functions);

    // Vérifier que les fonctions ont été trouvées
    assert!(found_functions.contains(&"processData".to_string()));
    assert!(found_functions.contains(&"transformData".to_string()));
}

/// Test d'extraction des méthodes dans les object literals
/// Pattern: var obj = { methodName: function() { ... } }
#[test]
fn test_javascript_object_literal_methods_extraction() {
    let javascript = tree_sitter_javascript::language();
    let mut parser = Parser::new();
    parser
        .set_language(javascript)
        .expect("Failed to set language");

    let source = r#"
var compasNotification = {
    handlerOnClick: null,
    nomPortletEnCours: null,
    
    initPopupNotificationActeur: function(pIndicManuel) {
        compasNotification.indicManuel = pIndicManuel;
        openPopup('popupTraitementAttente');
    },
    
    chargeActeur: function(pIdChamps, pType, pSource) {
        var lRadioBoutonEquipeTous = null;
        if(pSource && pSource == "delegation") {
            lRadioBoutonEquipeTous = $('acteursDelegationRadioBoutonEquipeDelegation');
        }
        openPopupAffectationCommercialSeul(pIdChamps, pType, lRadioBoutonValue);
    },
    
    validerNotificationActeur: function() {
        var lFlagErreur = compasNotification.checkFieldsNotificationActeur();
        if(lFlagErreur < 1) {
            closePopupSansClone("popupNotificationActeur");
        }
    },
    
    // Arrow function syntax also supported
    checkFields: (pForm) => {
        return pForm.validate();
    }
}
"#;

    let tree = parser.parse(source, None).expect("Parse failed");
    let root = tree.root_node();

    // Chercher les méthodes dans les object literals (pair avec function comme valeur)
    let mut found_methods = Vec::new();
    let mut found_objects = Vec::new();

    fn find_object_methods(
        node: tree_sitter::Node,
        source: &str,
        methods: &mut Vec<String>,
        objects: &mut Vec<String>,
    ) {
        // Trouver les variable_declarator avec un object literal
        if node.kind() == "variable_declarator" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Some(value_node) = node.child_by_field_name("value") {
                    if value_node.kind() == "object" {
                        if let Ok(obj_name) = name_node.utf8_text(source.as_bytes()) {
                            objects.push(obj_name.to_string());
                        }
                    }
                }
            }
        }

        // Trouver les pair avec function comme valeur
        // Note: Tree-sitter JS utilise "function_expression" pour les fonctions anonymes dans les objets
        if node.kind() == "pair" {
            if let Some(key_node) = node.child_by_field_name("key") {
                if let Some(value_node) = node.child_by_field_name("value") {
                    let value_kind = value_node.kind();
                    // function_expression pour `function() {}` et arrow_function pour `() => {}`
                    if value_kind == "function"
                        || value_kind == "function_expression"
                        || value_kind == "arrow_function"
                    {
                        if let Ok(method_name) = key_node.utf8_text(source.as_bytes()) {
                            methods.push(method_name.to_string());
                        }
                    }
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            find_object_methods(child, source, methods, objects);
        }
    }

    find_object_methods(root, source, &mut found_methods, &mut found_objects);

    // Debug: afficher ce qu'on a trouvé
    println!("Found objects: {:?}", found_objects);
    println!("Found methods: {:?}", found_methods);

    // Vérifier que l'objet a été trouvé
    assert!(
        found_objects.contains(&"compasNotification".to_string()),
        "Object 'compasNotification' should be found. Found: {:?}",
        found_objects
    );

    // Vérifier que les méthodes dans l'object literal ont été trouvées
    assert!(
        found_methods.contains(&"initPopupNotificationActeur".to_string()),
        "Method 'initPopupNotificationActeur' should be found. Found: {:?}",
        found_methods
    );
    assert!(
        found_methods.contains(&"chargeActeur".to_string()),
        "Method 'chargeActeur' should be found. Found: {:?}",
        found_methods
    );
    assert!(
        found_methods.contains(&"validerNotificationActeur".to_string()),
        "Method 'validerNotificationActeur' should be found. Found: {:?}",
        found_methods
    );
    assert!(
        found_methods.contains(&"checkFields".to_string()),
        "Arrow function method 'checkFields' should be found. Found: {:?}",
        found_methods
    );

    // Vérifier le nombre total de méthodes trouvées
    assert_eq!(
        found_methods.len(),
        4,
        "Should find 4 methods in object literal. Found: {:?}",
        found_methods
    );
}
