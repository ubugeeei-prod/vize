//! Generic live namespace recovery; authored heads are recorded uninterpreted.

use vize_l0::{Allocator, Namespace, Vec, is_void_tag};

use crate::build::scope::{Frame, Recovery};
use crate::event::Recorder;
use crate::markup::entity::DecodedEntity;
use crate::markup::{LexErrorCode, QuoteType, Sink};

struct OpenFrame<'a> {
    tag: &'a str,
    ns: Namespace,
}

impl<'a> Frame<'a> for OpenFrame<'a> {
    fn tag(&self) -> &'a str {
        self.tag
    }

    fn namespace(&self) -> Namespace {
        self.ns
    }
}

pub(super) struct NamespaceSink<'a, 'v> {
    source: &'a str,
    recorder: Recorder<'a, 'v>,
    stack: Vec<'a, OpenFrame<'a>>,
    recovery: Recovery<'a>,
    tag: Option<&'a str>,
}

impl<'a, 'v> NamespaceSink<'a, 'v> {
    pub(super) fn new(
        allocator: &'a Allocator,
        source: &'a str,
        recorder: Recorder<'a, 'v>,
    ) -> Self {
        Self {
            source,
            recorder,
            stack: Vec::new_in(&allocator),
            recovery: Recovery::new(allocator, true),
            tag: None,
        }
    }

    fn finish_tag(&mut self, self_closing: bool) {
        let Some(tag) = self.tag.take() else { return };
        let (ns, implicit_depth) = self.recovery.open(&self.stack, tag, self_closing);
        if let Some(depth) = implicit_depth {
            self.stack.truncate(depth);
        }
        if !self_closing && !is_void_tag(tag) {
            self.stack.push(OpenFrame { tag, ns });
        }
    }
}

macro_rules! forward {
    ($($name:ident($($arg:ident: $ty:ty),*);)*) => {
        $(fn $name(&mut self, $($arg: $ty),*) {
            self.recorder.$name($($arg),*);
        })*
    };
}

impl Sink for NamespaceSink<'_, '_> {
    forward! {
        on_text(start: usize, end: usize);
        on_text_entity(ch: char, start: usize, end: usize);
        on_text_entity_value(value: DecodedEntity, start: usize, end: usize);
        on_interpolation(start: usize, end: usize);
        on_attrib_data(start: usize, end: usize);
        on_attrib_entity(ch: char, start: usize, end: usize);
        on_attrib_entity_value(value: DecodedEntity, start: usize, end: usize);
        on_attrib_end(quote: QuoteType, end: usize);
        on_attrib_name(start: usize, end: usize);
        on_attrib_name_end(end: usize);
        on_dir_name(start: usize, end: usize);
        on_dir_arg(start: usize, end: usize);
        on_dir_modifier(start: usize, end: usize);
        on_comment(start: usize, end: usize);
        on_cdata(start: usize, end: usize);
        on_processing_instruction(start: usize, end: usize);
        on_end();
        on_error(code: LexErrorCode, index: usize);
    }

    fn on_open_tag_name(&mut self, start: usize, end: usize) {
        self.tag = self.source.get(start..end);
        self.recorder.on_open_tag_name(start, end);
    }

    fn svg_title_children(&self) -> bool {
        self.stack.last().is_some_and(|parent| {
            parent.ns == Namespace::Svg && !matches!(parent.tag, "foreignObject" | "desc" | "title")
        })
    }

    fn on_open_tag_end(&mut self, end: usize) {
        self.finish_tag(false);
        self.recorder.on_open_tag_end(end);
    }

    fn on_self_closing_tag(&mut self, end: usize) {
        self.finish_tag(true);
        self.recorder.on_self_closing_tag(end);
    }

    fn on_close_tag(&mut self, start: usize, end: usize) {
        if let Some(tag) = self.source.get(start..end)
            && let Some(depth) = self.recovery.close(&self.stack, tag)
        {
            self.stack.truncate(depth);
        }
        self.recorder.on_close_tag(start, end);
    }
}
