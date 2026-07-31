use crate::provide::{InjectPattern, ProvideKey};
use crate::reactivity::{ReactiveKind, ReactivityLossKind};
use vize_carton::{CompactString, String, appends};
use vize_relief::BindingType;

pub(super) fn semantic_id(kind: &str, name: &str, offset: u32) -> CompactString {
    let mut id = String::default();
    appends!(id, kind, #':', name, #'@', @offset);
    CompactString::new(id.as_str())
}

pub(super) fn provide_key_value(key: &ProvideKey) -> CompactString {
    match key {
        ProvideKey::String(value) | ProvideKey::Symbol(value) => value.clone(),
    }
}

pub(super) fn provide_key_kind(key: &ProvideKey) -> &'static str {
    match key {
        ProvideKey::String(_) => "string",
        ProvideKey::Symbol(_) => "symbol",
    }
}

pub(super) fn inject_pattern_kind(pattern: &InjectPattern) -> &'static str {
    match pattern {
        InjectPattern::Simple => "simple",
        InjectPattern::ObjectDestructure(_) => "objectDestructure",
        InjectPattern::ArrayDestructure(_) => "arrayDestructure",
        InjectPattern::IndirectDestructure { .. } => "indirectDestructure",
    }
}

pub(super) fn inject_pattern_names(pattern: &InjectPattern) -> Vec<CompactString> {
    match pattern {
        InjectPattern::Simple => Vec::new(),
        InjectPattern::ObjectDestructure(names) | InjectPattern::ArrayDestructure(names) => {
            names.clone()
        }
        InjectPattern::IndirectDestructure { props, .. } => props.clone(),
    }
}

pub(super) fn binding_kind(kind: BindingType) -> &'static str {
    match kind {
        BindingType::SetupLet => "setupLet",
        BindingType::SetupMaybeRef => "setupMaybeRef",
        BindingType::SetupRef => "setupRef",
        BindingType::SetupReactiveConst => "setupReactiveConst",
        BindingType::SetupConst => "setupConst",
        BindingType::Props => "props",
        BindingType::PropsAliased => "propsAliased",
        BindingType::Data => "data",
        BindingType::Options => "options",
        BindingType::LiteralConst => "literalConst",
        BindingType::JsGlobalUniversal => "jsGlobalUniversal",
        BindingType::JsGlobalBrowser => "jsGlobalBrowser",
        BindingType::JsGlobalNode => "jsGlobalNode",
        BindingType::JsGlobalDeno => "jsGlobalDeno",
        BindingType::JsGlobalBun => "jsGlobalBun",
        BindingType::VueGlobal => "vueGlobal",
        BindingType::ExternalModule => "externalModule",
    }
}

pub(super) fn binding_category(kind: BindingType) -> &'static str {
    match kind {
        BindingType::SetupLet
        | BindingType::SetupMaybeRef
        | BindingType::SetupRef
        | BindingType::SetupReactiveConst
        | BindingType::SetupConst
        | BindingType::LiteralConst => "setup",
        BindingType::Props | BindingType::PropsAliased => "props",
        BindingType::Data => "data",
        BindingType::Options => "options",
        BindingType::JsGlobalUniversal
        | BindingType::JsGlobalBrowser
        | BindingType::JsGlobalNode
        | BindingType::JsGlobalDeno
        | BindingType::JsGlobalBun => "jsGlobal",
        BindingType::VueGlobal => "vueGlobal",
        BindingType::ExternalModule => "externalModule",
    }
}

pub(super) fn template_expression_kind(kind: super::super::TemplateExpressionKind) -> &'static str {
    match kind {
        super::super::TemplateExpressionKind::Interpolation => "interpolation",
        super::super::TemplateExpressionKind::VBind => "vBind",
        super::super::TemplateExpressionKind::VOn => "vOn",
        super::super::TemplateExpressionKind::VIf => "vIf",
        super::super::TemplateExpressionKind::VShow => "vShow",
        super::super::TemplateExpressionKind::VModel => "vModel",
        super::super::TemplateExpressionKind::DynamicDirectiveArgument => {
            "dynamicDirectiveArgument"
        }
        super::super::TemplateExpressionKind::CustomDirective => "customDirective",
    }
}

pub(super) fn reactive_kind_name(kind: ReactiveKind) -> &'static str {
    match kind {
        ReactiveKind::Ref => "ref",
        ReactiveKind::ShallowRef => "shallowRef",
        ReactiveKind::Reactive => "reactive",
        ReactiveKind::ShallowReactive => "shallowReactive",
        ReactiveKind::Computed => "computed",
        ReactiveKind::Readonly => "readonly",
        ReactiveKind::ShallowReadonly => "shallowReadonly",
        ReactiveKind::ToRef => "toRef",
        ReactiveKind::ToRefs => "toRefs",
    }
}

pub(super) fn reactive_kind_category(kind: ReactiveKind) -> &'static str {
    match kind {
        ReactiveKind::Ref
        | ReactiveKind::ShallowRef
        | ReactiveKind::ToRef
        | ReactiveKind::ToRefs => "ref",
        ReactiveKind::Reactive | ReactiveKind::ShallowReactive => "reactive",
        ReactiveKind::Computed => "computed",
        ReactiveKind::Readonly | ReactiveKind::ShallowReadonly => "readonly",
    }
}

pub(super) fn reactivity_loss_kind_name(kind: &ReactivityLossKind) -> &'static str {
    match kind {
        ReactivityLossKind::ReactiveDestructure { .. } => "reactiveDestructure",
        ReactivityLossKind::RefValueDestructure { .. } => "refValueDestructure",
        ReactivityLossKind::RefValueExtract { .. } => "refValueExtract",
        ReactivityLossKind::ReactivePropertyExtract { .. } => "reactivePropertyExtract",
        ReactivityLossKind::PropsDestructure { .. } => "propsDestructure",
        ReactivityLossKind::FunctionArgumentExtract { .. } => "functionArgumentExtract",
        ReactivityLossKind::GetterCallExtract { .. } => "getterCallExtract",
        ReactivityLossKind::PlainValueAlias { .. } => "plainValueAlias",
        ReactivityLossKind::ReactiveSpread { .. } => "reactiveSpread",
        ReactivityLossKind::ReactiveReassign { .. } => "reactiveReassign",
    }
}
