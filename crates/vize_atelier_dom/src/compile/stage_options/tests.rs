use super::{
    DomLaneSelection, dom_lane_selection, dom_lane_selection_from_flag, s2_emit_supported,
};
use crate::DomCompilerOptions;
use std::ffi::OsString;
use vize_atelier_core::options::{CodegenOptions, CustomElementMatcher, TemplateSyntaxMode};
use vize_s0::config::VueVersion;

#[test]
fn dom_lane_selection_maps_the_legacy_flag_value() {
    assert_eq!(
        dom_lane_selection_from_flag(Some("legacy")),
        DomLaneSelection::Legacy
    );
    assert_eq!(
        dom_lane_selection_from_flag(Some("s2")),
        DomLaneSelection::S2
    );
    assert_eq!(dom_lane_selection_from_flag(None), DomLaneSelection::S2);
}

#[test]
fn dom_lane_selection_reads_the_legacy_env_value() {
    let _env_lock = lock_env();
    let _flag = ScopedEnvVar::set(vize_s1_to_s2::DOM_LANE_FLAG, "legacy");

    assert_eq!(dom_lane_selection(), DomLaneSelection::Legacy);
}

#[test]
fn the_dom_legacy_lane_flag_disarms_s2_selection() {
    assert!(supported(
        DomCompilerOptions::default(),
        DomLaneSelection::S2
    ));
    assert!(!supported(
        DomCompilerOptions::default(),
        DomLaneSelection::Legacy
    ));
}

#[test]
fn legacy_dialects_stay_on_the_compatibility_lane() {
    assert!(!supported(
        DomCompilerOptions {
            dialect: VueVersion::V2,
            ..Default::default()
        },
        DomLaneSelection::S2
    ));
    assert!(!supported(
        DomCompilerOptions {
            dialect: VueVersion::V2_7,
            ..Default::default()
        },
        DomLaneSelection::S2
    ));
}

#[test]
fn ssr_optimize_imports_codegen_option_stays_on_the_compatibility_lane() {
    assert!(!s2_emit_supported(
        &DomCompilerOptions::default(),
        &CodegenOptions {
            optimize_imports: true,
            ..Default::default()
        },
        &CustomElementMatcher::default(),
        TemplateSyntaxMode::Standard,
        false,
        DomLaneSelection::S2,
        super::S2EmitSelection::Allowed,
    ));
}

#[test]
fn opaque_custom_element_predicates_stay_on_the_compatibility_lane() {
    assert!(!s2_emit_supported(
        &DomCompilerOptions::default(),
        &CodegenOptions::default(),
        &CustomElementMatcher::from_static_predicate(is_custom_element),
        TemplateSyntaxMode::Standard,
        false,
        DomLaneSelection::S2,
        super::S2EmitSelection::Allowed,
    ));
}

fn supported(options: DomCompilerOptions, dom_lane: DomLaneSelection) -> bool {
    s2_emit_supported(
        &options,
        &CodegenOptions::default(),
        &CustomElementMatcher::default(),
        TemplateSyntaxMode::Standard,
        false,
        dom_lane,
        super::S2EmitSelection::Allowed,
    )
}

fn is_custom_element(tag: &str) -> bool {
    tag.starts_with("x-")
}

struct ScopedEnvVar {
    key: &'static str,
    previous: Option<OsString>,
}

impl ScopedEnvVar {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(key);
        // SAFETY: This test holds the local environment lock for the full
        // lifetime of the scoped override.
        unsafe { std::env::set_var(key, value) };
        Self { key, previous }
    }
}

impl Drop for ScopedEnvVar {
    fn drop(&mut self) {
        // SAFETY: The guard is dropped before the local environment lock,
        // restoring the process environment while mutations are serialized.
        unsafe {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }
}

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    static ENV_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    ENV_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
