use vize_l0::SmallVec;

use super::{LexOptions, Lexer};
use crate::markup::profile::Component;
use crate::markup::token::{LexErrorCode, QuoteType, Sink};

mod basic;
mod entities;
mod entity_expansion;
mod errors;
mod raw;
mod special;

// ========================================================================
// Test callback infrastructure
// ========================================================================

#[derive(Debug, PartialEq)]
pub(super) enum TokenEvent {
    Text(usize, usize),
    TextEntity(char, usize, usize),
    Interpolation(usize, usize),
    RawInterpolation(usize, usize),
    OpenTagName(usize, usize),
    OpenTagEnd(usize),
    SelfClosingTag(usize),
    CloseTag(usize, usize),
    AttribName(usize, usize),
    AttribData(usize, usize),
    AttribEnd(QuoteType, usize),
    AttribEntity(char, usize, usize),
    DirName(usize, usize),
    DirArg(usize, usize),
    DirModifier(usize, usize),
    Comment(usize, usize),
    Cdata(usize, usize),
    End,
}

#[derive(Debug, Default)]
pub(super) struct TestCallbacks {
    pub(super) events: SmallVec<[TokenEvent; 16]>,
    pub(super) errors: SmallVec<[(LexErrorCode, usize); 4]>,
}

impl Sink for TestCallbacks {
    fn on_text(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::Text(start, end));
    }
    fn on_text_entity(&mut self, _char: char, start: usize, end: usize) {
        self.events.push(TokenEvent::TextEntity(_char, start, end));
    }
    fn on_interpolation(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::Interpolation(start, end));
    }
    fn on_raw_interpolation(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::RawInterpolation(start, end));
    }
    fn on_open_tag_name(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::OpenTagName(start, end));
    }
    fn on_open_tag_end(&mut self, end: usize) {
        self.events.push(TokenEvent::OpenTagEnd(end));
    }
    fn on_self_closing_tag(&mut self, end: usize) {
        self.events.push(TokenEvent::SelfClosingTag(end));
    }
    fn on_close_tag(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::CloseTag(start, end));
    }
    fn on_attrib_name(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::AttribName(start, end));
    }
    fn on_attrib_name_end(&mut self, _end: usize) {}
    fn on_attrib_data(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::AttribData(start, end));
    }
    fn on_attrib_entity(&mut self, ch: char, start: usize, end: usize) {
        self.events.push(TokenEvent::AttribEntity(ch, start, end));
    }
    fn on_attrib_end(&mut self, quote: QuoteType, end: usize) {
        self.events.push(TokenEvent::AttribEnd(quote, end));
    }
    fn on_dir_name(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::DirName(start, end));
    }
    fn on_dir_arg(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::DirArg(start, end));
    }
    fn on_dir_modifier(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::DirModifier(start, end));
    }
    fn on_comment(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::Comment(start, end));
    }
    fn on_cdata(&mut self, start: usize, end: usize) {
        self.events.push(TokenEvent::Cdata(start, end));
    }
    fn on_processing_instruction(&mut self, _start: usize, _end: usize) {}
    fn on_end(&mut self) {
        self.events.push(TokenEvent::End);
    }
    fn on_error(&mut self, code: LexErrorCode, index: usize) {
        self.errors.push((code, index));
    }
}

pub(super) fn lex_with(input: &str, options: LexOptions<'_>) -> TestCallbacks {
    let mut lexer = Lexer::<Component, _>::new(input, TestCallbacks::default(), options);
    lexer.run();
    lexer.into_sink()
}

fn tokenize(input: &str) -> TestCallbacks {
    lex_with(input, LexOptions::default())
}

// ========================================================================
// Utility function tests
// ========================================================================
