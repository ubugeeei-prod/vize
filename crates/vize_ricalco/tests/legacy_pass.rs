//! Vue 2 sugar legalization (P2-9 installment 7): the pass rewrites
//! dialect payloads into the Vue 3 surface. Vue 3 stays on the 6-pass
//! table (`walks=6`).

mod support;

use support::{
    assert_transformed_sound, assert_transformed_sound_caps, with_transformed,
    with_transformed_caps,
};
use vize_carton::config::VueVersion;
use vize_davinci::folio::{Folio, FolioMode};
use vize_ricalco::LegacyCaps;

fn vue2() -> LegacyCaps {
    LegacyCaps::for_version(VueVersion::V2)
}

#[test]
fn vue3_keeps_six_walks_on_legacy_spellings() {
    let source = r#"<Comp :title.sync="heading"/>"#;
    with_transformed(source, |lowered, folio, _, budget| {
        assert!(
            folio.print_to_string(FolioMode::Full).contains("ui.bind"),
            "Vue 3 must not expand .sync"
        );
        assert!(
            !folio.print_to_string(FolioMode::Full).contains("onUpdate")
                && !folio
                    .print_to_string(FolioMode::Full)
                    .contains("update:title"),
            "Vue 3 must not synthesize update:title"
        );
        assert_eq!(lowered.caps, LegacyCaps::VUE3);
        assert_eq!(
            Folio::print_to_string(budget, FolioMode::Full).as_str(),
            "[budget-observer]\nwalks=6\npasses=6\nanalyses=0\npipelines=1\nfailures=0\n\n"
        );
    });
    assert_transformed_sound(source, "vue3-sync-inert-pass");
}

#[test]
fn vue2_expands_sync_into_bind_plus_update_listener() {
    let source = r#"<Comp :title.sync="heading"/>"#;
    with_transformed_caps(source, vue2(), |lowered, folio, _, budget| {
        let text = folio.print_to_string(FolioMode::Full);
        assert!(
            text.contains("ui.bind name=\"title\"") && text.contains("ui.on name=\"update:title\""),
            "vue.sync must expand: {text}"
        );
        assert!(!text.contains("vue.sync"), "vue.sync must be gone: {text}");
        assert!(
            text.contains("$event => ((heading) = $event)"),
            "handler must assign through $event: {text}"
        );
        assert_eq!(
            Folio::print_to_string(budget, FolioMode::Full).as_str(),
            "[budget-observer]\nwalks=7\npasses=7\nanalyses=0\npipelines=1\nfailures=0\n\n"
        );
        assert_eq!(u64::from(lowered.op_count), folio.op_count());
    });
    assert_transformed_sound_caps(source, vue2(), "vue2-sync-expand");
}

#[test]
fn vue2_keeps_camel_on_the_bind() {
    let source = r#"<Comp :title.sync.camel="heading"/>"#;
    with_transformed_caps(source, vue2(), |_, folio, _, _| {
        let text = folio.print_to_string(FolioMode::Full);
        assert!(
            text.contains("ui.bind name=\"title\" mods=\"camel\""),
            "remaining modifiers stay on the bind: {text}"
        );
    });
    assert_transformed_sound_caps(source, vue2(), "vue2-sync-camel-pass");
}

#[test]
fn vue2_rewrites_a_pipe_filter_to_the_asset_call() {
    let source = "{{msg | cap}}";
    with_transformed_caps(source, vue2(), |_, folio, _, _| {
        let text = folio.print_to_string(FolioMode::Full);
        assert!(
            text.contains("js(\"_filter_cap(msg)\""),
            "filter must wrap: {text}"
        );
        assert!(
            !text.contains("vue.filter"),
            "vue.filter must be gone: {text}"
        );
    });
    assert_transformed_sound_caps(source, vue2(), "vue2-filter-wrap");
}

#[test]
fn vue2_rewrites_a_filter_with_args() {
    let source = "{{a | f(b)}}";
    with_transformed_caps(source, vue2(), |_, folio, _, _| {
        let text = folio.print_to_string(FolioMode::Full);
        assert!(
            text.contains("js(\"_filter_f(a,b)\""),
            "call-style filter must wrap: {text}"
        );
    });
    assert_transformed_sound_caps(source, vue2(), "vue2-filter-args");
}

#[test]
fn vue2_converts_slot_scope_into_slot_content() {
    let source = r#"<Comp><template slot-scope="props">x</template></Comp>"#;
    with_transformed_caps(source, vue2(), |_, folio, facts, _| {
        let text = folio.print_to_string(FolioMode::Full);
        assert!(
            text.contains("ui.slot-content") && !text.contains("vue.slot-scope"),
            "slot-scope must become slot-content: {text}"
        );
        assert_eq!(facts.slot_facts.len(), 1);
    });
    assert_transformed_sound_caps(source, vue2(), "vue2-slot-scope-pass");
}

#[test]
fn vue2_strips_native_and_rewrites_keycodes() {
    let source = r#"<Comp @click.native @keyup.13="onKey"/>"#;
    with_transformed_caps(source, vue2(), |_, folio, _, _| {
        let text = folio.print_to_string(FolioMode::Full);
        assert!(
            !text.contains("mods=\"native\""),
            ".native must be stripped: {text}"
        );
        assert!(
            text.contains("mods=\"enter\""),
            "keyCode 13 must become enter: {text}"
        );
    });
    assert_transformed_sound_caps(source, vue2(), "vue2-on-sugar");
}

#[test]
fn vue3_leaves_native_and_keycodes() {
    let source = r#"<Comp @click.native @keyup.13="onKey"/>"#;
    with_transformed(source, |_, folio, _, _| {
        let text = folio.print_to_string(FolioMode::Full);
        assert!(
            text.contains("mods=\"native\""),
            "Vue 3 keeps .native: {text}"
        );
        assert!(
            text.contains("mods=\"13\""),
            "Vue 3 keeps numeric keyCodes: {text}"
        );
    });
    assert_transformed_sound(source, "vue3-on-inert");
}
