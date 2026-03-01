use std::io::{self, Write};
use tracing::info;

/// Affiche une barre de progression visuelle en mode info
///
/// # Arguments
/// * `current` - Le nombre d'éléments traités actuellement
/// * `total` - Le nombre total d'éléments à traiter
/// * `label` - Le label à afficher (ex: "Analyzing files", "Exporting nodes")
/// * `width` - La largeur de la barre en caractères (défaut: 40)
pub fn display_progress(current: usize, total: usize, label: &str, width: usize) {
    if total == 0 {
        return;
    }

    let percentage = (current * 100) / total;
    let filled = (current * width) / total;
    let empty = width - filled;

    let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

    // Affichage sur une seule ligne avec \r (carriage return)
    // À la fin (100%), on passe à la ligne suivante
    if current == total {
        eprintln!(
            "\r{} {} {}/{} ({}%)",
            label, bar, current, total, percentage
        );
    } else {
        eprint!(
            "\r{} {} {}/{} ({}%)",
            label, bar, current, total, percentage
        );
        let _ = io::stderr().flush();
    }
}

/// Affiche une barre de progression simplifiée avec calcul automatique
///
/// # Arguments
/// * `current` - Le nombre d'éléments traités actuellement
/// * `total` - Le nombre total d'éléments à traiter
/// * `label` - Le label à afficher
pub fn show_progress(current: usize, total: usize, label: &str) {
    display_progress(current, total, label, 40);
}

/// Affiche une barre de progression uniquement tous les N éléments pour éviter le spam
///
/// # Arguments
/// * `current` - Le nombre d'éléments traités actuellement
/// * `total` - Le nombre total d'éléments à traiter
/// * `label` - Le label à afficher
/// * `step` - Afficher tous les N éléments (ex: 10, 50, 100)
pub fn show_progress_stepped(current: usize, total: usize, label: &str, step: usize) {
    // Toujours afficher à 0%, 100%, ou tous les 'step' éléments
    if current == 0 || current == total || current % step == 0 {
        show_progress(current, total, label);
    }
}

/// Affiche une barre de progression par batch
///
/// # Arguments
/// * `batch_idx` - L'index du batch actuel (commence à 0)
/// * `batch_size` - La taille d'un batch
/// * `total` - Le nombre total d'éléments
/// * `label` - Le label à afficher
#[allow(dead_code)]
pub fn show_batch_progress(batch_idx: usize, batch_size: usize, total: usize, label: &str) {
    let current = (batch_idx + 1) * batch_size;
    let current_clamped = current.min(total);
    show_progress(current_clamped, total, label);
}

/// Affiche le début d'une phase avec un label formaté
pub fn phase_start(phase_name: &str) {
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("▶ {}", phase_name);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

/// Affiche la fin d'une phase avec un label formaté
pub fn phase_complete(phase_name: &str) {
    info!("✓ {} - Terminé", phase_name);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_progress() {
        // Teste que la fonction ne panique pas
        display_progress(0, 100, "Test", 40);
        display_progress(50, 100, "Test", 40);
        display_progress(100, 100, "Test", 40);
    }

    #[test]
    fn test_show_progress_stepped() {
        // Teste la logique de step
        show_progress_stepped(0, 100, "Test", 10);
        show_progress_stepped(5, 100, "Test", 10);
        show_progress_stepped(10, 100, "Test", 10);
        show_progress_stepped(100, 100, "Test", 10);
    }

    #[test]
    fn test_show_batch_progress() {
        show_batch_progress(0, 50, 1000, "Batch Test");
        show_batch_progress(19, 50, 1000, "Batch Test");
    }
}
