use super::{
    Argument, CallExpression, ClientOnlyScopeData, ClosureScopeData, CompactString, Expression,
    ScriptParseResult, Statement, detect_call_argument_reactivity_loss, detect_provide_inject_call,
    detect_race_condition_call, extract_function_params_with_occurrences, is_client_only_hook,
    walk_expression, walk_statement,
};

/// Walk call expression arguments to find callbacks
#[inline]
pub(in crate::script_parser) fn walk_call_arguments(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    source: &str,
) {
    super::super::super::ssr_calls::note_call(result, call);
    result.refuse_type_arguments(call.type_arguments.as_deref());
    // First, walk the callee (might be a chained call like foo.bar().baz())
    walk_expression(result, &call.callee, source);

    // Check for provide/inject calls
    detect_provide_inject_call(result, call, source);
    if !result.skip_diagnostics {
        detect_race_condition_call(result, call, source);
    }
    detect_call_argument_reactivity_loss(result, call, source);
    super::super::super::extract::check_reactive_plain_call_mutation(result, call, source);

    // onScopeDispose is cleanup for onMounted, but it is not a client-only
    // lifecycle hook for race tracking.
    let hook_name = if let Expression::Identifier(id) = &call.callee {
        let name = id.name.as_str();
        (is_client_only_hook(name) || name == "onScopeDispose").then_some(name)
    } else {
        None
    };
    let mut lifecycle_callback_scope_recorded = false;

    // Then walk each argument
    for arg in call.arguments.iter() {
        match arg {
            Argument::SpreadElement(spread) => {
                super::super::super::extract::check_reactive_spread_expression(
                    result,
                    &spread.argument,
                    source,
                    spread.span.start,
                    spread.span.end,
                );
                walk_expression(result, &spread.argument, source);
            }
            _ => {
                if let Some(expr) = arg.as_expression() {
                    // If this is a lifecycle hook and the argument is a function,
                    // wrap it in a ClientOnly scope
                    if let Some(name) = hook_name {
                        super::super::super::occurrences::refuse_expression_type_reads(
                            result, expr,
                        );
                        match expr {
                            Expression::ArrowFunctionExpression(arrow) => {
                                lifecycle_callback_scope_recorded = true;
                                // Enter client-only scope
                                result.scopes.enter_client_only_scope(
                                    client_only_data(
                                        name,
                                        source,
                                        arrow.span.start,
                                        arrow.span.end,
                                    ),
                                    call.span.start,
                                    call.span.end,
                                );

                                // Now create the closure scope inside the client-only scope
                                let params =
                                    extract_function_params_with_occurrences(result, &arrow.params);
                                result.scopes.enter_closure_scope(
                                    ClosureScopeData {
                                        name: None,
                                        param_names: params,
                                        is_arrow: true,
                                        is_async: arrow.r#async,
                                        is_generator: false,
                                    },
                                    arrow.span.start,
                                    arrow.span.end,
                                );
                                result.install_parameter_occurrences();

                                // Walk the body
                                if arrow.expression {
                                    if let Some(Statement::ExpressionStatement(expr_stmt)) =
                                        arrow.body.statements.first()
                                    {
                                        walk_expression(result, &expr_stmt.expression, source);
                                    }
                                } else {
                                    for stmt in arrow.body.statements.iter() {
                                        walk_statement(result, stmt, source);
                                    }
                                }

                                result.scopes.exit_scope(); // Exit closure scope
                                result.scopes.exit_scope(); // Exit client-only scope
                                continue;
                            }
                            Expression::FunctionExpression(func) => {
                                if func.id.is_some() {
                                    result.refuse_occurrences();
                                }
                                lifecycle_callback_scope_recorded = true;
                                // Enter client-only scope
                                result.scopes.enter_client_only_scope(
                                    client_only_data(name, source, func.span.start, func.span.end),
                                    call.span.start,
                                    call.span.end,
                                );

                                // Create closure scope inside client-only scope
                                let params =
                                    extract_function_params_with_occurrences(result, &func.params);
                                let fn_name = func
                                    .id
                                    .as_ref()
                                    .map(|id| CompactString::new(id.name.as_str()));

                                result.scopes.enter_closure_scope(
                                    ClosureScopeData {
                                        name: fn_name,
                                        param_names: params,
                                        is_arrow: false,
                                        is_async: func.r#async,
                                        is_generator: func.generator,
                                    },
                                    func.span.start,
                                    func.span.end,
                                );
                                result.install_parameter_occurrences();

                                if let Some(body) = &func.body {
                                    for stmt in body.statements.iter() {
                                        walk_statement(result, stmt, source);
                                    }
                                }

                                result.scopes.exit_scope(); // Exit closure scope
                                result.scopes.exit_scope(); // Exit client-only scope
                                continue;
                            }
                            _ => {}
                        }
                    }
                    walk_expression(result, expr, source);
                }
            }
        }
    }

    if let Some(name) = hook_name
        && !lifecycle_callback_scope_recorded
    {
        result.scopes.enter_client_only_scope(
            client_only_data(name, source, call.span.start, call.span.end),
            call.span.start,
            call.span.end,
        );
        result.scopes.exit_scope();
    }
}

