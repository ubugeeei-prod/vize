//! Positive component values from the existing clean, original script parses.
//!
//! This is not receiver inference and never authorizes negative conclusions.
//! A private constructor prevents a caller-authored name list from becoming
//! component ownership. Ordinary module locals and factory names prove nothing.

use oxc_ast::ast::{
    ExportDefaultDeclarationKind, ImportDeclarationSpecifier, ImportOrExportKind, Statement,
    VariableDeclarationKind,
};
use oxc_parser::ParserReturn;
use vize_atelier_sfc::{SfcDescriptor, SfcScriptBlock};
use vize_croquis::{OptionGroup, script_parser::collect_options_descriptor};
use vize_l0::{CompactString, Span};

const MAX_DECLARATIONS: usize = 4096;

#[derive(Debug)]
struct AuthoredValue {
    name: CompactString,
    declaration: Span,
}

/// Positive, source-authenticated component declarations only. An empty packet
/// does not establish absence of a binding or turn unknown syntax into a global.
#[derive(Debug, Default)]
pub struct TemplateComponentBindings {
    values: Vec<AuthoredValue>,
    unknown: [bool; 2],
}

impl TemplateComponentBindings {
    /// Refused inspection is not evidence that component lookup is next.
    pub fn can_report_component_reference(&self, name: &str) -> bool {
        let index = match name {
            "$listeners" => 0,
            "$scopedSlots" => 1,
            _ => return false,
        };
        !self.unknown[index] && !self.has_authored_value(name)
    }

    fn refuse(&mut self) {
        self.unknown = [true; 2];
    }

    pub fn has_authored_value(&self, name: &str) -> bool {
        self.values
            .iter()
            .any(|value| value.name == name && value.declaration.start < value.declaration.end)
    }

    pub(crate) fn from_parsed_sfc(
        descriptor: &SfcDescriptor<'_>,
        setup: Option<&ParserReturn<'_>>,
        script: Option<&ParserReturn<'_>>,
    ) -> Self {
        let mut result = Self::default();
        for (block, parsed) in [
            (descriptor.script_setup.as_ref(), setup),
            (descriptor.script.as_ref(), script),
        ] {
            if block.is_some() && clean_block(block, parsed).is_none() {
                result.refuse();
            }
        }
        if let Some((block, parsed)) = clean_block(descriptor.script_setup.as_ref(), setup) {
            for statement in &parsed.program.body {
                match statement {
                    Statement::VariableDeclaration(declaration)
                        if matches!(
                            declaration.kind,
                            VariableDeclarationKind::Const
                                | VariableDeclarationKind::Let
                                | VariableDeclarationKind::Var
                        ) && !declaration.declare =>
                    {
                        if declaration.declarations.len() > MAX_DECLARATIONS {
                            result.refuse();
                            continue;
                        }
                        for declarator in &declaration.declarations {
                            if let Some(bindings) =
                                vize_croquis::drawer::retained_binding_declarations(&declarator.id)
                            {
                                for binding in bindings {
                                    result.record(block, binding.name.as_str(), binding.span);
                                }
                            } else {
                                result.refuse();
                            }
                        }
                    }
                    Statement::FunctionDeclaration(function)
                        if !function.declare && function.body.is_some() =>
                    {
                        if let Some(binding) = &function.id {
                            result.record(block, binding.name.as_str(), binding.span);
                        }
                    }
                    Statement::ClassDeclaration(class) if !class.declare => {
                        if let Some(binding) = &class.id {
                            result.record(block, binding.name.as_str(), binding.span);
                        }
                    }
                    Statement::ImportDeclaration(declaration)
                        if declaration.import_kind == ImportOrExportKind::Value =>
                    {
                        if declaration
                            .specifiers
                            .as_ref()
                            .is_some_and(|items| items.len() > MAX_DECLARATIONS)
                        {
                            result.refuse();
                            continue;
                        }
                        for specifier in declaration.specifiers.iter().flatten() {
                            let binding = match specifier {
                                ImportDeclarationSpecifier::ImportSpecifier(specifier)
                                    if specifier.import_kind == ImportOrExportKind::Value =>
                                {
                                    &specifier.local
                                }
                                ImportDeclarationSpecifier::ImportDefaultSpecifier(specifier) => {
                                    &specifier.local
                                }
                                ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                                    &specifier.local
                                }
                                _ => continue,
                            };
                            result.record(block, binding.name.as_str(), binding.span);
                        }
                    }
                    _ => {}
                }
            }
        }
        // A script-setup compile replaces the normal setup function. Its
        // presence cannot authenticate an unexecuted sibling's return keys.
        if descriptor.script_setup.is_none()
            && let Some((block, parsed)) = clean_block(descriptor.script.as_ref(), script)
            && parsed.program.body.iter().any(|statement| {
                matches!(statement, Statement::ExportDefaultDeclaration(export)
                    if matches!(export.declaration, ExportDefaultDeclarationKind::ObjectExpression(_)))
            })
            && let Some(options) = collect_options_descriptor(&parsed.program)
        {
            for member in options.members {
                if member.group == OptionGroup::Setup {
                    result.record(
                        block,
                        member.name.as_str(),
                        oxc_span::Span::new(member.start, member.end),
                    );
                }
            }
        }
        result
    }

    fn record(&mut self, block: &SfcScriptBlock<'_>, name: &str, span: oxc_span::Span) {
        if !matches!(name, "$listeners" | "$scopedSlots")
            || span.start >= span.end
            || block
                .content
                .get(span.start as usize..span.end as usize)
                .is_none()
        {
            if name == "$listeners" {
                self.unknown[0] = true;
            }
            if name == "$scopedSlots" {
                self.unknown[1] = true;
            }
            return;
        }
        self.values.push(AuthoredValue {
            name: CompactString::new(name),
            declaration: Span::new(
                block.loc.start as u32 + span.start,
                block.loc.start as u32 + span.end,
            ),
        });
    }
}

