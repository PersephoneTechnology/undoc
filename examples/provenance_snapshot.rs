//! Compatibility probe: run unchanged on official and fork builds.
fn main() {
    let path = std::env::args().nth(1).expect("DOCX path");
    let doc = undoc::parse_bytes(&std::fs::read(path).unwrap()).unwrap();
    let markdown =
        undoc::render::to_markdown(&doc, &undoc::render::RenderOptions::default()).unwrap();
    println!(
        "{}",
        serde_json::json!({"model": doc, "markdown": markdown})
    );
}
