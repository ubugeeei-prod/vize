//! TS-34 spec for `Bindings` over `<script setup>`: the classification
//! rules as data and their naive evaluator.
//!
//! ```text
//! binding(n, t)   :- decl(n, form), rule(form, t)                 -- last declaration wins
//! binding(p, Props) :- macro_prop(p), ¬binding(p, _)              -- an authored name owns p
//! span(n, s)      :- decl(n, _, s)                                -- last declaration wins
//! script_setup()  :- the artifact has a <script setup> block
//! ```
//!
//! A macro is recognized by its own spelling; any other call's callee first
//! resolves through `alias(a, api) :- decl(a, const a = api)` declared earlier
//! in the script, as production resolves `const r = ref`.

use std::collections::BTreeMap;

use vize_carton::CompactString;
use vize_relief::BindingType;

use super::bindings_extract::{Decl, Form, Init, PatternSource, VarKind};
use crate::facts::FactTable;
use crate::facts::bindings::{BindingFact, BindingKey, Bindings};

/// What a rule yields for a declared name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Yield {
    /// A binding of this type.
    Type(BindingType),
    /// The keyword's default: `SetupConst` for `const`, `SetupLet` for
    /// `let`/`var`.
    ByKind,
    /// No binding (the name keeps its span only).
    Nothing,
}

/// A declaration class the rules key on — [`Form`] with the call callee
/// already resolved against [`MACRO_RULES`] and [`REACTIVITY_RULES`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    NamedImport,
    DefaultImport,
    TypeImport,
    Declaration,
    /// A simple variable initialized by a [`MACRO_RULES`] macro.
    Macro(Yield),
    /// A simple variable initialized by a [`REACTIVITY_RULES`] constructor.
    Reactivity(BindingType),
    ConstAbsent,
    ConstLiteral,
    ConstFunctionOrAggregate,
    ConstOther,
    Mutable,
    PropsDestructure {
        direct: bool,
    },
    ModelArray,
    ConstPatternFunction,
    ConstPatternOther,
    ConstPatternAbsent,
    MutablePattern,
}

/// Every declaration class and what it yields, scanned first-match.
pub const DECLARATION_RULES: &[(Class, Yield)] = &[
    (Class::NamedImport, Yield::Type(BindingType::SetupMaybeRef)),
    (Class::DefaultImport, Yield::Type(BindingType::SetupConst)),
    (Class::TypeImport, Yield::Nothing),
    (Class::Declaration, Yield::Type(BindingType::SetupConst)),
    (Class::ConstAbsent, Yield::Type(BindingType::SetupConst)),
    (Class::ConstLiteral, Yield::Type(BindingType::LiteralConst)),
    (
        Class::ConstFunctionOrAggregate,
        Yield::Type(BindingType::SetupConst),
    ),
    (Class::ConstOther, Yield::Type(BindingType::SetupMaybeRef)),
    (Class::Mutable, Yield::Type(BindingType::SetupLet)),
    (
        Class::PropsDestructure { direct: true },
        Yield::Type(BindingType::Props),
    ),
    (Class::PropsDestructure { direct: false }, Yield::Nothing),
    (Class::ModelArray, Yield::Type(BindingType::SetupMaybeRef)),
    (
        Class::ConstPatternFunction,
        Yield::Type(BindingType::SetupConst),
    ),
    (
        Class::ConstPatternOther,
        Yield::Type(BindingType::SetupMaybeRef),
    ),
    (
        Class::ConstPatternAbsent,
        Yield::Type(BindingType::SetupConst),
    ),
    (Class::MutablePattern, Yield::Type(BindingType::SetupLet)),
];

/// Compiler macros, recognized by their own spelling, and what the
/// variable they initialize binds as.
pub const MACRO_RULES: &[(&str, Yield)] = &[
    ("defineProps", Yield::Type(BindingType::SetupReactiveConst)),
    ("withDefaults", Yield::Type(BindingType::SetupReactiveConst)),
    ("defineModel", Yield::Type(BindingType::SetupRef)),
    ("defineEmits", Yield::ByKind),
    ("defineExpose", Yield::ByKind),
    ("defineOptions", Yield::ByKind),
    ("defineSlots", Yield::ByKind),
    ("defineArt", Yield::ByKind),
];

/// Reactivity constructors, recognized through [`ALIAS_APIS`] aliases, and
/// what the variable they initialize binds as, whatever its keyword.
pub const REACTIVITY_RULES: &[(&str, BindingType)] = &[
    ("ref", BindingType::SetupRef),
    ("shallowRef", BindingType::SetupRef),
    ("computed", BindingType::SetupRef),
    ("toRef", BindingType::SetupRef),
    ("toRefs", BindingType::SetupRef),
    ("customRef", BindingType::SetupRef),
    ("useTemplateRef", BindingType::SetupRef),
    ("reactive", BindingType::SetupReactiveConst),
    ("shallowReactive", BindingType::SetupReactiveConst),
    ("readonly", BindingType::SetupReactiveConst),
    ("shallowReadonly", BindingType::SetupReactiveConst),
];

