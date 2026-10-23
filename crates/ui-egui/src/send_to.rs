//! Send to: Oracle Edit hands its document to another Oracle app (File › Send to Oracle Vector…),
//! and takes what the others send it. A document sent is placed, linked, in the receiving app's
//! open document, so a change saved here shows there (its link watch). The exchange goes through
//! `oracle_suite_link::inbox`: a small request file the receiving app takes about every second,
//! starting it when it isn't running. A running app takes it on its next frame; an idle one is woken for it
//! (`inbox::watch`, which also keeps it marked as running).

use std::path::PathBuf;

use oracle_suite_link::App;
use oracle_suite_link::inbox::{self, Action};
use serde_json::{Value, json};

use crate::OracleEditApp;

/// How often the inbox is looked at, in seconds.
const EVERY: f64 = 1.0;

/// The file that stands for the active document: itself when it is saved as an Oracle Edit document
/// and unchanged since, else a copy stored in the library ("Mes créations").
fn document_file(app: &mut OracleEditApp) -> Result<PathBuf, String> {
    let st = app.session.active().ok_or("no document is open")?;
    if let Some(path) = st.path.as_deref().filter(|p| p.to_ascii_lowercase().ends_with(".pcraft") && st.revision == st.saved_revision) {
        return Ok(PathBuf::from(path));
    }
    crate::library_send::store_active(app)
}

/// File › Send to `target`: the document placed, linked, in `target`'s open document.
pub fn send_active(app: &mut OracleEditApp, target: App) -> Result<Value, String> {
    let file = document_file(app)?;
    inbox::send(target, Action::Place, &file)?;
    let name = target.name();
    app.ui.status = crate::i18n::fmt(tl!("Sent to {app}"), &[("app", name)]);
    app.ui.status_error = false;
    Ok(json!({"sentTo": target.id(), "path": file}))
}

/// Takes what the other Oracle apps sent: each file placed, linked, in the active document (opened
/// as a document when none is open, or when asked to open it). Looks at most every second; the
/// watch wakes an idle app when something waits.
pub fn poll(app: &mut OracleEditApp, ctx: &egui::Context) {
    // Once: a background thread keeps this app marked as running, and wakes it when something is sent.
    let watching = egui::Id::new("oracle-suite-inbox-watch");
    if !ctx.data(|d| d.get_temp::<bool>(watching).unwrap_or(false)) {
        ctx.data_mut(|d| d.insert_temp(watching, true));
        let waker = ctx.clone();
        inbox::watch(App::Edit, move || waker.request_repaint());
    }
    let now = ctx.input(|i| i.time);
    if now - app.inbox_last < EVERY {
        return;
    }
    app.inbox_last = now;
    let requests = inbox::take(App::Edit);
    if requests.is_empty() {
        return;
    }
    for r in requests {
        let path = r.file.to_string_lossy().into_owned();
        let placed = match r.action {
            Action::Place if app.session.active().is_some() => app.run("file.placeLinked", json!({"path": path})).map(|_| ()),
            _ => app.open_path(&path).map(|_| ()),
        };
        if let Err(e) = placed {
            app.ui.status = e;
            app.ui.status_error = true;
        }
    }
    app.sync_views();
    ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
}
