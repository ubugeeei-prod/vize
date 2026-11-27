//! Key spellings in a script block: `useTemplateRef('box')` calls and
//! `this.$refs.box` / `this.$refs['box']` reads.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, CallExpression, ComputedMemberExpression, Expression, ImportDeclarationSpecifier,
    Program, Statement, StaticMemberExpression,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::{Site, Spellings, every_refs_is_known, verbatim_key};

/// Modules whose `useTemplateRef` export is Vue's.
const VUE_MODULES: [&str; 4] = ["vue", "@vue/runtime-core", "@vue/runtime-dom", "#imports"];

pub(super) fn collect(content: &str, lang: Option<&str>, base: usize, spellings: &mut Spellings) {
    let source_type = match lang {
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        Some("js") => SourceType::mjs(),
        _ => SourceType::ts(),
    };
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, content, source_type).parse();
    // A recovered parse may have dropped a spelling.
    if !parsed.diagnostics.is_empty() {
        spellings.complete = false;
    }
    if parsed.panicked {
        return;
    }
    let mut collector = Collector {
        content,
        base,
        callees: callee_names(&parsed.program),
        spellings,
        refs: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    let complete = every_refs_is_known(content, &collector.refs);
    spellings.complete &= complete;
}

/// Local names that call Vue's `useTemplateRef`. An unimported
/// `useTemplateRef` is an auto-import and counts too.
fn callee_names<'a>(program: &Program<'a>) -> Vec<&'a str> {
    let mut names = Vec::new();
    let mut bare_is_foreign = false;
    for statement in &program.body {
        let Statement::ImportDeclaration(import) = statement else {
            continue;
        };
        let from_vue = VUE_MODULES.contains(&import.source.value.as_str());
        for specifier in import.specifiers.iter().flatten() {
            let local = specifier.local().name.as_str();
            let is_use_template_ref = matches!(
                specifier,
                ImportDeclarationSpecifier::ImportSpecifier(named)
                    if named.imported.name().as_str() == "useTemplateRef"
            );
            if from_vue && is_use_template_ref {
                names.push(local);
            } else if local == "useTemplateRef" {
                bare_is_foreign = true;
            }
        }
    }
    if !bare_is_foreign && !names.contains(&"useTemplateRef") {
        names.push("useTemplateRef");
    }
    names
}

struct Collector<'s, 'a> {
    content: &'s str,
    base: usize,
    callees: Vec<&'a str>,
    spellings: &'s mut Spellings,
    /// Offsets of the `$refs` names that start a known spelling.
    refs: Vec<usize>,
}

impl Collector<'_, '_> {
    fn push_literal(&mut self, start: u32, end: u32, value: &str, site: Site) {
        match verbatim_key(self.content, start as usize, end as usize, value) {
            Some((start, end)) => {
                self.spellings
                    .push(value, self.base + start, self.base + end, site);
            }
            // An escaped key has no single span to rewrite.
            None => self.spellings.complete = false,
        }
    }
}

/// The offset of `$refs` when `object` is `this.$refs`.
fn this_refs(object: &Expression<'_>) -> Option<u32> {
    let Expression::StaticMemberExpression(member) = object else {
        return None;
    };
    (matches!(member.object, Expression::ThisExpression(_)) && member.property.name == "$refs")
        .then_some(member.property.span.start)
}

impl<'a> Visit<'a> for Collector<'_, 'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if let Expression::Identifier(callee) = &call.callee
            && self.callees.contains(&callee.name.as_str())
        {
            match call.arguments.first() {
                Some(Argument::StringLiteral(literal)) => self.push_literal(
                    literal.span.start,
                    literal.span.end,
                    literal.value.as_str(),
                    Site::Call,
                ),
                _ => self.spellings.complete = false,
            }
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_static_member_expression(&mut self, member: &StaticMemberExpression<'a>) {
        if let Some(refs) = this_refs(&member.object) {
            self.refs.push(refs as usize);
            let span = member.property.span;
            self.spellings.push(
                member.property.name.as_str(),
                self.base + span.start as usize,
                self.base + span.end as usize,
                Site::Member,
            );
            return;
        }
        walk::walk_static_member_expression(self, member);
    }

    fn visit_computed_member_expression(&mut self, member: &ComputedMemberExpression<'a>) {
        if let Some(refs) = this_refs(&member.object)
            && let Expression::StringLiteral(literal) = &member.expression
        {
            self.refs.push(refs as usize);
            self.push_literal(
                literal.span.start,
                literal.span.end,
                literal.value.as_str(),
                Site::Index,
            );
            return;
        }
        walk::walk_computed_member_expression(self, member);
    }
}