fn clean_block<'source, 'borrow, 'ast>(
    block: Option<&'borrow SfcScriptBlock<'source>>,
    parsed: Option<&'borrow ParserReturn<'ast>>,
) -> Option<(
    &'borrow SfcScriptBlock<'source>,
    &'borrow ParserReturn<'ast>,
)> {
    let (block, parsed) = (block?, parsed?);
    (block.src.is_none()
        && block
            .lang
            .as_deref()
            .is_none_or(|lang| matches!(lang, "js" | "ts" | "jsx" | "tsx"))
        && !parsed.panicked
        && parsed.diagnostics.is_empty()
        && parsed.program.body.len() <= MAX_DECLARATIONS
        && core::ptr::eq(parsed.program.source_text, block.content.as_ref()))
    .then_some((block, parsed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_allocator::Allocator;
    use oxc_parser::Parser;
    use oxc_span::SourceType;
    use serde_json::Value;

    #[test]
    fn appended_original_program_controls_prove_only_real_setup_values() {
        for (json, count) in [
            (
                include_str!(
                    "../../../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-component-bindings.json"
                ),
                30,
            ),
            (
                include_str!(
                    "../../../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-component-declaration-bindings.json"
                ),
                40,
            ),
        ] {
            let corpus: Value = serde_json::from_str(json).unwrap();
            let cases = corpus["records"].as_array().unwrap();
            assert_eq!(cases.len(), count);
            for case in cases {
                let source = case["source"].as_str().unwrap();
                let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
                let setup_arena = Allocator::default();
                let script_arena = Allocator::default();
                let setup = descriptor.script_setup.as_ref().map(|block| {
                    Parser::new(&setup_arena, &block.content, SourceType::ts()).parse()
                });
                let script = descriptor.script.as_ref().map(|block| {
                    Parser::new(&script_arena, &block.content, SourceType::ts()).parse()
                });
                let bindings = TemplateComponentBindings::from_parsed_sfc(
                    &descriptor,
                    setup.as_ref(),
                    script.as_ref(),
                );
                assert_eq!(
                    bindings.has_authored_value(case["surface"].as_str().unwrap()),
                    matches!(
                        case["ownerExpectation"].as_str().unwrap(),
                        "setup-owned"
                            | "setup-owned-template-script-member-unchanged"
                            | "bare-setup-owned-template-this-unqualified"
                    ),
                    "{}: {bindings:?}",
                    case["id"],
                );
            }
        }
    }

    #[test]
    fn byte_equal_foreign_program_does_not_prove_component_binding() {
        let source =
            "<script setup>const $listeners = {}</script><template>{{ $listeners }}</template>";
        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
        let copied = descriptor
            .script_setup
            .as_ref()
            .unwrap()
            .content
            .to_string();
        let arena = Allocator::default();
        let parsed = Parser::new(&arena, &copied, SourceType::ts()).parse();
        let bindings = TemplateComponentBindings::from_parsed_sfc(&descriptor, Some(&parsed), None);
        assert!(!bindings.has_authored_value("$listeners"));
        assert!(!bindings.can_report_component_reference("$listeners"));
    }

    #[test]
    fn unavailable_script_authority_is_not_absence() {
        for source in [
            "<script setup>const $listeners = {}</script><template>{{ $listeners }}</template>",
            "<script setup src=\"./setup.ts\"/><template>{{ $listeners }}</template>",
            "<script lang=\"custom\">unknown</script><template>{{ $listeners }}</template>",
        ] {
            let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
            let bindings = TemplateComponentBindings::from_parsed_sfc(&descriptor, None, None);
            assert!(
                !bindings.can_report_component_reference("$listeners"),
                "{source}"
            );
            assert!(
                !bindings.can_report_component_reference("$scopedSlots"),
                "{source}"
            );
        }
        assert!(TemplateComponentBindings::default().can_report_component_reference("$listeners"));
    }
    #[test]
    fn parse_failure_does_not_authorize_a_template_warning() {
        let source = "<script setup>const $listeners = {}; const broken = </script><template>{{ $listeners }}</template>";
        let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
        let arena = Allocator::default();
        let block = descriptor.script_setup.as_ref().unwrap();
        let parsed = Parser::new(&arena, &block.content, SourceType::ts()).parse();
        assert!(parsed.panicked || !parsed.diagnostics.is_empty());
        let bindings = TemplateComponentBindings::from_parsed_sfc(&descriptor, Some(&parsed), None);
        assert!(!bindings.can_report_component_reference("$listeners"));
    }
    #[test]
    fn over_budget_setup_declarations_retain_unknown_authority() {
        let overlay: Value = serde_json::from_str(include_str!(
            "../../../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-authority-review-corrections.json"
        )).unwrap();
        let guards = overlay["guardRecords"].as_array().unwrap();
        assert_eq!(guards.len(), 4);
        for case in guards {
            let descriptor =
                vize_atelier_sfc::parse_sfc(case["source"].as_str().unwrap(), Default::default())
                    .unwrap();
            let arena = Allocator::default();
            let block = descriptor.script_setup.as_ref().unwrap();
            let parsed = Parser::new(&arena, &block.content, SourceType::ts()).parse();
            assert!(
                !parsed.panicked && parsed.diagnostics.is_empty(),
                "{}",
                case["id"]
            );
            let bindings =
                TemplateComponentBindings::from_parsed_sfc(&descriptor, Some(&parsed), None);
            assert!(
                !bindings.can_report_component_reference("$listeners"),
                "{}",
                case["id"]
            );
        }
    }
}
