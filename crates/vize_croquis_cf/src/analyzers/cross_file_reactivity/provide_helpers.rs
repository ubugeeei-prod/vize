use vize_carton::{CompactString, cstr};
use vize_croquis::provide::ProvideKey;
use vize_croquis::reactivity::ReactiveKind;

pub(super) fn provided_value_reactive_kind(
    analysis: &vize_croquis::Croquis,
    value: &str,
) -> Option<ReactiveKind> {
    let value = value.trim();

    if let Some(source) = analysis
        .reactivity
        .sources()
        .iter()
        .find(|source| source.name.as_str() == value)
    {
        return Some(source.kind);
    }

    let callee = value
        .split_once('(')
        .map(|(callee, _)| callee.trim())
        .unwrap_or_default();

    match callee {
        "ref" => Some(ReactiveKind::Ref),
        "shallowRef" => Some(ReactiveKind::ShallowRef),
        "reactive" => Some(ReactiveKind::Reactive),
        "shallowReactive" => Some(ReactiveKind::ShallowReactive),
        "computed" => Some(ReactiveKind::Computed),
        "readonly" => Some(ReactiveKind::Readonly),
        "shallowReadonly" => Some(ReactiveKind::ShallowReadonly),
        "toRef" => Some(ReactiveKind::ToRef),
        "toRefs" => Some(ReactiveKind::ToRefs),
        _ => None,
    }
}

pub(super) fn provide_key_display(key: &ProvideKey) -> CompactString {
    match key {
        ProvideKey::String(s) | ProvideKey::Symbol(s) => s.clone(),
    }
}

pub(super) fn provide_key_identity(key: &ProvideKey) -> CompactString {
    match key {
        ProvideKey::String(s) => cstr!("string:{s}"),
        ProvideKey::Symbol(s) => cstr!("symbol:{s}"),
    }
}
