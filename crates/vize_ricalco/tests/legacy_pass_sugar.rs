//! The legacy port's template-sugar pins (P2-9 series 7, `_legacy`
//! only; split from `legacy_pass.rs` under the source budget): the
//! `.sync` expansion, the scoped-slot conversion, the v-on event sugar,
//! the live lane's bails, the capability split across legacy lines, and
//! the zero-cost behavioural pin.

mod support;

use vize_davinci::folio::{Folio, FolioMode};
use vize_disegno::folio::FolioOp;
use vize_ricalco::LegacyVueLine;

use support::{assert_transformed_sound_legacy, with_lowered, with_transformed_legacy};

fn v2<R>(
    source: &str,
    f: impl FnOnce(
        &vize_ricalco::Lowered<'_>,
        &vize_disegno::folio::DisegnoFolio,
        &vize_ricalco::pass::S2Facts,
    ) -> R,
) -> R {
    with_transformed_legacy(source, LegacyVueLine::V2, |lowered, folio, facts, _| {
        f(lowered, folio, facts)
    })
}

/// The source text of an owned expression mirror, whatever its class.
fn expr_source(expr: &vize_disegno::folio::FolioExpr) -> &str {
    match expr {
        vize_disegno::folio::FolioExpr::Js { source, .. }
        | vize_disegno::folio::FolioExpr::Foreign { source, .. }
        | vize_disegno::folio::FolioExpr::Opaque { source, .. } => source.as_str(),
    }
}

#[test]
fn sync_expands_to_the_update_listener() {
    v2(
        r#"<MyPane :title.sync="pane.title"></MyPane>"#,
        |lowered, folio, _| {
            let FolioOp::Component(component) = &folio.ops[0] else {
                panic!("expected the component");
            };
            assert_eq!(component.bindings.len(), 2);
            let vize_disegno::folio::FolioBinding::Bind(bind) = &component.bindings[0] else {
                panic!("expected the bind");
            };
            // The `sync` modifier stripped, exactly as the live desugar
            // strips it in place.
            assert!(bind.modifiers.is_empty());
            let vize_disegno::folio::FolioBinding::On(on) = &component.bindings[1] else {
                panic!("expected the appended listener");
            };
            assert_eq!(
                on.name,
                Some(vize_disegno::folio::FolioName::Static(
                    vize_carton::String::from("update:title")
                ))
            );
            // The exact live handler shape, `$event => ((exp) = $event)`.
            let handler = on.handler.as_ref().expect("the listener has a handler");
            assert_eq!(expr_source(handler), "$event => ((pane.title) = $event)");
            assert!(
                lowered
                    .provenance
                    .iter()
                    .any(|r| r.rule.as_str() == "normalize.legacy.sync")
            );
        },
    );
    assert_transformed_sound_legacy(
        r#"<MyPane :title.sync="pane.title"></MyPane>"#,
        LegacyVueLine::V2,
        "sync expansion",
    );
}

#[test]
fn a_valueless_sync_reads_the_same_name_shorthand() {
    // The shipped parser fills `:model-name` with camelized `modelName`
    // before the live desugar reads it — mirrored.
    v2("<Widget :model-name.sync></Widget>", |_, folio, _| {
        let FolioOp::Component(component) = &folio.ops[0] else {
            panic!("expected the component");
        };
        assert_eq!(component.bindings.len(), 2);
        let vize_disegno::folio::FolioBinding::On(on) = &component.bindings[1] else {
            panic!("expected the appended listener");
        };
        let handler = on.handler.as_ref().expect("handler");
        assert_eq!(expr_source(handler), "$event => ((modelName) = $event)");
    });
}

#[test]
fn a_dynamic_argument_sync_is_left_untouched() {
    // The live desugar's bounded subset: dynamic args skip, the `sync`
    // modifier stays authored.
    v2(r#"<Row :[k].sync="v"></Row>"#, |_, folio, _| {
        let FolioOp::Component(component) = &folio.ops[0] else {
            panic!("expected the component");
        };
        assert_eq!(component.bindings.len(), 1);
        let vize_disegno::folio::FolioBinding::Bind(bind) = &component.bindings[0] else {
            panic!("expected the bind");
        };
        assert_eq!(bind.modifiers.len(), 1);
        assert_eq!(bind.modifiers[0].as_str(), "sync");
    });
}

