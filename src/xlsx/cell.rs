use super::*;
impl XlsxParser {
    pub(super) fn build_sheet_cell(
        &self,
        current_cell_value: &str,
        current_cell_type: Option<&str>,
        current_cell_style: Option<usize>,
        current_cell_ref: Option<&str>,
        context: BuildSheetCellContext<'_>,
    ) -> Result<Cell> {
        let value =
            self.resolve_cell_value(current_cell_value, current_cell_type, current_cell_style)?;

        let hyperlink_url = current_cell_ref
            .and_then(|r| context.hyperlink_map.get(r))
            .cloned();

        let mut text_run = TextRun::plain(&value);
        if let Some(url) = hyperlink_url {
            text_run.hyperlink = Some(url);
        }

        let mut runs = vec![text_run];

        if let Some(comment_text) = current_cell_ref.and_then(|r| context.comment_map.get(r)) {
            let mut comment_run = TextRun::plain(format!(" [Comment: {}]", comment_text));
            comment_run.style.italic = true;
            runs.push(comment_run);
        }

        Ok(Cell {
            spreadsheet_source: None,
            content: vec![Paragraph {
                runs,
                ..Default::default()
            }],
            nested_tables: Vec::new(),
            col_span: 1,
            row_span: 1,
            alignment: CellAlignment::Left,
            vertical_alignment: Default::default(),
            is_header: context.is_header,
            background: None,
            background_image: None,
        })
    }
}
