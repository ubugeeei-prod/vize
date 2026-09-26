//! The production lattice verdict and every loss payload cross unchanged.
#![expect(clippy::disallowed_macros, reason = "JSON uses serde macros")]
use serde_json::{Value, json};
use vize_croquis::facts::{ReactivityFact, ReactivityKey};
use vize_croquis::reactivity::{ReactiveKind, ReactivityLossKind};
use vize_impeto::lattice::EffectKind;
fn strings(values: &[vize_l0::String]) -> Vec<&str> {
    values.iter().map(|value| value.as_str()).collect()
}
pub fn project((key, value): (&ReactivityKey, &ReactivityFact)) -> Value {
    let key = match key {
        ReactivityKey::Source(id) => format!("source:{id}"),
        ReactivityKey::Loss(id) => format!("loss:{id}"),
    };
    let value = match value {
        ReactivityFact::Source(value) => {
            let kind = match value.kind {
                ReactiveKind::Ref => "ref",
                ReactiveKind::ShallowRef => "shallow-ref",
                ReactiveKind::Reactive => "reactive",
                ReactiveKind::ShallowReactive => "shallow-reactive",
                ReactiveKind::Computed => "computed",
                ReactiveKind::Readonly => "readonly",
                ReactiveKind::ShallowReadonly => "shallow-readonly",
                ReactiveKind::ToRef => "to-ref",
                ReactiveKind::ToRefs => "to-refs",
            };
            let effects: Vec<_> = [
                EffectKind::Freeze,
                EffectKind::Capture,
                EffectKind::ReadProp,
                EffectKind::ReadReactive,
                EffectKind::MutateLocal,
                EffectKind::MutateGlobal,
                EffectKind::CallUnknown,
                EffectKind::Allocate,
            ]
            .into_iter()
            .filter(|effect| value.effects.contains(*effect))
            .map(EffectKind::as_str)
            .collect();
            json!({"kind": "source", "name": value.name.as_str(), "sourceKind": kind, "declarationOffset": value.declaration_offset, "class": value.class.as_str(), "verdict": value.verdict.as_str(), "effects": effects})
        }
        ReactivityFact::Loss(value) => {
            json!({"kind": "loss", "loss": loss(&value.kind), "start": value.start, "end": value.end})
        }
    };
    json!([key, value])
}
fn loss(value: &ReactivityLossKind) -> Value {
    match value {
        ReactivityLossKind::ReactiveDestructure {
            source_name,
            destructured_props,
        } => {
            json!({"kind": "reactive-destructure", "sourceName": source_name.as_str(), "destructuredProps": strings(destructured_props)})
        }
        ReactivityLossKind::RefValueDestructure {
            source_name,
            destructured_props,
        } => {
            json!({"kind": "ref-value-destructure", "sourceName": source_name.as_str(), "destructuredProps": strings(destructured_props)})
        }
        ReactivityLossKind::RefValueExtract {
            source_name,
            target_name,
        } => {
            json!({"kind": "ref-value-extract", "sourceName": source_name.as_str(), "targetName": target_name.as_str()})
        }
        ReactivityLossKind::ReactivePropertyExtract {
            source_name,
            prop_name,
            target_name,
        } => {
            json!({"kind": "reactive-property-extract", "sourceName": source_name.as_str(), "propName": prop_name.as_str(), "targetName": target_name.as_str()})
        }
        ReactivityLossKind::PropsDestructure { destructured_props } => {
            json!({"kind": "props-destructure", "destructuredProps": strings(destructured_props)})
        }
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name,
        } => {
            json!({"kind": "function-argument-extract", "sourceName": source_name.as_str(), "argumentName": argument_name.as_str(), "calleeName": callee_name.as_str()})
        }
        ReactivityLossKind::GetterCallExtract {
            context_name,
            getter_name,
            target_name,
            callee_name,
            source_name,
        } => {
            json!({"kind": "getter-call-extract", "contextName": context_name.as_str(), "getterName": getter_name.as_str(), "targetName": target_name.as_str(), "calleeName": callee_name.as_str(), "sourceName": source_name.as_str()})
        }
        ReactivityLossKind::PlainValueAlias {
            source_name,
            alias_name,
            target_name,
        } => {
            json!({"kind": "plain-value-alias", "sourceName": source_name.as_str(), "aliasName": alias_name.as_str(), "targetName": target_name.as_str()})
        }
        ReactivityLossKind::ReactiveSpread { source_name } => {
            json!({"kind": "reactive-spread", "sourceName": source_name.as_str()})
        }
        ReactivityLossKind::ReactiveReassign { source_name } => {
            json!({"kind": "reactive-reassign", "sourceName": source_name.as_str()})
        }
    }
}
