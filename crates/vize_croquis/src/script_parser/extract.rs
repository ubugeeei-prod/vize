//! Extraction functions for props, emits, and reactivity detection.

use oxc_ast::ast::{
    Argument, AssignmentTarget, CallExpression, Declaration, Expression, ObjectPropertyKind,
    PropertyKey, SimpleAssignmentTarget, Statement, TSType, VariableDeclarationKind,
};
use oxc_span::{GetSpan, Span};

use crate::analysis::{InvalidExport, InvalidExportKind, TypeExport, TypeExportKind};
use crate::macros::{EmitDefinition, MacroKind, ModelDefinition, PropDefinition};
use crate::provide::ProvideKey;
use crate::race::RaceConditionRiskKind;
use crate::reactivity::ReactiveKind;
use crate::setup_context::SetupContextViolationKind;
use vize_carton::{cstr, CompactString, FxHashMap, FxHashSet, String};
use vize_relief::BindingType;

use super::{ReactiveGetterContext, ReactiveValueOrigin, ScriptParseResult};

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

/// Detect async reactive mutation patterns that can race with later updates,
/// unmounting, or sibling consumers.
pub fn detect_race_condition_call(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    _source: &str,
) {
    let Some(callee_name) = resolved_call_name(result, call) else {
        return;
    };

    match callee_name.as_str() {
        "watch" => {
            if let Some(callback) = call.arguments.get(1).and_then(argument_expression) {
                record_watcher_risk(result, call, callback, "watch");
            }
        }
        "watchEffect" | "watchPostEffect" | "watchSyncEffect" => {
            if let Some(callback) = call.arguments.first().and_then(argument_expression) {
                record_watcher_risk(result, call, callback, callee_name.as_str());
            }
        }
        name if super::walk::is_client_only_hook(name) => {
            if let Some(callback) = call.arguments.first().and_then(argument_expression) {
                record_lifecycle_risk(result, call, callback, name);
            }
        }
        name if is_scheduler_api(name) => {
            if let Some(callback) = call.arguments.first().and_then(argument_expression) {
                record_scheduler_risk(result, call, callback, name);
            }
        }
        "then" | "catch" | "finally" => {
            for arg in &call.arguments {
                let Some(callback) = argument_expression(arg) else {
                    continue;
                };
                record_promise_risk(result, call, callback, callee_name.as_str());
            }
        }
        _ => {}
    }
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

fn record_watcher_risk(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    callback: &Expression<'_>,
    watcher_name: &str,
) {
    let scan = scan_callback_for_race(result, callback);
    if !scan.has_async_boundary() || scan.mutated_targets.is_empty() || scan.has_cleanup_call {
        return;
    }

    let async_operation = scan.primary_async_operation();
    let mutated_targets = scan.mutated_targets();

    let kind = if matches!(
        watcher_name,
        "watchEffect" | "watchPostEffect" | "watchSyncEffect"
    ) {
        RaceConditionRiskKind::AsyncWatchEffect {
            async_operation,
            mutated_targets,
        }
    } else {
        RaceConditionRiskKind::AsyncWatcherMutation {
            watcher_name: CompactString::new(watcher_name),
            async_operation,
            mutated_targets,
        }
    };

    result
        .race_conditions
        .record(kind, call.span.start, call.span.end);
}

fn record_lifecycle_risk(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    callback: &Expression<'_>,
    hook_name: &str,
) {
    let scan = scan_callback_for_race(result, callback);
    if !scan.has_async_boundary() || scan.mutated_targets.is_empty() || scan.has_cleanup_call {
        return;
    }

    result.race_conditions.record(
        RaceConditionRiskKind::AsyncLifecycleMutation {
            hook_name: CompactString::new(hook_name),
            async_operation: scan.primary_async_operation(),
            mutated_targets: scan.mutated_targets(),
        },
        call.span.start,
        call.span.end,
    );
}

fn record_scheduler_risk(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    callback: &Expression<'_>,
    scheduler_name: &str,
) {
    let scan = scan_callback_for_race(result, callback);
    if scan.mutated_targets.is_empty() || scan.has_cleanup_call {
        return;
    }

    result.race_conditions.record(
        RaceConditionRiskKind::ScheduledMutation {
            scheduler_name: CompactString::new(scheduler_name),
            mutated_targets: scan.mutated_targets(),
        },
        call.span.start,
        call.span.end,
    );
}

fn record_promise_risk(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    callback: &Expression<'_>,
    operation_name: &str,
) {
    let scan = scan_callback_for_race(result, callback);
    if scan.mutated_targets.is_empty() || scan.has_cleanup_call {
        return;
    }

    result.race_conditions.record(
        RaceConditionRiskKind::PromiseContinuationMutation {
            async_operation: CompactString::new(operation_name),
            mutated_targets: scan.mutated_targets(),
        },
        call.span.start,
        call.span.end,
    );
}

#[derive(Default)]
struct RaceScan {
    async_operations: Vec<CompactString>,
    mutated_targets: FxHashSet<CompactString>,
    cleanup_names: FxHashSet<CompactString>,
    has_cleanup_call: bool,
}

impl RaceScan {
    fn has_async_boundary(&self) -> bool {
        !self.async_operations.is_empty()
    }

    fn add_async_operation(&mut self, operation: &str) {
        if !self
            .async_operations
            .iter()
            .any(|existing| existing.as_str() == operation)
        {
            self.async_operations.push(CompactString::new(operation));
        }
    }

    fn primary_async_operation(&self) -> CompactString {
        self.async_operations
            .iter()
            .find(|operation| operation.as_str() != "async callback")
            .or_else(|| self.async_operations.first())
            .cloned()
            .unwrap_or_else(|| CompactString::new("async callback"))
    }

    fn mutated_targets(&self) -> Vec<CompactString> {
        let mut targets = self.mutated_targets.iter().cloned().collect::<Vec<_>>();
        targets.sort();
        targets
    }
}

fn scan_callback_for_race(result: &ScriptParseResult, callback: &Expression<'_>) -> RaceScan {
    let mut scan = RaceScan::default();
    for name in callback_param_names(callback) {
        scan.cleanup_names.insert(name);
    }

    match callback {
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.r#async {
                scan.add_async_operation("async callback");
            }
            for stmt in arrow.body.statements.iter() {
                scan_statement_for_race(result, stmt, &mut scan);
            }
        }
        Expression::FunctionExpression(func) => {
            if func.r#async {
                scan.add_async_operation("async callback");
            }
            if let Some(body) = &func.body {
                for stmt in body.statements.iter() {
                    scan_statement_for_race(result, stmt, &mut scan);
                }
            }
        }
        _ => scan_expression_for_race(result, callback, &mut scan),
    }

    scan
}

