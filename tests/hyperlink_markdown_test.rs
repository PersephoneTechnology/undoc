//! A hyperlink's text is underlined because it is a link: Word's `Hyperlink` character style
//! draws it so, and some producers write the underline on the run itself. Markdown marks the
//! link as a link, so the underline is not written as emphasis -- for a link to an address, a
//! link to a bookmark (a table of contents), and a `HYPERLINK` field's result alike. Text the
//! author underlined outside a link keeps its underline. Checked in body text and in a table
//! cell, which read their paragraphs the same way.

use std::io::{Cursor, Write};

use undoc::parse_bytes;
use undoc::render::{to_markdown, RenderOptions};
use zip::write::SimpleFileOptions;

const CONTENT_TYPES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
<Override PartName="/word/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"/>
</Types>"#;

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;

const DOC_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
<Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.com/download" TargetMode="External"/>
</Relationships>"#;

const STYLES: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:styles xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
<w:style w:type="character" w:styleId="Hyperlink"><w:name w:val="Hyperlink"/><w:rPr><w:color w:val="0000FF"/><w:u w:val="single"/></w:rPr></w:style>
</w:styles>"#;

const LINK_RUN: &str =
    r#"<w:r><w:rPr><w:rStyle w:val="Hyperlink"/></w:rPr><w:t>download page</w:t></w:r>"#;

fn docx(body: &str) -> Vec<u8> {
    let document = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><w:body>{body}</w:body></w:document>"#
    );
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    for (name, data) in [
        ("[Content_Types].xml", CONTENT_TYPES),
        ("_rels/.rels", ROOT_RELS),
        ("word/_rels/document.xml.rels", DOC_RELS),
        ("word/styles.xml", STYLES),
        ("word/document.xml", document.as_str()),
    ] {
        zip.start_file(name, opts).unwrap();
        zip.write_all(data.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn markdown(body: &str) -> String {
    to_markdown(
        &parse_bytes(&docx(body)).unwrap(),
        &RenderOptions::default(),
    )
    .unwrap()
}

#[test]
fn a_link_in_the_hyperlink_style_is_a_link_not_underlined_text() {
    let md = markdown(&format!(
        r#"<w:p><w:r><w:t xml:space="preserve">See the </w:t></w:r><w:hyperlink r:id="rId2">{LINK_RUN}</w:hyperlink></w:p>"#
    ));
    assert!(
        md.contains("See the [download page](https://example.com/download)"),
        "{md}"
    );
    assert!(!md.contains("<u>"), "{md}");
}

#[test]
fn an_underline_written_on_a_link_run_is_the_link_look_too() {
    let md = markdown(
        r#"<w:p><w:hyperlink r:id="rId2"><w:r><w:rPr><w:u w:val="single"/><w:color w:val="1155CC"/></w:rPr><w:t>download page</w:t></w:r></w:hyperlink></w:p>"#,
    );
    assert!(
        md.contains("[download page](https://example.com/download)"),
        "{md}"
    );
    assert!(!md.contains("<u>"), "{md}");
}

#[test]
fn a_link_to_a_bookmark_is_not_underlined() {
    let md = markdown(&format!(
        r#"<w:p><w:hyperlink w:anchor="_Toc1">{LINK_RUN}</w:hyperlink></w:p>"#
    ));
    assert!(md.contains("download page"), "{md}");
    assert!(!md.contains("<u>"), "{md}");
}

#[test]
fn a_hyperlink_field_result_is_a_link() {
    let md = markdown(&format!(
        r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> HYPERLINK "https://example.com/field" </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r>{LINK_RUN}<w:r><w:fldChar w:fldCharType="end"/></w:r><w:r><w:t xml:space="preserve"> after</w:t></w:r></w:p>"#
    ));
    assert!(
        md.contains("[download page](https://example.com/field) after"),
        "{md}"
    );
    assert!(!md.contains("<u>") && !md.contains("HYPERLINK"), "{md}");
}

#[test]
fn a_simple_hyperlink_field_is_a_link() {
    let md = markdown(&format!(
        r#"<w:p><w:fldSimple w:instr=" HYPERLINK &quot;https://example.com/simple&quot; ">{LINK_RUN}</w:fldSimple></w:p>"#
    ));
    assert!(
        md.contains("[download page](https://example.com/simple)"),
        "{md}"
    );
}

#[test]
fn text_underlined_outside_a_link_keeps_its_underline() {
    let md = markdown(
        r#"<w:p><w:r><w:rPr><w:u w:val="single"/></w:rPr><w:t>underlined</w:t></w:r></w:p>"#,
    );
    assert!(md.contains("<u>underlined</u>"), "{md}");
}

#[test]
fn a_link_in_a_table_cell_is_a_link() {
    let md = markdown(&format!(
        r#"<w:tbl><w:tr><w:tc><w:p><w:r><w:t>Name</w:t></w:r></w:p></w:tc><w:tc><w:p><w:r><w:t>Where</w:t></w:r></w:p></w:tc></w:tr><w:tr><w:tc><w:p><w:r><w:t>Site</w:t></w:r></w:p></w:tc><w:tc><w:p><w:hyperlink r:id="rId2">{LINK_RUN}</w:hyperlink></w:p></w:tc></w:tr></w:tbl>"#
    ));
    assert!(
        md.contains("| Site | [download page](https://example.com/download) |"),
        "{md}"
    );
    assert!(!md.contains("<u>"), "{md}");
}

/// A `HYPERLINK \l` field leads to a bookmark in the document: no anchor in the output to
/// point at, so the text is written as text -- and not underlined.
#[test]
fn a_hyperlink_field_to_a_bookmark_is_text() {
    let md = markdown(&format!(
        r#"<w:p><w:r><w:fldChar w:fldCharType="begin"/></w:r><w:r><w:instrText xml:space="preserve"> HYPERLINK \l "_Toc1" </w:instrText></w:r><w:r><w:fldChar w:fldCharType="separate"/></w:r>{LINK_RUN}<w:r><w:fldChar w:fldCharType="end"/></w:r></w:p>"#
    ));
    assert!(md.contains("download page"), "{md}");
    assert!(!md.contains("](") && !md.contains("<u>"), "{md}");
}
