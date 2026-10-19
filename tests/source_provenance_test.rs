use std::io::{Cursor, Write};
use undoc::{Block, XmlSourceLocation};
use zip::write::SimpleFileOptions;

const W: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";
fn document(body: &str) -> undoc::Document {
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for (name, xml) in [
        ("[Content_Types].xml", r#"<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#.to_owned()),
        ("word/document.xml", format!(r#"<w:document xmlns:w="{W}" xmlns:w14="http://schemas.microsoft.com/office/word/2010/wordml"><w:body>{body}</w:body></w:document>"#)),
    ] {
        zip.start_file(name, SimpleFileOptions::default()).unwrap();
        zip.write_all(xml.as_bytes()).unwrap();
    }
    undoc::parse_bytes(&zip.finish().unwrap().into_inner()).unwrap()
}
fn path(source: &XmlSourceLocation) -> Vec<(&str, u32)> {
    assert_eq!(source.part, "word/document.xml");
    source
        .path
        .iter()
        .map(|s| {
            assert_eq!(s.namespace, W);
            (s.local_name.as_str(), s.index)
        })
        .collect()
}
#[test]
fn original_nodes_count_empty_and_duplicate_paragraphs() {
    let doc = document(
        r#"<w:p/><w:p></w:p><w:p w14:paraId="A"><w:r><w:t>same</w:t></w:r></w:p><w:p><w:r><w:t>same</w:t></w:r></w:p><w:sdt><w:sdtContent><w:p><w:r><w:t>wrapped</w:t></w:r></w:p></w:sdtContent></w:sdt>"#,
    );
    let paragraphs: Vec<_> = doc.sections[0]
        .content
        .iter()
        .filter_map(|b| match b {
            Block::Paragraph(p) if !p.is_empty() => Some(p),
            _ => None,
        })
        .collect();
    assert_eq!(paragraphs.len(), 3);
    assert_eq!(
        path(paragraphs[0].source.as_ref().unwrap()),
        [("document", 1), ("body", 1), ("p", 3)]
    );
    assert_eq!(
        paragraphs[0]
            .source
            .as_ref()
            .unwrap()
            .paragraph_id
            .as_deref(),
        Some("A")
    );
    assert_eq!(
        path(paragraphs[1].source.as_ref().unwrap()).last(),
        Some(&("p", 4))
    );
    assert_eq!(
        path(paragraphs[2].source.as_ref().unwrap()),
        [
            ("document", 1),
            ("body", 1),
            ("sdt", 1),
            ("sdtContent", 1),
            ("p", 1)
        ]
    );
}
#[test]
fn tables_keep_original_cell_paths_after_filtering() {
    let doc = document(
        r#"<w:p/><w:tbl><w:tr><w:tc><w:p/><w:p><w:r><w:t>cell</w:t></w:r></w:p><w:p><w:r><w:t>cell</w:t></w:r></w:p><w:p><w:r><w:t>next</w:t></w:r></w:p><w:tbl><w:tr><w:tc><w:p><w:r><w:t>nested</w:t></w:r></w:p></w:tc></w:tr></w:tbl></w:tc></w:tr></w:tbl>"#,
    );
    let table = doc.sections[0]
        .content
        .iter()
        .find_map(|b| match b {
            Block::Table(t) => Some(t),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        path(table.source.as_ref().unwrap()),
        [("document", 1), ("body", 1), ("tbl", 1)]
    );
    let cell = &table.rows[0].cells[0];
    assert_eq!(cell.content.len(), 2); // Preserve upstream's adjacent-cell deduplication.
    assert_eq!(
        path(cell.content[0].source.as_ref().unwrap()),
        [
            ("document", 1),
            ("body", 1),
            ("tbl", 1),
            ("tr", 1),
            ("tc", 1),
            ("p", 2)
        ]
    );
    assert_eq!(
        path(cell.content[1].source.as_ref().unwrap()).last(),
        Some(&("p", 4))
    );
    assert_eq!(
        path(
            cell.nested_tables[0].rows[0].cells[0].content[0]
                .source
                .as_ref()
                .unwrap()
        ),
        [
            ("document", 1),
            ("body", 1),
            ("tbl", 1),
            ("tr", 1),
            ("tc", 1),
            ("tbl", 1),
            ("tr", 1),
            ("tc", 1),
            ("p", 1)
        ]
    );
}
#[test]
fn source_is_optional_in_old_models() {
    let p: undoc::Paragraph = serde_json::from_str(r#"{"runs":[],"heading":"None"}"#).unwrap();
    assert!(p.source.is_none());
    assert!(serde_json::to_value(p).unwrap().get("source").is_none());
}
#[test]
fn expanded_names_count_aliases_without_changing_upstream_prefix_support() {
    // Upstream ignores q:p. It still occupies the first expanded-name sibling slot.
    let doc = document(&format!(
        r#"<q:p xmlns:q="{W}"/><w:p><w:r><w:t>visible</w:t></w:r></w:p>"#
    ));
    let Block::Paragraph(p) = &doc.sections[0].content[0] else {
        panic!()
    };
    assert_eq!(p.plain_text(), "visible");
    assert_eq!(path(p.source.as_ref().unwrap()).last(), Some(&("p", 2)));
}

#[test]
fn real_decision_d03_retains_seventh_original_paragraph() {
    let bytes = include_bytes!("fixtures/provenance-decisions.docx");
    let doc = undoc::parse_bytes(bytes).unwrap();
    let p = doc.sections[0]
        .content
        .iter()
        .find_map(|b| match b {
            Block::Paragraph(p) if p.plain_text().contains("D-03") => Some(p),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        path(p.source.as_ref().unwrap()),
        [("document", 1), ("body", 1), ("p", 7)]
    );
    assert_eq!(p.source.as_ref().unwrap().paragraph_id, None);
}
