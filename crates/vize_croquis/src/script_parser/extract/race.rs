mod expression;
mod scan;

use oxc_ast::ast::{Argument, CallExpression, Expression};

use crate::race::RaceConditionRiskKind;
use vize_carton::CompactString;

use super::super::ScriptParseResult;
use scan::scan_callback_for_race;

pub fn detect_race_condition_call(
    result: &mut ScriptParseResult,
    call: &CallExpression<'_>,
    _source: &str,
) {
    let Some(callee_name) = super::common::resolved_call_name(result, call) else {
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
        name if super::super::walk::is_client_only_hook(name) => {
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

/// Record race risk for a watcher callback.
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