#[test]
fn v2_event_sugar_rewrites_modifiers() {
    v2(
        r#"<button @click.native.stop="go" @keyup.13="submit" @keydown.99.native="odd"></button>"#,
        |lowered, folio, _| {
            let FolioOp::Element(element) = &folio.ops[0] else {
                panic!("expected the element");
            };
            let mods = |i: usize| -> Vec<&str> {
                let vize_disegno::folio::FolioBinding::On(on) = &element.bindings[i] else {
                    panic!("expected an on binding");
                };
                on.modifiers.iter().map(|m| m.as_str()).collect()
            };
            // `.native` stripped, keycodes renamed, unmapped numbers
            // kept — the shipped `desugar_v2_v_on_modifiers` exactly.
            assert_eq!(mods(0), ["stop"]);
            assert_eq!(mods(1), ["enter"]);
            assert_eq!(mods(2), ["99"]);
            assert!(
                lowered
                    .provenance
                    .iter()
                    .any(|r| r.rule.as_str() == "normalize.legacy.native")
            );
            assert!(
                lowered
                    .provenance
                    .iter()
                    .any(|r| r.rule.as_str() == "normalize.legacy.keycode")
            );
        },
    );
}

#[test]
fn scoped_slot_attrs_desugar_to_slot_content() {
    let source = r#"<Card><template slot="header" slot-scope="props"><b>{{ props.title }}</b></template></Card>"#;
    v2(source, |lowered, folio, facts| {
        let FolioOp::Component(component) = &folio.ops[0] else {
            panic!("expected the component");
        };
        let FolioOp::Element(template) = &component.children[0] else {
            panic!("expected the template carrier");
        };
        // Both attributes consumed; the appended spelling carries the
        // companion name and the props.
        assert!(template.attributes.is_empty());
        assert_eq!(template.bindings.len(), 1);
        let vize_disegno::folio::FolioBinding::SlotContent(content) = &template.bindings[0] else {
            panic!("expected the slot-content op");
        };
        assert_eq!(
            content.name,
            Some(vize_disegno::folio::FolioName::Static(
                vize_carton::String::from("header")
            ))
        );
        let params = content.params.as_ref().expect("the props position");
        assert_eq!(expr_source(params), "props");
        // The props scope registered through the one params rule, and
        // the slot pass groups the carrier as an authored named group.
        assert_eq!(lowered.scopes.len(), 1);
        assert_eq!(facts.slot_facts.len(), 1);
        assert!(
            lowered
                .provenance
                .iter()
                .any(|r| r.rule.as_str() == "normalize.legacy.slot-scope")
        );
        assert!(
            lowered
                .provenance
                .iter()
                .any(|r| r.rule.as_str() == "consume.legacy.slot-name")
        );
    });
    assert_transformed_sound_legacy(source, LegacyVueLine::V2, "scoped slot");
}

#[test]
fn the_scope_alias_converts_and_defaults_without_a_companion() {
    v2(
        "<List><template scope=\"row\">item</template></List>",
        |_, folio, _| {
            let FolioOp::Component(component) = &folio.ops[0] else {
                panic!("expected the component");
            };
            let FolioOp::Element(template) = &component.children[0] else {
                panic!("expected the template");
            };
            assert!(template.attributes.is_empty());
            let vize_disegno::folio::FolioBinding::SlotContent(content) = &template.bindings[0]
            else {
                panic!("expected the slot-content op");
            };
            // No companion `slot` attribute: the implicit default — the
            // name stays `None` so the slot pass records the synthesized
            // default-name origin, exactly like a bare `v-slot`.
            assert_eq!(content.name, None);
        },
    );
}

