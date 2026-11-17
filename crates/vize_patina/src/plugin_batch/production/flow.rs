//! Provide/inject and race records; variant payloads remain explicit.
#![expect(clippy::disallowed_macros, reason = "JSON uses serde macros")]
use serde_json::{Value, json};
use vize_croquis::facts::{ProvideInjectFact, ProvideInjectKey};
use vize_croquis::provide::{InjectPattern, ProvideKey};
use vize_croquis::race::{RaceConditionRisk, RaceConditionRiskKind};

fn strings(values: &[vize_l0::String]) -> Vec<&str> {
    values.iter().map(|value| value.as_str()).collect()
}
fn key(value: &ProvideKey) -> Value {
    match value {
        ProvideKey::String(name) => json!({"kind": "string", "name": name.as_str()}),
        ProvideKey::Symbol(name) => json!({"kind": "symbol", "name": name.as_str()}),
    }
}
fn pattern(value: &InjectPattern) -> Value {
    match value {
        InjectPattern::Simple => json!({"kind": "simple"}),
        InjectPattern::ObjectDestructure(props) => {
            json!({"kind": "object", "props": strings(props)})
        }
        InjectPattern::ArrayDestructure(props) => json!({"kind": "array", "props": strings(props)}),
        InjectPattern::IndirectDestructure {
            inject_var,
            props,
            offset,
        } => {
            json!({"kind": "indirect", "injectVar": inject_var.as_str(), "props": strings(props), "offset": offset})
        }
    }
}
pub fn provide((id, fact): (&ProvideInjectKey, &ProvideInjectFact)) -> Value {
    let name = match id {
        ProvideInjectKey::Provide(id) => format!("provide:{id}"),
        ProvideInjectKey::Inject(id) => format!("inject:{id}"),
        ProvideInjectKey::Composable(id) => format!("composable:{id}"),
    };
    let value = match fact {
        ProvideInjectFact::Provide(value) => {
            json!({"kind": "provide", "id": value.id.as_u32(), "key": key(&value.key), "value": value.value.as_str(), "valueType": value.value_type.as_deref(), "fromComposable": value.from_composable.as_deref(), "start": value.start, "end": value.end})
        }
        ProvideInjectFact::Inject(value) => {
            json!({"kind": "inject", "key": key(&value.key), "localName": value.local_name.as_str(), "defaultValue": value.default_value.as_deref(), "expectedType": value.expected_type.as_deref(), "pattern": pattern(&value.pattern), "fromComposable": value.from_composable.as_deref(), "start": value.start, "end": value.end})
        }
        ProvideInjectFact::Composable(value) => {
            json!({"kind": "composable", "name": value.name.as_str(), "source": value.source.as_str(), "localName": value.local_name.as_deref(), "usesProvide": value.uses_provide, "usesInject": value.uses_inject, "usesReactivity": value.uses_reactivity, "start": value.start, "end": value.end})
        }
    };
    json!([name, value])
}
pub fn race((id, value): (&u32, &RaceConditionRisk)) -> Value {
    let kind = match &value.kind {
        RaceConditionRiskKind::AsyncWatcherMutation {
            watcher_name,
            async_operation,
            mutated_targets,
        } => {
            json!({"kind": "async-watcher", "watcherName": watcher_name.as_str(), "asyncOperation": async_operation.as_str(), "mutatedTargets": strings(mutated_targets)})
        }
        RaceConditionRiskKind::AsyncWatchEffect {
            async_operation,
            mutated_targets,
        } => {
            json!({"kind": "async-watch-effect", "asyncOperation": async_operation.as_str(), "mutatedTargets": strings(mutated_targets)})
        }
        RaceConditionRiskKind::AsyncLifecycleMutation {
            hook_name,
            async_operation,
            mutated_targets,
        } => {
            json!({"kind": "async-lifecycle", "hookName": hook_name.as_str(), "asyncOperation": async_operation.as_str(), "mutatedTargets": strings(mutated_targets)})
        }
        RaceConditionRiskKind::ScheduledMutation {
            scheduler_name,
            mutated_targets,
        } => {
            json!({"kind": "scheduled", "schedulerName": scheduler_name.as_str(), "mutatedTargets": strings(mutated_targets)})
        }
        RaceConditionRiskKind::PromiseContinuationMutation {
            async_operation,
            mutated_targets,
        } => {
            json!({"kind": "promise-continuation", "asyncOperation": async_operation.as_str(), "mutatedTargets": strings(mutated_targets)})
        }
    };
    json!([id, {"kind": kind, "start": value.start, "end": value.end}])
}
