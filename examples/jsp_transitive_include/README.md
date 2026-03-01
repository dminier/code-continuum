# Fixtures pour test d'inclusions JSP transitives

Ce répertoire contient des fixtures pour tester la détection des inclusions JSP/JSPF transitives.

## Structure des inclusions

```
web/
├── main.jsp                    # Page principale
│   └── INCLUDES_JSP → page1.jspx
│
├── page1.jspx                  # Page XML JSP
│   ├── INCLUDES_JSP → includes/fragment.jspf
│   └── INCLUDES_JSP → includes/sidebar.jspf
│
└── includes/
    ├── fragment.jspf           # Fragment principal
    │   └── INCLUDES_JSP → sub/nested-fragment.jspf
    │
    ├── sidebar.jspf            # Fragment sidebar
    │   └── INCLUDES_JSP → sub/menu-items.jspf
    │
    └── sub/
        ├── nested-fragment.jspf    # Fragment imbriqué niveau 2
        └── menu-items.jspf         # Items de menu
```

## Relations attendues

| Source | Target | Type |
|--------|--------|------|
| main.jsp | page1.jspx | dynamic |
| page1.jspx | includes/fragment.jspf | dynamic |
| page1.jspx | includes/sidebar.jspf | static |
| includes/fragment.jspf | sub/nested-fragment.jspf | static |
| includes/sidebar.jspf | sub/menu-items.jspf | static |

## Cas de test

1. **Inclusion simple** : main.jsp → page1.jspx
2. **Inclusion JSPF** : page1.jspx → fragment.jspf
3. **Inclusion transitive** : main.jsp → page1.jspx → fragment.jspf → nested-fragment.jspf
4. **Multiples inclusions** : page1.jspx inclut plusieurs fragments
