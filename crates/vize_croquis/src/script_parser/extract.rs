//! Extraction functions for props, emits, and reactivity detection.

use oxc_ast::ast::{
    Argument, CallExpression, Declaration, Expression, ObjectPropertyKind, PropertyKey, TSType,
    VariableDeclarationKind,
};
use oxc_span::{GetSpan, Span};

use crate::analysis::{InvalidExport, InvalidExportKind, TypeExport, TypeExportKind};
use crate::macros::{EmitDefinition, MacroKind, ModelDefinition, PropDefinition};
use crate::provide::ProvideKey;
use crate::reactivity::ReactiveKind;
use crate::setup_context::SetupContextViolationKind;
use vize_carton::{CompactString, FxHashMap, String};
use vize_relief::BindingType;

use super::ScriptParseResult;

/// Extract a CallExpression from an expression, unwrapping type assertions (as/satisfies)
pub fn extract_call_expression<'a>(expr: &'a Expression<'a>) -> Option<&'a CallExpression<'a>> {
    match expr {
        Expression::CallExpression(call) => Some(call),
        Expression::TSAsExpression(ts_as) => extract_call_expression(&ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_call_expression(&ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_call_expression(&ts_non_null.expression)
        }
        Expression::ParenthesizedExpression(paren) => extract_call_expression(&paren.expression),
        _ => None,
    }
}

/// Process a call expression, returns the MacroKind if it was a macro call
pub fn process_call_expression(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) -> Option<MacroKind> {
    let callee_name = match &call.callee {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return None,
    };

    let macro_kind = MacroKind::from_name(callee_name)?;

    let span = call.span;

    // Extract type arguments if present
    let type_args = call.type_arguments.as_ref().map(|tp| {
        let type_source = &source[tp.span.start as usize..tp.span.end as usize];
        CompactString::new(type_source)
    });

    // Extract runtime arguments
    let runtime_args = if !call.arguments.is_empty() {
        let args_start = call.arguments.first().map(|a| match a {
            Argument::SpreadElement(s) => s.span.start,
            Argument::Identifier(id) => id.span.start,
            _ => span.start,
        });
        let args_end = call.arguments.last().map(|a| match a {
            Argument::SpreadElement(s) => s.span.end,
            Argument::Identifier(id) => id.span.end,
            _ => span.end,
        });
        if let (Some(start), Some(end)) = (args_start, args_end) {
            Some(CompactString::new(&source[start as usize..end as usize]))
        } else {
            None
        }
    } else {
        None
    };

    // Add macro call
    result.macros.add_call(
        callee_name,
        macro_kind,
        span.start,
        span.end,
        runtime_args,
        type_args.clone(),
    );

    // Process macro-specific content
    match macro_kind {
        MacroKind::DefineProps => {
            // Extract props from type or runtime arguments
            if let Some(ref type_params) = call.type_arguments {
                extract_props_from_type(result, &type_params.params, source);
            } else if let Some(first_arg) = call.arguments.first() {
                extract_props_from_runtime(result, first_arg, source);
            }
        }

        MacroKind::DefineEmits => {
            // Extract emits from type or runtime arguments
            if let Some(ref type_params) = call.type_arguments {
                extract_emits_from_type(result, &type_params.params, source);
            } else if let Some(first_arg) = call.arguments.first() {
                extract_emits_from_runtime(result, first_arg, source);
            }
        }

        MacroKind::DefineModel => {
            // Extract model name (first string argument or 'modelValue' by default)
            let model_name = call
                .arguments
                .first()
                .and_then(|arg| {
                    if let Argument::StringLiteral(s) = arg {
                        Some(s.value.as_str())
                    } else {
                        None
                    }
                })
                .unwrap_or("modelValue");

            result.macros.add_model(ModelDefinition {
                name: CompactString::new(model_name),
                local_name: CompactString::new(model_name),
                model_type: None,
                required: false,
                default_value: None,
            });
        }

        MacroKind::WithDefaults => {
            // withDefaults wraps defineProps - find the inner call
            if let Some(Argument::CallExpression(inner_call)) = call.arguments.first() {
                process_call_expression(result, inner_call, source);
            }
        }

        _ => {}
    }

    Some(macro_kind)
}

