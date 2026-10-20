use std::io::{Cursor, Write};
use undoc::Block;
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

fn boxes(prefix: &str) -> String {
    format!(
        r#"<w:p><w:r><w:pict xmlns:{prefix}="urn:schemas-microsoft-com:vml"><{prefix}:group><{prefix}:shape/><{prefix}:shape><{prefix}:textbox><w:txbxContent><w:p/><w:p><w:r><w:t>same</w:t></w:r></w:p><w:p><w:r><w:t>same</w:t></w:r></w:p></w:txbxContent></{prefix}:textbox></{prefix}:shape></{prefix}:group></w:pict></w:r></w:p>"#
    )
}
#[test]
fn textbox_sources_keep_full_path_indices_and_namespace_identity() {
    for prefix in ["v", "drawingAlias"] {
        let doc = document(&boxes(prefix));
        let paragraphs: Vec<_> = doc.sections[0]
            .content
            .iter()
            .filter_map(|b| match b {
                Block::Paragraph(p) if p.plain_text() == "same" => Some(p),
                _ => None,
            })
            .collect();
        assert_eq!(paragraphs.len(), 2);
        for (i, p) in paragraphs.iter().enumerate() {
            let source = p.source.as_ref().expect("textbox source");
            assert_eq!(source.part, "word/document.xml");
            assert_eq!(
                source
                    .path
                    .iter()
                    .map(|s| (s.local_name.as_str(), s.index))
                    .collect::<Vec<_>>(),
                vec![
                    ("document", 1),
                    ("body", 1),
                    ("p", 1),
                    ("r", 1),
                    ("pict", 1),
                    ("group", 1),
                    ("shape", 2),
                    ("textbox", 1),
                    ("txbxContent", 1),
                    ("p", i as u32 + 2)
                ]
            );
            assert_eq!(source.path[6].namespace, "urn:schemas-microsoft-com:vml");
        }
    }
}
#[test]
fn cell_textbox_sources_keep_table_context() {
    let doc = document(&format!(
        "<w:tbl><w:tr><w:tc>{}</w:tc></w:tr></w:tbl>",
        boxes("v")
    ));
    let Block::Table(table) = &doc.sections[0].content[0] else {
        panic!("table")
    };
    let paragraphs = &table.rows[0].cells[0].content;
    assert_eq!(paragraphs.len(), 1); // Preserve existing cell paragraph deduplication.
    for p in paragraphs {
        let path = &p.source.as_ref().expect("cell textbox source").path;
        assert_eq!(path[2].local_name, "tbl");
        assert_eq!(path[5].local_name, "p");
        assert_eq!(path.last().unwrap().local_name, "p");
    }
}
#[test]
fn choice_source_does_not_inherit_fallback_or_host() {
    let doc = document(
        r#"<w:p xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006" xmlns:wps="http://schemas.microsoft.com/office/word/2010/wordprocessingShape"><w:r><mc:AlternateContent><mc:Choice Requires="wps"><w:drawing><wps:wsp><wps:txbx><w:txbxContent><w:p><w:r><w:t>choice</w:t></w:r></w:p></w:txbxContent></wps:txbx></wps:wsp></w:drawing></mc:Choice><mc:Fallback><w:txbxContent><w:p><w:r><w:t>fallback</w:t></w:r></w:p></w:txbxContent></mc:Fallback></mc:AlternateContent></w:r></w:p>"#,
    );
    let paragraphs: Vec<_> = doc.sections[0]
        .content
        .iter()
        .filter_map(|b| match b {
            Block::Paragraph(p) if !p.is_empty() => Some(p),
            _ => None,
        })
        .collect();
    assert_eq!(paragraphs.len(), 1);
    assert_eq!(paragraphs[0].plain_text(), "choice");
    let path = &paragraphs[0].source.as_ref().expect("choice source").path;
    assert!(path.iter().any(|s| s.local_name == "Choice"));
    assert!(!path.iter().any(|s| s.local_name == "Fallback"));
}
