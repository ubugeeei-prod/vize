//! Static template sinks share serialization without recording plain output.

use vize_atelier_core::codegen::document::EmitDocument;
use vize_carton::{Span, String};

pub(super) trait TemplateWriter {
    fn push_str(&mut self, text: &str);
    fn push_linked(&mut self, text: &str, authored: Span);
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