fn callback_param_names(callback: &Expression<'_>) -> Vec<CompactString> {
    match callback {
        Expression::ArrowFunctionExpression(arrow) => {
            super::walk::extract_function_params(&arrow.params).into_vec()
        }
        Expression::FunctionExpression(func) => {
            super::walk::extract_function_params(&func.params).into_vec()
        }
        _ => Vec::new(),
    }
}

fn scan_statement_for_race(result: &ScriptParseResult, stmt: &Statement<'_>, scan: &mut RaceScan) {
    match stmt {
        Statement::ExpressionStatement(expr_stmt) => {
            scan_expression_for_race(result, &expr_stmt.expression, scan);
        }
        Statement::VariableDeclaration(var_decl) => {
            for decl in var_decl.declarations.iter() {
                if let Some(init) = &decl.init {
                    scan_expression_for_race(result, init, scan);
                }
            }
        }
        Statement::ReturnStatement(ret) => {
            if let Some(arg) = &ret.argument {
                scan_expression_for_race(result, arg, scan);
            }
        }
        Statement::BlockStatement(block) => {
            for stmt in block.body.iter() {
                scan_statement_for_race(result, stmt, scan);
            }
        }
        Statement::IfStatement(if_stmt) => {
            scan_expression_for_race(result, &if_stmt.test, scan);
            scan_statement_for_race(result, &if_stmt.consequent, scan);
            if let Some(alt) = &if_stmt.alternate {
                scan_statement_for_race(result, alt, scan);
            }
        }
        Statement::ForStatement(for_stmt) => {
            if let Some(init) = &for_stmt.init {
                if let Some(expr) = init.as_expression() {
                    scan_expression_for_race(result, expr, scan);
                }
            }
            if let Some(test) = &for_stmt.test {
                scan_expression_for_race(result, test, scan);
            }
            if let Some(update) = &for_stmt.update {
                scan_expression_for_race(result, update, scan);
            }
            scan_statement_for_race(result, &for_stmt.body, scan);
        }
        Statement::ForInStatement(for_in) => {
            scan_expression_for_race(result, &for_in.right, scan);
            scan_statement_for_race(result, &for_in.body, scan);
        }
        Statement::ForOfStatement(for_of) => {
            scan_expression_for_race(result, &for_of.right, scan);
            scan_statement_for_race(result, &for_of.body, scan);
        }
        Statement::WhileStatement(while_stmt) => {
            scan_expression_for_race(result, &while_stmt.test, scan);
            scan_statement_for_race(result, &while_stmt.body, scan);
        }
        Statement::DoWhileStatement(do_while) => {
            scan_statement_for_race(result, &do_while.body, scan);
            scan_expression_for_race(result, &do_while.test, scan);
        }
        Statement::SwitchStatement(switch_stmt) => {
            scan_expression_for_race(result, &switch_stmt.discriminant, scan);
            for case in switch_stmt.cases.iter() {
                if let Some(test) = &case.test {
                    scan_expression_for_race(result, test, scan);
                }
                for stmt in case.consequent.iter() {
                    scan_statement_for_race(result, stmt, scan);
                }
            }
        }
        Statement::TryStatement(try_stmt) => {
            for stmt in try_stmt.block.body.iter() {
                scan_statement_for_race(result, stmt, scan);
            }
            if let Some(handler) = &try_stmt.handler {
                for stmt in handler.body.body.iter() {
                    scan_statement_for_race(result, stmt, scan);
                }
            }
            if let Some(finalizer) = &try_stmt.finalizer {
                for stmt in finalizer.body.iter() {
                    scan_statement_for_race(result, stmt, scan);
                }
            }
        }
        _ => {}
    }
}

