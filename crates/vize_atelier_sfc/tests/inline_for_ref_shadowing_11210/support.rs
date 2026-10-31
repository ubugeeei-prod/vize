//! Inspect resolved JavaScript symbols, rather than stripping `.value` text.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, AssignmentExpression, AssignmentTarget, BindingPattern, CallExpression, Expression,
    IdentifierReference, StaticMemberExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_semantic::{Scoping, SemanticBuilder};
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::symbol::SymbolId;
use serde_json::json;
use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_sfc::{
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, compile_sfc_for_adapter, parse_sfc,
};

#[derive(Debug, Default)]
pub struct Facts {
    pub aliases: Vec<String>,
    pub sources: Vec<String>,
    pub alias_references: usize,
    pub incorrect_alias_unwrapping: Vec<String>,
    pub setup_value_references: Vec<String>,
    pub assignment_targets: Vec<String>,
    assignment_members: Vec<(String, String)>,
}

impl Facts {
    pub fn assignment_members(&self) -> Vec<(&str, &str)> {
        self.assignment_members
            .iter()
            .map(|(object, property)| (object.as_str(), property.as_str()))
            .collect()
    }
}

#[derive(Default)]
struct Loops<'s> {
    code: &'s str,
    symbols: Vec<SymbolId>,
    names: Vec<String>,
    sources: Vec<String>,
}

impl Loops<'_> {
    fn binding(&mut self, pattern: &BindingPattern<'_>) {
        match pattern {
            BindingPattern::BindingIdentifier(id) => {
                self.symbols
                    .push(id.symbol_id.get().expect("resolved callback binding"));
                self.names.push(id.name.to_string());
            }
            BindingPattern::ArrayPattern(array) => {
                for item in array.elements.iter().flatten() {
                    self.binding(item);
                }
                if let Some(rest) = &array.rest {
                    self.binding(&rest.argument);
                }
            }
            BindingPattern::ObjectPattern(object) => {
                for property in &object.properties {
                    self.binding(&property.value);
                }
                if let Some(rest) = &object.rest {
                    self.binding(&rest.argument);
                }
            }
            BindingPattern::AssignmentPattern(assignment) => self.binding(&assignment.left),
        }
    }
}

impl<'a> Visit<'a> for Loops<'_> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && matches!(callee.name.as_str(), "_renderList" | "_ssrRenderList")
        {
            let [collection, callback, ..] = call.arguments.as_slice() else {
                panic!("renderList must retain its collection and callback");
            };
            self.sources
                .push(collection.span().source_text(self.code).to_string());
            let parameters = match callback {
                Argument::ArrowFunctionExpression(arrow) => &arrow.params,
                Argument::FunctionExpression(function) => &function.params,
                _ => panic!("renderList must retain an actual callback"),
            };
            for parameter in &parameters.items {
                self.binding(&parameter.pattern);
            }
        }
        walk::walk_call_expression(self, call);
    }
}

struct Uses<'s> {
    code: &'s str,
    scoping: &'s Scoping,
    aliases: &'s [SymbolId],
    facts: Facts,
}

impl Uses<'_> {
    fn is_alias(&self, identifier: &IdentifierReference<'_>) -> bool {
        self.scoping
            .get_reference(identifier.reference_id.get().expect("resolved identifier"))
            .symbol_id()
            .is_some_and(|symbol| self.aliases.contains(&symbol))
    }
}

impl<'a> Visit<'a> for Uses<'_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        if self.is_alias(identifier) {
            self.facts.alias_references += 1;
        }
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if member.property.name == "value"
            && let Expression::Identifier(object) = member.object.without_parentheses()
        {
            if self.is_alias(object) {
                self.facts
                    .incorrect_alias_unwrapping
                    .push(member.span.source_text(self.code).to_string());
            } else {
                self.facts
                    .setup_value_references
                    .push(object.name.to_string());
            }
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && callee.name == "_unref"
            && let Some(argument) = call.arguments.first().and_then(|arg| arg.as_expression())
            && let Expression::Identifier(argument) = argument.without_parentheses()
            && self.is_alias(argument)
        {
            self.facts
                .incorrect_alias_unwrapping
                .push(call.span.source_text(self.code).to_string());
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_assignment_expression(&mut self, assignment: &AssignmentExpression<'a>) {
        if let AssignmentTarget::StaticMemberExpression(member) = &assignment.left {
            self.facts
                .assignment_targets
                .push(member.span.source_text(self.code).to_string());
            if let Expression::Identifier(object) = member.object.without_parentheses() {
                self.facts
                    .assignment_members
                    .push((object.name.to_string(), member.property.name.to_string()));
            }
        }
        walk::walk_assignment_expression(self, assignment);
    }
}

pub fn compile(id: &str, source: &str, inline: bool, ssr: bool) -> Facts {
    let parse = SfcParseOptions {
        filename: "Repro.vue".into(),
        ..Default::default()
    };
    let descriptor = parse_sfc(source, parse.clone()).expect("authored SFC parses");
    let mut options = SfcCompileOptions {
        parse,
        ..Default::default()
    };
    // Public `inlineTemplate` selects the script output below; the Rust flag
    // instead requests standalone `mode: "function"` output.
    options.script.id = Some("Repro.vue".into());
    options.template.id = Some("Repro.vue".into());
    options.template.is_prod = true;
    options.template.ssr = ssr;
    let result = compile_sfc_for_adapter(
        &descriptor,
        options,
        TemplateSyntaxMode::Standard,
        Default::default(),
        Default::default(),
        if inline {
            SfcScriptOutputMode::InlineTemplate
        } else {
            SfcScriptOutputMode::SeparateTemplate
        },
    )
    .expect("complete production SFC result");
    // Retain complete source/result before assertions, including failed observations.
    println!(
        "inline-ref-shadow-result {}",
        json!({"id":id,"source":source,"inline":inline,"ssr":ssr,
            "filename":"Repro.vue","mode":"module","isProd":true,"result":result})
    );
    assert!(result.errors.is_empty(), "{id}: {:?}", result.errors);
    assert!(result.warnings.is_empty(), "{id}: {:?}", result.warnings);
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, result.code.as_str(), SourceType::mjs()).parse();
    assert!(
        !parsed.panicked && parsed.diagnostics.is_empty(),
        "{id}: {:?}\n{}",
        parsed.diagnostics,
        result.code
    );
    let semantic = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .build(&parsed.program);
    assert!(
        semantic.diagnostics.is_empty(),
        "{id}: {:?}\n{}",
        semantic.diagnostics,
        result.code
    );
    let mut loops = Loops {
        code: result.code.as_str(),
        ..Default::default()
    };
    loops.visit_program(&parsed.program);
    let mut uses = Uses {
        code: result.code.as_str(),
        scoping: semantic.semantic.scoping(),
        aliases: &loops.symbols,
        facts: Facts {
            aliases: loops.names,
            sources: loops.sources,
            ..Default::default()
        },
    };
    uses.visit_program(&parsed.program);
    println!(
        "inline-ref-shadow-facts {}",
        json!({"id":id,"aliases":uses.facts.aliases,"collections":uses.facts.sources,
            "aliasReferences":uses.facts.alias_references,
            "incorrectAliasUnwrapping":uses.facts.incorrect_alias_unwrapping,
            "setupValueReferences":uses.facts.setup_value_references,
            "wholeAssignmentTargets":uses.facts.assignment_targets,
            "assignmentMembers":uses.facts.assignment_members})
    );
    assert!(
        uses.facts.incorrect_alias_unwrapping.is_empty(),
        "{id}: {:?}\n{}",
        uses.facts,
        result.code
    );
    uses.facts
}
