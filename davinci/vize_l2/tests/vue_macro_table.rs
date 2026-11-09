//! The native Vue macro table answers every lookup exactly as the Croquis
//! table it replaces (#6844). Croquis is only this test's oracle.

use vize_croquis::macros as croquis;
use vize_l2::framework::vue::{
    self, DEFINE_EMITS, DEFINE_EXPOSE, DEFINE_MODEL, DEFINE_OPTIONS, DEFINE_PROPS, DEFINE_SLOTS,
    MACROS, MacroKind, MacroLifecycle, WITH_DEFAULTS,
};

/// Names next to the table that must stay ordinary calls.
const ORDINARY: &[&str] = &[
    "",
    "define",
    "defineprops",
    "DefineProps",
    "definePropsX",
    "defineComponent",
    "defineAsyncComponent",
    "useSlots",
    "useAttrs",
    "defineArt.call",
    "definePageMetaX",
];

fn every_name() -> impl Iterator<Item = &'static str> {
    MACROS
        .iter()
        .map(|row| row.name)
        .chain(croquis::BUILTIN_MACROS.iter().copied())
        .chain(
            croquis::ECOSYSTEM_COMPILE_TIME_MACROS
                .iter()
                .map(|row| row.name),
        )
        .chain(ORDINARY.iter().copied())
}

fn croquis_lifecycle(lifecycle: MacroLifecycle) -> croquis::MacroLifecycle {
    match lifecycle {
        MacroLifecycle::AnalyzeAndErase => croquis::MacroLifecycle::AnalyzeAndErase,
        MacroLifecycle::Expand => croquis::MacroLifecycle::Expand,
        MacroLifecycle::ExtractAndErase => croquis::MacroLifecycle::ExtractAndErase,
    }
}

#[test]
fn every_lookup_matches_croquis() {
    for name in every_name() {
        let row = vue::lookup(name);
        assert_eq!(
            row.map(|row| croquis_lifecycle(row.lifecycle)),
            croquis::macro_lifecycle(name),
            "{name:?}"
        );
        assert_eq!(
            vue::is_builtin_macro(name),
            croquis::is_builtin_macro(name),
            "{name:?}"
        );
        assert_eq!(
            vue::is_runtime_erased_macro(name),
            croquis::is_runtime_erased_macro(name),
            "{name:?}"
        );
        assert_eq!(
            row.is_some_and(|row| row.lifecycle.produces_artifact()),
            croquis::is_artifact_macro(name),
            "{name:?}"
        );
        assert_eq!(
            vue::macro_artifact_kind(name),
            croquis::macro_artifact_kind(name),
            "{name:?}"
        );
    }
}

#[test]
fn name_sequences_match_croquis() {
    assert_eq!(
        vue::runtime_erased_macro_names().collect::<Vec<_>>(),
        croquis::runtime_erased_macro_names().collect::<Vec<_>>()
    );
    assert_eq!(
        vue::artifact_macro_names().collect::<Vec<_>>(),
        croquis::artifact_macro_names().collect::<Vec<_>>()
    );
}

#[test]
fn builtin_kinds_and_names_match_croquis() {
    assert_eq!(
        [
            DEFINE_PROPS,
            DEFINE_EMITS,
            DEFINE_EXPOSE,
            DEFINE_OPTIONS,
            DEFINE_SLOTS,
            DEFINE_MODEL,
            WITH_DEFAULTS,
        ],
        [
            croquis::DEFINE_PROPS,
            croquis::DEFINE_EMITS,
            croquis::DEFINE_EXPOSE,
            croquis::DEFINE_OPTIONS,
            croquis::DEFINE_SLOTS,
            croquis::DEFINE_MODEL,
            croquis::WITH_DEFAULTS,
        ]
    );
    let builtin = MACROS
        .iter()
        .filter(|row| row.builtin)
        .map(|row| (row.name, croquis::MacroKind::from_name(row.name)))
        .collect::<Vec<_>>();
    assert_eq!(
        builtin,
        [
            (DEFINE_PROPS, Some(croquis::MacroKind::DefineProps)),
            (DEFINE_EMITS, Some(croquis::MacroKind::DefineEmits)),
            (DEFINE_EXPOSE, Some(croquis::MacroKind::DefineExpose)),
            (DEFINE_OPTIONS, Some(croquis::MacroKind::DefineOptions)),
            (DEFINE_SLOTS, Some(croquis::MacroKind::DefineSlots)),
            (DEFINE_MODEL, Some(croquis::MacroKind::DefineModel)),
            (WITH_DEFAULTS, Some(croquis::MacroKind::WithDefaults)),
        ]
    );
}

#[test]
fn each_kind_names_one_row() {
    let kinds = MACROS.iter().map(|row| row.kind).collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            MacroKind::DefineProps,
            MacroKind::DefineEmits,
            MacroKind::DefineExpose,
            MacroKind::DefineOptions,
            MacroKind::DefineSlots,
            MacroKind::DefineModel,
            MacroKind::WithDefaults,
            MacroKind::DefineArt,
            MacroKind::DefinePage,
            MacroKind::DefinePageMeta,
            MacroKind::DefineRouteRules,
            MacroKind::DefineLazyHydrationComponent,
        ]
    );
}
