//! P2-11: the option-surface audit guards.
//!
//! Several late S2 DOM parity fixes came from widening batteries until an
//! option-specific path failed. These tests preserve the structural rows from
//! that audit: cases whose parity should hold because the emitter routes the
//! construct through the correct owner, not because a caller passed another
//! flag.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_atelier_core::options::CodegenOptions;
use vize_atelier_dom::DomCompilerOptions;
use vize_s1_to_s2::DomEmitOptions;

const SCOPE_ID: &str = "data-v-abc123";

/// Slot outlet props are not props on a real rendered element, so scoped CSS
/// attrs stay out of the outlet itself while fallback elements still receive
/// the normal scope pair.
#[test]
fn slot_outlets_take_no_scope_pair() {
    let cases: &[(&str, &str)] = &[
        ("outlet_props", r#"<slot :item="x" name="a">fb</slot>"#),
        ("outlet_static_name", r#"<slot name="a"></slot>"#),
        ("outlet_bare", "<slot />"),
        (
            "outlet_element_fallback",
            r#"<slot><div class="c">f</div></slot>"#,
        ),
        ("outlet_dynamic_name", r#"<slot :name="n" />"#),
        (
            "outlet_in_for",
            r#"<slot v-for="i in n" :key="i" :item="i" />"#,
        ),
    ];
    support::assert_s2_matches_shipped_with_options(
        cases,
        &DomCompilerOptions {
            cache_handlers: true,
            scope_id: Some(SCOPE_ID.into()),
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            cache_handlers: true,
            scope_id: Some(SCOPE_ID),
            ..DomEmitOptions::DEFAULT
        },
    );
}

/// `cache_handlers` is uniform across structural emit sites; the variation is
/// only Vue's slot-param scope gate, where both lanes suppress caching.
#[test]
fn cached_handlers_are_uniform_across_structural_positions() {
    let cases: &[(&str, &str)] = &[
        ("once_handler", r#"<div v-once @click="a++">x</div>"#),
        (
            "once_child_handler",
            r#"<div v-once><a @click="a++">x</a></div>"#,
        ),
        ("memo_handler", r#"<div v-memo="[a]" @click="a++">x</div>"#),
        (
            "slot_scope_handler",
            r#"<MyComp v-slot="{ r }"><a @click="go(r)">x</a></MyComp>"#,
        ),
        (
            "outlet_fallback_handler",
            r#"<slot :on="1"><a @click="a++">x</a></slot>"#,
        ),
        ("vif_handler", r#"<div v-if="ok" @click="a++">x</div>"#),
        (
            "vfor_handler",
            r#"<div v-for="i in n" :key="i" @click="a++">x</div>"#,
        ),
    ];
    support::assert_s2_matches_shipped_with_options(
        cases,
        &DomCompilerOptions {
            cache_handlers: true,
            ..Default::default()
        },
        &CodegenOptions::default(),
        &DomEmitOptions {
            cache_handlers: true,
            ..DomEmitOptions::DEFAULT
        },
    );
}
