//! Documents of the other Oracle apps in Oracle Edit, without an export step.
//!
//! An `.oracle-vector` logo is drawn by Oracle Vector as SVG and placed as a **vector** smart
//! object (sharp at any size); a Brand Guide's first page is drawn as PNG. Placed linked, the
//! smart object keeps the original's path: when the logo is saved again in Oracle Vector, the link
//! watch ([`crate::link_watch`]) sees the file change and the layer is drawn again from it, as for
//! any linked file. The drawing itself is `oracle_suite_link::render`, cached per version of the
//! file. Opening such a document makes a new Oracle Edit document of its size with it placed, linked,
//! so a save never writes over the original in another format.

use serde_json::{Value, json};

use crate::{EngineError, Result, Session};

/// Whether `name` (a path or file name) is a document of another Oracle app.
#[cfg(not(target_arch = "wasm32"))]
pub fn is_foreign(name: &str) -> bool {
    foreign(name).is_some()
}

#[cfg(target_arch = "wasm32")]
pub fn is_foreign(_name: &str) -> bool {
    false
}

#[cfg(not(target_arch = "wasm32"))]
fn foreign(name: &str) -> Option<&'static oracle_suite_link::OracleFormat> {
    oracle_suite_link::oracle_format(std::path::Path::new(name)).filter(|f| f.app != oracle_suite_link::App::Edit)
}

/// The drawing Oracle Edit places for `path`, a document of another Oracle app, as
/// (file name telling its format, bytes): Oracle Vector's SVG, which stays vector, or the first
/// page of a Brand Guide as PNG. `None` when `path` isn't such a document.
#[cfg(not(target_arch = "wasm32"))]
pub fn drawn(path: &str) -> Option<Result<(String, Vec<u8>)>> {
    use oracle_suite_link::Preview;
    let format = foreign(path)?;
    let preview = if format.can(Preview::Svg) { Preview::Svg } else { Preview::Png };
    let file = std::path::Path::new(path);
    let stem = file.file_stem().map_or_else(|| "document".to_string(), |s| s.to_string_lossy().into_owned());
    Some(
        oracle_suite_link::render(file, preview)
            .map_err(EngineError::Other)
            // A stale drawing (the app is missing) still shows the document as last drawn.
            .and_then(|r| std::fs::read(&r.path).map_err(|e| EngineError::Other(format!("{}: {e}", r.path.display()))))
            .map(|bytes| (format!("{stem}.{}", preview.ext()), bytes)),
    )
}

#[cfg(target_arch = "wasm32")]
pub fn drawn(_path: &str) -> Option<Result<(String, Vec<u8>)>> {
    None
}