fn client_only_data(name: &str, source: &str, start: u32, end: u32) -> ClientOnlyScopeData {
    // Keep the scan in this function so the call walker's codegen stays the
    // shape that fits the instruction ceilings. The bool is not stored:
    // `ClientOnlyScopeData` cannot grow a field in a patch.
    let acquires = name == "onMounted" && callback_acquires_resource(source, start, end);
    std::hint::black_box(acquires);
    ClientOnlyScopeData {
        hook_name: CompactString::new(name),
    }
}

fn callback_acquires_resource(source: &str, start: u32, end: u32) -> bool {
    let Some(text) = source.get(start as usize..end as usize) else {
        return false;
    };
    const NAMES: &[&str] = &[
        "addEventListener",
        "setInterval",
        "setTimeout",
        "requestAnimationFrame",
        "subscribe",
    ];
    NAMES.iter().any(|name| contains_ident(text, name))
        || text.contains("new WebSocket")
        || text.contains("new EventSource")
        || text.contains("new Worker")
        || text.contains("new IntersectionObserver")
        || text.contains("new ResizeObserver")
        || text.contains("new MutationObserver")
        || text.contains(".observe(")
}

fn contains_ident(haystack: &str, ident: &str) -> bool {
    let bytes = haystack.as_bytes();
    let needle = ident.as_bytes();
    if needle.is_empty() {
        return false;
    }
    let mut start = 0;
    while start + needle.len() <= bytes.len() {
        if bytes.get(start..start + needle.len()) == Some(needle) {
            let before_ok = start == 0
                || bytes
                    .get(start - 1)
                    .is_some_and(|byte| !is_ident_byte(*byte));
            let after = start + needle.len();
            let after_ok = bytes.get(after).is_none_or(|byte| !is_ident_byte(*byte));
            if before_ok && after_ok {
                return true;
            }
        }
        start += 1;
    }
    false
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

#[inline(always)]
pub(super) fn identifier_might_be_browser_global(name: &str) -> bool {
    let bytes = name.as_bytes();
    let len = bytes.len();
    // Names shorter than `alert` or longer than `requestAnimationFrame`
    // cannot be one of the reported globals. Most script identifiers miss here.
    if !(5..=21).contains(&len) {
        return false;
    }
    let Some((&first, &second)) = bytes.first().zip(bytes.get(1)) else {
        return false;
    };
    // Each browser global is unique on (length, first byte, second byte), so
    // the hot walk rejects ordinary identifiers before any string compare.
    matches!(
        (len, first, second),
        (5, b'a', b'l')
            | (5, b'f', b'e')
            | (6, b'p', b'r')
            | (6, b'w', b'i')
            | (7, b'c', b'o')
            | (7, b'h', b'i')
            | (8, b'd', b'o')
            | (8, b'l', b'o')
            | (9, b'W', b'e')
            | (9, b'n', b'a')
            | (10, b'm', b'a')
            | (12, b'l', b'o')
            | (14, b'X', b'M')
            | (14, b's', b'e')
            | (14, b'R', b'e')
            | (16, b'M', b'u')
            | (16, b'g', b'e')
            | (20, b'I', b'n')
            | (20, b'c', b'a')
            | (21, b'r', b'e')
    )
}

pub(super) fn note_script_browser_global(result: &mut ScriptParseResult, name: &str, offset: u32) {
    if !is_browser_global(name) {
        return;
    }
    let shadowed = result.scopes.lookup(name).is_some_and(|(scope, _)| {
        !matches!(
            scope.kind,
            crate::scope::ScopeKind::JsGlobalUniversal
                | crate::scope::ScopeKind::JsGlobalBrowser
                | crate::scope::ScopeKind::JsGlobalNode
                | crate::scope::ScopeKind::JsGlobalDeno
                | crate::scope::ScopeKind::JsGlobalBun
                | crate::scope::ScopeKind::VueGlobal
        )
    });
    if shadowed {
        return;
    }
    result
        .setup_context
        .note_browser_global(CompactString::new(name), offset);
}

fn is_browser_global(name: &str) -> bool {
    matches!(
        name,
        "window"
            | "document"
            | "navigator"
            | "localStorage"
            | "sessionStorage"
            | "location"
            | "history"
            | "fetch"
            | "XMLHttpRequest"
            | "WebSocket"
            | "IntersectionObserver"
            | "ResizeObserver"
            | "MutationObserver"
            | "requestAnimationFrame"
            | "cancelAnimationFrame"
            | "getComputedStyle"
            | "matchMedia"
            | "alert"
            | "confirm"
            | "prompt"
    )
}