fn scan_expression_for_race(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    scan: &mut RaceScan,
) {
    match expr {
        Expression::AwaitExpression(await_expr) => {
            scan.add_async_operation("await");
            scan_expression_for_race(result, &await_expr.argument, scan);
        }
        Expression::CallExpression(call) => {
            scan_call_expression_for_race(result, call, scan);
        }
        Expression::AssignmentExpression(assign) => {
            if let Some(target) = assignment_target_root(result, &assign.left) {
                scan.mutated_targets.insert(target);
            }
            scan_expression_for_race(result, &assign.right, scan);
        }
        Expression::UpdateExpression(update) => {
            if let Some(target) = simple_assignment_target_root(result, &update.argument) {
                scan.mutated_targets.insert(target);
            }
        }
        Expression::StaticMemberExpression(member) => {
            scan_expression_for_race(result, &member.object, scan);
        }
        Expression::ComputedMemberExpression(member) => {
            scan_expression_for_race(result, &member.object, scan);
            scan_expression_for_race(result, &member.expression, scan);
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::CallExpression(call) => {
                scan_call_expression_for_race(result, call, scan);
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(expr) => {
                scan_expression_for_race(result, &expr.expression, scan);
            }
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                scan_expression_for_race(result, &member.object, scan);
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                scan_expression_for_race(result, &member.object, scan);
                scan_expression_for_race(result, &member.expression, scan);
            }
            oxc_ast::ast::ChainElement::PrivateFieldExpression(field) => {
                scan_expression_for_race(result, &field.object, scan);
            }
        },
        Expression::ConditionalExpression(cond) => {
            scan_expression_for_race(result, &cond.test, scan);
            scan_expression_for_race(result, &cond.consequent, scan);
            scan_expression_for_race(result, &cond.alternate, scan);
        }
        Expression::LogicalExpression(logical) => {
            scan_expression_for_race(result, &logical.left, scan);
            scan_expression_for_race(result, &logical.right, scan);
        }
        Expression::BinaryExpression(binary) => {
            scan_expression_for_race(result, &binary.left, scan);
            scan_expression_for_race(result, &binary.right, scan);
        }
        Expression::ArrayExpression(arr) => {
            for elem in arr.elements.iter() {
                if let Some(expr) = elem.as_expression() {
                    scan_expression_for_race(result, expr, scan);
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            for prop in obj.properties.iter() {
                match prop {
                    ObjectPropertyKind::ObjectProperty(prop) => {
                        scan_expression_for_race(result, &prop.value, scan);
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        scan_expression_for_race(result, &spread.argument, scan);
                    }
                }
            }
        }
        Expression::UnaryExpression(unary) => {
            scan_expression_for_race(result, &unary.argument, scan);
        }
        Expression::SequenceExpression(seq) => {
            for expr in seq.expressions.iter() {
                scan_expression_for_race(result, expr, scan);
            }
        }
        Expression::ParenthesizedExpression(paren) => {
            scan_expression_for_race(result, &paren.expression, scan);
        }
        Expression::TSAsExpression(ts_as) => {
            scan_expression_for_race(result, &ts_as.expression, scan);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            scan_expression_for_race(result, &ts_satisfies.expression, scan);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            scan_expression_for_race(result, &ts_non_null.expression, scan);
        }
        _ => {}
    }
}

fn scan_call_expression_for_race(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
    scan: &mut RaceScan,
) {
    if let Some(name) = resolved_call_name(result, call) {
        if name == "fetch" {
            scan.add_async_operation("fetch");
        } else if matches!(name.as_str(), "then" | "catch" | "finally") {
            scan.add_async_operation("promise callback");
        } else if is_scheduler_api(name.as_str()) {
            scan.add_async_operation(name.as_str());
        }

        if name == "onWatcherCleanup" || scan.cleanup_names.contains(name.as_str()) {
            scan.has_cleanup_call = true;
        }
    }

    if let Some(target) = mutation_call_target(result, call) {
        scan.mutated_targets.insert(target);
    }

    scan_expression_for_race(result, &call.callee, scan);
    for arg in call.arguments.iter() {
        if let Some(expr) = arg.as_expression() {
            scan_expression_for_race(result, expr, scan);
        }
    }
}

fn mutation_call_target(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
) -> Option<CompactString> {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };
    if !is_mutating_method(member.property.name.as_str()) {
        return None;
    }
    expression_reactive_root(result, &member.object)
}

fn assignment_target_root(
    result: &ScriptParseResult,
    target: &AssignmentTarget<'_>,
) -> Option<CompactString> {
    match target {
        AssignmentTarget::AssignmentTargetIdentifier(id) => {
            tracked_mutation_root(result, id.name.as_str())
        }
        AssignmentTarget::StaticMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        AssignmentTarget::ComputedMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        _ => None,
    }
}

fn simple_assignment_target_root(
    result: &ScriptParseResult,
    target: &SimpleAssignmentTarget<'_>,
) -> Option<CompactString> {
    match target {
        SimpleAssignmentTarget::AssignmentTargetIdentifier(id) => {
            tracked_mutation_root(result, id.name.as_str())
        }
        SimpleAssignmentTarget::StaticMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        SimpleAssignmentTarget::ComputedMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        _ => None,
    }
}

fn expression_reactive_root(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
) -> Option<CompactString> {
    match expr {
        Expression::Identifier(id) => tracked_mutation_root(result, id.name.as_str()),
        Expression::StaticMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        Expression::ComputedMemberExpression(member) => {
            expression_reactive_root(result, &member.object)
        }
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                expression_reactive_root(result, &member.object)
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                expression_reactive_root(result, &member.object)
            }
            oxc_ast::ast::ChainElement::PrivateFieldExpression(field) => {
                expression_reactive_root(result, &field.object)
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(expr) => {
                expression_reactive_root(result, &expr.expression)
            }
            _ => None,
        },
        _ => None,
    }
}

