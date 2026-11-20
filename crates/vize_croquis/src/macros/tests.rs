use super::{
    MacroKind, MacroLifecycle, MacroTracker, artifact_macro_names, is_artifact_macro,
    is_compile_time_macro, is_ecosystem_compile_time_macro, is_runtime_erased_macro,
    macro_artifact_kind, macro_lifecycle, runtime_erased_macro_names,
};
use vize_carton::CompactString;

#[test]
fn test_macro_lifecycle_classifies_builtin_and_ecosystem_macros() {
    assert_eq!(
        macro_lifecycle("defineProps"),
        Some(MacroLifecycle::AnalyzeAndErase)
    );
    assert_eq!(
        macro_lifecycle("definePage"),
        Some(MacroLifecycle::ExtractAndErase)
    );
    assert_eq!(
        macro_lifecycle("definePageMeta"),
        Some(MacroLifecycle::ExtractAndErase)
    );
    assert_eq!(
        macro_lifecycle("defineRouteRules"),
        Some(MacroLifecycle::ExtractAndErase)
    );
    assert_eq!(
        macro_lifecycle("defineLazyHydrationComponent"),
        Some(MacroLifecycle::Expand)
    );
    assert_eq!(
        macro_lifecycle("defineArt"),
        Some(MacroLifecycle::AnalyzeAndErase)
    );
    assert_eq!(
        macro_lifecycle("defineArtMeta"),
        Some(MacroLifecycle::AnalyzeAndErase)
    );
    assert_eq!(macro_lifecycle("notAMacro"), None);
    assert_eq!(macro_lifecycle("useTemplateRef"), None);

    assert!(is_compile_time_macro("defineArt"));
    assert!(is_ecosystem_compile_time_macro("defineArt"));
    assert!(is_runtime_erased_macro("defineArt"));
    assert!(!is_artifact_macro("defineArt"));
    assert_eq!(macro_artifact_kind("defineArt"), None);
    assert!(is_compile_time_macro("defineArtMeta"));
    assert!(is_ecosystem_compile_time_macro("defineArtMeta"));
    assert!(is_runtime_erased_macro("defineArtMeta"));
    assert!(!is_artifact_macro("defineArtMeta"));
    assert_eq!(macro_artifact_kind("defineArtMeta"), None);
    assert!(is_compile_time_macro("definePage"));
    assert!(is_ecosystem_compile_time_macro("definePage"));
    assert!(is_runtime_erased_macro("definePage"));
    assert!(is_artifact_macro("definePage"));
    assert_eq!(
        macro_artifact_kind("definePage"),
        Some("vue-router.definePage")
    );
    assert!(is_compile_time_macro("definePageMeta"));
    assert!(is_ecosystem_compile_time_macro("definePageMeta"));
    assert!(is_runtime_erased_macro("definePageMeta"));
    assert!(is_artifact_macro("definePageMeta"));
    assert_eq!(
        macro_artifact_kind("definePageMeta"),
        Some("nuxt.definePageMeta")
    );
    assert!(is_compile_time_macro("defineRouteRules"));
    assert!(is_ecosystem_compile_time_macro("defineRouteRules"));
    assert!(is_runtime_erased_macro("defineRouteRules"));
    assert!(is_artifact_macro("defineRouteRules"));
    assert_eq!(
        macro_artifact_kind("defineRouteRules"),
        Some("nuxt.defineRouteRules")
    );
    assert!(is_compile_time_macro("defineLazyHydrationComponent"));
    assert!(is_ecosystem_compile_time_macro(
        "defineLazyHydrationComponent"
    ));
    assert!(!is_runtime_erased_macro("defineLazyHydrationComponent"));
    assert!(!is_artifact_macro("defineLazyHydrationComponent"));
    assert_eq!(macro_artifact_kind("defineLazyHydrationComponent"), None);
    assert!(runtime_erased_macro_names().any(|name| name == "definePage"));
    assert!(runtime_erased_macro_names().any(|name| name == "definePageMeta"));
    assert!(runtime_erased_macro_names().any(|name| name == "defineRouteRules"));
    assert!(!runtime_erased_macro_names().any(|name| name == "defineLazyHydrationComponent"));
    assert!(runtime_erased_macro_names().any(|name| name == "defineArt"));
    assert!(runtime_erased_macro_names().any(|name| name == "defineArtMeta"));
    assert!(artifact_macro_names().any(|name| name == "definePage"));
    assert!(artifact_macro_names().any(|name| name == "definePageMeta"));
    assert!(artifact_macro_names().any(|name| name == "defineRouteRules"));
    assert!(!artifact_macro_names().any(|name| name == "defineLazyHydrationComponent"));
}

#[test]
fn test_macro_tracker() {
    let mut tracker = MacroTracker::new();

    let id = tracker.add_call(
        "defineProps",
        MacroKind::DefineProps,
        0,
        20,
        None,
        Some(CompactString::new("{ msg: string }")),
    );

    assert_eq!(id.as_u32(), 0);
    assert!(tracker.define_props().is_some());
    assert!(tracker.define_emits().is_none());
}

#[test]
fn test_top_level_await() {
    let mut tracker = MacroTracker::new();
    assert!(!tracker.is_async());

    tracker.add_top_level_await(CompactString::new("fetch('/api')"), 10, 30);
    assert!(tracker.is_async());
    assert_eq!(tracker.top_level_awaits().len(), 1);
}
