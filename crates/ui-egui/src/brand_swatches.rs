//! The brands of Oracle Brand Kit in the Swatches panel: a group "Marque · <brand>" per brand,
//! kept as the brand is (`oracle_edit_engine::swatch_cmds::sync_brands`). Every two seconds the
//! brand registry's stamp is looked at (a few file times); when a brand changed, its colours are
//! read again. Only in the desktop app: tests and the web have no brands.

use crate::OracleEditApp;

/// How often the brands are looked at, in seconds.
const EVERY: f64 = 2.0;

pub fn poll(app: &mut OracleEditApp, ctx: &egui::Context) {
    if cfg!(test) || app.services.file_stamp.is_none() {
        return;
    }
    let id = egui::Id::new("oracle-brand-swatches");
    let now = ctx.input(|i| i.time);
    let (last, stamp) = ctx.data(|d| d.get_temp::<(f64, u64)>(id)).unwrap_or((f64::NEG_INFINITY, 0));
    if now - last < EVERY {
        return;
    }
    let fresh = oracle_suite_link::brands::stamp();
    ctx.data_mut(|d| d.insert_temp(id, (now, fresh)));
    // The first look always reads the brands: groups saved last time may be out of date.
    if fresh == stamp && last.is_finite() {
        return;
    }
    let brands = oracle_suite_link::brands::brands();
    if oracle_edit_engine::swatch_cmds::sync_brands(&mut app.session, &brands) {
        ctx.request_repaint();
    }
}