fn tracked_mutation_root(result: &ScriptParseResult, name: &str) -> Option<CompactString> {
    (result.reactivity.is_reactive(name) || result.inject_var_names.contains(name))
        .then(|| CompactString::new(name))
}

fn resolved_call_name(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
) -> Option<CompactString> {
    let raw_name = match &call.callee {
        Expression::Identifier(id) => Some(id.name.as_str()),
        Expression::StaticMemberExpression(member) => Some(member.property.name.as_str()),
        Expression::ComputedMemberExpression(_) => None,
        _ => None,
    }?;

    Some(
        result
            .reactivity_aliases
            .get(raw_name)
            .cloned()
            .unwrap_or_else(|| CompactString::new(raw_name)),
    )
}

fn argument_expression<'a>(arg: &'a Argument<'a>) -> Option<&'a Expression<'a>> {
    arg.as_expression()
}

fn is_scheduler_api(name: &str) -> bool {
    matches!(
        name,
        "setTimeout"
            | "setInterval"
            | "requestAnimationFrame"
            | "requestIdleCallback"
            | "queueMicrotask"
    )
}

fn is_mutating_method(name: &str) -> bool {
    matches!(
        name,
        "push"
            | "pop"
            | "shift"
            | "unshift"
            | "splice"
            | "sort"
            | "reverse"
            | "fill"
            | "copyWithin"
            | "set"
            | "add"
            | "delete"
            | "clear"
    )
}

struct ReactivePlainValue {
    source_name: CompactString,
    argument_name: CompactString,
    getter_name: CompactString,
    start: u32,
    end: u32,
}

