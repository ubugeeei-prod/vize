//! [`CompileOptions::erase_types`](super::CompileOptions::erase_types): the
//! compiled module's TypeScript erased by oxc's transformer with the recipe of
//! the repository's whole-script erasure for `.vue` (`vize build
//! --script-ext downcompile`): `only_remove_type_imports: false`, default
//! printing, tabs printed as two spaces. The output equals it byte for byte.
//!
//! The compiled module is parsed again because the emitted render code holds
//! authored expressions too; host code and markup expressions are erased in
//! one pass. Any diagnostic of the parse, the semantic pass or the transform
//! refuses the module instead of keeping TypeScript, and so does a tab inside
//! a printed literal, whose value the two-space print would change (#13151).

use oxc_ast::ast::{Directive, RegExpLiteral, StringLiteral, TemplateElement};
use oxc_ast_visit::Visit;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer, TypeScriptOptions};
use vize_l0::{Allocator, String, cstr};

/// `code` as JavaScript, or why it was refused.
pub(super) fn erase(code: &str) -> Result<String, String> {
    let allocator = Allocator::new();
    let parsed = Parser::new(allocator.as_oxc(), code, SourceType::ts()).parse();
    if let Some(diagnostic) = parsed.diagnostics.first() {
        return Err(refusal("parse", &diagnostic.message));
    }
    let mut program = parsed.program;
    // `with_enum_eval`: the enum transform panics without evaluated members.
    let semantic = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .with_enum_eval(true)
        .build(&program);
    if let Some(diagnostic) = semantic.diagnostics.first() {
        return Err(refusal("semantic", &diagnostic.message));
    }
    let options = TransformOptions {
        typescript: TypeScriptOptions {
            only_remove_type_imports: false,
            ..TypeScriptOptions::default()
        },
        ..TransformOptions::default()
    };
    let transformed = Transformer::new(allocator.as_oxc(), std::path::Path::new(""), &options)
        .build_with_scoping(semantic.semantic.into_scoping(), &mut program);
    if let Some(diagnostic) = transformed.diagnostics.first() {
        return Err(refusal("transform", &diagnostic.message));
    }
    let mut tabs = LiteralTabs(false);
    tabs.visit_program(&program);
    if tabs.0 {
        return Err(String::from(
            "type erasure of TSRX modules does not keep a tab character inside a literal yet",
        ));
    }
    Ok(Codegen::new()
        .build(&program)
        .code
        .replace('\t', "  ")
        .into())
}

fn refusal(pass: &str, message: &str) -> String {
    cstr!("type erasure of the compiled TSRX module failed ({pass}): {message}")
}

/// Whether a literal prints a tab character as is.
struct LiteralTabs(bool);

impl<'a> Visit<'a> for LiteralTabs {
    fn visit_string_literal(&mut self, it: &StringLiteral<'a>) {
        self.0 |= it.value.contains('\t');
    }

    fn visit_template_element(&mut self, it: &TemplateElement<'a>) {
        self.0 |= it.value.raw.contains('\t');
    }

    fn visit_reg_exp_literal(&mut self, it: &RegExpLiteral<'a>) {
        self.0 |= it.regex.pattern.text.contains('\t');
    }

    /// A directive prints its raw text, where `\t` stays an escape.
    fn visit_directive(&mut self, it: &Directive<'a>) {
        self.0 |= it.directive.contains('\t');
    }
}
