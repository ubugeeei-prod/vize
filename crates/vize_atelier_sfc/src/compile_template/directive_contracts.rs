//! Declared `vue.directive` contracts (#13074) checked before any lane emits.
//!
//! One tokenizer pass over the template, only when a plan declares
//! directives, finds every `v-<name>` binding of a declared directive and
//! refuses one that carries an undeclared modifier or lacks a required one.

use vize_armature::tokenizer::{Callbacks, QuoteType, Tokenizer};
use vize_atelier_core::ErrorCode;
use vize_l0::config::{DirectiveContract, declared_directive};
use vize_l0::{ToCompactString, cstr};

use crate::types::{BlockLocation, SfcError, SfcTemplateBlock};

/// One `v-<name>` binding: its name, modifiers and authored span.
struct Binding<'a> {
    name: &'a str,
    modifiers: Vec<&'a str>,
    start: usize,
    end: usize,
}

#[derive(Default)]
struct Bindings<'a> {
    source: &'a str,
    current: Option<Binding<'a>>,
    found: Vec<Binding<'a>>,
}

impl<'a> Bindings<'a> {
    fn finish(&mut self) {
        if let Some(binding) = self.current.take() {
            self.found.push(binding);
        }
    }
}

/// Refuse the template when a binding breaks its directive's contract. The
/// error lists every violation and points at the first one.
pub(crate) fn check(
    template: &SfcTemplateBlock,
    contracts: &[DirectiveContract],
) -> Result<(), SfcError> {
    if contracts.is_empty() {
        return Ok(());
    }
    let source: &str = &template.content;
    let mut bindings = Bindings {
        source,
        ..Bindings::default()
    };
    Tokenizer::new(source, &mut bindings).tokenize();
    bindings.finish();
    let mut violations = bindings.found.iter().filter_map(|binding| {
        let contract = declared_directive(contracts, binding.name)?;
        let error = contract.normalized_modifiers(&binding.modifiers).err()?;
        Some((binding, cstr!("{error} (declared by {})", contract.plugin)))
    });
    let Some((first, message)) = violations.next() else {
        return Ok(());
    };
    let message = violations.fold(message, |mut message, (_, next)| {
        message.push('\n');
        message.push_str(&next);
        message
    });
    Err(SfcError {
        message,
        code: Some("PLUGIN_DIRECTIVE_CONTRACT".to_compact_string()),
        loc: Some(location(template, first.start, first.end)),
    })
}

/// The SFC location of `start..end` within the template content.
fn location(template: &SfcTemplateBlock, start: usize, end: usize) -> BlockLocation {
    let content: &str = &template.content;
    let position = |offset: usize| {
        let before = content.get(..offset).unwrap_or_default();
        match before.rfind('\n') {
            Some(newline) => (
                template.loc.start_line + before.matches('\n').count(),
                offset - newline,
            ),
            None => (template.loc.start_line, template.loc.start_column + offset),
        }
    };
    let (start_line, start_column) = position(start);
    let (end_line, end_column) = position(end);
    BlockLocation {
        start: template.loc.start + start,
        end: template.loc.start + end,
        tag_start: template.loc.start + start,
        tag_end: template.loc.start + end,
        start_line,
        start_column,
        end_line,
        end_column,
    }
}

impl<'a> Callbacks for &mut Bindings<'a> {
    fn on_text(&mut self, _start: usize, _end: usize) {}
    fn on_text_entity(&mut self, _ch: char, _start: usize, _end: usize) {}
    fn on_interpolation(&mut self, _start: usize, _end: usize) {}
    fn on_open_tag_name(&mut self, _start: usize, _end: usize) {
        self.finish();
    }
    fn on_open_tag_end(&mut self, _end: usize) {
        self.finish();
    }
    fn on_self_closing_tag(&mut self, _end: usize) {
        self.finish();
    }
    fn on_close_tag(&mut self, _start: usize, _end: usize) {}
    fn on_attrib_data(&mut self, _start: usize, _end: usize) {}
    fn on_attrib_entity(&mut self, _ch: char, _start: usize, _end: usize) {}
    fn on_attrib_end(&mut self, _quote: QuoteType, _end: usize) {
        self.finish();
    }
    fn on_attrib_name(&mut self, _start: usize, _end: usize) {
        self.finish();
    }
    fn on_attrib_name_end(&mut self, _end: usize) {}
    fn on_dir_name(&mut self, start: usize, end: usize) {
        self.finish();
        let name = self.source.get(start..end).unwrap_or_default();
        // Shorthands (`:`, `@`, `#`) are built-in directives.
        if let Some(name) = name.strip_prefix("v-") {
            self.current = Some(Binding {
                name,
                modifiers: Vec::new(),
                start,
                end,
            });
        }
    }
    fn on_dir_arg(&mut self, _start: usize, end: usize) {
        if let Some(current) = self.current.as_mut() {
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
    fn on_end(&mut self) {
        self.finish();
    }
    fn on_error(&mut self, _code: ErrorCode, _index: usize) {}
}

#[cfg(test)]
mod tests {
    use super::check;
    use crate::types::{BlockLocation, SfcTemplateBlock};
    use vize_l0::config::{DirectiveContract, DirectiveModifier};

    fn track() -> DirectiveContract {
        let modifier = |name: &str, optional, default| DirectiveModifier {
            name: name.into(),
            optional,
            default,
        };
        DirectiveContract {
            plugin: "@acme/tracking".into(),
            name: "track".into(),
            modifiers: vec![
                modifier("once", true, false),
                modifier("passive", true, true),
                modifier("strict", false, false),
            ],
        }
    }

    fn template(content: &str) -> SfcTemplateBlock<'_> {
        SfcTemplateBlock {
            content: content.into(),
            loc: BlockLocation {
                start: 10,
                end: 10 + content.len(),
                tag_start: 0,
                tag_end: 21 + content.len(),
                start_line: 1,
                start_column: 11,
                end_line: 1,
                end_column: 11,
            },
            lang: None,
            src: None,
            attrs: Default::default(),
        }
    }

    #[test]
    fn a_binding_that_breaks_its_contract_refuses_the_template() {
        let content = "\n  <a v-track.strict.once=\"e\" v-track:click.bogus.strict v-track v-focus.x>go</a>\n";
        let error = check(&template(content), &[track()]).unwrap_err();
        assert_eq!(
            error.message,
            "v-track does not declare the modifier `bogus` (declared by @acme/tracking)\nv-track requires the modifier `strict` (declared by @acme/tracking)"
        );
        assert_eq!(error.code.as_deref(), Some("PLUGIN_DIRECTIVE_CONTRACT"));
        let loc = error.loc.unwrap();
        let start = content.find("v-track:click").unwrap();
        assert_eq!(
            (loc.start, loc.end, loc.start_line, loc.start_column),
            (
                10 + start,
                10 + start + "v-track:click.bogus.strict".len(),
                2,
                start
            )
        );
    }

    #[test]
    fn valid_bindings_and_plans_without_directives_pass() {
        let content = "<a v-track.strict v-track.passive.strict=\"e\" v-focus.x>go</a>";
        check(&template(content), &[track()]).unwrap();
        check(&template("<a v-track.bogus>go</a>"), &[]).unwrap();
    }
}