/// Record reactivity loss when a plain reactive snapshot crosses a call boundary.
#[inline]
pub fn detect_call_argument_reactivity_loss(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) {
    let callee_name = call_label(result, call, source);

    for arg in call.arguments.iter() {
        match arg {
            Argument::SpreadElement(spread) => {
                record_reactive_plain_values_in_call_arg(
                    result,
                    &spread.argument,
                    &callee_name,
                    source,
                );
            }
            _ => {
                if let Some(expr) = arg.as_expression() {
                    if getter_source_from_function(result, expr, source).is_some() {
                        continue;
                    }
                    record_reactive_plain_values_in_call_arg(result, expr, &callee_name, source);
                }
            }
        }
    }
}

/// Track call results whose arguments are getters of reactive snapshots.
#[inline]
pub fn record_getter_context_from_call(
    result: &mut ScriptParseResult,
    target_name: &str,
    call: &CallExpression<'_>,
    source: &str,
) {
    let mut getters = FxHashMap::default();

    for arg in call.arguments.iter() {
        let Some(expr) = arg.as_expression() else {
            continue;
        };
        let Some(value) = getter_source_from_function(result, expr, source) else {
            continue;
        };
        getters.insert(value.getter_name, value.source_name);
    }

    if getters.is_empty() {
        return;
    }

    result.reactive_getter_contexts.insert(
        CompactString::new(target_name),
        ReactiveGetterContext {
            callee_name: call_label(result, call, source),
            getters,
        },
    );
}

/// Check `const x = ctx.count()` where `ctx` was produced from getter arguments.
#[inline]
pub fn check_getter_call_extraction(
    result: &mut ScriptParseResult,
    id: &oxc_ast::ast::BindingPattern<'_>,
    init: &Expression<'_>,
) {
    let target_name = match id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => id.name.as_str(),
        _ => return,
    };

    let Some((context_name, getter_name, source_name, callee_name)) =
        getter_call_source(result, init)
    else {
        return;
    };

    use crate::reactivity::{ReactivityLoss, ReactivityLossKind};
    result.reactivity.add_loss(ReactivityLoss {
        kind: ReactivityLossKind::GetterCallExtract {
            context_name: context_name.clone(),
            getter_name: getter_name.clone(),
            target_name: CompactString::new(target_name),
            callee_name,
            source_name: source_name.clone(),
        },
        start: init.span().start,
        end: init.span().end,
    });
    result.reactive_value_origins.insert(
        CompactString::new(target_name),
        ReactiveValueOrigin::GetterCall {
            context_name,
            getter_name,
            source_name,
        },
    );
}

/// Check `const alias = count` where `count` is already a plain reactive snapshot.
#[inline]
pub fn check_reactive_plain_alias_extraction(
    result: &mut ScriptParseResult,
    id: &oxc_ast::ast::BindingPattern<'_>,
    init: &Expression<'_>,
) {
    let target_name = match id {
        oxc_ast::ast::BindingPattern::BindingIdentifier(id) => id.name.as_str(),
        _ => return,
    };

    let Some(value) = reactive_plain_identifier_value_from_expr(result, init) else {
        return;
    };
    if value.argument_name.as_str() == target_name {
        return;
    }

    result.reactivity.record_plain_value_alias(
        value.source_name.clone(),
        value.argument_name,
        CompactString::new(target_name),
        value.start,
        value.end,
    );
    result.reactive_value_origins.insert(
        CompactString::new(target_name),
        ReactiveValueOrigin::PlainAlias {
            source_name: value.source_name,
        },
    );
}

/// Check `alias = count` where `count` is already a plain reactive snapshot.
#[inline]
pub fn check_reactive_plain_assignment_alias(
    result: &mut ScriptParseResult,
    target_name: &str,
    init: &Expression<'_>,
) {
    if result.reactivity.is_reactive(target_name) {
        return;
    }

    let Some(value) = reactive_plain_identifier_value_from_expr(result, init) else {
        return;
    };
    if value.argument_name.as_str() == target_name {
        return;
    }

    result.reactivity.record_plain_value_alias(
        value.source_name.clone(),
        value.argument_name,
        CompactString::new(target_name),
        value.start,
        value.end,
    );
    result.reactive_value_origins.insert(
        CompactString::new(target_name),
        ReactiveValueOrigin::PlainAlias {
            source_name: value.source_name,
        },
    );
}

