//! Vue policy is applied at complete authored heads, outside the generic sink.

use vize_l0::{Allocator, Namespace, Vec, is_void_tag};

use super::SurfacePolicy;
use crate::build::scope::{Frame, Recovery};
use crate::event::{EventKind, Recorder};
use crate::markup::entity::DecodedEntity;
use crate::markup::token::{LexErrorCode, LexMode, QuoteType, Sink};
use core::marker::PhantomData;

struct ModeFrame<'a> {
    tag: &'a str,
    ns: Namespace,
    verbatim: bool,
}

impl<'a> Frame<'a> for ModeFrame<'a> {
    fn tag(&self) -> &'a str {
        self.tag
    }
    fn namespace(&self) -> Namespace {
        self.ns
    }
}

pub(crate) struct VueSink<'a, 'v, P: SurfacePolicy> {
    source: &'a str,
    recorder: Recorder<'a, 'v>,
    unsupported: &'v mut Vec<'a, P::Boundary>,
    stack: Vec<'a, ModeFrame<'a>>,
    recovery: Recovery<'a>,
    tag: Option<&'a str>,
    tag_events: usize,
    policy: PhantomData<P>,
}

impl<'a, 'v, P: SurfacePolicy> VueSink<'a, 'v, P> {
    pub(crate) fn new(
        allocator: &'a Allocator,
        source: &'a str,
        recorder: Recorder<'a, 'v>,
        unsupported: &'v mut Vec<'a, P::Boundary>,
    ) -> Self {
        Self {
            source,
            recorder,
            unsupported,
            stack: Vec::new_in(&allocator),
            recovery: Recovery::new(allocator, true),
            tag: None,
            tag_events: 0,
            policy: PhantomData,
        }
    }

    fn finish_tag(&mut self, self_closing: bool) -> bool {
        let Some(tag) = self.tag.take() else {
            return false;
        };
        let (ns, implicit_depth) = self.recovery.open(&self.stack, tag, self_closing);
        if let Some(depth) = implicit_depth {
            self.stack.truncate(depth);
        }
        // Inherit after recovery: an implicitly closed owner cannot affect
        // the newly opened sibling, even when it has the same authored name.
        let verbatim = self.mode() == LexMode::Verbatim || self.resolve_heads();
        if !self_closing && !is_void_tag(tag) {
            self.stack.push(ModeFrame { tag, ns, verbatim });
        }
        verbatim
    }

    /// Consume the current tag's existing complete heads after structural
    /// recovery. Inherited verbatim content short-circuits this hook entirely.
    fn resolve_heads(&mut self) -> bool {
        let unsupported_start = self.unsupported.len();
        let mut name_start = None;
        let mut pre = false;
        for event in self.recorder.events.iter().skip(self.tag_events) {
            match event.kind {
                EventKind::AttrName => {
                    name_start.get_or_insert(event.start);
                }
                EventKind::AttrNameEnd => {
                    if let Some(start) = name_start.take()
                        && let Some(raw) = self.source.get(start as usize..event.start as usize)
                    {
                        match P::pre(raw, start, self.source) {
                            Ok(control) => pre |= control,
                            Err(boundary) => self.unsupported.push(boundary),
                        }
                    }
                }
                _ => {}
            }
        }
        if pre {
            // An admitted control makes every other head on this same tag raw,
            // regardless of attribute order. Preserve facts from prior tags.
            self.unsupported.truncate(unsupported_start);
        }
        pre
    }
}

// Forward the generic vocabulary without rebuilding entities or callbacks.
macro_rules! forward {
    ($($name:ident($($arg:ident: $ty:ty),*);)*) => {
        $(fn $name(&mut self, $($arg: $ty),*) {
            self.recorder.$name($($arg),*);
        })*
    };
}

impl<P: SurfacePolicy> Sink for VueSink<'_, '_, P> {
    forward! {
        on_text(start: usize, end: usize);
        on_text_entity(ch: char, start: usize, end: usize);
        on_text_entity_value(value: DecodedEntity, start: usize, end: usize);
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

    fn on_interpolation(&mut self, start: usize, end: usize) {
        P::interpolation(
            self.source,
            &mut self.recorder,
            self.unsupported,
            start,
            end,
            false,
        );
    }

    fn on_raw_interpolation(&mut self, start: usize, end: usize) {
        P::interpolation(
            self.source,
            &mut self.recorder,
            self.unsupported,
            start,
            end,
            true,
        );
    }

    fn on_open_tag_name(&mut self, start: usize, end: usize) {
        self.tag = self.source.get(start..end);
        self.tag_events = self.recorder.events.len();
        self.recorder.on_open_tag_name(start, end);
    }

    fn on_open_tag_end(&mut self, end: usize) {
        let verbatim = self.finish_tag(false);
        self.recorder
            .opening_end(EventKind::OpenTagEnd, end, verbatim);
    }

    fn on_self_closing_tag(&mut self, end: usize) {
        let verbatim = self.finish_tag(true);
        self.recorder
            .opening_end(EventKind::SelfClosingTag, end, verbatim);
    }

    fn on_close_tag(&mut self, start: usize, end: usize) {
        if let Some(tag) = self.source.get(start..end)
            && let Some(depth) = self.recovery.close(&self.stack, tag)
        {
            self.stack.truncate(depth);
        }
        self.recorder.on_close_tag(start, end);
    }

    fn mode(&self) -> LexMode {
        if self.stack.last().is_some_and(|frame| frame.verbatim) {
            LexMode::Verbatim
        } else {
            LexMode::Normal
        }
    }
}
