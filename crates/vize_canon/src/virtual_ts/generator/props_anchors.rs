//! Setup-scope anchors for props bindings shadowed by template generation.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, BindingPattern, CallExpression, Expression, VariableDeclarator};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, FxHashSet, String, append, profile};
use vize_croquis::{
    Croquis,
    macros::{DEFINE_PROPS, WITH_DEFAULTS},
};

use crate::virtual_ts::props::{strip_outer_angle_brackets, type_reference_lookup_key};

pub(super) fn emit_setup_scope_prop_anchors(
    ts: &mut String,
    summary: &Croquis,
    script_content: Option<&str>,
    template_referenced_names: Option<&FxHashSet<String>>,
    preserve_unused_diagnostics: bool,
) {
    if preserve_unused_diagnostics {
        let bindings = profile!(
            "canon.virtual_ts.collect_define_props_result_anchors",
            collect_define_props_result_anchor_names(
                summary,
                script_content,
                template_referenced_names,
            )
        );
        if !bindings.is_empty() {
            ts.push_str("\n  // Reference defineProps result (prevent TS6133)\n  ");
            for (index, binding) in bindings.iter().enumerate() {
                if index > 0 {
                    ts.push(' ');
                }
                append!(*ts, "void {binding};");
            }
            ts.push('\n');
        }
    }

    if let Some(destructure) = summary.macros.props_destructure()
        && !destructure.bindings.is_empty()
    {
        ts.push_str("\n  // Reference destructured props (prevent TS6133)\n  ");
        let mut first = true;
        for binding in destructure.bindings.values() {
            if !first {
                ts.push(' ');
            }
            append!(*ts, "void {};", binding.local);
            first = false;
        }
        if let Some(ref rest) = destructure.rest_id {
            if !first {
                ts.push(' ');
            }
            append!(*ts, "void {};", rest);
        }
        ts.push('\n');
    }
}

fn collect_define_props_result_anchor_names<'a>(
    summary: &'a Croquis,
    script_content: Option<&str>,
    template_referenced_names: Option<&FxHashSet<String>>,
) -> Vec<&'a str> {
    if !template_referenced_names
        .is_some_and(|names| template_references_define_props_binding(summary, names))
    {
        return Vec::new();
    }
    let Some(script) = script_content else {
        return Vec::new();
    };

    let bindings = collect_define_props_result_bindings(script);
    let mut names = summary
        .bindings
        .bindings
        .keys()
        .map(|name| name.as_str())
        .filter(|name| bindings.iter().any(|candidate| candidate.as_str() == *name))
        .collect::<Vec<_>>();
    names.sort_unstable();
    names
}

fn template_references_define_props_binding(
    summary: &Croquis,
    template_referenced_names: &FxHashSet<String>,
) -> bool {
    if summary.macros.define_props().is_none() {
        return false;
    }

    if summary
        .macros
        .props()
        .iter()
        .any(|prop| template_referenced_names.contains(prop.name.as_str()))
    {
        return true;
    }

    if summary
        .macros
        .models()
        .iter()
        .any(|model| template_referenced_names.contains(model.name.as_str()))
    {
        return true;
    }

    let Some(type_args) = summary
        .macros
        .define_props()
        .and_then(|macro_call| macro_call.type_args.as_ref())
    else {
        return false;
    };
    let type_name = strip_outer_angle_brackets(type_args.trim());
    summary
        .types
        .extract_properties(type_reference_lookup_key(type_name))
        .iter()
        .any(|prop| template_referenced_names.contains(prop.name.as_str()))
}

fn collect_define_props_result_bindings(script: &str) -> FxHashSet<CompactString> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    let mut collector = DefinePropsResultBindingCollector::default();
    collector.visit_program(&parsed.program);
    collector.bindings
}

#[derive(Default)]
struct DefinePropsResultBindingCollector {
    bindings: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for DefinePropsResultBindingCollector {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(binding) = &declarator.id
            && declarator
                .init
                .as_ref()
                .is_some_and(is_define_props_result_expression)
        {
            self.bindings
                .insert(CompactString::new(binding.name.as_str()));
        }

        walk::walk_variable_declarator(self, declarator);
    }
}

fn is_define_props_result_expression(expr: &Expression<'_>) -> bool {
    let Expression::CallExpression(call) = expr else {
        return false;
    };
    is_define_props_call(call) || is_with_defaults_define_props_call(call)
}

fn is_define_props_call(call: &CallExpression<'_>) -> bool {
    matches!(
        &call.callee,
        Expression::Identifier(ident) if ident.name.as_str() == DEFINE_PROPS
    )
}

fn is_with_defaults_define_props_call(call: &CallExpression<'_>) -> bool {
    if !matches!(
        &call.callee,
        Expression::Identifier(ident) if ident.name.as_str() == WITH_DEFAULTS
    ) {
        return false;
    }

    matches!(
        call.arguments.first(),
        Some(Argument::CallExpression(inner)) if is_define_props_call(inner)
    )
}