fn record_reactive_plain_values_in_call_arg(
    result: &mut ScriptParseResult,
    expr: &Expression<'_>,
    callee_name: &CompactString,
    source: &str,
) {
    if let Some(value) = reactive_plain_value_from_expr(result, expr, source) {
        result.reactivity.record_function_argument_extract(
            value.source_name.clone(),
            value.argument_name.clone(),
            callee_name.clone(),
            value.start,
            value.end,
        );
        result.reactive_value_origins.insert(
            value.argument_name,
            ReactiveValueOrigin::FunctionArgument {
                source_name: value.source_name,
                callee_name: callee_name.clone(),
            },
        );
        return;
    }

    match expr {
        Expression::ArrayExpression(arr) => {
            for elem in arr.elements.iter() {
                match elem {
                    oxc_ast::ast::ArrayExpressionElement::SpreadElement(spread) => {
                        record_reactive_plain_values_in_call_arg(
                            result,
                            &spread.argument,
                            callee_name,
                            source,
                        );
                    }
                    oxc_ast::ast::ArrayExpressionElement::Elision(_) => {}
                    _ => {
                        if let Some(expr) = elem.as_expression() {
                            record_reactive_plain_values_in_call_arg(
                                result,
                                expr,
                                callee_name,
                                source,
                            );
                        }
                    }
                }
            }
        }
        Expression::ObjectExpression(obj) => {
            for prop in obj.properties.iter() {
                match prop {
                    ObjectPropertyKind::ObjectProperty(prop) => {
                        record_reactive_plain_values_in_call_arg(
                            result,
                            &prop.value,
                            callee_name,
                            source,
                        );
                    }
                    ObjectPropertyKind::SpreadProperty(spread) => {
                        record_reactive_plain_values_in_call_arg(
                            result,
                            &spread.argument,
                            callee_name,
                            source,
                        );
                    }
                }
            }
        }
        Expression::ConditionalExpression(cond) => {
            record_reactive_plain_values_in_call_arg(result, &cond.test, callee_name, source);
            record_reactive_plain_values_in_call_arg(result, &cond.consequent, callee_name, source);
            record_reactive_plain_values_in_call_arg(result, &cond.alternate, callee_name, source);
        }
        Expression::LogicalExpression(logical) => {
            record_reactive_plain_values_in_call_arg(result, &logical.left, callee_name, source);
            record_reactive_plain_values_in_call_arg(result, &logical.right, callee_name, source);
        }
        Expression::SequenceExpression(seq) => {
            for expr in seq.expressions.iter() {
                record_reactive_plain_values_in_call_arg(result, expr, callee_name, source);
            }
        }
        Expression::ParenthesizedExpression(paren) => {
            record_reactive_plain_values_in_call_arg(
                result,
                &paren.expression,
                callee_name,
                source,
            );
        }
        Expression::TSAsExpression(ts_as) => {
            record_reactive_plain_values_in_call_arg(
                result,
                &ts_as.expression,
                callee_name,
                source,
            );
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            record_reactive_plain_values_in_call_arg(
                result,
                &ts_satisfies.expression,
                callee_name,
                source,
            );
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            record_reactive_plain_values_in_call_arg(
                result,
                &ts_non_null.expression,
                callee_name,
                source,
            );
        }
        _ => {}
    }
}

fn getter_source_from_function(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) -> Option<ReactivePlainValue> {
    let returned = match expr {
        Expression::ArrowFunctionExpression(arrow) => {
            if !arrow.params.items.is_empty() {
                return None;
            }
            arrow_return_expression(arrow)?
        }
        Expression::FunctionExpression(func) => {
            if !func.params.items.is_empty() {
                return None;
            }
            function_return_expression(func)?
        }
        Expression::ParenthesizedExpression(paren) => {
            return getter_source_from_function(result, &paren.expression, source);
        }
        Expression::TSAsExpression(ts_as) => {
            return getter_source_from_function(result, &ts_as.expression, source);
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            return getter_source_from_function(result, &ts_satisfies.expression, source);
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            return getter_source_from_function(result, &ts_non_null.expression, source);
        }
        _ => return None,
    };

    reactive_plain_value_from_expr(result, returned, source)
}

fn arrow_return_expression<'a>(
    arrow: &'a oxc_ast::ast::ArrowFunctionExpression<'a>,
) -> Option<&'a Expression<'a>> {
    if !arrow.expression {
        return function_body_return_expression(&arrow.body.statements);
    }

    let Statement::ExpressionStatement(expr_stmt) = arrow.body.statements.first()? else {
        return None;
    };
    Some(&expr_stmt.expression)
}

fn function_return_expression<'a>(
    func: &'a oxc_ast::ast::Function<'a>,
) -> Option<&'a Expression<'a>> {
    function_body_return_expression(&func.body.as_ref()?.statements)
}

