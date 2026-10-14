//! Whole files: format detection, the ZIP and CFB containers, every format behind them (DOCX,
//! XLSX, PPTX and the 97-2003 binaries), then the Markdown render.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(doc) = undoc::parse_bytes(data) {
        let _ = undoc::render::to_markdown(&doc, &undoc::render::RenderOptions::default());
    }
});
