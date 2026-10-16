//! The compatibility parser consumes v-pre itself. Read only a candidate's
//! existing authored opening span with the canonical lexer; quoted text cannot
//! impersonate a directive and no whole-template parser mode changes.

use vize_l1::markup::{Component, LexErrorCode, LexOptions, Lexer, QuoteType, Sink};
use vize_relief::ElementNode;

pub(super) fn opens(element: &ElementNode<'_>, source: &str) -> bool {
    let Some(header) = source.get(element.loc.span.start as usize..element.loc.span.end as usize)
    else {
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