fn function_body_return_expression<'a>(
    statements: &'a oxc_allocator::Vec<'a, Statement<'a>>,
) -> Option<&'a Expression<'a>> {
    for stmt in statements.iter() {
        if let Statement::ReturnStatement(ret) = stmt {
            if let Some(argument) = &ret.argument {
                return Some(argument);
            }
        }
    }
    None
}

fn reactive_plain_value_from_expr(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) -> Option<ReactivePlainValue> {
    match expr {
        Expression::Identifier(id) => {
            let binding_name = id.name.as_str();
            let origin = result.reactive_value_origins.get(binding_name)?;
            let (source_name, getter_name) = plain_origin_labels(origin, binding_name);
            Some(ReactivePlainValue {
                source_name,
                argument_name: CompactString::new(binding_name),
                getter_name,
                start: id.span.start,
                end: id.span.end,
            })
        }
        Expression::StaticMemberExpression(member) => {
            if member.property.name.as_str() == "value" {
                if let Some(root) = member_chain_root_identifier(&member.object) {
                    if result.reactivity.needs_value_access(root.as_str()) {
                        return Some(ReactivePlainValue {
                            source_name: expression_label(source, member.span),
                            argument_name: expression_label(source, member.span),
                            getter_name: root,
                            start: member.span.start,
                            end: member.span.end,
                        });
                    }
                }
            }

            let (root, prop_name) = extract_member_chain_root(expr)?;
            if result
                .reactivity
                .lookup(root.as_str())
                .is_some_and(|source| !source.kind.needs_value_access())
            {
                return Some(ReactivePlainValue {
                    source_name: expression_label(source, member.span),
                    argument_name: expression_label(source, member.span),
                    getter_name: prop_name,
                    start: member.span.start,
                    end: member.span.end,
                });
            }

            let root_origin = result.reactive_value_origins.get(root.as_str())?;
            let (source_name, _) = plain_origin_labels(root_origin, root.as_str());
            Some(ReactivePlainValue {
                source_name,
                argument_name: expression_label(source, member.span),
                getter_name: prop_name,
                start: member.span.start,
                end: member.span.end,
            })
        }
        Expression::ComputedMemberExpression(member) => {
            let root = member_chain_root_identifier(&member.object)?;
            if result
                .reactivity
                .lookup(root.as_str())
                .is_some_and(|source| !source.kind.needs_value_access())
            {
                return Some(ReactivePlainValue {
                    source_name: expression_label(source, member.span),
                    argument_name: expression_label(source, member.span),
                    getter_name: expression_label(source, member.span),
                    start: member.span.start,
                    end: member.span.end,
                });
            }
            None
        }
        Expression::CallExpression(_) => getter_call_plain_value(result, expr, source),
        Expression::ChainExpression(chain) => match &chain.expression {
            oxc_ast::ast::ChainElement::CallExpression(_) => {
                getter_call_plain_value(result, expr, source)
            }
            oxc_ast::ast::ChainElement::StaticMemberExpression(member) => {
                reactive_plain_value_from_expr(result, &member.object, source)
            }
            oxc_ast::ast::ChainElement::ComputedMemberExpression(member) => {
                reactive_plain_value_from_expr(result, &member.object, source)
            }
            oxc_ast::ast::ChainElement::TSNonNullExpression(expr) => {
                reactive_plain_value_from_expr(result, &expr.expression, source)
            }
            oxc_ast::ast::ChainElement::PrivateFieldExpression(field) => {
                reactive_plain_value_from_expr(result, &field.object, source)
            }
        },
        Expression::ParenthesizedExpression(paren) => {
            reactive_plain_value_from_expr(result, &paren.expression, source)
        }
        Expression::TSAsExpression(ts_as) => {
            reactive_plain_value_from_expr(result, &ts_as.expression, source)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            reactive_plain_value_from_expr(result, &ts_satisfies.expression, source)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            reactive_plain_value_from_expr(result, &ts_non_null.expression, source)
        }
        _ => None,
    }
}

fn reactive_plain_identifier_value_from_expr(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
) -> Option<ReactivePlainValue> {
    match expr {
        Expression::Identifier(id) => {
            let binding_name = id.name.as_str();
            let origin = result.reactive_value_origins.get(binding_name)?;
            let (source_name, _) = plain_origin_labels(origin, binding_name);
            Some(ReactivePlainValue {
                source_name,
                argument_name: CompactString::new(binding_name),
                getter_name: CompactString::new(binding_name),
                start: id.span.start,
                end: id.span.end,
            })
        }
        Expression::ParenthesizedExpression(paren) => {
            reactive_plain_identifier_value_from_expr(result, &paren.expression)
        }
        Expression::TSAsExpression(ts_as) => {
            reactive_plain_identifier_value_from_expr(result, &ts_as.expression)
        }
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            reactive_plain_identifier_value_from_expr(result, &ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            reactive_plain_identifier_value_from_expr(result, &ts_non_null.expression)
        }
        _ => None,
    }
}