#[test]
fn a_scoped_slot_on_a_plain_element_misplaces_exactly_like_the_live_lane() {
    // The shipped desugar converts `slot-scope` on *any* element into a
    // `v-slot` directive, and the shipped validation then rejects a
    // plain-element carrier — so the S2 mirror's slot pass fires the
    // same `VSlotMisplaced`, deliberate parity rather than a Vue 2
    // legality question (Vue 2 allowed scoped slots on plain children;
    // the shipped implementation does not, and the port mirrors the
    // implementation).
    v2(
        "<List><li scope=\"row\">item</li></List>",
        |lowered, _, _| {
            assert_eq!(lowered.diagnostics.len(), 1);
            assert_eq!(
                lowered.diagnostics[0].message.as_str(),
                "v-slot can only be used on components or <template> tags."
            );
        },
    );
}

#[test]
fn an_existing_v_slot_spelling_wins_over_the_legacy_attrs() {
    // The live desugar's conflict bail: the element keeps its authored
    // `v-slot` and the legacy attributes stay plain attributes.
    v2(
        r#"<Grid><template slot-scope="cell" #conflict="c">y</template></Grid>"#,
        |_, folio, _| {
            let FolioOp::Component(component) = &folio.ops[0] else {
                panic!("expected the component");
            };
            let FolioOp::Element(template) = &component.children[0] else {
                panic!("expected the template");
            };
            assert_eq!(template.attributes.len(), 1);
            assert_eq!(template.attributes[0].name.as_str(), "slot-scope");
            assert_eq!(template.bindings.len(), 1);
            assert!(matches!(
                template.bindings[0],
                vize_disegno::folio::FolioBinding::SlotContent(_)
            ));
        },
    );
}

#[test]
fn a_plain_slot_attribute_without_slot_scope_stays_an_attribute() {
    // The live desugar converts only when `slot-scope`/`scope` is
    // present; a bare `slot="named"` stays authored surface.
    v2(
        "<Tab><template slot=\"named\">z</template></Tab>",
        |_, folio, _| {
            let FolioOp::Component(component) = &folio.ops[0] else {
                panic!("expected the component");
            };
            let FolioOp::Element(template) = &component.children[0] else {
                panic!("expected the template");
            };
            assert_eq!(template.attributes.len(), 1);
            assert_eq!(template.attributes[0].name.as_str(), "slot");
            assert!(template.bindings.is_empty());
        },
    );
}

#[test]
fn v1_keeps_filters_but_not_the_v2_sugar() {
    // The capability split, mirrored from `LegacyDialectCapabilities`:
    // V1 supports filters, has no scoped-slot attrs and no V2 event
    // sugar.
    with_transformed_legacy(
        r#"<div :a.sync="b" @keyup.13="go" data-scope="x">{{ m | f }}</div>"#,
        LegacyVueLine::V1,
        |_, folio, facts, _| {
            assert_eq!(facts.legacy.sites.len(), 1);
            let FolioOp::Element(element) = &folio.ops[0] else {
                panic!("expected the element");
            };
            // `.sync` untouched, `13` untouched.
            let vize_disegno::folio::FolioBinding::Bind(bind) = &element.bindings[0] else {
                panic!("expected the bind");
            };
            assert_eq!(bind.modifiers[0].as_str(), "sync");
            let vize_disegno::folio::FolioBinding::On(on) = &element.bindings[1] else {
                panic!("expected the on");
            };
            assert_eq!(on.modifiers[0].as_str(), "13");
        },
    );
}

#[test]
fn the_legacy_entry_under_a_filter_free_template_matches_the_default_lowering() {
    // The zero-cost clause's behavioural half: with no legacy surface
    // authored, the legacy entry's artifact is byte-identical to the
    // default entry's (the desugars are recognition-gated, not
    // tree-shaped).
    let source = r#"<div v-if="ok" :class="c">{{ plain }}</div><p v-else>n</p>"#;
    let default_folio = with_lowered(source, |_, folio| folio.print_to_string(FolioMode::Full));
    v2(source, |lowered, folio, facts| {
        assert_eq!(folio.print_to_string(FolioMode::Full), default_folio);
        assert_eq!(facts.legacy.sites.len(), 0);
        assert!(facts.legacy.assets.is_empty());
        assert_eq!(lowered.filters.len(), 0);
    });
}
