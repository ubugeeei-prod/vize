//! Scope walking functions for tracking nested JavaScript scopes.
//!
//! These functions recursively walk the AST to discover:
//! - Arrow functions and function expressions (closure scopes)
//! - Block statements (if, for, while, try/catch, etc.)
//! - Client-only lifecycle hooks (onMounted, etc.)
//! - Reactivity losses (destructuring, spreading, reassignment)
//!
//! This module is split into:
//! - `statements`: Walking statement nodes (blocks, loops, declarations)
//! - `expressions`: Walking expression nodes (functions, calls, operators)

mod expressions;
mod statements;

use oxc_ast::ast::{BindingPattern, Expression};
use oxc_span::GetSpan;

use crate::ScopeBinding;
use crate::scope::{BlockKind, BlockScopeData, ClientOnlyScopeData, ClosureScopeData};
use vize_carton::CompactString;
use vize_relief::BindingType;

use super::ScriptParseResult;
use super::extract::{
    detect_call_argument_reactivity_loss, detect_provide_inject_call, detect_race_condition_call,
};

pub(in crate::script_parser) use expressions::{walk_call_arguments, walk_expression};
pub(in crate::script_parser) use statements::walk_statement;

/// Check if a function name is a client-only lifecycle hook
#[inline]
pub(in crate::script_parser) fn is_client_only_hook(name: &str) -> bool {
    matches!(
        name,
        "onMounted"
            | "onBeforeMount"
            | "onUnmounted"
            | "onBeforeUnmount"
            | "onUpdated"
            | "onBeforeUpdate"
            | "onActivated"
            | "onDeactivated"
    )
}

/// Add variable bindings from a binding pattern to the current scope
#[inline]
pub(in crate::script_parser) fn add_binding_pattern_to_scope(
    result: &mut ScriptParseResult,
    pattern: &oxc_ast::ast::BindingPattern<'_>,
    offset: u32,
) {
    let mut names = vize_carton::SmallVec::<[CompactString; 4]>::new();
    if let Some(capture) = result.occurrence_capture.as_mut() {
        let scope = result.scopes.current_id();
        let has_default = collect_param_sites(pattern, &mut |name, span| {
            names.push(CompactString::new(name));
            capture.declaration(scope, name, span);
        });
        if has_default {
            capture.refuse();
        }
    } else {
        extract_param_names(pattern, &mut names);
    }
    for name in names {
        result
            .scopes
            .add_binding(name, ScopeBinding::new(BindingType::SetupConst, offset));
    }
}

/// Extract parameter names from function params
#[inline]
pub(in crate::script_parser) fn extract_function_params(
    params: &oxc_ast::ast::FormalParameters<'_>,
) -> vize_carton::SmallVec<[CompactString; 4]> {
    let mut names = vize_carton::SmallVec::new();

    for param in params.items.iter() {
        extract_param_names(&param.pattern, &mut names);
    }

    if let Some(rest) = &params.rest {
        extract_param_names(&rest.rest.argument, &mut names);
    }

    names
}

/// Extract parameter names from a binding pattern
#[inline]
pub(in crate::script_parser) fn extract_param_names(
    pattern: &oxc_ast::ast::BindingPattern<'_>,
    names: &mut vize_carton::SmallVec<[CompactString; 4]>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            names.push(CompactString::new(id.name.as_str()));
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in obj.properties.iter() {
                extract_param_names(&prop.value, names);
            }
            if let Some(rest) = &obj.rest {
                extract_param_names(&rest.argument, names);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for elem in arr.elements.iter().flatten() {
                extract_param_names(elem, names);
            }
            if let Some(rest) = &arr.rest {
                extract_param_names(&rest.argument, names);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            extract_param_names(&assign.left, names);
        }
    }
}

/// Replace the existing parameter-name walk only when authored sites are demanded.
#[inline]
pub(in crate::script_parser) fn extract_function_params_with_occurrences(
    result: &mut ScriptParseResult,
    params: &oxc_ast::ast::FormalParameters<'_>,
) -> vize_carton::SmallVec<[CompactString; 4]> {
    let Some(capture) = result.occurrence_capture.as_mut() else {
        return extract_function_params(params);
    };
    let mut names = vize_carton::SmallVec::new();
    let mut type_read = false;
    let has_default = {
        let mut note = |name: &str, span| {
            names.push(CompactString::new(name));
            capture.parameters.push((CompactString::new(name), span));
        };
        let mut has_default = false;
        for param in &params.items {
            type_read |= param.type_annotation.as_ref().is_some_and(|annotation| {
                super::typeof_refs::has_value_type_reads(&annotation.type_annotation)
            });
            has_default |= collect_param_sites(&param.pattern, &mut note);
            has_default |= param.initializer.is_some();
        }
        if let Some(rest) = &params.rest {
            type_read |= rest.type_annotation.as_ref().is_some_and(|annotation| {
                super::typeof_refs::has_value_type_reads(&annotation.type_annotation)
            });
            has_default |= collect_param_sites(&rest.rest.argument, &mut note);
        }
        has_default
    };
    if has_default || type_read {
        capture.refuse();
    }
    names
}

pub(in crate::script_parser) fn collect_param_sites(
    pattern: &BindingPattern<'_>,
    note: &mut impl FnMut(&str, oxc_span::Span),
) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            note(id.name.as_str(), id.span);
            false
        }
        BindingPattern::ObjectPattern(object) => {
            let mut has_default = false;
            for property in &object.properties {
                has_default |= property.computed;
                has_default |= collect_param_sites(&property.value, note);
            }
            if let Some(rest) = &object.rest {
                has_default |= collect_param_sites(&rest.argument, note);
            }
            has_default
        }
        BindingPattern::ArrayPattern(array) => {
            let mut has_default = false;
            for item in array.elements.iter().flatten() {
                has_default |= collect_param_sites(item, note);
            }
            if let Some(rest) = &array.rest {
                has_default |= collect_param_sites(&rest.argument, note);
            }
            has_default
        }
        BindingPattern::AssignmentPattern(assignment) => {
            collect_param_sites(&assignment.left, note);
            true
        }
    }
}