fn getter_call_plain_value(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
    source: &str,
) -> Option<ReactivePlainValue> {
    let (context_name, getter_name, source_name, _) = getter_call_source(result, expr)?;
    Some(ReactivePlainValue {
        source_name,
        argument_name: expression_label(source, expr.span()),
        getter_name,
        start: expr.span().start,
        end: expr.span().end,
    })
    .filter(|_| !context_name.is_empty())
}

fn getter_call_source(
    result: &ScriptParseResult,
    expr: &Expression<'_>,
) -> Option<(CompactString, CompactString, CompactString, CompactString)> {
    match expr {
        Expression::CallExpression(call) => getter_call_source_from_call(result, call),
        Expression::ParenthesizedExpression(paren) => getter_call_source(result, &paren.expression),
        Expression::TSAsExpression(ts_as) => getter_call_source(result, &ts_as.expression),
        Expression::TSSatisfiesExpression(ts_satisfies) => {
            getter_call_source(result, &ts_satisfies.expression)
        }
        Expression::TSNonNullExpression(ts_non_null) => {
            getter_call_source(result, &ts_non_null.expression)
        }
        _ => None,
    }
}

fn getter_call_source_from_call(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
) -> Option<(CompactString, CompactString, CompactString, CompactString)> {
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };
    let Expression::Identifier(context) = &member.object else {
        return None;
    };

    let context_name = CompactString::new(context.name.as_str());
    let getter_name = CompactString::new(member.property.name.as_str());
    let context = result.reactive_getter_contexts.get(context_name.as_str())?;
    let source_name = context.getters.get(getter_name.as_str())?.clone();

    Some((
        context_name,
        getter_name,
        source_name,
        context.callee_name.clone(),
    ))
}

fn plain_origin_labels(
    origin: &ReactiveValueOrigin,
    binding_name: &str,
) -> (CompactString, CompactString) {
    match origin {
        ReactiveValueOrigin::PropsDestructure { prop_name } => {
            (prop_name.clone(), prop_name.clone())
        }
        ReactiveValueOrigin::ReactiveProperty {
            source_name,
            prop_name,
        } => (cstr!("{source_name}.{prop_name}"), prop_name.clone()),
        ReactiveValueOrigin::RefValue { source_name } => {
            (cstr!("{source_name}.value"), source_name.clone())
        }
        ReactiveValueOrigin::FunctionArgument {
            source_name,
            callee_name: _callee_name,
        } => (source_name.clone(), CompactString::new(binding_name)),
        ReactiveValueOrigin::GetterCall {
            context_name: _context_name,
            getter_name,
            source_name,
        } => (source_name.clone(), getter_name.clone()),
        ReactiveValueOrigin::PlainAlias { source_name } => {
            (source_name.clone(), CompactString::new(binding_name))
        }
    }
}

fn call_label(
    result: &ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) -> CompactString {
    resolved_call_name(result, call).unwrap_or_else(|| expression_label(source, call.callee.span()))
}

fn expression_label(source: &str, span: Span) -> CompactString {
    source
        .get(span.start as usize..span.end as usize)
        .map(str::trim)
        .filter(|label| !label.is_empty())
        .map(CompactString::new)
        .unwrap_or_else(|| CompactString::new("<expression>"))
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
                            source_name: ref_name.clone(),
                            target_name: CompactString::new(target_name),
                        },
                        start: member.span.start,
                        end: member.span.end,
                    });
                    result.reactive_value_origins.insert(
                        CompactString::new(target_name),
                        ReactiveValueOrigin::RefValue {
                            source_name: ref_name,
                        },
                    );
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

    let is_reactive_property = result
        .reactivity
        .lookup(source_name.as_str())
        .is_some_and(|source| !source.kind.needs_value_access());
    if !is_reactive_property {
        return;
    }

    result.reactivity.record_property_extract(
        source_name.clone(),
        prop_name.clone(),
        CompactString::new(target_name),
        init.span().start,
        init.span().end,
    );
    result.reactive_value_origins.insert(
        CompactString::new(target_name),
        ReactiveValueOrigin::ReactiveProperty {
            source_name,
            prop_name,
        },
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
