use std::path::Path;
use tracing::debug;

pub fn read_text_with_encoding_detection(path: &Path) -> Result<String, std::io::Error> {
    let bytes = std::fs::read(path)?;

    // Fast-path: tenter UTF-8 direct
    if let Ok(s) = String::from_utf8(bytes.clone()) {
        return Ok(s);
    }

    // Détection d'encodage puis décodage via encoding_rs
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(&bytes, true);
    let enc = detector.guess(None, true); // None => pas d'indication de langue
    let (text, _used_encoding, _had_errors) = enc.decode(&bytes);

    // Journaliser l'encodage détecté pour faciliter le debug
    debug!(file = %path.display(), encoding = enc.name(), "Converting encoding to UTF-8");
    Ok(text.into_owned())
}
