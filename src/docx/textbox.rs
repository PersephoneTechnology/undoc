//! Textbox extraction retains the original paragraph context.
use super::{DocxParser, Paragraph, SourceContext, SourceTracker, escape_xml};

impl DocxParser {
    /// Extract paragraphs from `w:txbxContent` elements (text boxes/shapes).
    ///
    /// Text boxes in DOCX appear inside `w:drawing` or `mc:AlternateContent` elements.
    /// This method finds all `w:txbxContent` sections (skipping those inside `mc:Fallback`
    /// to avoid duplication) and parses the inner `<w:p>` elements as regular paragraphs.
    pub(super) fn extract_textbox_paragraphs(
        &mut self,
        xml: &str,
        context: Option<SourceContext>,
    ) -> Vec<Paragraph> {
        let mut paragraphs = Vec::new();
        let mut reader = crate::decode::reader_for(xml);
        reader.config_mut().trim_text(false);

        let has_context = context.is_some();
        let mut sources = SourceTracker::new(context);
        let mut paragraph_source = None;
        let mut buf = Vec::new();
        let mut mc_fallback_depth: u32 = 0;
        let mut txbx_content_depth: u32 = 0;
        let mut in_txbx_para = false;
        let mut txbx_para_xml = String::new();
        let mut txbx_para_depth: u32 = 0; // Track nested elements inside the text box <w:p>

        loop {
            let event = reader.read_event_into(&mut buf);
            if let Ok(ref event) = event {
                sources.observe(event);
            }
            match event {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        "mc:Fallback" => {
                            mc_fallback_depth += 1;
                        }
                        "w:txbxContent" if mc_fallback_depth == 0 => {
                            txbx_content_depth += 1;
                        }
                        "w:p" if txbx_content_depth > 0 && !in_txbx_para => {
                            paragraph_source =
                                has_context.then(|| sources.context().location()).flatten();
                            in_txbx_para = true;
                            txbx_para_depth = 0;
                            txbx_para_xml.clear();
                            txbx_para_xml.push_str("<w:p");
                            for attr in e.attributes().flatten() {
                                txbx_para_xml.push_str(&format!(
                                    " {}=\"{}\"",
                                    attr.key.as_ref(),
                                    attr.value.as_ref()
                                ));
                            }
                            txbx_para_xml.push('>');
                        }
                        _ if in_txbx_para => {
                            txbx_para_depth += 1;
                            txbx_para_xml.push('<');
                            txbx_para_xml.push_str(name.as_ref());
                            for attr in e.attributes().flatten() {
                                txbx_para_xml.push_str(&format!(
                                    " {}=\"{}\"",
                                    attr.key.as_ref(),
                                    attr.value.as_ref()
                                ));
                            }
                            txbx_para_xml.push('>');
                        }
                        _ => {}
                    }
                }
                Ok(quick_xml::events::Event::Empty(ref e)) if in_txbx_para => {
                    let name = e.name();
                    txbx_para_xml.push('<');
                    txbx_para_xml.push_str(name.as_ref());
                    for attr in e.attributes().flatten() {
                        txbx_para_xml.push_str(&format!(
                            " {}=\"{}\"",
                            attr.key.as_ref(),
                            attr.value.as_ref()
                        ));
                    }
                    txbx_para_xml.push_str("/>");
                }
                Ok(quick_xml::events::Event::Text(ref e)) if in_txbx_para => {
                    let text = crate::decode::decode_text_lossy(e);
                    txbx_para_xml.push_str(&escape_xml(&text));
                }
                // Re-serialized buffer (parsed a second time): re-emit the resolved
                // entity through escape_xml so the second pass decodes it again.
                Ok(quick_xml::events::Event::GeneralRef(ref e)) if in_txbx_para => {
                    let decoded = crate::decode::resolve_general_ref(e);
                    txbx_para_xml.push_str(&escape_xml(&decoded));
                }
                Ok(quick_xml::events::Event::End(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        "mc:Fallback" if mc_fallback_depth > 0 => {
                            mc_fallback_depth -= 1;
                        }
                        "w:txbxContent" if txbx_content_depth > 0 => {
                            txbx_content_depth -= 1;
                        }
                        "w:p" if in_txbx_para && txbx_para_depth == 0 => {
                            txbx_para_xml.push_str("</w:p>");
                            if let Ok(mut para) = self.parse_paragraph(&txbx_para_xml) {
                                para.source = paragraph_source.take();
                                if !para.plain_text().is_empty() {
                                    paragraphs.push(para);
                                }
                            }
                            in_txbx_para = false;
                        }
                        _ if in_txbx_para => {
                            txbx_para_depth = txbx_para_depth.saturating_sub(1);
                            txbx_para_xml.push_str("</");
                            txbx_para_xml.push_str(name.as_ref());
                            txbx_para_xml.push('>');
                        }
                        _ => {}
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }

        paragraphs
    }
}