/// Extract props from TypeScript type parameters
pub fn extract_props_from_type(
    result: &mut ScriptParseResult,
    type_params: &oxc_allocator::Vec<'_, TSType<'_>>,
    _source: &str,
) {
    for tp in type_params.iter() {
        if let TSType::TSTypeLiteral(lit) = tp {
            for member in lit.members.iter() {
                if let oxc_ast::ast::TSSignature::TSPropertySignature(prop) = member {
                    if let PropertyKey::StaticIdentifier(id) = &prop.key {
                        let name = id.name.as_str();
                        result.macros.add_prop(PropDefinition {
                            name: CompactString::new(name),
                            required: !prop.optional,
                            prop_type: None,
                            default_value: None,
                        });
                        result.bindings.add(name, BindingType::Props);
                    }
                }
            }
        }
    }
}

/// Extract props from runtime arguments (array or object)
pub fn extract_props_from_runtime(
    result: &mut ScriptParseResult,
    arg: &Argument<'_>,
    source: &str,
) {
    match arg {
        // Array syntax: ['prop1', 'prop2']
        Argument::ArrayExpression(arr) => {
            for elem in arr.elements.iter() {
                if let oxc_ast::ast::ArrayExpressionElement::StringLiteral(s) = elem {
                    let name = s.value.as_str();
                    result.macros.add_prop(PropDefinition {
                        name: CompactString::new(name),
                        required: false,
                        prop_type: None,
                        default_value: None,
                    });
                    result.bindings.add(name, BindingType::Props);
                }
            }
        }

        // Object syntax: { prop1: Type, prop2: { type: Type, required: true } }
        Argument::ObjectExpression(obj) => {
            for prop in obj.properties.iter() {
                if let ObjectPropertyKind::ObjectProperty(p) = prop {
                    let name = match &p.key {
                        PropertyKey::StaticIdentifier(id) => id.name.as_str(),
                        PropertyKey::StringLiteral(s) => s.value.as_str(),
                        _ => continue,
                    };
                    let required = detect_required_prop(&p.value);
                    let prop_type = extract_runtime_prop_type(&p.value, source);
                    let default_value = extract_runtime_prop_default(&p.value, source);
                    result.macros.add_prop(PropDefinition {
                        name: CompactString::new(name),
                        required,
                        prop_type,
                        default_value,
                    });
                    result.bindings.add(name, BindingType::Props);
                }
            }
        }

        _ => {}
    }
}

fn extract_runtime_prop_type(value: &Expression<'_>, source: &str) -> Option<CompactString> {
    match value {
        Expression::Identifier(id) => runtime_ctor_type(id.name.as_str()).map(CompactString::new),
        Expression::ArrayExpression(arr) => {
            let mut union = String::default();
            let mut has_type = false;
            for elem in arr.elements.iter() {
                let Some(prop_type) = extract_runtime_prop_type_from_array_element(elem, source)
                else {
                    continue;
                };
                if has_type {
                    union.push_str(" | ");
                }
                union.push_str(prop_type.as_str());
                has_type = true;
            }
            has_type.then(|| CompactString::new(union.as_str()))
        }
        Expression::ObjectExpression(obj) => obj.properties.iter().find_map(|prop| {
            let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                return None;
            };
            let PropertyKey::StaticIdentifier(id) = &prop.key else {
                return None;
            };
            (id.name.as_str() == "type")
                .then(|| extract_runtime_prop_type(&prop.value, source))
                .flatten()
        }),
        Expression::TSAsExpression(ts_as) => {
            extract_runtime_prop_type_from_annotation(source, ts_as.type_annotation.span())
                .or_else(|| extract_runtime_prop_type(&ts_as.expression, source))
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            extract_runtime_prop_type_from_annotation(source, ts_satisfies.type_annotation.span())
                .or_else(|| extract_runtime_prop_type(&ts_satisfies.expression, source))
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            extract_runtime_prop_type(&ts_non_null.expression, source)
        }
        Expression::ParenthesizedExpression(paren) => {
            extract_runtime_prop_type(&paren.expression, source)
        }
        _ => None,
    }
}

