//! Historical Vue dialects and parser quirks use the production L2 emitter.
//! The differential override keeps the retired lane available only as a byte
//! oracle for this test.

#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types,
    reason = "differential fixtures own strings and install a shared profiler"
)]

use std::{ffi::OsString, sync::Arc};
use vize_atelier_core::options::{CodegenMode, CodegenOptions, TemplateSyntaxMode};
use vize_atelier_dom::{
    DomCompilerOptions,
    compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options,
};
use vize_l0::Allocator;
use vize_l0::config::VueVersion;
use vize_l0::profiler::{CounterSummary, Profiler, ProfilerScope};

#[test]
fn historical_dialects_and_quirks_match_the_compatibility_lane_byte_for_byte() {
    let cases = [
        (
            "vue-0.10-repeat",
            VueVersion::V0_10,
            TemplateSyntaxMode::Standard,
            include_str!(
                "../../../tests/_fixtures/differential/dialect-inputs/generated/vue0-10-repeat.html"
            ),
        ),
        (
            "vue-0.11-repeat",
            VueVersion::V0_11,
            TemplateSyntaxMode::Standard,
            include_str!(
                "../../../tests/_fixtures/differential/dialect-inputs/generated/vue0-11-repeat.html"
            ),
        ),
        (
            "vue-1-list",
            VueVersion::V1,
            TemplateSyntaxMode::Standard,
            include_str!(
                "../../../tests/_fixtures/differential/dialect-inputs/generated/vue1-list.html"
            ),
        ),
        (
            "vue-1-attribute-interpolation",
            VueVersion::V1,
            TemplateSyntaxMode::Standard,
            include_str!(
                "../../../tests/_fixtures/differential/dialect-inputs/generated/vue1-attribute-interpolation.html"
            ),
        ),
        (
            "vue-1-raw-html",
            VueVersion::V1,
            TemplateSyntaxMode::Standard,
            include_str!(
                "../../../tests/_fixtures/differential/dialect-inputs/generated/vue1-raw-html.html"
            ),
        ),
        (
            "vue-1-raw-html-with-text",
            VueVersion::V1,
            TemplateSyntaxMode::Standard,
            "<div>before {{{ html }}} after</div>",
        ),
        (
            "vue-1-mixed-escaped-and-raw-html",
            VueVersion::V1,
            TemplateSyntaxMode::Standard,
            "<div>{{ escaped }} {{{ html }}}</div>",
        ),
        (
            "vue-2-filter-and-event-sugar",
            VueVersion::V2,
            TemplateSyntaxMode::Standard,
            r#"<button @click.native="go">{{ label | cap }}</button>"#,
        ),
        (
            "vue-2-duplicate-static-class",
            VueVersion::V2,
            TemplateSyntaxMode::Standard,
            r#"<h4 :class="item.is_premium ? 'premium' : ''" class="" class="shop_title ellipsis">{{ item.name }}</h4>"#,
        ),
        (
            "vue-2.7-slot-scope",
            VueVersion::V2_7,
            TemplateSyntaxMode::Standard,
            r#"<Comp><template slot-scope="props">{{ props.label }}</template></Comp>"#,
        ),
        (
            "quirks-adjacent-attributes",
            VueVersion::V3,
            TemplateSyntaxMode::Quirks,
            include_str!(
                "../../../tests/_fixtures/differential/dialect-inputs/generated/quirks-adjacent-attributes.html"
            ),
        ),
        (
            "quirks-void-close",
            VueVersion::V3,
            TemplateSyntaxMode::Quirks,
            include_str!(
                "../../../tests/_fixtures/differential/dialect-inputs/generated/quirks-void-close.html"
            ),
        ),
        (
            "quirks-authored-nesting",
            VueVersion::V3,
            TemplateSyntaxMode::Quirks,
            "<button><span><button>share</button></span></button>",
        ),
    ];

    for (name, dialect, syntax, source) in cases {
        for (lane, mode, prefix_identifiers) in [
            ("function", CodegenMode::Function, false),
            ("module-prefixed", CodegenMode::Module, true),
        ] {
            let options = DomCompilerOptions {
                dialect,
                mode,
                prefix_identifiers,
                ..Default::default()
            };
            let compatibility = vize_atelier_dom::differential::with_legacy_lane(|| {
                compile_result(source, options.clone(), syntax)
            });
            let (native, counters) = compile_with_l2_profiler(source, options, syntax);
            let refusal = vize_atelier_dom::differential::take_emit_refusal();
            assert_eq!(
                native.errors, compatibility.errors,
                "{name}/{lane}: diagnostics"
            );
            assert_eq!(
                native.preamble, compatibility.preamble,
                "{name}/{lane}: preamble\nnative code:\n{}\ncompatibility code:\n{}",
                native.code, compatibility.code
            );
            assert_eq!(native.code, compatibility.code, "{name}/{lane}: code");
            assert_eq!(
                native.sections, compatibility.sections,
                "{name}/{lane}: sections"
            );
            assert_eq!(
                counter_total(&counters, "l2.dom.accepted"),
                Some(1),
                "{name}/{lane}: native selector; refusal={refusal:?}; counters={counters:?}"
            );
        }
    }
}

#[test]
fn removed_dom_lane_flag_does_not_disarm_vue3_l2_selection() {
    let _env_lock = lock_env();
    let _flag = ScopedEnvVar::set(removed_dom_lane_flag_name(), "legacy");
    let (result, counters) = compile_with_l2_profiler(
        r#"<button @click="go">{{ label }}</button>"#,
        DomCompilerOptions::default(),
        TemplateSyntaxMode::Standard,
    );

    assert!(
        result.sections.is_some(),
        "L2 DOM must keep codegen sections available"
    );
    assert_eq!(
        counter_total(&counters, "l2.dom.files"),
        Some(1),
        "removed DOM lane env values must not disarm the L2 production selector",
    );
}

fn removed_dom_lane_flag_name() -> &'static str {
    concat!("VIZE", "_LEVEL", "_DOM")
}

#[derive(Debug, PartialEq, Eq)]
struct Compiled {
    errors: String,
    preamble: String,
    code: String,
    sections: Option<vize_atelier_core::CodegenSections>,
}

fn compile_with_l2_profiler(
    source: &str,
    options: DomCompilerOptions,
    syntax: TemplateSyntaxMode,
) -> (Compiled, CounterSummary) {
    let profiler = Arc::new(Profiler::new());
    profiler.clear();
    profiler.set_enabled(true);
    let _scope = ProfilerScope::install(&profiler);
    let result = compile_result(source, options, syntax);
    let counters = profiler.counter_summary();
    (result, counters)
}

fn compile_result(
    source: &str,
    options: DomCompilerOptions,
    syntax: TemplateSyntaxMode,
) -> Compiled {
    let allocator = Allocator::new();
    let (_, errors, result) =
        compile_template_with_template_syntax_and_hoisted_scope_id_with_sections_and_codegen_options(
            &allocator,
            source,
            options,
            syntax,
            None,
            CodegenOptions::default(),
        );
    Compiled {
        errors: format!("{errors:?}"),
        preamble: result.result.preamble.to_string(),
        code: result.result.code.to_string(),
        sections: result.sections,
    }
}

fn counter_total(counters: &CounterSummary, name: &str) -> Option<u64> {
    counters
        .entries
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.total)
}

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    static ENV_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    ENV_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
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
