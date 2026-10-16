//! The compatibility parser consumes v-pre itself. Read only a candidate's
//! existing authored opening span with the canonical lexer; quoted text cannot
//! impersonate a directive and no whole-template parser mode changes.

use vize_l1::markup::{Component, LexErrorCode, LexOptions, Lexer, QuoteType, Sink};
use vize_relief::ElementNode;

pub(super) fn opens(element: &ElementNode<'_>, source: &str) -> bool {
    // The canonical parser gives opening-only locations. Authenticate that
    // contract before lexing, including externally supplied/lowered nodes:
    // a widened whole-element range must never lex its subtree per ancestor.
    let Some(header) = super::super::authored_element::opening(source, element) else {
        return false;
    };
    if !header.contains("v-pre") {
        return false;
    }
    let mut lexer = Lexer::<Component, _>::new(
        header,
        Pre {
            header,
            found: false,
            done: false,
        },
        LexOptions::default(),
    );
    lexer.run();
    lexer.into_sink().found
}

struct Pre<'a> {
    header: &'a str,
    found: bool,
    done: bool,
}

impl Sink for Pre<'_> {
    fn on_dir_name(&mut self, start: usize, end: usize) {
        if !self.done {
            self.found |= self.header.get(start..end) == Some("v-pre");
        }
    }
    fn on_text(&mut self, _: usize, _: usize) {}
    fn on_text_entity(&mut self, _: char, _: usize, _: usize) {}
    fn on_interpolation(&mut self, _: usize, _: usize) {}
    fn on_open_tag_name(&mut self, _: usize, _: usize) {}
    fn on_open_tag_end(&mut self, _: usize) {
        self.done = true;
    }
    fn on_self_closing_tag(&mut self, _: usize) {
        self.done = true;
    }
    fn on_close_tag(&mut self, _: usize, _: usize) {}
    fn on_attrib_data(&mut self, _: usize, _: usize) {}
    fn on_attrib_entity(&mut self, _: char, _: usize, _: usize) {}
    fn on_attrib_end(&mut self, _: QuoteType, _: usize) {}
    fn on_attrib_name(&mut self, _: usize, _: usize) {}
    fn on_attrib_name_end(&mut self, _: usize) {}
    fn on_dir_arg(&mut self, _: usize, _: usize) {}
    fn on_dir_modifier(&mut self, _: usize, _: usize) {}
    fn on_comment(&mut self, _: usize, _: usize) {}
    fn on_cdata(&mut self, _: usize, _: usize) {}
    fn on_processing_instruction(&mut self, _: usize, _: usize) {}
    fn on_end(&mut self) {}
    fn on_error(&mut self, _: LexErrorCode, _: usize) {}
}

#[cfg(test)]
mod tests {
    use super::opens;
    use vize_l0::Allocator;
    use vize_relief::TemplateChildNode;

    #[test]
    fn only_an_authenticated_opening_header_can_reach_the_lexer() {
        let source = r#"<Panel v-pre><Nested slot-scope="{ item }" /></Panel>"#;
        let allocator = Allocator::new();
        let (mut root, errors) = vize_armature::parse(&allocator, source);
        assert!(errors.is_empty(), "{errors:?}");
        let Some(TemplateChildNode::Element(element)) = root.children.first_mut() else {
            panic!("original element");
        };
        assert!(opens(element, source));
        element.loc.span.end = source.len() as u32;
        assert!(
            !opens(element, source),
            "a subtree is not an opening header"
        );
    }

    #[test]
    fn quoted_text_and_descendant_pre_do_not_freeze_the_owner() {
        let source = r#"<Panel title="v-pre"><Nested v-pre /></Panel>"#;
        let allocator = Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, source);
        assert!(errors.is_empty(), "{errors:?}");
        let Some(TemplateChildNode::Element(element)) = root.children.first() else {
            panic!("original element");
        };
        assert!(!opens(element, source));
    }
}
