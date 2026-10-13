//! Word field instructions shared by the formats that carry them (`.docx` and `.doc`).

/// The target of a `HYPERLINK` field instruction: `HYPERLINK "url"`, or `HYPERLINK \l "name"`
/// for a bookmark in the same document.
pub(crate) fn hyperlink_target(instruction: &str) -> Option<String> {
    let rest = instruction.trim_start().strip_prefix("HYPERLINK")?;
    let mut url = None;
    let mut bookmark = false;
    let mut chars = rest.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if chars.next() == Some('l') {
                    bookmark = true;
                }
            }
            '"' => {
                let quoted: String = chars.by_ref().take_while(|&c| c != '"').collect();
                if url.is_none() {
                    url = Some(quoted);
                }
            }
            c if !c.is_whitespace() && url.is_none() => {
                let mut bare = String::from(c);
                while let Some(&next) = chars.peek() {
                    if next.is_whitespace() {
                        break;
                    }
                    bare.push(next);
                    chars.next();
                }
                url = Some(bare);
            }
            _ => {}
        }
    }
    let url = url.filter(|u| !u.is_empty())?;
    Some(if bookmark { format!("#{url}") } else { url })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hyperlink_targets_are_read_from_the_instruction() {
        assert_eq!(
            hyperlink_target(" HYPERLINK \"https://example.com/a b\" ").as_deref(),
            Some("https://example.com/a b")
        );
        assert_eq!(
            hyperlink_target("HYPERLINK \\l \"_Toc1\"").as_deref(),
            Some("#_Toc1")
        );
        assert_eq!(
            hyperlink_target("HYPERLINK https://example.com").as_deref(),
            Some("https://example.com")
        );
        assert_eq!(hyperlink_target(" PAGEREF _Toc1 \\h "), None);
    }
}