/// Vue APIs a bare `const a = api` aliases: a later call of `a` resolves to
/// `api` before [`REACTIVITY_RULES`] is consulted.
pub const ALIAS_APIS: &[&str] = &[
    "inject",
    "provide",
    "ref",
    "shallowRef",
    "reactive",
    "shallowReactive",
    "computed",
    "readonly",
    "shallowReadonly",
    "toRef",
    "toRefs",
    "toValue",
    "toRaw",
    "isRef",
    "isReactive",
    "isReadonly",
    "isProxy",
    "unref",
    "triggerRef",
    "customRef",
    "markRaw",
    "effectScope",
    "getCurrentScope",
    "onScopeDispose",
    "watch",
    "watchEffect",
    "watchPostEffect",
    "watchSyncEffect",
    "onMounted",
    "onUnmounted",
    "onBeforeMount",
    "onBeforeUnmount",
    "onUpdated",
    "onBeforeUpdate",
    "onActivated",
    "onDeactivated",
    "onErrorCaptured",
    "onRenderTracked",
    "onRenderTriggered",
    "onServerPrefetch",
    "defineComponent",
    "defineAsyncComponent",
    "getCurrentInstance",
    "nextTick",
    "InjectionKey",
];

fn macro_rule(callee: &str) -> Option<Yield> {
    MACRO_RULES
        .iter()
        .find(|(name, _)| *name == callee)
        .map(|(_, rule)| *rule)
}

fn reactivity_rule(callee: &str) -> Option<BindingType> {
    REACTIVITY_RULES
        .iter()
        .find(|(name, _)| *name == callee)
        .map(|(_, kind)| *kind)
}

fn class_of(form: &Form, aliases: &BTreeMap<CompactString, &'static str>) -> Class {
    match form {
        Form::NamedImport => Class::NamedImport,
        Form::DefaultImport => Class::DefaultImport,
        Form::TypeImport => Class::TypeImport,
        Form::Declaration => Class::Declaration,
        Form::Simple(kind, init) => {
            if let Init::Call(callee) = init {
                if let Some(rule) = macro_rule(callee) {
                    return Class::Macro(rule);
                }
                let resolved = aliases.get(callee).copied().unwrap_or(callee.as_str());
                if let Some(kind) = reactivity_rule(resolved) {
                    return Class::Reactivity(kind);
                }
            }
            match (kind, init) {
                (VarKind::Mutable, _) => Class::Mutable,
                (VarKind::Const, Init::Absent) => Class::ConstAbsent,
                (VarKind::Const, Init::Literal) => Class::ConstLiteral,
                (VarKind::Const, Init::Function | Init::Aggregate) => {
                    Class::ConstFunctionOrAggregate
                }
                (VarKind::Const, _) => Class::ConstOther,
            }
        }
        Form::Pattern {
            kind,
            source,
            direct,
        } => match (source, kind) {
            (PatternSource::DefineProps, _) => Class::PropsDestructure { direct: *direct },
            (PatternSource::DefineModel, _) => Class::ModelArray,
            (_, VarKind::Mutable) => Class::MutablePattern,
            (PatternSource::Function, VarKind::Const) => Class::ConstPatternFunction,
            (PatternSource::Absent, VarKind::Const) => Class::ConstPatternAbsent,
            (PatternSource::Other, VarKind::Const) => Class::ConstPatternOther,
        },
    }
}

fn yield_of(class: Class) -> Yield {
    match class {
        Class::Macro(rule) => return rule,
        Class::Reactivity(kind) => return Yield::Type(kind),
        _ => {}
    }
    DECLARATION_RULES
        .iter()
        .find(|(candidate, _)| *candidate == class)
        .map_or(Yield::Nothing, |(_, rule)| *rule)
}

fn kind_of(form: &Form) -> VarKind {
    match form {
        Form::Simple(kind, _) | Form::Pattern { kind, .. } => *kind,
        _ => VarKind::Const,
    }
}

/// Evaluate the rules over one script's declarations.
///
/// `macro_props` is the macro tracker's prop-name relation; `script_setup`
/// the artifact-level marker.
#[must_use]
pub fn evaluate(
    decls: &[Decl],
    macro_props: &[CompactString],
    script_setup: bool,
) -> FactTable<Bindings> {
    let mut aliases: BTreeMap<CompactString, &'static str> = BTreeMap::new();
    let mut kinds: BTreeMap<CompactString, BindingType> = BTreeMap::new();
    let mut spans: BTreeMap<CompactString, (u32, u32)> = BTreeMap::new();
    for decl in decls {
        let class = class_of(&decl.form, &aliases);
        let binding = match yield_of(class) {
            Yield::Type(kind) => Some(kind),
            Yield::ByKind => Some(match kind_of(&decl.form) {
                VarKind::Const => BindingType::SetupConst,
                VarKind::Mutable => BindingType::SetupLet,
            }),
            Yield::Nothing => None,
        };
        if let Some(kind) = binding {
            kinds.insert(decl.name.clone(), kind);
        }
        if let Some(span) = decl.span {
            spans.insert(decl.name.clone(), span);
        }
        if let Form::Simple(_, Init::Identifier(api)) = &decl.form
            && let Some(canonical) = ALIAS_APIS.iter().find(|name| **name == api.as_str())
        {
            aliases.insert(decl.name.clone(), canonical);
        }
    }
    for prop in macro_props {
        kinds.entry(prop.clone()).or_insert(BindingType::Props);
    }
    let mut facts: BTreeMap<BindingKey, BindingFact> = BTreeMap::new();
    if script_setup {
        facts.insert(BindingKey::ScriptSetup, BindingFact::default());
    }
    for (name, kind) in kinds {
        facts.entry(BindingKey::Name(name)).or_default().kind = Some(kind);
    }
    for (name, span) in spans {
        facts.entry(BindingKey::Name(name)).or_default().span = Some(span);
    }
    facts.into_iter().collect()
}
