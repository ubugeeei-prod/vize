//! Vue 2 dialect payloads at S1→S2 lowering (P2-9 installment 7).
//!
//! The legalizing pass is a later PR; this suite pins that the
//! lowering admits `.sync` / `slot-scope` / pipe filters as dialect
//! ops and expressions under Vue 2, and that Vue 3 stays inert.

mod support;

use support::{artifact, artifact_caps, assert_sound, assert_sound_caps};
use vize_carton::config::VueVersion;
use vize_ricalco::LegacyCaps;

fn vue2() -> LegacyCaps {
    LegacyCaps::for_version(VueVersion::V2)
}

#[test]
fn vue3_leaves_sync_as_a_bind_modifier() {
    let source = r#"<Comp :title.sync="heading"/>"#;
    let art = artifact(source);
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=2\n\
         \n\
         [disegno.ops]\n\
         ui.component Comp @0:29\n\
         \x20 ui.bind name=\"title\" mods=\"sync\" value=js(\"heading\" @19:26) @6:27\n\
         \n"
    );
    assert_sound(source, "vue3-sync-inert");
}

#[test]
fn vue2_admits_sync_as_the_dialect_op() {
    let source = r#"<Comp :title.sync="heading"/>"#;
    let art = artifact_caps(source, vue2());
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=2\n\
         \n\
         [disegno.ops]\n\
         ui.component Comp @0:29\n\
         \x20 vue.sync name=\"title\" value=js(\"heading\" @19:26) @6:27\n\
         \n"
    );
    assert_sound_caps(source, vue2(), "vue2-sync");
}

#[test]
fn vue2_keeps_non_sync_bind_modifiers_on_the_dialect_op() {
    let source = r#"<Comp :title.sync.camel="heading"/>"#;
    let art = artifact_caps(source, vue2());
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=2\n\
         \n\
         [disegno.ops]\n\
         ui.component Comp @0:35\n\
         \x20 vue.sync name=\"title\" mods=\"camel\" value=js(\"heading\" @25:32) @6:33\n\
         \n"
    );
    assert_sound_caps(source, vue2(), "vue2-sync-camel");
}

#[test]
fn vue2_leaves_dynamic_sync_as_bind() {
    let source = r#"<Comp :[foo].sync="heading"/>"#;
    let art = artifact_caps(source, vue2());
    assert!(
        art.folio.contains("ui.bind"),
        "dynamic .sync must stay ui.bind: {}",
        art.folio
    );
    assert!(
        !art.folio.contains("vue.sync"),
        "dynamic .sync must not become vue.sync: {}",
        art.folio
    );
    assert_sound_caps(source, vue2(), "vue2-dynamic-sync");
}

#[test]
fn vue3_reads_a_pipe_as_js() {
    let source = "{{msg | cap}}";
    let art = artifact(source);
    assert!(
        art.folio.contains("js(\"msg | cap\""),
        "Vue 3 must read `|` as JS: {}",
        art.folio
    );
    assert!(
        !art.folio.contains("vue.filter"),
        "Vue 3 must not admit vue.filter: {}",
        art.folio
    );
    assert_sound(source, "vue3-pipe-js");
}

#[test]
fn vue2_admits_a_pipe_as_the_filter_expression() {
    let source = "{{msg | cap}}";
    let art = artifact_caps(source, vue2());
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=1\n\
         \n\
         [disegno.ops]\n\
         ui.interpolation vue.filter(\"msg | cap\" @2:11) @0:13\n\
         \n"
    );
    assert_sound_caps(source, vue2(), "vue2-filter");
}

#[test]
fn vue3_leaves_slot_scope_as_an_attribute() {
    let source = r#"<Comp><template slot-scope="props">x</template></Comp>"#;
    let art = artifact(source);
    assert!(
        art.folio.contains("attr slot-scope=\"props\""),
        "Vue 3 must keep slot-scope as an attribute: {}",
        art.folio
    );
    assert!(
        !art.folio.contains("vue.slot-scope"),
        "Vue 3 must not emit vue.slot-scope: {}",
        art.folio
    );
    assert_sound(source, "vue3-slot-scope-inert");
}

#[test]
fn vue2_admits_slot_scope_as_the_dialect_op() {
    let source = r#"<Comp><template slot-scope="props">x</template></Comp>"#;
    let art = artifact_caps(source, vue2());
    assert_eq!(
        art.folio,
        "[disegno]\n\
         ops=4\n\
         \n\
         [disegno.ops]\n\
         ui.component Comp @0:54\n\
         \x20 ui.element template @6:47\n\
         \x20   vue.slot-scope params=js(\"props\" @28:33) @16:34\n\
         \x20   ui.text \"x\" @35:36\n\
         \n"
    );
    assert_eq!(art.scopes.len(), 1);
    assert_sound_caps(source, vue2(), "vue2-slot-scope");
}

