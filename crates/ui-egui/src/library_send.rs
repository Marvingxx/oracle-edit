//! File › Add to Oracle Library: the active document goes into the library ("Mes créations") as an
//! Oracle Edit document, as it is now (saved or not); the library draws its thumbnail with Oracle
//! Edit and places it, linked, in the other Oracle apps.

use crate::OracleEditApp;

/// Characters a file name can't hold on Windows (and `/` anywhere).
fn file_name_safe(name: &str) -> String {
    let cleaned: String = name.chars().map(|c| if matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || c.is_control() { '-' } else { c }).collect();
    let trimmed = cleaned.trim().trim_end_matches('.').to_string();
    if trimmed.is_empty() { "Document".into() } else { trimmed }
}

/// Adds the active document to the library.
pub fn add_active(app: &mut OracleEditApp) -> Result<(), String> {
    let st = app.session.active().ok_or("no document is open")?;
    let stem = st
        .path
        .as_deref()
        .and_then(|p| std::path::Path::new(p).file_stem())
        .map_or_else(|| st.doc.name.clone(), |s| s.to_string_lossy().into_owned());
    let bytes = oracle_edit_format::save_to_bytes(&st.doc, &Default::default()).map_err(|e| e.to_string())?;
    let dir = std::env::temp_dir().join(format!("oracle-edit-to-library-{}", std::process::id()));
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let file = dir.join(format!("{}.{}", file_name_safe(&stem), oracle_edit_format::EXTENSION));
    std::fs::write(&file, bytes).map_err(|e| format!("{}: {e}", file.display()))?;
    app.oracle_library.add_files(std::slice::from_ref(&file));
    let _ = std::fs::remove_file(&file); // Copied into the library; a leftover is only a temp file.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::file_name_safe;

    #[test]
    fn names_become_file_names() {
        assert_eq!(file_name_safe("Affiche: été 2026?"), "Affiche- été 2026-");
        assert_eq!(file_name_safe("  ..  "), "Document");
        assert_eq!(file_name_safe("logo"), "logo");
    }
}