fn extract_runtime_prop_type_from_array_element(
    value: &oxc_ast::ast::ArrayExpressionElement<'_>,
    source: &str,
) -> Option<CompactString> {
    match value {
        oxc_ast::ast::ArrayExpressionElement::Identifier(id) => {
            runtime_ctor_type(id.name.as_str()).map(CompactString::new)
        }
        oxc_ast::ast::ArrayExpressionElement::StringLiteral(_) => {
            Some(CompactString::new("string"))
        }
        oxc_ast::ast::ArrayExpressionElement::NumericLiteral(_) => {
            Some(CompactString::new("number"))
        }
        oxc_ast::ast::ArrayExpressionElement::BooleanLiteral(_) => {
            Some(CompactString::new("boolean"))
        }
        oxc_ast::ast::ArrayExpressionElement::ObjectExpression(_) => {
            Some(CompactString::new("Record<string, unknown>"))
        }
        oxc_ast::ast::ArrayExpressionElement::ArrayExpression(_) => {
            Some(CompactString::new("unknown[]"))
        }
        oxc_ast::ast::ArrayExpressionElement::TSAsExpression(ts_as) => {
            extract_runtime_prop_type_from_annotation(source, ts_as.type_annotation.span())
                .or_else(|| extract_runtime_prop_type(&ts_as.expression, source))
        }
        oxc_ast::ast::ArrayExpressionElement::TSSatisfiesExpression(ts_satisfies) => {
            extract_runtime_prop_type_from_annotation(source, ts_satisfies.type_annotation.span())
                .or_else(|| extract_runtime_prop_type(&ts_satisfies.expression, source))
        }
        oxc_ast::ast::ArrayExpressionElement::TSNonNullExpression(ts_non_null) => {
            extract_runtime_prop_type(&ts_non_null.expression, source)
        }
        oxc_ast::ast::ArrayExpressionElement::ParenthesizedExpression(paren) => {
            extract_runtime_prop_type(&paren.expression, source)
        }
        _ => None,
    }
}

fn extract_runtime_prop_type_from_annotation(source: &str, span: Span) -> Option<CompactString> {
    let annotation = source.get(span.start as usize..span.end as usize)?.trim();
    extract_prop_type_generic(annotation, "PropType")
        .or_else(|| extract_prop_type_generic(annotation, "ReadonlyArray"))
}

fn extract_prop_type_generic(annotation: &str, type_name: &str) -> Option<CompactString> {
    let mut marker = String::default();
    marker.push_str(type_name);
    marker.push('<');
    let start = annotation.find(marker.as_str())? + marker.len();
    let mut depth = 1i32;

    for (idx, ch) in annotation[start..].char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    let inner = annotation[start..start + idx].trim();
                    return (!inner.is_empty()).then(|| CompactString::new(inner));
                }
            }
            _ => {}
        }
    }

    None
}

fn extract_runtime_prop_default(value: &Expression<'_>, source: &str) -> Option<CompactString> {
    let Expression::ObjectExpression(obj) = value else {
        return None;
    };

    obj.properties.iter().find_map(|prop| {
        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
            return None;
        };
        let PropertyKey::StaticIdentifier(id) = &prop.key else {
            return None;
        };
        if id.name.as_str() != "default" {
            return None;
        }

        source
            .get(prop.value.span().start as usize..prop.value.span().end as usize)
            .map(CompactString::new)
    })
}

fn runtime_ctor_type(name: &str) -> Option<&'static str> {
    match name {
        "String" => Some("string"),
        "Number" => Some("number"),
        "Boolean" => Some("boolean"),
        "Array" => Some("unknown[]"),
        "Object" => Some("Record<string, unknown>"),
        "Date" => Some("Date"),
        "Function" => Some("(...args: any[]) => any"),
        _ => None,
    }
}