#[test]
fn vue2_consumes_the_companion_slot_attribute() {
    let source = r#"<Comp><template slot="header" slot-scope="props">x</template></Comp>"#;
    let art = artifact_caps(source, vue2());
    assert!(
        art.folio
            .contains("vue.slot-scope name=\"header\" params=js(\"props\""),
        "companion slot must become the dialect op name: {}",
        art.folio
    );
    assert!(
        !art.folio.contains("attr slot="),
        "companion slot must not remain an attribute: {}",
        art.folio
    );
    assert_sound_caps(source, vue2(), "vue2-slot-scope-named");
}

#[test]
fn vue2_leaves_slot_scope_when_v_slot_is_already_authored() {
    let source = r#"<Comp><template v-slot:header slot-scope="props">x</template></Comp>"#;
    let art = artifact_caps(source, vue2());
    assert!(
        art.folio.contains("ui.slot-content"),
        "authored v-slot must still lower: {}",
        art.folio
    );
    assert!(
        art.folio.contains("attr slot-scope=\"props\""),
        "conflicting slot-scope must stay an attribute: {}",
        art.folio
    );
    assert!(
        !art.folio.contains("vue.slot-scope"),
        "must not emit a conflicting dialect op: {}",
        art.folio
    );
    assert_sound_caps(source, vue2(), "vue2-slot-scope-conflict");
}

#[test]
fn vue1_admits_filters_but_not_sync() {
    let caps = LegacyCaps::for_version(VueVersion::V1);
    let sync = artifact_caps(r#"<Comp :title.sync="heading"/>"#, caps);
    assert!(
        sync.folio.contains("ui.bind") && !sync.folio.contains("vue.sync"),
        "Vue 1 has no .sync sugar: {}",
        sync.folio
    );
    let filter = artifact_caps("{{msg | cap}}", caps);
    assert!(
        filter.folio.contains("vue.filter"),
        "Vue 1 still has filters: {}",
        filter.folio
    );
    assert_sound_caps("{{msg | cap}}", caps, "vue1-filter");
}

#[test]
fn vue2_admits_a_pipe_on_a_bind_value() {
    let source = r#"<div :id="raw | formatId"/>"#;
    let art = artifact_caps(source, vue2());
    assert!(
        art.folio.contains("vue.filter(\"raw | formatId\""),
        "v-bind values are filter sites: {}",
        art.folio
    );
    assert_sound_caps(source, vue2(), "vue2-bind-filter");
}

#[test]
fn vue2_reads_an_event_handler_pipe_as_js() {
    let source = r#"<div @click="left | right"/>"#;
    let art = artifact_caps(source, vue2());
    assert!(
        art.folio.contains("js(\"left | right\""),
        "v-on handlers are not filter sites: {}",
        art.folio
    );
    assert!(
        !art.folio.contains("vue.filter"),
        "v-on must not admit vue.filter: {}",
        art.folio
    );
    assert_sound_caps(source, vue2(), "vue2-on-bitwise-or");
}

#[test]
fn vue2_scope_on_template_is_the_dialect_op() {
    let source = r#"<Comp><template scope="props">x</template></Comp>"#;
    let art = artifact_caps(source, vue2());
    assert!(
        art.folio.contains("vue.slot-scope params=js(\"props\""),
        "template scope is the 2.1 alias: {}",
        art.folio
    );
    assert_sound_caps(source, vue2(), "vue2-template-scope");
}

#[test]
fn vue2_scope_on_a_div_stays_an_attribute() {
    let source = r#"<div scope="props">x</div>"#;
    let art = artifact_caps(source, vue2());
    assert!(
        art.folio.contains("attr scope=\"props\""),
        "div scope is ordinary HTML: {}",
        art.folio
    );
    assert!(
        !art.folio.contains("vue.slot-scope"),
        "div scope must not become vue.slot-scope: {}",
        art.folio
    );
    assert_sound_caps(source, vue2(), "vue2-div-scope");
}

#[test]
fn vue2_slot_scope_on_a_div_is_the_dialect_op() {
    let source = r#"<div slot-scope="props">x</div>"#;
    let art = artifact_caps(source, vue2());
    assert!(
        art.folio.contains("vue.slot-scope params=js(\"props\""),
        "slot-scope is legal on any element: {}",
        art.folio
    );
    assert_sound_caps(source, vue2(), "vue2-div-slot-scope");
}
