//! Validate complete SFC header identities before structural lowering.
//! Reuse the actual DOM surface where admitted; retain the callback preflight
//! for unsupported observations, actual compatibility entries and SSR.

use vize_armature::tokenizer::{Callbacks, QuoteType, Tokenizer};
use vize_atelier_core::{CompilerError, CompilerErrorWithSource, ErrorCode, SourceLocation};
use vize_l0::{String, ToCompactString};

use crate::types::{SfcError, SfcTemplateBlock};

struct Candidate<'a> {
    name: &'a str,
    argument: Option<(&'a str, bool)>,
    modifiers: Vec<&'a str>,
    start: usize,
    end: usize,
    directive: bool,
}

impl Candidate<'_> {
    fn same_name(&self, other: &Self) -> bool {
        if self.directive != other.directive {
            return false;
        }
        if self.directive {
            self.name == other.name
                && self.argument == other.argument
                && self.modifiers == other.modifiers
        } else {
            self.name.eq_ignore_ascii_case(other.name)
        }
    }

    fn may_repeat(&self) -> bool {
        self.directive
            && (matches!(self.name, "@" | "v-on")
                || (self.name == "v-bind" && self.argument.is_none()))
    }
}

struct DuplicateCollector<'a> {
    source: &'a str,
    current: Option<Candidate<'a>>,
    seen: Vec<Candidate<'a>>,
    duplicates: Vec<SourceLocation>,
}

pub(super) fn reject_duplicate_props(template: &SfcTemplateBlock) -> Result<(), SfcError> {
    reject_spans(template, find_duplicate_props(&template.content))
}

fn reject_spans(template: &SfcTemplateBlock, spans: Vec<SourceLocation>) -> Result<(), SfcError> {
    let errors: Vec<_> = spans
        .into_iter()
        .map(|span| CompilerError::new(ErrorCode::DuplicateAttribute, Some(span)))
        .collect();
    if errors.is_empty() {
        return Ok(());
    }
    let with_source: Vec<_> = errors
        .iter()
        .map(|error| CompilerErrorWithSource::new(error, &template.content))
        .collect();
    let mut message = String::from("Template compilation errors: ");
    use std::fmt::Write as _;
    let _ = write!(&mut message, "{:?}", with_source);
    Err(SfcError {
        message,
        code: Some("TEMPLATE_ERROR".to_compact_string()),
        loc: Some(template.loc.clone()),
    })
}

fn find_duplicate_props(source: &str) -> Vec<SourceLocation> {
    let mut collector = DuplicateCollector {
        source,
        current: None,
        seen: Vec::new(),
        duplicates: Vec::new(),
    };
    Tokenizer::new(source, &mut collector).tokenize();
    collector.duplicates
}

impl<'a> Callbacks for &mut DuplicateCollector<'a> {
    fn on_text(&mut self, _start: usize, _end: usize) {}
    fn on_text_entity(&mut self, _ch: char, _start: usize, _end: usize) {}
    fn on_interpolation(&mut self, _start: usize, _end: usize) {}
    fn on_open_tag_name(&mut self, _start: usize, _end: usize) {
        self.current = None;
        self.seen.clear();
    }
    fn on_open_tag_end(&mut self, _end: usize) {}
    fn on_self_closing_tag(&mut self, _end: usize) {}
    fn on_close_tag(&mut self, _start: usize, _end: usize) {}
    fn on_attrib_data(&mut self, _start: usize, _end: usize) {}
    fn on_attrib_entity(&mut self, _ch: char, _start: usize, _end: usize) {}
    fn on_attrib_end(&mut self, _quote: QuoteType, _end: usize) {
        let Some(candidate) = self.current.take() else {
            return;
        };
        if candidate.may_repeat() {
            return;
        }
        if self.seen.iter().any(|seen| seen.same_name(&candidate)) {
            self.duplicates.push(SourceLocation::new(
                candidate.start as u32,
                candidate.end as u32,
            ));
        }
        self.seen.push(candidate);
    }
    fn on_attrib_name(&mut self, start: usize, end: usize) {
        self.current = Some(Candidate {
            name: self.source.get(start..end).unwrap_or_default(),
            argument: None,
            modifiers: Vec::new(),
            start,
            end,
            directive: false,
        });
    }
    fn on_attrib_name_end(&mut self, end: usize) {
        if let Some(current) = self.current.as_mut() {
            current.end = end;
        }
    }
    fn on_dir_name(&mut self, start: usize, end: usize) {
        self.current = Some(Candidate {
            name: self.source.get(start..end).unwrap_or_default(),
            argument: None,
            modifiers: Vec::new(),
            start,
            end,
            directive: true,
        });
    }
    fn on_dir_arg(&mut self, start: usize, end: usize) {
        if let Some(current) = self.current.as_mut() {
            let argument = self.source.get(start..end).unwrap_or_default();
            let dynamic = self.source.as_bytes().get(start.wrapping_sub(1)) == Some(&b'[');
            current.argument = Some((argument, dynamic));
            current.end = end;
        }
    }
    fn on_dir_modifier(&mut self, start: usize, end: usize) {
        if let Some(current) = self.current.as_mut() {
            current
                .modifiers
                .push(self.source.get(start..end).unwrap_or_default());
            current.end = end;
        }
    }
    fn on_comment(&mut self, _start: usize, _end: usize) {}
    fn on_cdata(&mut self, _start: usize, _end: usize) {}
    fn on_processing_instruction(&mut self, _start: usize, _end: usize) {}
    fn on_end(&mut self) {}
    fn on_error(&mut self, _code: ErrorCode, _index: usize) {}
}