/// Opens `path`, a document of another Oracle app: a new document the size of its drawing (named
/// after it, transparent, untitled so a save asks where), with the document placed in it, linked.
pub fn open(s: &mut Session, path: &str) -> Result<Value> {
    let Some(drawing) = drawn(path) else {
        return Err(EngineError::Other(format!("{path}: not a document of another Oracle app")));
    };
    let (name, bytes) = drawing?;
    let size = oracle_edit_io::import(&name, &bytes).map_err(|e| EngineError::Other(format!("{name}: {e}")))?.document.size;
    let stem = name.rsplit_once('.').map_or(name.as_str(), |(stem, _)| stem).to_string();
    s.execute("file.new", json!({"width": size.width.max(1), "height": size.height.max(1), "background": "transparent", "name": stem}))?;
    crate::file_cmds::place_bytes(s, &name, bytes, Some(path.to_string()), &json!({"fit": false}))
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use oracle_edit_doc::{LayerContent, SmartSource};

    fn session() -> Session {
        let mut s = Session::new();
        s.execute("file.new", json!({"width": 200, "height": 100, "background": "white"})).unwrap();
        s
    }

    /// An `.oracle-vector` made by Oracle Vector's engine from `svg`, in a fresh folder.
    fn vector_doc(name: &str, svg: &str) -> Option<std::path::PathBuf> {
        let dir = std::env::temp_dir().join(format!("oracle-edit-ov-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let doc = dir.join(format!("{name}.oracle-vector"));
        save_in_vector(svg, &doc)?;
        Some(doc)
    }

    /// Oracle Vector saving `svg`'s artwork as `doc` (a real save: new contents, new time).
    fn save_in_vector(svg: &str, doc: &std::path::Path) -> Option<()> {
        let cli = oracle_suite_link::App::Vector.cli()?;
        let src = doc.with_extension("svg");
        std::fs::write(&src, svg).unwrap();
        assert!(std::process::Command::new(cli).arg("convert").arg(&src).arg(doc).status().unwrap().success());
        Some(())
    }

    const RED: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="#d31c3b"/></svg>"##;
    const BLUE: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="#1c3bd3"/></svg>"##;

    /// The composited colour at (`x`, `y`), 0..1.
    fn pixel(s: &Session, x: i32, y: i32) -> [f32; 3] {
        let p = oracle_edit_compose::render(&s.active().unwrap().doc, oracle_edit_geom::Rect::new(x, y, x + 1, y + 1)).px[0];
        [p[0], p[1], p[2]]
    }

    #[test]
    fn other_apps_documents_are_recognised_but_not_edits_own() {
        assert!(is_foreign("C:/logos/Logo.ORACLE-VECTOR"));
        assert!(is_foreign("charte.oracle-layout"));
        assert!(!is_foreign("photo.oracle-edit"));
        assert!(!is_foreign("logo.svg"));
        assert!(drawn("logo.svg").is_none());
    }

    /// The example of the plan: a logo made in Oracle Vector, placed linked in Oracle Edit as a
    /// vector smart object, follows the logo when Oracle Vector saves it again.
    #[test]
    fn a_vector_logo_placed_linked_is_vector_and_follows_its_file() {
        let Some(doc) = vector_doc("logo", RED) else { return };
        let path = doc.to_string_lossy().to_string();
        let mut s = session();
        s.execute("file.placeLinked", json!({"path": path, "fit": false})).unwrap();
        let d = s.active().unwrap();
        let (layer, sm) = d.doc.layers.iter().find_map(|l| if let LayerContent::Smart(sm) = &l.content { Some((l, sm)) } else { None }).expect("a smart object");
        assert_eq!(sm.source, SmartSource::Linked { path: path.clone() }, "linked to the .oracle-vector itself");
        assert_eq!(layer.name, "logo");
        let red = pixel(&s, 100, 50);
        assert!(red[0] > 0.7 && red[2] < 0.4, "the logo's red in the middle: {red:?}");
        // The link watch has seen the file once (the app looks every two seconds)…
        s.execute("layer.smartObjects.updateChanged", json!({})).unwrap();
        // …then Oracle Vector saves the logo again, blue: the layer follows.
        std::thread::sleep(std::time::Duration::from_millis(30));
        save_in_vector(BLUE, &doc).unwrap();
        s.execute("layer.smartObjects.updateChanged", json!({})).unwrap();
        let blue = pixel(&s, 100, 50);
        assert!(blue[2] > 0.7 && blue[0] < 0.4, "the new blue: {blue:?}");
    }

    #[test]
    fn opening_another_apps_document_makes_an_untitled_document_with_it_linked() {
        let Some(doc) = vector_doc("ouvert", RED) else { return };
        let mut s = Session::new();
        open(&mut s, &doc.to_string_lossy()).unwrap();
        let d = s.active().unwrap();
        assert_eq!((d.doc.size.width, d.doc.size.height), (40, 20));
        assert!(d.path.is_none(), "untitled: saving asks where");
        assert!(d.doc.layers.iter().any(|l| matches!(&l.content, LayerContent::Smart(sm) if matches!(&sm.source, SmartSource::Linked { path } if path.ends_with(".oracle-vector")))));
    }
}
