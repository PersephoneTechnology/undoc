//! Original worksheet addresses, independent of presentation spans.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpreadsheetCellSource {
    pub row_start: u32,
    pub column_start: u32,
    pub row_end: u32,
    pub column_end: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpreadsheetTableSource {
    pub part: String,
    pub worksheet_index: u32,
    pub worksheet_name: String,
}
