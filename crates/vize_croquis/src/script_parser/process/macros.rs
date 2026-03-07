//! Variable declarator processing for macros, reactivity, and inject patterns.
//!
//! Handles the complex logic of processing variable declarations including:
//! - Compiler macro detection (defineProps, defineEmits, etc.)
//! - Reactivity wrapper detection (ref, computed, reactive)
//! - Inject call detection and destructuring patterns
//! - Object/array destructuring from defineProps and reactive sources

use oxc_ast::ast::{Argument, BindingPattern, Expression, PropertyKey, VariableDeclarationKind};
use oxc_span::GetSpan;

use crate::macros::{MacroKind, PropsDestructuredBindings};
use crate::provide::InjectPattern;
use crate::reactivity::ReactiveKind;
use vize_carton::CompactString;
use vize_relief::BindingType;

use super::super::extract::{
    check_ref_value_extraction, detect_reactivity_call, detect_setup_context_violation,
    extract_argument_source, extract_call_expression, extract_provide_key,
    get_binding_type_from_kind, process_call_expression,
};
use super::super::walk::{walk_call_arguments, walk_expression};
use super::super::ScriptParseResult;
use super::bindings::{
    get_binding_pattern_name, infer_destructure_binding_type, is_function_expression,
    is_literal_expression,
};

