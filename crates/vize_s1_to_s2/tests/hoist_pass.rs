//! The hoist-static analysis pass (P2-9 series 6): the static-type
//! lattice and the three subtree predicates, pinned per owner against
//! the shipped `static_type.rs`/`props.rs` rules — including the
//! mirrored quirks (the `svg` directive block, the prefixed-key
//! `ref`/`class` escape) and the deliberately weaker const rule
//! (`pass/hoist/consts.rs` module docs), whose every narrowing is
//! pinned here so a silent widening is a loud diff.

mod support;

use support::{assert_transformed_sound, with_transformed};
use vize_s1_to_s2::pass::{StaticFacts, StaticLevel};

/// The facts as `(op index, level, props, nested, native)` rows.
fn rows(facts: &vize_davinci::side_table::SideTable<StaticFacts>) -> Vec<(u32, StaticFacts)> {
    facts
        .sorted_entries()
        .into_iter()
        .map(|(id, fact)| (id.index(), *fact))
        .collect()
}

fn fact(
    level: StaticLevel,
    props_hoistable: bool,
    nested_static: bool,
    native_descendants: bool,
) -> StaticFacts {
    StaticFacts {
        level,
        props_hoistable,
        nested_static,
        native_descendants,
        foreign: false,
    }
}

/// The same rows inside a foreign markup namespace.
fn foreign_fact(
    level: StaticLevel,
    props_hoistable: bool,
    nested_static: bool,
    native_descendants: bool,
) -> StaticFacts {
    StaticFacts {
        foreign: true,
        ..fact(level, props_hoistable, nested_static, native_descendants)
    }
}

#[test]
fn a_static_tree_is_fully_static_with_hoistable_props() {
    with_transformed(
        r#"<div id="a" class="x"><b>t</b></div>"#,
        |_, _, facts, _| {
            assert_eq!(
                rows(&facts.static_facts),
                vec![
                    (0, fact(StaticLevel::FullyStatic, true, true, true)),
                    (1, fact(StaticLevel::FullyStatic, false, true, true)),
                ]
            );
        },
    );
    assert_transformed_sound(r#"<div id="a" class="x"><b>t</b></div>"#, "static tree");
}

#[test]
fn interpolation_children_make_dynamic_text_not_dynamic() {
    with_transformed(r#"<p id="k">{{ x }}</p>"#, |_, _, facts, _| {
        // The shipped ladder's middle rung: still prop-hoistable at
        // root, never whole-hoistable; the interpolation counts as a
        // static nested child (the shipped nested rule, mirrored).
        assert_eq!(
            rows(&facts.static_facts),
            vec![(0, fact(StaticLevel::HasDynamicText, true, true, true))]
        );
    });
}

#[test]
fn structural_children_force_not_static() {
    with_transformed(r#"<div><span v-if="a">x</span></div>"#, |_, _, facts, _| {
        // div(0) holds a ui.if(1) child: dynamic, nested and native
        // both broken. The span(2) inside the branch is itself fully
        // static (its v-if left the surface at lowering).
        assert_eq!(
            rows(&facts.static_facts),
            vec![
                (0, fact(StaticLevel::NotStatic, false, false, false)),
                (2, fact(StaticLevel::FullyStatic, false, true, true)),
            ]
        );
    });
}

#[test]
fn the_svg_directive_block_is_mirrored() {
    with_transformed(
        r#"<svg :width="100"><rect></rect></svg>"#,
        |_, _, facts, _| {
            // Any directive on a literal `<svg>` tag blocks staticness
            // (the shipped quirk) — but the binding itself is hoistable
            // (constant value), so the props surface stays hoistable and
            // the foreign-namespace props-hoist arm can fire on it; both
            // owners carry the foreign bit (the `ns != Html` arm's input).
            assert_eq!(
                rows(&facts.static_facts),
                vec![
                    (0, foreign_fact(StaticLevel::NotStatic, true, true, true)),
                    (
                        2,
                        foreign_fact(StaticLevel::FullyStatic, false, false, true)
                    ),
                ]
            );
        },
    );
}

