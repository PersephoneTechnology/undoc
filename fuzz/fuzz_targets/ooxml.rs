//! The XML of a package's main part: the fuzzed bytes become the body of a DOCX, an XLSX or a
//! PPTX whose other parts are well formed, so the mutations reach the format parsers instead
//! of stopping at the ZIP container. The first byte picks the format.
#![no_main]

use std::io::{Cursor, Write};

use libfuzzer_sys::fuzz_target;
use zip::write::SimpleFileOptions;

const RELS: &str = "http://schemas.openxmlformats.org/officeDocument/2006/relationships";

fn content_types(overrides: &[(&str, &str)]) -> String {
    let mut types = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/>"#,
    );
    for (part, kind) in overrides {
        types.push_str(&format!(
            r#"<Override PartName="/{part}" ContentType="application/vnd.openxmlformats-officedocument.{kind}+xml"/>"#
        ));
    }
    types.push_str("</Types>");
    types
}

fn relationships(links: &[(&str, &str)]) -> String {
    let mut rels = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">"#,
    );
    for (i, (kind, target)) in links.iter().enumerate() {
        rels.push_str(&format!(
            r#"<Relationship Id="rId{}" Type="{RELS}/{kind}" Target="{target}"/>"#,
            i + 1
        ));
    }
    rels.push_str("</Relationships>");
    rels
}

fn package(parts: &[(&str, &[u8])]) -> Vec<u8> {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, body) in parts {
        zip.start_file(*name, options).unwrap();
        zip.write_all(body).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn docx(document: &[u8]) -> Vec<u8> {
    let types = content_types(&[("word/document.xml", "wordprocessingml.document.main")]);
    let rels = relationships(&[("officeDocument", "word/document.xml")]);
    package(&[
        ("[Content_Types].xml", types.as_bytes()),
        ("_rels/.rels", rels.as_bytes()),
        ("word/document.xml", document),
    ])
}

fn xlsx(sheet: &[u8]) -> Vec<u8> {
    let types = content_types(&[("xl/workbook.xml", "spreadsheetml.sheet.main")]);
    let rels = relationships(&[("officeDocument", "xl/workbook.xml")]);
    let workbook = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="{RELS}"><sheets><sheet name="Sheet1" sheetId="1" r:id="rId1"/></sheets></workbook>"#
    );
    let workbook_rels = relationships(&[("worksheet", "worksheets/sheet1.xml")]);
    package(&[
        ("[Content_Types].xml", types.as_bytes()),
        ("_rels/.rels", rels.as_bytes()),
        ("xl/workbook.xml", workbook.as_bytes()),
        ("xl/_rels/workbook.xml.rels", workbook_rels.as_bytes()),
        ("xl/worksheets/sheet1.xml", sheet),
    ])
}

fn pptx(slide: &[u8]) -> Vec<u8> {
    let types = content_types(&[
        ("ppt/presentation.xml", "presentationml.presentation.main"),
        ("ppt/slides/slide1.xml", "presentationml.slide"),
    ]);
    let rels = relationships(&[("officeDocument", "ppt/presentation.xml")]);
    let presentation_rels = relationships(&[("slide", "slides/slide1.xml")]);
    let presentation = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="{RELS}"><p:sldIdLst><p:sldId id="256" r:id="rId1"/></p:sldIdLst></p:presentation>"#
    );
    let slide_rels = relationships(&[]);
    package(&[
        ("[Content_Types].xml", types.as_bytes()),
        ("_rels/.rels", rels.as_bytes()),
        (
            "ppt/_rels/presentation.xml.rels",
            presentation_rels.as_bytes(),
        ),
        ("ppt/presentation.xml", presentation.as_bytes()),
        ("ppt/slides/_rels/slide1.xml.rels", slide_rels.as_bytes()),
        ("ppt/slides/slide1.xml", slide),
    ])
}

fuzz_target!(|data: &[u8]| {
    let Some((&format, part)) = data.split_first() else {
        return;
    };
    let file = match format % 3 {
        0 => docx(part),
        1 => xlsx(part),
        _ => pptx(part),
    };
    if let Ok(doc) = undoc::parse_bytes(&file) {
        let _ = undoc::render::to_markdown(&doc, &undoc::render::RenderOptions::default());
    }
});
