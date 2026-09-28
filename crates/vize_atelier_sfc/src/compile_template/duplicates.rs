//! Validate SFC attribute names with tokenizer callbacks before any backend
//! can lower a repeated structural directive into nested control flow.

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
    let errors: Vec<_> = find_duplicate_props(&template.content)
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
