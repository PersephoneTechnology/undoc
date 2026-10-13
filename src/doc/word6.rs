//! What a Word 6.0 or Word 95 document keeps differently from Word 97: text one byte per
//! character in the document's code page, and the font table that names that code page.

use super::fib::FcLcb;
use super::text::{self, DocChar, Piece};
use crate::error::{Error, Result};

/// Windows `charset` values that name no code page of their own: `ANSI_CHARSET`,
/// `DEFAULT_CHARSET` and `SYMBOL_CHARSET`.
const ANSI_CHARSET: u8 = 0;
const DEFAULT_CHARSET: u8 = 1;
const SYMBOL_CHARSET: u8 = 2;

/// The code page the document's text is in: that of the first font whose character set names
/// one, else Windows-1252. Word 6.0 and Word 95 store no code page for the text; the fonts it
/// is set in say which script it is.
pub(super) fn code_page(word_document: &[u8], sttbf_ffn: FcLcb) -> u16 {
    let start = sttbf_ffn.fc as usize;
    let Some(table) = start
        .checked_add(sttbf_ffn.lcb as usize)
        .and_then(|end| word_document.get(start..end))
    else {
        return 1252;
    };
    font_charsets(table)
        .find(|&chs| !matches!(chs, ANSI_CHARSET | DEFAULT_CHARSET | SYMBOL_CHARSET))
        .and_then(crate::codepage::from_charset)
        .unwrap_or(1252)
}

/// The character set of each font in a Word 6.0 font table: a 16-bit total length, then
/// records each led by its length less one, with the character set at offset 4.
fn font_charsets(table: &[u8]) -> impl Iterator<Item = u8> + '_ {
    let mut at = 2;
    std::iter::from_fn(move || {
        let len = *table.get(at)? as usize + 1;
        let record = table.get(at..at + len)?;
        at += len;
        record.get(4).copied()
    })
}

/// The pieces of a fast-saved document. Their descriptors are laid out as Word 97's, but a
/// piece's offset is a plain byte offset -- all of the text is one byte per character -- and
/// never reaches the bit Word 97 marks one-byte pieces with.
pub(super) fn pieces(word_document: &[u8], clx: FcLcb) -> Result<Vec<Piece>> {
    let mut pieces = text::parse_clx(word_document, clx)?;
    for piece in &mut pieces {
        piece.compressed = true;
    }
    Ok(pieces)
}

/// The main document text, characters `[0, ccp_text)`, decoded from `codepage`.
///
/// A double-byte code page decodes a lead byte and its trail byte into one character, which
/// then stands at the position of the next byte; nothing is looked up by position in this
/// text, so the positions only need to keep their order.
pub(super) fn read_chars(
    word_document: &[u8],
    pieces: &[Piece],
    ccp_text: u32,
    codepage: u16,
) -> Result<Vec<DocChar>> {
    let mut out = Vec::with_capacity(ccp_text as usize);
    for piece in pieces {
        let (start, end) = (piece.cp_start, piece.cp_end.min(ccp_text));
        if start >= end {
            continue;
        }
        let first = piece.fc as usize;
        let bytes = word_document
            .get(first..first + (end - start) as usize)
            .ok_or_else(|| {
                Error::InvalidData(
                    "Word document: a piece of text lies past the end of the WordDocument stream"
                        .into(),
                )
            })?;
        let text = crate::codepage::decode_lossy(bytes, codepage);
        for (i, ch) in text.chars().enumerate() {
            out.push(DocChar {
                ch,
                fc: (first + i) as u32,
                cp: start + i as u32,
            });
        }
    }
    Ok(out)
}