#[test]
fn a_ref_attribute_blocks_staticness() {
    with_transformed(r#"<div ref="el">x</div>"#, |_, _, facts, _| {
        assert_eq!(
            rows(&facts.static_facts),
            vec![(0, fact(StaticLevel::NotStatic, false, true, true))]
        );
    });
}

#[test]
fn the_prefixed_key_escapes_the_ref_class_block() {
    // `:ref`/`:class` are unhoistable by key; the `.` shorthand's
    // prefixed key `.ref` deliberately passes — the shipped
    // `hoistable_static_bind_parts` quirk, mirrored byte-for-byte.
    with_transformed(r#"<i :ref="1">t</i>"#, |_, _, facts, _| {
        assert_eq!(rows(&facts.static_facts)[0].1.level, StaticLevel::NotStatic);
    });
    with_transformed(r#"<i :class="'x'">t</i>"#, |_, _, facts, _| {
        assert_eq!(rows(&facts.static_facts)[0].1.level, StaticLevel::NotStatic);
    });
    with_transformed(r#"<i .ref="1">t</i>"#, |_, _, facts, _| {
        assert_eq!(
            rows(&facts.static_facts),
            vec![(0, fact(StaticLevel::FullyStatic, true, true, true))]
        );
    });
}

#[test]
fn a_constant_bind_value_keeps_the_element_static() {
    with_transformed(
        r#"<i :tabindex="12 + 3" data-a="b">t</i>"#,
        |_, _, facts, _| {
            assert_eq!(
                rows(&facts.static_facts),
                vec![(0, fact(StaticLevel::FullyStatic, true, true, true))]
            );
        },
    );
}

#[test]
fn the_weaker_const_rule_refuses_every_recorded_narrowing() {
    // The four narrowings of `pass/hoist/consts.rs`, each pinned: any
    // identifier (the shipped classifier admits allowlisted globals),
    // `this` (the shipped visitor's blind spot), a TS-only spelling
    // (the shipped mjs re-parse refuses it, so admitting it would break
    // one-sidedness), and the mirrored context substrings.
    for (name, source) in [
        ("allowlist identifier", r#"<i :max="Math.PI">m</i>"#),
        ("this expression", r#"<i :n="this.x">m</i>"#),
        ("ts-only spelling", r#"<i :n="100 as any">m</i>"#),
        ("context substring", r#"<i :n="'_ctx.'">m</i>"#),
    ] {
        with_transformed(source, |_, _, facts, _| {
            let rows = rows(&facts.static_facts);
            assert_eq!(
                (rows[0].1.level, rows[0].1.props_hoistable),
                (StaticLevel::NotStatic, false),
                "{name} must refuse constness"
            );
        });
    }
}

#[test]
fn an_opaque_bind_value_is_never_constant() {
    // Pessimal law 3's first real consumer: `a b` fails the one shared
    // admission rule, arrives as `ExprRef::Opaque`, and no hoisting may
    // be justified by it — `OpaqueExpr::is_constant` is the answer, not
    // a re-derivation.
    with_transformed(r#"<i :n="a b">t</i>"#, |_, _, facts, _| {
        let rows = rows(&facts.static_facts);
        assert_eq!(
            (rows[0].1.level, rows[0].1.props_hoistable),
            (StaticLevel::NotStatic, false)
        );
    });
}

#[test]
fn a_slot_carrier_template_is_neither_nested_static_nor_native() {
    with_transformed(
        r#"<Card><template #head><b>x</b></template></Card>"#,
        |_, _, facts, _| {
            let rows = rows(&facts.static_facts);
            // Card(0): component — level fixed NotStatic, its only
            // child a carrier template: not a static nested child, not
            // a native descendant (the shipped `ElementType::Template`
            // gates, re-derived from the ui.slot-content binding).
            assert_eq!(
                rows[0],
                (0, fact(StaticLevel::NotStatic, false, false, false))
            );
            // The template(1) itself: the slot-content binding is
            // unhoistable, so NotStatic with an unhoistable surface.
            assert_eq!(rows[1].1.level, StaticLevel::NotStatic);
            assert!(!rows[1].1.props_hoistable);
        },
    );
    assert_transformed_sound(
        r#"<Card><template #head><b>x</b></template></Card>"#,
        "slot carrier",
    );
}

#[test]
fn a_static_props_outlet_counts_as_a_static_nested_child() {
    with_transformed(r#"<div><slot name="s"></slot></div>"#, |_, _, facts, _| {
        // The outlet keeps div dynamic (never whole-hoistable) and
        // breaks the native predicate, but the shipped nested rule's
        // slot arm counts a static-props outlet as a static nested
        // child — the `has_only_static_nested_children` props-hoist
        // gate can still fire on div.
        assert_eq!(
            rows(&facts.static_facts),
            vec![(0, fact(StaticLevel::NotStatic, false, true, false))]
        );
    });
}

#[test]
fn a_dynamic_name_outlet_breaks_the_nested_gate() {
    with_transformed(r#"<div><slot :name="n"></slot></div>"#, |_, _, facts, _| {
        // The separated name position is part of the outlet's props
        // surface for the nested rule (the legacy `:name` bind), and an
        // identifier name is not constant.
        assert_eq!(
            rows(&facts.static_facts),
            vec![(0, fact(StaticLevel::NotStatic, false, false, false))]
        );
    });
}

#[test]
fn the_camel_modifier_key_is_mirrored() {
    // `:foo-bar.camel="1"` hoists under its camelized key (the shipped
    // precedence chain: camel over prop over attr).
    with_transformed(r#"<i :foo-bar.camel="1">t</i>"#, |_, _, facts, _| {
        assert!(rows(&facts.static_facts)[0].1.props_hoistable);
    });
    // An unknown modifier refuses the whole binding.
    with_transformed(r#"<i :foo.sync="1">t</i>"#, |_, _, facts, _| {
        assert_eq!(rows(&facts.static_facts)[0].1.level, StaticLevel::NotStatic);
    });
}

