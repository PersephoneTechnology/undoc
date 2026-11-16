#![cfg(feature = "xls")]

use undoc::{
    detect::detect_format_from_reader, detect_format_from_bytes, parse_bytes, parse_file,
    FormatType,
};

const INPUT: &[u8] = include_bytes!("fixtures/persephone/chinese.xls");

#[test]
fn xls_surplus_fat_padding_preserves_workbook_content_and_input() {
    let bytes = INPUT.to_vec();
    assert_eq!(detect_format_from_bytes(&bytes).unwrap(), FormatType::Xls);
    assert_eq!(
        detect_format_from_reader(std::io::Cursor::new(&bytes)).unwrap(),
        FormatType::Xls
    );
    let document = parse_bytes(&bytes).unwrap();
    assert_eq!(document.sections.len(), 2);
    assert_eq!(document.sections[0].name.as_deref(), Some("利润表"));
    assert_eq!(document.sections[1].name.as_deref(), Some("附注"));
    let text = document.plain_text();
    for expected in ["营业收入", "1234.5", "1434.5", "2024-01-02", "第二张表中文"] {
        assert!(text.contains(expected), "missing {expected}: {text}");
    }
    let direct = undoc::xls::XlsParser::from_bytes(bytes.clone())
        .unwrap()
        .parse()
        .unwrap();
    assert_eq!(direct.plain_text(), text);
    assert_eq!(bytes, INPUT);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/persephone/chinese.xls");
    assert_eq!(parse_file(&path).unwrap().plain_text(), text);
    assert_eq!(std::fs::read(path).unwrap(), INPUT);
}

#[test]
fn xls_padding_does_not_hide_truncation_or_broken_live_chains() {
    for len in [0, 511, 512, INPUT.len() - 1, INPUT.len() - 512] {
        assert!(
            parse_bytes(&INPUT[..len]).is_err(),
            "accepted truncated length {len}"
        );
    }
    // Sector 3 is a live stream sector. A cycle or an out-of-file pointer is not padding.
    for next in [3_u32, 128] {
        let mut bytes = INPUT.to_vec();
        bytes[512 + 3 * 4..512 + 4 * 4].copy_from_slice(&next.to_le_bytes());
        assert!(
            parse_bytes(&bytes).is_err(),
            "accepted corrupt live chain {next}"
        );
    }
}
