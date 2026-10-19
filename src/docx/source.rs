//! Structural tracking alongside the existing parser, including ignored XML nodes.
use crate::model::{XmlPathSegment, XmlSourceLocation};
use quick_xml::events::{BytesStart, Event};
use std::collections::HashMap;

#[derive(Clone, Default)]
struct Frame {
    namespaces: HashMap<String, String>,
    children: HashMap<(String, String), u32>,
    path: Vec<XmlPathSegment>,
    paragraph_id: Option<String>,
    valid: bool,
}

/// Captured before fragment serialization loses its original parent context.
#[derive(Clone)]
pub(super) struct SourceContext(Frame);
impl SourceContext {
    pub(super) fn location(&self) -> Option<XmlSourceLocation> {
        self.0.valid.then(|| XmlSourceLocation {
            part: "word/document.xml".into(),
            path: self.0.path.clone(),
            paragraph_id: self.0.paragraph_id.clone(),
        })
    }
}

pub(super) struct SourceTracker {
    frames: Vec<Frame>,
    fragment: Option<SourceContext>,
}
impl SourceTracker {
    pub(super) fn new(fragment: Option<SourceContext>) -> Self {
        Self {
            frames: vec![Frame {
                valid: fragment.is_none(),
                ..Frame::default()
            }],
            fragment,
        }
    }
    pub(super) fn context(&self) -> SourceContext {
        SourceContext(self.frames.last().unwrap().clone())
    }
    // End events are consumed before parser branches which can `continue`.
    pub(super) fn observe(&mut self, event: &Event<'_>) {
        match event {
            Event::Start(e) => self.enter(e),
            Event::Empty(e) => {
                self.enter(e);
                self.frames.pop();
            }
            Event::End(_) if self.frames.len() > 1 => {
                self.frames.pop();
            }
            _ => {}
        }
    }
    fn enter(&mut self, e: &BytesStart<'_>) {
        if let Some(SourceContext(mut frame)) = self.fragment.take() {
            frame.children.clear();
            self.frames.push(frame);
            return;
        }
        let parent = self.frames.last_mut().unwrap();
        let mut frame = Frame {
            namespaces: parent.namespaces.clone(),
            path: parent.path.clone(),
            valid: parent.valid,
            ..Frame::default()
        };
        for attr in e.attributes().flatten() {
            if attr.key.as_ref() == "xmlns" {
                frame
                    .namespaces
                    .insert(String::new(), crate::decode::attr_value(&attr));
            } else if let Some(prefix) = attr.key.as_ref().strip_prefix("xmlns:") {
                frame
                    .namespaces
                    .insert(prefix.into(), crate::decode::attr_value(&attr));
            }
        }
        let name = e.name();
        let (prefix, local) = name.as_ref().split_once(':').unwrap_or(("", name.as_ref()));
        let namespace = frame.namespaces.get(prefix).cloned().unwrap_or_default();
        if !prefix.is_empty() && namespace.is_empty() {
            frame.valid = false;
        }
        let index = parent
            .children
            .entry((namespace.clone(), local.into()))
            .or_default();
        *index = index.saturating_add(1);
        frame.path.push(XmlPathSegment {
            namespace,
            local_name: local.into(),
            index: *index,
        });
        for attr in e.attributes().flatten() {
            if let Some((prefix, "paraId")) = attr.key.as_ref().split_once(':') {
                if frame
                    .namespaces
                    .get(prefix)
                    .is_some_and(|ns| ns == "http://schemas.microsoft.com/office/word/2010/wordml")
                {
                    frame.paragraph_id = Some(crate::decode::attr_value(&attr));
                }
            }
        }
        self.frames.push(frame);
    }
}
