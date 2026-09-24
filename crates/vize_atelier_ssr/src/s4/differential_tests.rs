//! Differential battery: both SSR emitters over the same inputs.
//!
//! Every fixture compiles once on the selected lane and once with the legacy
//! AST walker pinned. Code, preamble, source map, and diagnostics must be
//! byte-identical. Admitted fixtures must also be *emitted from the string
//! plan*, so an accidental legacy fallback cannot make the battery vacuous;
//! refused fixtures pin that the selector keeps them on the legacy lane.
#![expect(clippy::disallowed_macros, reason = "insta and fixtures use format!")]

mod component_fixtures;
mod control_fixtures;
mod fixtures;
mod slot_fixtures;
mod vue_align_fixtures;

use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_core::options::{BindingMetadata, BindingType, CustomElementMatcher};
use vize_s0::Allocator;

use super::{SsrS4Request, SsrS4Selection, select_ssr_lane};
use crate::compile::{SsrLane, compile_ssr_on_lane};
use crate::options::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

/// The option surfaces every admitted fixture runs under.
fn option_sets() -> std::vec::Vec<(
    &'static str,
    SsrCompilerOptions,
    SsrCompilerExperimentalOptions,
)> {
    let mut metadata = BindingMetadata::default();
    for (name, kind) in [
        ("title", BindingType::Props),
        ("count", BindingType::SetupRef),
        ("state", BindingType::SetupReactiveConst),
        ("label", BindingType::Data),
        ("format", BindingType::Options),
        ("cls", BindingType::SetupMaybeRef),
    ] {
        metadata.bindings.insert(name.into(), kind);
    }
    std::vec![
        (
            "default",
            SsrCompilerOptions::default(),
            SsrCompilerExperimentalOptions::default()
        ),
        (
            "scoped+css-vars",
            SsrCompilerOptions {
                scope_id: Some("data-v-7ba5bd90".into()),
                ssr_css_vars: Some("{ \"--color\": (_ctx.color) }".into()),
                ..SsrCompilerOptions::default()
            },
            SsrCompilerExperimentalOptions::default(),
        ),
        (
            "bindings+ts",
            SsrCompilerOptions {
                binding_metadata: Some(metadata),
                is_ts: true,
                ..SsrCompilerOptions::default()
            },
            SsrCompilerExperimentalOptions::default(),
        ),
        (
            "source-map",
            SsrCompilerOptions::default(),
            SsrCompilerExperimentalOptions {
                source_map: true,
                source_map_filename: Some("Fixture.vue".into()),
                ..SsrCompilerExperimentalOptions::default()
            },
        ),
    ]
}

fn assert_parity(
    source: &str,
    options: &SsrCompilerOptions,
    experimental: &SsrCompilerExperimentalOptions,
    context: &str,
) {
    let compile = |lane| {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_ssr_on_lane(
            &allocator,
            source,
            options.clone(),
            TemplateSyntaxMode::Standard,
            CustomElementMatcher::default(),
            experimental.clone(),
            lane,
        );
        (std::format!("{errors:?}"), result)
    };
    let (legacy_errors, legacy) = compile(SsrLane::LegacyOnly);
    let (selected_errors, selected) = compile(SsrLane::Selected);
    assert_eq!(selected.code, legacy.code, "{context}: code diverged");
    assert_eq!(
        selected.preamble, legacy.preamble,
        "{context}: preamble diverged"
    );
    assert_eq!(selected.map, legacy.map, "{context}: source map diverged");
    assert_eq!(
        selected_errors, legacy_errors,
        "{context}: diagnostics diverged"
    );
}

fn selection(
    source: &str,
    options: &SsrCompilerOptions,
    experimental: &SsrCompilerExperimentalOptions,
) -> SsrS4Selection {
    let allocator = Allocator::new();
    select_ssr_lane(
        &allocator,
        source,
        &SsrS4Request {
            options,
            experimental,
            template_syntax: TemplateSyntaxMode::Standard,
            has_custom_elements: false,
        },
    )
}

#[test]
fn admitted_fixtures_emit_from_the_plan_with_legacy_byte_parity() {
    for (set, options, experimental) in option_sets() {
        for (name, source) in fixtures::ADMITTED
            .iter()
            .chain(control_fixtures::ADMITTED)
            .chain(component_fixtures::ADMITTED)
            .chain(slot_fixtures::ADMITTED)
            .chain(vue_align_fixtures::ADMITTED)
        {
            let context = std::format!("{set}/{name}: {source}");
            let selected = selection(source, &options, &experimental);
            assert!(
                matches!(selected, SsrS4Selection::Emitted(_)),
                "{context}: expected plan emission, got {selected:?}"
            );
            assert_parity(source, &options, &experimental, &context);
        }
    }
}

#[test]
fn typescript_fixtures_are_owned_only_under_is_ts() {
    for (set, options, experimental) in option_sets() {
        for (name, source) in fixtures::ADMITTED_TS {
            let context = std::format!("{set}/{name}: {source}");
            let selected = selection(source, &options, &experimental);
            let emitted = matches!(selected, SsrS4Selection::Emitted(_));
            assert_eq!(
                emitted, options.is_ts,
                "{context}: plan emission must follow is_ts, got {selected:?}"
            );
            assert_parity(source, &options, &experimental, &context);
        }
    }
}

#[test]
fn refused_fixtures_stay_on_the_legacy_lane_with_parity() {
    for (set, options, experimental) in option_sets() {
        for (name, source) in fixtures::REFUSED {
            let context = std::format!("{set}/{name}: {source}");
            let selected = selection(source, &options, &experimental);
            assert!(
                matches!(selected, SsrS4Selection::Legacy(_)),
                "{context}: expected the legacy lane, got {selected:?}"
            );
            assert_parity(source, &options, &experimental, &context);
        }
    }
}
