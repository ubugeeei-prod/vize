//! Static template sinks share serialization without recording plain output.

use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{Span, String};

pub(super) trait TemplateWriter {
    fn push_str(&mut self, text: &str);
    fn push_linked(&mut self, text: &str, authored: Span);
}

/// A static template is parsed as HTML again. Compensate for its first-newline
/// rule without assigning the generated extra byte an authored source range.
pub(super) struct LeadingNewlineWriter<'a> {
    pub(super) sink: &'a mut dyn TemplateWriter,
    pub(super) first: bool,
}

impl LeadingNewlineWriter<'_> {
    fn prefix(&mut self, text: &str) {
        if self.first && !text.is_empty() {
            self.first = false;
            if text.starts_with('\n') || text.starts_with("\r\n") {
                self.sink.push_str("\n");
            }
        }
    }
}

impl TemplateWriter for LeadingNewlineWriter<'_> {
    fn push_str(&mut self, text: &str) {
        self.prefix(text);
        self.sink.push_str(text);
    }

    fn push_linked(&mut self, text: &str, authored: Span) {
        self.prefix(text);
        self.sink.push_linked(text, authored);
    }
}

impl TemplateWriter for String {
    fn push_str(&mut self, text: &str) {
        String::push_str(self, text);
    }

    fn push_linked(&mut self, text: &str, _authored: Span) {
        String::push_str(self, text);
    }
}

impl TemplateWriter for EmitDocument {
    fn push_str(&mut self, text: &str) {
        EmitDocument::push_str(self, text);
    }

    fn push_linked(&mut self, text: &str, authored: Span) {
        EmitDocument::push_linked(self, text, authored);
    }
}
