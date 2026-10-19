use undoc::Block;
#[test]
fn coordinates_survive_sparse_rows_and_merge_clamping() {
    let doc = undoc::parse_bytes(include_bytes!("fixtures/xlsx-source.xlsx")).unwrap();
    for (i, section) in doc.sections.iter().enumerate() {
        let Block::Table(table) = &section.content[0] else {
            panic!()
        };
        let sheet = table.spreadsheet_source.as_ref().unwrap();
        assert_eq!(sheet.worksheet_index, i as u32);
        assert_eq!(Some(&sheet.worksheet_name), section.name.as_ref());
        for cell in table.rows.iter().flat_map(|r| &r.cells) {
            let text = cell.plain_text();
            if text == "合并标题" {
                let r = cell.spreadsheet_source.unwrap();
                assert_eq!(
                    (r.row_start, r.column_start, r.row_end, r.column_end),
                    (39, 1, 44, 4)
                );
            }
            if text == "尾部目标" {
                let r = cell.spreadsheet_source.unwrap();
                assert_eq!((r.row_start, r.column_start), (89, 5));
            }
        }
    }
}
#[test]
fn generic_tables_have_no_spreadsheet_identity() {
    let table = undoc::Table::new();
    assert!(table.spreadsheet_source.is_none());
    let cell = undoc::Cell::with_text("text");
    assert!(cell.spreadsheet_source.is_none());
    assert!(!serde_json::to_string(&cell)
        .unwrap()
        .contains("spreadsheet_source"));
}
