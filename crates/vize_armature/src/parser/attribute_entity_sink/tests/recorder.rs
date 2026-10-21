//! Whole public callback vectors used only by the private consumer tests.

use crate::tokenizer::{Callbacks, QuoteType};
use vize_l0::ErrorCode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Event {
    Range(&'static str, usize, usize),
    EndAt(&'static str, usize),
    Entity(&'static str, char, usize, usize),
    Quote(QuoteType, usize),
    Error(ErrorCode, usize),
    End,
}

#[derive(Default)]
pub(super) struct Recorder {
    pub(super) events: Vec<Event>,
    pub(super) verbatim: bool,
}

#[derive(Default)]
pub(super) struct FullRecorder {
    pub(super) events: Vec<Event>,
    pub(super) verbatim: bool,
}

macro_rules! original_callbacks {
    () => {
        fn on_text(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("text", start, end));
        }

        fn on_interpolation(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("interpolation", start, end));
        }

        fn on_raw_interpolation(&mut self, start: usize, end: usize) {
            self.events
                .push(Event::Range("raw_interpolation", start, end));
        }

        fn on_open_tag_name(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("open_tag_name", start, end));
        }

        fn on_close_tag(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("close_tag", start, end));
        }

        fn on_attrib_data(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("attrib_data", start, end));
        }

        fn on_attrib_name(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("attrib_name", start, end));
        }

        fn on_dir_name(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("dir_name", start, end));
        }

        fn on_dir_arg(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("dir_arg", start, end));
        }

        fn on_dir_modifier(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("dir_modifier", start, end));
        }

        fn on_comment(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("comment", start, end));
        }

        fn on_in_tag_comment(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("in_tag_comment", start, end));
        }

        fn on_cdata(&mut self, start: usize, end: usize) {
            self.events.push(Event::Range("cdata", start, end));
        }

        fn on_processing_instruction(&mut self, start: usize, end: usize) {
            self.events
                .push(Event::Range("processing_instruction", start, end));
        }

        fn on_open_tag_end(&mut self, end: usize) {
            self.events.push(Event::EndAt("open_tag_end", end));
        }

        fn on_self_closing_tag(&mut self, end: usize) {
            self.events.push(Event::EndAt("self_closing_tag", end));
        }

        fn on_attrib_name_end(&mut self, end: usize) {
            self.events.push(Event::EndAt("attrib_name_end", end));
        }

        fn on_text_entity(&mut self, ch: char, start: usize, end: usize) {
            self.events.push(Event::Entity("text", ch, start, end));
        }

        fn on_attrib_entity(&mut self, ch: char, start: usize, end: usize) {
            self.events.push(Event::Entity("attribute", ch, start, end));
        }

        fn on_attrib_end(&mut self, quote: QuoteType, end: usize) {
            self.events.push(Event::Quote(quote, end));
        }

        fn on_end(&mut self) {
            self.events.push(Event::End);
        }

        fn on_error(&mut self, code: ErrorCode, index: usize) {
            self.events.push(Event::Error(code, index));
        }

        fn is_in_v_pre(&self) -> bool {
            self.verbatim
        }
    };
}

impl Callbacks for Recorder {
    original_callbacks!();
}

impl Callbacks for FullRecorder {
    original_callbacks!();

    fn on_attrib_entity_value(
        &mut self,
        value: vize_l1::markup::entity::DecodedEntity,
        start: usize,
        end: usize,
    ) {
        value.for_each(|ch| self.on_attrib_entity(ch, start, end));
    }
}