#[test]
fn a_component_inherits_the_foreign_namespace_context() {
    // The corpus catch that forced the fact's `foreign` bit (directus
    // `arrows.vue`): `ui.component` carries no namespace of its own,
    // but the shipped `ns != Html` props-hoist arm reads the parser's
    // inherited namespace — so the analysis records the context.
    with_transformed(
        r#"<svg><Anim name="fade"><b>t</b></Anim></svg>"#,
        |_, _, facts, _| {
            let rows = rows(&facts.static_facts);
            // svg(0): foreign, dynamic (component child).
            assert!(rows[0].1.foreign);
            // Anim(1): foreign by context, NotStatic by kind, props
            // hoistable — exactly the combination the `ns != Html` arm
            // props-hoists in the shipped lane.
            assert_eq!(
                rows[1],
                (1, foreign_fact(StaticLevel::NotStatic, true, true, true))
            );
            // The b(2) under the component is back in its own element
            // namespace facts (still the svg context: no integration point
            // intervenes).
            assert!(rows[2].1.foreign);
        },
    );
}

#[test]
fn an_integration_point_returns_the_context_to_html() {
    with_transformed(
        r#"<svg><foreignObject><Card x="1">t</Card></foreignObject></svg>"#,
        |_, _, facts, _| {
            let rows = rows(&facts.static_facts);
            // foreignObject(1) is itself SVG; its children re-enter
            // HTML (the lowering's integration-point rule, mirrored),
            // so Card(2) is not foreign.
            assert!(rows[1].1.foreign);
            assert!(!rows[2].1.foreign);
        },
    );
}

#[test]
fn every_owner_gets_exactly_one_fact() {
    // Dense over the owner family: elements and components in, outlets
    // and text-family ops out.
    with_transformed(
        r#"<div><Card><template #a>x</template></Card><slot></slot><p>t {{ x }}</p></div>"#,
        |lowered, _, facts, _| {
            let owners = rows(&facts.static_facts).len();
            assert_eq!(owners, 4, "div, Card, template, p");
            let recorded = lowered
                .provenance
                .iter()
                .filter(|record| record.rule.as_str() == "pass.hoist-static.fact")
                .count();
            assert_eq!(recorded, owners, "one provenance record per fact");
        },
    );
}