/// Process a variable declarator
pub(in crate::script_parser) fn process_variable_declarator(
    result: &mut ScriptParseResult,
    declarator: &oxc_ast::ast::VariableDeclarator<'_>,
    kind: VariableDeclarationKind,
    source: &str,
) {
    // Handle destructuring patterns
    match &declarator.id {
        BindingPattern::BindingIdentifier(id) => {
            let name = id.name.as_str();

            // Record definition span for Go-to-Definition
            result
                .binding_spans
                .insert(CompactString::new(name), (id.span.start, id.span.end));

            // Check if the init is a macro or reactivity call
            // Use extract_call_expression to handle type assertions (as/satisfies)
            let call_extracted = if let Some(call) =
                declarator.init.as_ref().and_then(extract_call_expression)
            {
                // Check for macro calls (defineProps, defineEmits, etc.)
                if let Some(macro_kind) = process_call_expression(result, call, source) {
                    // Assign binding type based on macro kind
                    let binding_type = match macro_kind {
                        MacroKind::DefineProps | MacroKind::WithDefaults => {
                            BindingType::SetupReactiveConst
                        }
                        MacroKind::DefineModel => BindingType::SetupRef,
                        _ => get_binding_type_from_kind(kind),
                    };
                    // defineModel returns a ref, register in reactivity tracker
                    if macro_kind == MacroKind::DefineModel {
                        result
                            .reactivity
                            .register(CompactString::new(name), ReactiveKind::Ref, 0);
                    }
                    result.bindings.add(name, binding_type);
                    // Walk into the call's callback arguments to track nested scopes
                    walk_call_arguments(result, call, source);
                    return;
                }

                // Check for reactivity wrappers (also handles aliases)
                if let Some((reactive_kind, binding_type)) =
                    detect_reactivity_call(call, &result.reactivity_aliases)
                {
                    // Detect setup context violations for module-level state
                    detect_setup_context_violation(result, call);

                    result
                        .reactivity
                        .register(CompactString::new(name), reactive_kind, 0);
                    result.bindings.add(name, binding_type);
                    // Walk into the call's callback arguments to track nested scopes
                    walk_call_arguments(result, call, source);
                    return;
                }

                // Check for inject() call - track with local_name for indirect destructure detection
                // Also handles inject aliases (e.g., const a = inject; const state = a('key'))
                if let Expression::Identifier(callee_id) = &call.callee {
                    let callee_name = callee_id.name.as_str();
                    let is_inject =
                        callee_name == "inject" || result.inject_aliases.contains(callee_name);
                    if is_inject && !call.arguments.is_empty() {
                        // Detect setup context violation for inject
                        detect_setup_context_violation(result, call);

                        if let Some(key) = extract_provide_key(&call.arguments[0], source) {
                            let default_value = call.arguments.get(1).map(|arg| {
                                CompactString::new(extract_argument_source(arg, source))
                            });
                            let local_name = CompactString::new(name);
                            // Track inject variable name for indirect destructure detection
                            result.inject_var_names.insert(local_name.clone());
                            result.provide_inject.add_inject(
                                key,
                                local_name, // local_name is the binding name
                                default_value,
                                None, // expected_type
                                InjectPattern::Simple,
                                None, // from_composable
                                call.span.start,
                                call.span.end,
                            );
                            // Walk into the call's callback arguments to track nested scopes
                            walk_call_arguments(result, call, source);
                            // Add binding and return
                            let binding_type = get_binding_type_from_kind(kind);
                            result.bindings.add(name, binding_type);
                            return;
                        }
                    }
                }

                // Not a known macro/reactivity/inject, but still walk for nested scopes
                walk_call_arguments(result, call, source);
                true // Call was extracted and processed
            } else {
                false
            };

            // Walk other expression types for nested scopes
            // Skip if we already extracted and processed a call expression to avoid double processing
            if !call_extracted {
                if let Some(init) = &declarator.init {
                    walk_expression(result, init, source);

                    // Check for ref.value extraction: const x = someRef.value
                    check_ref_value_extraction(result, &declarator.id, init);

                    // Check for Vue API aliases: const a = inject, const r = ref, etc.
                    if let Expression::Identifier(id) = init {
                        let api_name = id.name.as_str();
                        match api_name {
                            "inject" => {
                                result.inject_aliases.insert(CompactString::new(name));
                            }
                            "provide" => {
                                result.provide_aliases.insert(CompactString::new(name));
                            }
                            // Reactivity APIs
                            "ref" | "shallowRef" | "reactive" | "shallowReactive"
                            | "computed" | "readonly" | "shallowReadonly"
                            | "toRef" | "toRefs" | "toValue" | "toRaw"
                            | "isRef" | "isReactive" | "isReadonly" | "isProxy"
                            | "unref" | "triggerRef" | "customRef"
                            | "markRaw" | "effectScope" | "getCurrentScope" | "onScopeDispose"
                            // Watch APIs
                            | "watch" | "watchEffect" | "watchPostEffect" | "watchSyncEffect"
                            // Lifecycle hooks
                            | "onMounted" | "onUnmounted" | "onBeforeMount" | "onBeforeUnmount"
                            | "onUpdated" | "onBeforeUpdate" | "onActivated" | "onDeactivated"
                            | "onErrorCaptured" | "onRenderTracked" | "onRenderTriggered"
                            | "onServerPrefetch"
                            // Component APIs
                            | "defineComponent" | "defineAsyncComponent"
                            | "getCurrentInstance" | "nextTick"
                            // Types (for InjectionKey tracking)
                            | "InjectionKey" => {
                                result.reactivity_aliases.insert(
                                    CompactString::new(name),
                                    CompactString::new(api_name),
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Regular binding - for const, detect literal/function expressions
            let binding_type = if kind == VariableDeclarationKind::Const {
                if let Some(init) = &declarator.init {
                    if is_literal_expression(init) {
                        BindingType::LiteralConst
                    } else if is_function_expression(init)
                        || matches!(
                            init,
                            Expression::ObjectExpression(_) | Expression::ArrayExpression(_)
                        )
                    {
                        BindingType::SetupConst
                    } else {
                        BindingType::SetupMaybeRef
                    }
                } else {
                    BindingType::SetupConst
                }
            } else {
                get_binding_type_from_kind(kind)
            };
            result.bindings.add(name, binding_type);
        }

        BindingPattern::ObjectPattern(obj) => {
            // Check if this is destructuring from defineProps or withDefaults(defineProps())
            let is_define_props = declarator.init.as_ref().is_some_and(|init| {
                match init {
                    Expression::CallExpression(call) => {
                        if let Expression::Identifier(id) = &call.callee {
                            let name = id.name.as_str();
                            if name == "defineProps" {
                                return true;
                            }
                            // withDefaults(defineProps<...>(), {...})
                            if name == "withDefaults" {
                                if let Some(Argument::CallExpression(inner)) =
                                    call.arguments.first()
                                {
                                    if let Expression::Identifier(inner_id) = &inner.callee {
                                        return inner_id.name.as_str() == "defineProps";
                                    }
                                }
                            }
                        }
                        false
                    }
                    _ => false,
                }
            });

            // Check if this is destructuring from inject() - this loses reactivity!
            let inject_call = declarator.init.as_ref().and_then(|init| {
                let call = extract_call_expression(init)?;
                if let Expression::Identifier(id) = &call.callee {
                    if id.name.as_str() == "inject" {
                        return Some(call);
                    }
                }
                None
            });

            // Check if this is indirect destructuring from an inject variable
            // e.g., const state = inject('state'); const { count } = state;
            let indirect_inject_var = declarator.init.as_ref().and_then(|init| {
                if let Expression::Identifier(id) = init {
                    let var_name = CompactString::new(id.name.as_str());
                    if result.inject_var_names.contains(&var_name) {
                        return Some((var_name, id.span.start));
                    }
                }
                None
            });

            // Check if this is destructuring from a reactive variable
            // e.g., const state = reactive({...}); const { count } = state;
            let reactive_destructure_var = declarator.init.as_ref().and_then(|init| {
                if let Expression::Identifier(id) = init {
                    let var_name = CompactString::new(id.name.as_str());
                    if result.reactivity.is_reactive(var_name.as_str()) {
                        return Some((var_name, id.span.start, id.span.end));
                    }
                }
                None
            });

            // Check if this is destructuring directly from reactive() or ref().value
            // e.g., const { count } = reactive({ count: 0 })
            let direct_reactive_call = declarator.init.as_ref().and_then(|init| {
                let call = extract_call_expression(init)?;
                if let Expression::Identifier(id) = &call.callee {
                    let name = id.name.as_str();
                    if matches!(name, "reactive" | "shallowReactive") {
                        return Some((CompactString::new(name), call.span.start, call.span.end));
                    }
                }
                None
            });

            // If inject(), track it with ObjectDestructure pattern
            if let Some(call) = inject_call {
                // Extract destructured property names
                let mut destructured_props: Vec<CompactString> = Vec::new();
                for prop in obj.properties.iter() {
                    if let Some(name) = get_binding_pattern_name(&prop.value) {
                        destructured_props.push(CompactString::new(&name));
                    }
                }

                // Extract inject key
                if let Some(key) = call
                    .arguments
                    .first()
                    .and_then(|arg| extract_provide_key(arg, source))
                {
                    result.provide_inject.add_inject(
                        key,
                        CompactString::new("(destructured)"),
                        call.arguments
                            .get(1)
                            .map(|arg| CompactString::new(extract_argument_source(arg, source))),
                        None,
                        InjectPattern::ObjectDestructure(destructured_props.clone()),
                        None,
                        call.span.start,
                        call.span.end,
                    );
                }
            } else if let Some((inject_var, offset)) = indirect_inject_var {
                // Indirect destructuring: const { count } = injectVar
                let mut destructured_props: Vec<CompactString> = Vec::new();
                for prop in obj.properties.iter() {
                    if let Some(name) = get_binding_pattern_name(&prop.value) {
                        destructured_props.push(CompactString::new(&name));
                    }
                }

                // Find the original inject entry and update it with indirect destructure info
                // We need to record this as a new pattern variant
                result.provide_inject.add_indirect_destructure(
                    inject_var.clone(),
                    destructured_props,
                    offset,
                );
            } else if let Some((source_name, start, end)) = reactive_destructure_var {
                // Destructuring reactive variable: const { count } = state
                let mut destructured_props: Vec<CompactString> = Vec::new();
                for prop in obj.properties.iter() {
                    if let Some(name) = get_binding_pattern_name(&prop.value) {
                        destructured_props.push(CompactString::new(&name));
                    }
                }
                result
                    .reactivity
                    .record_destructure(source_name, destructured_props, start, end);
            } else if let Some((fn_name, start, end)) = direct_reactive_call {
                // Direct destructuring: const { count } = reactive({ count: 0 })
                let mut destructured_props: Vec<CompactString> = Vec::new();
                for prop in obj.properties.iter() {
                    if let Some(name) = get_binding_pattern_name(&prop.value) {
                        destructured_props.push(CompactString::new(&name));
                    }
                }
                use crate::reactivity::{ReactivityLoss, ReactivityLossKind};
                result.reactivity.add_loss(ReactivityLoss {
                    kind: ReactivityLossKind::ReactiveDestructure {
                        source_name: fn_name,
                        destructured_props,
                    },
                    start,
                    end,
                });
            }

            // If defineProps, process it first to extract prop definitions
            if is_define_props {
                if let Some(Expression::CallExpression(call)) = &declarator.init {
                    process_call_expression(result, call, source);
                }
            }

            // Track props destructure bindings
            let mut props_destructure = if is_define_props {
                Some(PropsDestructuredBindings::default())
            } else {
                None
            };

            // Handle object destructuring
            for prop in obj.properties.iter() {
                // Get the key (prop name in defineProps)
                let key_name = match &prop.key {
                    PropertyKey::StaticIdentifier(id) => Some(id.name.as_str()),
                    PropertyKey::StringLiteral(s) => Some(s.value.as_str()),
                    _ => None,
                };

                if let Some(local_name) = get_binding_pattern_name(&prop.value) {
                    // If destructuring from defineProps, use Props binding type
                    let binding_type = if is_define_props {
                        BindingType::Props
                    } else {
                        infer_destructure_binding_type(kind, declarator.init.as_ref())
                    };
                    result.bindings.add(local_name.as_str(), binding_type);

                    // Track destructure binding
                    if let Some(ref mut destructure) = props_destructure {
                        let key = key_name
                            .map(CompactString::new)
                            .unwrap_or_else(|| CompactString::new(&local_name));

                        // Extract default value if present (assignment pattern)
                        let default_value = if prop.shorthand {
                            // Check if the value is an assignment pattern with default
                            if let BindingPattern::AssignmentPattern(assign) = &prop.value {
                                Some(CompactString::new(
                                    &source[assign.right.span().start as usize
                                        ..assign.right.span().end as usize],
                                ))
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        destructure.insert(key, CompactString::new(&local_name), default_value);
                    }
                }
            }

            // Handle rest element
            if let Some(rest) = &obj.rest {
                if let Some(name) = get_binding_pattern_name(&rest.argument) {
                    let binding_type = if is_define_props {
                        BindingType::Props
                    } else {
                        infer_destructure_binding_type(kind, declarator.init.as_ref())
                    };
                    result.bindings.add(name.as_str(), binding_type);

                    // Track rest binding
                    if let Some(ref mut destructure) = props_destructure {
                        destructure.rest_id = Some(CompactString::new(&name));
                    }
                }
            }

            // Set props destructure in macro tracker
            if let Some(destructure) = props_destructure {
                if !destructure.is_empty() {
                    result.macros.set_props_destructure(destructure);
                }
            }
        }

        BindingPattern::ArrayPattern(arr) => {
            // Handle array destructuring
            let arr_binding_type = infer_destructure_binding_type(kind, declarator.init.as_ref());
            for elem in arr.elements.iter().flatten() {
                if let Some(name) = get_binding_pattern_name(elem) {
                    result.bindings.add(name.as_str(), arr_binding_type);
                }
            }
            if let Some(rest) = &arr.rest {
                if let Some(name) = get_binding_pattern_name(&rest.argument) {
                    result.bindings.add(name.as_str(), arr_binding_type);
                }
            }
        }

        BindingPattern::AssignmentPattern(assign) => {
            if let Some(name) = get_binding_pattern_name(&assign.left) {
                let binding_type = get_binding_type_from_kind(kind);
                result.bindings.add(name.as_str(), binding_type);
            }
        }
    }
}