/// Detect if a prop has required: true
fn detect_required_prop(value: &Expression<'_>) -> bool {
    if let Expression::ObjectExpression(obj) = value {
        for prop in obj.properties.iter() {
            if let ObjectPropertyKind::ObjectProperty(p) = prop {
                if let PropertyKey::StaticIdentifier(id) = &p.key {
                    if id.name.as_str() == "required" {
                        if let Expression::BooleanLiteral(b) = &p.value {
                            return b.value;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Extract emits from TypeScript type parameters
pub fn extract_emits_from_type(
    result: &mut ScriptParseResult,
    type_params: &oxc_allocator::Vec<'_, TSType<'_>>,
    _source: &str,
) {
    for tp in type_params.iter() {
        if let TSType::TSTypeLiteral(lit) = tp {
            // Handle call signatures like { (e: 'update', value: string): void }
            for member in lit.members.iter() {
                if let oxc_ast::ast::TSSignature::TSCallSignatureDeclaration(call_sig) = member {
                    // First parameter is usually the event name: (e: 'eventName', ...)
                    if let Some(first_param) = call_sig.params.items.first() {
                        if let Some(type_ann) = &first_param.type_annotation {
                            if let TSType::TSLiteralType(lit_type) = &type_ann.type_annotation {
                                if let oxc_ast::ast::TSLiteral::StringLiteral(s) = &lit_type.literal
                                {
                                    result.macros.add_emit(EmitDefinition {
                                        name: CompactString::new(s.value.as_str()),
                                        payload_type: None,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Extract emits from runtime arguments (array)
pub fn extract_emits_from_runtime(
    result: &mut ScriptParseResult,
    arg: &Argument<'_>,
    _source: &str,
) {
    if let Argument::ArrayExpression(arr) = arg {
        for elem in arr.elements.iter() {
            if let oxc_ast::ast::ArrayExpressionElement::StringLiteral(s) = elem {
                result.macros.add_emit(EmitDefinition {
                    name: CompactString::new(s.value.as_str()),
                    payload_type: None,
                });
            }
        }
    }
}

/// Detect reactivity wrappers (ref, computed, reactive, etc.)
/// Also handles aliases (e.g., const r = ref; const count = r(0))
pub fn detect_reactivity_call(
    call: &CallExpression<'_>,
    reactivity_aliases: &FxHashMap<CompactString, CompactString>,
) -> Option<(ReactiveKind, BindingType)> {
    let callee_name = match &call.callee {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return None,
    };

    // Resolve alias to original API name if needed
    let resolved_name = reactivity_aliases
        .get(callee_name)
        .map(|s| s.as_str())
        .unwrap_or(callee_name);

    match resolved_name {
        "ref" | "shallowRef" => Some((ReactiveKind::Ref, BindingType::SetupRef)),
        "computed" => Some((ReactiveKind::Computed, BindingType::SetupRef)),
        "reactive" | "shallowReactive" => {
            Some((ReactiveKind::Reactive, BindingType::SetupReactiveConst))
        }
        "toRef" => Some((ReactiveKind::ToRef, BindingType::SetupRef)),
        "toRefs" => Some((ReactiveKind::ToRefs, BindingType::SetupRef)),
        "customRef" => Some((ReactiveKind::Ref, BindingType::SetupRef)),
        "readonly" => Some((ReactiveKind::Readonly, BindingType::SetupReactiveConst)),
        "shallowReadonly" => Some((
            ReactiveKind::ShallowReadonly,
            BindingType::SetupReactiveConst,
        )),
        _ => None,
    }
}

/// Detect Vue API calls that violate setup context (CSRP/Memory Leak risks)
/// Returns true if a violation was detected and recorded
pub fn detect_setup_context_violation(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
) -> bool {
    // Only detect in non-setup scripts
    if !result.is_non_setup_script {
        return false;
    }

    let callee_name = match &call.callee {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return false,
    };

    if let Some(kind) = SetupContextViolationKind::from_api_name(callee_name) {
        result.setup_context.record_violation(
            kind,
            CompactString::new(callee_name),
            call.span.start,
            call.span.end,
        );
        return true;
    }

    false
}

/// Detect provide() and inject() calls and track them (including through aliases)
pub fn detect_provide_inject_call(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) {
    let callee_name = match &call.callee {
        Expression::Identifier(id) => id.name.as_str(),
        _ => return,
    };

    // Check if this is a direct call or an alias call
    let is_provide = callee_name == "provide" || result.provide_aliases.contains(callee_name);
    let is_inject = callee_name == "inject" || result.inject_aliases.contains(callee_name);

    if is_provide {
        // Detect setup context violation for provide
        detect_setup_context_violation(result, call);

        // provide(key, value)
        if call.arguments.len() >= 2 {
            let key = extract_provide_key(&call.arguments[0], source);
            let value = call
                .arguments
                .get(1)
                .map(|arg| extract_argument_source(arg, source))
                .unwrap_or_default();

            if let Some(key) = key {
                result.provide_inject.add_provide(
                    key,
                    CompactString::new(&value),
                    None, // value_type
                    None, // from_composable
                    call.span.start,
                    call.span.end,
                );
            }
        }
    } else if is_inject {
        // inject() called through an alias (e.g., const a = inject; a('key'))
        // We need to track this as an inject call
        // Note: When inject is assigned to a variable (const state = inject('key')),
        // it's handled in process_variable_declarator. This handles bare inject calls
        // like `a('key')` that appear in expression statements.
    }
}

/// Check for ref.value extraction to a plain variable (loses reactivity)
/// e.g., `const x = someRef.value` or `const primitiveValue = countRef.value`
#[inline]
pub fn check_ref_value_extraction(
    result: &mut ScriptParseResult,
    id: &oxc_ast::ast::BindingPattern<'_>,
    init: &Expression<'_>,
) {
    // Only check simple identifier bindings
    let target_name = match id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => id.name.as_str(),
        _ => return,
    };

    // Check for ref.value pattern: someRef.value
    if let Expression::StaticMemberExpression(member) = init {
        if member.property.name.as_str() == "value" {
            if let Expression::Identifier(obj_id) = &member.object {
                let ref_name = CompactString::new(obj_id.name.as_str());
                if result.reactivity.needs_value_access(ref_name.as_str()) {
                    use crate::reactivity::{ReactivityLoss, ReactivityLossKind};
                    result.reactivity.add_loss(ReactivityLoss {
                        kind: ReactivityLossKind::RefValueExtract {
                            source_name: ref_name,
                            target_name: CompactString::new(target_name),
                        },
                        start: member.span.start,
                        end: member.span.end,
                    });
                }
            }
        }
    }
}

/// Check for reactive property extraction to a plain variable.
/// e.g., `const x = state.x` or `const x = props.x`
#[inline]
pub fn check_reactive_property_extraction(
    result: &mut ScriptParseResult,
    id: &oxc_ast::ast::BindingPattern<'_>,
    init: &Expression<'_>,
) {
    let target_name = match id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => id.name.as_str(),
        _ => return,
    };

    let Some((source_name, prop_name)) = extract_member_chain_root(init) else {
        return;
    };

    result.reactivity.record_property_extract(
        source_name,
        prop_name,
        CompactString::new(target_name),
        init.span().start,
        init.span().end,
    );
}

fn extract_member_chain_root(expr: &Expression<'_>) -> Option<(CompactString, CompactString)> {
    match expr {
        Expression::StaticMemberExpression(member) => {
            if let Some((root, prop_name)) = extract_member_chain_root(&member.object) {
                Some((root, prop_name))
            } else {
                let root = member_chain_root_identifier(&member.object)?;
                Some((root, CompactString::new(member.property.name.as_str())))
            }
        }
        _ => None,
    }
}

fn member_chain_root_identifier(expr: &Expression<'_>) -> Option<CompactString> {
    match expr {
        Expression::Identifier(id) => Some(CompactString::new(id.name.as_str())),
        Expression::StaticMemberExpression(member) => member_chain_root_identifier(&member.object),
        _ => None,
    }
}

/// Extract a provide/inject key from an argument
pub fn extract_provide_key(arg: &Argument<'_>, source: &str) -> Option<ProvideKey> {
    match arg {
        Argument::StringLiteral(s) => {
            Some(ProvideKey::String(CompactString::new(s.value.as_str())))
        }
        Argument::Identifier(id) => {
            // Could be a Symbol or a variable reference - treat as Symbol for now
            Some(ProvideKey::Symbol(CompactString::new(id.name.as_str())))
        }
        _ => {
            // For complex expressions, extract source as string key
            let expr_source = extract_argument_source(arg, source);
            if !expr_source.is_empty() {
                Some(ProvideKey::String(CompactString::new(&expr_source)))
            } else {
                None
            }
        }
    }
}

/// Extract source code of an argument
pub fn extract_argument_source(arg: &Argument<'_>, source: &str) -> String {
    let span = match arg {
        Argument::SpreadElement(s) => s.span,
        Argument::Identifier(id) => id.span,
        Argument::StringLiteral(s) => s.span,
        Argument::NumericLiteral(n) => n.span,
        Argument::BooleanLiteral(b) => b.span,
        Argument::NullLiteral(n) => n.span,
        Argument::ArrayExpression(a) => a.span,
        Argument::ObjectExpression(o) => o.span,
        Argument::FunctionExpression(f) => f.span,
        Argument::ArrowFunctionExpression(a) => a.span,
        Argument::CallExpression(c) => c.span,
        _ => return String::default(),
    };
    String::from(
        source
            .get(span.start as usize..span.end as usize)
            .unwrap_or(""),
    )
}

/// Get binding type from variable declaration kind
pub fn get_binding_type_from_kind(kind: VariableDeclarationKind) -> BindingType {
    match kind {
        VariableDeclarationKind::Const => BindingType::SetupConst,
        VariableDeclarationKind::Let => BindingType::SetupLet,
        VariableDeclarationKind::Var => BindingType::SetupLet,
        VariableDeclarationKind::Using => BindingType::SetupConst,
        VariableDeclarationKind::AwaitUsing => BindingType::SetupConst,
    }
}

/// Process type export (export type / export interface)
pub fn process_type_export(result: &mut ScriptParseResult, decl: &Declaration<'_>, span: Span) {
    match decl {
        Declaration::TSTypeAliasDeclaration(type_alias) => {
            result.type_exports.push(TypeExport {
                name: CompactString::new(type_alias.id.name.as_str()),
                kind: TypeExportKind::Type,
                start: span.start,
                end: span.end,
                hoisted: true,
            });
        }
        Declaration::TSInterfaceDeclaration(interface) => {
            result.type_exports.push(TypeExport {
                name: CompactString::new(interface.id.name.as_str()),
                kind: TypeExportKind::Interface,
                start: span.start,
                end: span.end,
                hoisted: true,
            });
        }
        _ => {}
    }
}

/// Process invalid export in script setup
pub fn process_invalid_export(result: &mut ScriptParseResult, decl: &Declaration<'_>, span: Span) {
    let (name, kind) = match decl {
        Declaration::VariableDeclaration(var_decl) => {
            let first_name = var_decl
                .declarations
                .first()
                .and_then(|d| {
                    if let oxc_ast::ast::BindingPattern::BindingIdentifier(id) = &d.id {
                        Some(id.name.as_str())
                    } else {
                        None
                    }
                })
                .unwrap_or("unknown");

            let kind = match var_decl.kind {
                VariableDeclarationKind::Const => InvalidExportKind::Const,
                VariableDeclarationKind::Let => InvalidExportKind::Let,
                VariableDeclarationKind::Var => InvalidExportKind::Var,
                _ => InvalidExportKind::Const,
            };

            (first_name, kind)
        }
        Declaration::FunctionDeclaration(func) => {
            let name = func
                .id
                .as_ref()
                .map(|id| id.name.as_str())
                .unwrap_or("anonymous");
            (name, InvalidExportKind::Function)
        }
        Declaration::ClassDeclaration(class) => {
            let name = class
                .id
                .as_ref()
                .map(|id| id.name.as_str())
                .unwrap_or("anonymous");
            (name, InvalidExportKind::Class)
        }
        _ => return,
    };

    result.invalid_exports.push(InvalidExport {
        name: CompactString::new(name),
        kind,
        start: span.start,
        end: span.end,
    });
}
