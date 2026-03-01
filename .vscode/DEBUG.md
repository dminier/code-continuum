# VS Code Debug Configuration

## Debug Configurations Available

### 1. **Debug: examples folder** (Principal)
Lance l'analyse sur le dossier `examples/` avec tous les fichiers de test.

**Shorcut:** `F5` (debug principal)

**Comportement:**
- Analyse: `examples/`
- Logs: `RUST_LOG=info`
- Neo4j: Configuration locale (localhost:7687)
- Console: Integrated Terminal

### 2. **Debug: Analyser le projet demo**
Lance l'analyse sur le dossier `.samples/demo-portlets`

**Usage:** `Ctrl+Shift+D` → Sélectionner configuration

### 3. **Debug: Custom path (prompt)**
Demande le chemin du projet à analyser

**Usage:** `Ctrl+Shift+D` → Sélectionner configuration → Entrer le chemin

## Variables d'Environnement

Les configurations incluent automatiquement:
```
RUST_LOG=info                    # Niveau de log (console + fichier)
NEO4J_URI=bolt://localhost:7687  # Serveur Neo4j
NEO4J_USER=neo4j                 # Utilisateur Neo4j
NEO4J_PASSWORD=password          # Mot de passe Neo4j
```

Pour modifier, éditer `.vscode/launch.json` → section `env`

## Tâches de Build

Disponibles via `Ctrl+Shift+B`:

| Tâche | Commande |
|-------|----------|
| **build** (défaut) | `cargo build` |
| **build release** | `cargo build --release` |
| **test** | `cargo test -q` |
| **format** | `cargo fmt` |
| **lint** | `cargo clippy --all-targets` |

## Points de Rupture (Breakpoints)

1. Cliquer à gauche du numéro de ligne pour ajouter un breakpoint
2. La barre rouge apparaît
3. Au lancement du debug (F5), l'exécution s'arrête au breakpoint
4. Variables visibles dans le panneau "Variables"

## Inspection du Code

Lors de l'arrêt à un breakpoint:

- **Variables Panel** (gauche): Voir toutes les variables locales/globales
- **Watch**: Ajouter des expressions personnalisées à surveiller
- **Call Stack**: Voir la pile d'appels
- **Debug Console**: Évaluer des expressions Rust

## Commandes de Debug

| Touche | Action |
|--------|--------|
| F5 | Continuer / Lancer debug |
| F10 | Step over (ligne suivante) |
| F11 | Step into (entrer dans fonction) |
| Shift+F11 | Step out (sortir fonction) |
| Shift+F5 | Arrêter debug |

## Exemple d'Utilisation

1. Ouvrir `src/main.rs`
2. Ajouter un breakpoint à la ligne qui vous intéresse
3. Presser `F5` pour lancer le debug sur `examples/`
4. Le programme s'exécute jusqu'au breakpoint
5. Inspecter les variables, expressions, etc.
6. Presser F10 pour avancer ligne par ligne

## Logs et Sortie

- **Console intégrée** (bas): Affiche les logs en temps réel
- **Fichier log**: `.output/app.log` (logs persistants)
- **Rapport**: `.output/report.json` (résultats d'analyse)

## Variables disponibles

Dans `launch.json`, vous pouvez utiliser:
- `${workspaceFolder}`: Racine du projet
- `${workspaceFolderBasename}`: Nom du dossier racine
- `${file}`: Fichier actuellement ouvert
- `${input:projectPath}`: Demander à l'utilisateur une valeur

## Troubleshooting

### "Unable to debug: LLDB not found"
Solution: Installer CodeLLDB extension (`vadimcn.vscode-lldb`)

### "Program hung / infinite loop"
Solution: Shift+F5 pour arrêter

### "Breakpoints not working"
Solution: 
1. Rebuild: `Ctrl+Shift+B` → build
2. Redémarrer debug: F5

### "Cannot reach Neo4j"
Solution:
1. Vérifier que Neo4j est en cours d'exécution
2. Modifier les variables `NEO4J_*` dans `launch.json`
3. Ou désactiver le check en commentant la ligne dans `main.rs`
