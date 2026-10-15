//! Each format's parser is behind a cargo feature of the same name. `FormatType::is_enabled`
//! reports whether this build parses a format, and it has to agree with what parsing actually
//! does: a format it calls enabled reaches its parser, one it calls disabled is recognised and rejected
//! as unsupported. Run under a reduced feature set (for example
//! `--no-default-features --features xlsx`) to exercise the disabled side.

use std::io::{Cursor, Write};

use undoc::{parse_bytes, ErrorKind, FormatType};
use zip::write::SimpleFileOptions;

const ROOT_RELS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="{main}"/>
</Relationships>"#;

fn package(main_part: &str, content_type: &str, main_xml: &str) -> Vec<u8> {
    let content_types = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
<Default Extension="xml" ContentType="application/xml"/>
<Override PartName="/{main_part}" ContentType="{content_type}"/>
</Types>"#
    );
    let rels = ROOT_RELS.replace("{main}", main_part);
    let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let opts = SimpleFileOptions::default();
    for (name, data) in [
        ("[Content_Types].xml", content_types.as_str()),
        ("_rels/.rels", rels.as_str()),
        (main_part, main_xml),
    ] {
        zip.start_file(name, opts).unwrap();
        zip.write_all(data.as_bytes()).unwrap();
    }
    zip.finish().unwrap().into_inner()
}

fn docx() -> Vec<u8> {
    package(
        "word/document.xml",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>text</w:t></w:r></w:p></w:body></w:document>"#,
    )
}

fn xlsx() -> Vec<u8> {
    package(
        "xl/workbook.xml",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheets/></workbook>"#,
    )
}

fn pptx() -> Vec<u8> {
    package(
        "ppt/presentation.xml",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml",
        r#"<?xml version="1.0" encoding="UTF-8"?>
<p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main"/>"#,
    )
}

#[test]
fn is_enabled_agrees_with_what_parsing_does() {
    for (format, bytes) in [
        (FormatType::Docx, docx()),
        (FormatType::Xlsx, xlsx()),
        (FormatType::Pptx, pptx()),
    ] {
        assert_eq!(
            undoc::detect_format_from_bytes(&bytes).expect("the package is recognised"),
            format,
            "the fixture must be detected as {format:?} whatever the features"
        );
        let result = parse_bytes(&bytes);
        if format.is_enabled() {
            // The fixtures carry only the main part, so a parser may stop on a part it needs;
            // what matters is that the format's parser ran rather than being refused.
            if let Err(error) = result {
                assert_ne!(
                    error.kind(),
                    ErrorKind::UnsupportedFormat,
                    "{format:?} is reported enabled, so its parser must run: {error}"
                );
            }
        } else {
            let error = result.expect_err("a disabled format must not parse");
            assert_eq!(
                error.kind(),
                ErrorKind::UnsupportedFormat,
                "a disabled {format:?} is recognised and rejected as unsupported: {error}"
            );
        }
    }
}

/// The default build parses every format it knows.
#[test]
#[cfg(all(
    feature = "docx",
    feature = "xlsx",
    feature = "pptx",
    feature = "doc",
    feature = "xls",
    feature = "ppt"
))]
fn the_default_build_enables_every_format() {
    for format in FormatType::ALL {
        assert!(format.is_enabled(), "{format:?} is not enabled");
    }
}