mod surface;

/// The SFC caller alone owns the complete fatal diagnostic and exact-once state.
pub(super) struct Preflight<'t, 'a> {
    template: &'t SfcTemplateBlock<'a>,
    enabled: bool,
    native_in_tag_comments: bool,
    result: Option<Result<(), SfcError>>,
    #[cfg(test)]
    native_calls: usize,
    #[cfg(test)]
    retained_calls: usize,
}
impl<'t, 'a> Preflight<'t, 'a> {
    pub(super) fn new(
        template: &'t SfcTemplateBlock<'a>,
        enabled: bool,
        native_in_tag_comments: bool,
    ) -> Self {
        Self {
            template,
            enabled,
            native_in_tag_comments,
            result: None,
            #[cfg(test)]
            native_calls: 0,
            #[cfg(test)]
            retained_calls: 0,
        }
    }
    fn retained(&mut self) -> Result<(), SfcError> {
        #[cfg(test)]
        {
            self.retained_calls += 1;
        }
        reject_duplicate_props(self.template)
    }
    pub(super) fn finish(mut self) -> Result<(), SfcError> {
        match self.result.take() {
            Some(result) => result,
            None if self.enabled => self.retained(),
            None => Ok(()),
        }
    }
}
impl vize_l1_to_l2::SurfacePreflight for Preflight<'_, '_> {
    fn check(
        &mut self,
        surface: Option<(
            &vize_l1::markup::SurfaceTree<'_>,
            &[vize_l1::markup::SurfaceError],
        )>,
    ) -> bool {
        if self.result.is_none() {
            self.result = Some(if !self.enabled {
                Ok(())
            } else if let Some((tree, errors)) = surface {
                #[cfg(test)]
                {
                    self.native_calls += 1;
                }
                // The retained callback always uses default comment policy.
                // A comment-aware native tree cannot supply those facts.
                match (!self.native_in_tag_comments
                    && core::ptr::eq(tree.source, self.template.content.as_ref()))
                .then(|| surface::find_in_surface(tree, errors))
                .flatten()
                {
                    Some(spans) => reject_spans(self.template, spans),
                    None => self.retained(),
                }
            } else {
                self.retained()
            });
        }
        self.result.as_ref().is_some_and(Result::is_ok)
    }
}

#[cfg(test)]
mod preflight_tests;

#[cfg(all(test, feature = "legacy-dom-differential"))]
mod whole_parity;

#[cfg(all(test, feature = "legacy-dom-differential"))]
pub(super) fn original_preflight_requested() -> bool {
    whole_parity::original_preflight_requested()
}

#[cfg(test)]
mod tests {
    use super::find_duplicate_props;

    #[test]
    fn distinguishes_directive_arguments_and_ignores_repeatable_listeners() {
        assert!(
            find_duplicate_props(r#"<p :one="a" :two="b" @click="a" @click="b" />"#).is_empty()
        );
        assert_eq!(find_duplicate_props(r#"<p v-if="a" v-if="b" />"#).len(), 1);
        assert_eq!(
            find_duplicate_props(r#"<p CLASS="a" class="b" />"#).len(),
            1
        );
    }
}
