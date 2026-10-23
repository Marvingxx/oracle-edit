//! Edit Contents on a smart object linked to another Oracle app's document (an `.oracle-vector`
//! logo placed from Oracle Vector) opens the document in that app, as Photoshop opens a linked
//! Illustrator file in Illustrator. Saving it there updates the layer here: the link watch sees the
//! file change. Embedded smart objects and other linked files open here, as before.

use oracle_edit_doc::{LayerContent, LayerId, SmartSource};
use serde_json::{Value, json};

use crate::OracleEditApp;

/// The resolved path of the other Oracle app's document `params`' layer (default: the active one)
/// is linked to, if it is such a smart object.
fn linked_document(app: &OracleEditApp, params: &Value) -> Option<String> {
    let st = app.session.active()?;
    let id = params.get("layer").and_then(Value::as_u64).map(LayerId).or(st.active_layer)?;
    let LayerContent::Smart(sm) = &st.doc.layer(id)?.content else { return None };
    let SmartSource::Linked { path } = &sm.source else { return None };
    if !oracle_edit_engine::oracle_files::is_foreign(path) {
        return None;
    }
    let resolved = oracle_edit_engine::doc_path::resolve(path).ok()?;
    Some(resolved.to_string_lossy().into_owned())
}

/// Opens the document in its own app when Edit Contents targets such a smart object; `None` leaves
/// Edit Contents to the engine.
pub fn open(app: &mut OracleEditApp, params: &Value) -> Option<Result<Value, String>> {
    let path = linked_document(app, params)?;
    Some(oracle_suite_link::edit_original(std::path::Path::new(&path)).map(|owner| {
        app.ui.status = crate::i18n::fmt(tl!("Opened in {app}: save it there to update this layer"), &[("app", owner.name())]);
        app.ui.status_error = false;
        json!({"openedIn": owner.id(), "path": path})
    }))
}
