#![no_main]

use libfuzzer_sys::fuzz_target;

// The whole native reader (archive, object stream, model) and the preview, on arbitrary bytes.
fuzz_target!(|data: &[u8]| {
    let _ = oracle_edit_affinity::container::header(data);
    let _ = oracle_edit_affinity::preview(data);
    let _ = oracle_edit_affinity::read(data, oracle_edit_affinity::Limits { max_entry: 16 << 20, max_total: 64 << 20, max_pixel_bytes: 64 << 20 });
});
