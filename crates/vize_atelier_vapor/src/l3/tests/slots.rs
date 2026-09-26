use super::options;
use crate::l3::{LegacyReason, VaporL3BridgeStatus, lower_source_for_vapor};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn admitted(source: &str) -> bool {
    let allocator = Allocator::new();
    matches!(
        lower_source_for_vapor(&allocator, source, options()),
        VaporL3BridgeStatus::Accepted(_)
    )
}

#[test]
fn computed_outlet_names_use_native_l3_with_retained_expressions() {
    for source in [
        r#"<slot :name="name"></slot>"#,
        r#"<slot :name="names[selected]" :value="count"><b>{{ fallback }}</b></slot>"#,
        r#"<slot :name="enabled ? first : second"></slot>"#,
        r#"<slot :name="'prefix-' + selected"></slot>"#,
        r#"<slot :name="name.toLowerCase()"></slot>"#,
    ] {
        assert!(admitted(source), "{source}");
        for prefix_identifiers in [false, true] {
            let allocator = Allocator::new();
            let native = compile_vapor(
                &allocator,
                source,
                VaporCompilerOptions {
                    prefix_identifiers,
                    ..Default::default()
                },
            );
            assert_eq!(native.error_messages.len(), 0, "{source}");
            let retained = compile_vapor(
                &allocator,
                source,
                VaporCompilerOptions {
                    prefix_identifiers,
                    davinci_retained_lane: true,
                    ..Default::default()
                },
            );
            assert_eq!(
                native.code, retained.code,
                "{source}: prefix={prefix_identifiers}"
            );
        }
    }
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta formats exact generated-code snapshots"
)]
fn computed_outlet_name_is_owned_by_the_checked_graph() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(&allocator, r#"<slot :name="original"></slot>"#);
    let name = s3
        .program
        .operands
        .iter_mut()
        .find(|operand| operand.role == vize_l3::operand::OperandRole::Name)
        .unwrap();
    name.value.text = "replacement";
    let code = super::generated(
        crate::l3::admit(s3, &crate::l3::retained::Retained::new(&allocator)),
        &allocator,
    );
    insta::assert_snapshot!("computed_outlet_payload", code);
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta formats exact generated-code snapshots"
)]
fn computed_content_name_is_owned_by_the_checked_graph() {
    let allocator = Allocator::new();
    let mut s3 = super::lowered_source(
        &allocator,
        r#"<MyComp><template #[original]>content</template></MyComp>"#,
    );
    let name = s3
        .program
        .operands
        .iter_mut()
        .find(|operand| {
            operand.role == vize_l3::operand::OperandRole::Name && operand.target.is_some()
        })
        .unwrap();
    name.value.text = "replacement";
    let code = super::generated(
        crate::l3::admit(s3, &crate::l3::retained::Retained::new(&allocator)),
        &allocator,
    );
    insta::assert_snapshot!("computed_content_payload", code);
}

/// Named and scoped slots at the template root match the retained lane byte
/// for byte, including the shared generator's plain-identifier parameter.
#[test]
fn named_and_scoped_slots_match_the_retained_lane() {
    for source in [
        r#"<MyComponent><template #header>H</template><template #footer>F</template></MyComponent>"#,
        r#"<MyComponent v-slot:head="p">{{ p.x }}</MyComponent>"#,
        r#"<MyComponent v-slot:[name]="{ item }">{{ item }}</MyComponent>"#,
        r#"<MyComponent><template #[name]>x</template></MyComponent>"#,
        r#"<MyComponent><template #[names[selected]]="{ item }">{{ item }}</template><template #fixed>fixed</template></MyComponent>"#,
        r#"<MyComponent><template #['slot-'+selected]>x</template><template #[selected.toLowerCase()]>y</template></MyComponent>"#,
        r#"<MyComponent v-slot="{ item, index }">{{ index }}: {{ item }}</MyComponent>"#,
        r#"<MyComponent><template #item="{ data }">{{ data }}</template></MyComponent>"#,
        r#"<MyComponent v-slot="p">{{ p.x }}</MyComponent>"#,
        r#"<MyComponent v-slot:default="{ n }"><b :title="n">{{ n + 1 }}</b></MyComponent>"#,
        r#"<MyComponent><template #default>d</template><template #row="{ r }"><i v-if="r.on">{{ r.a }}</i></template></MyComponent>"#,
    ] {
        assert!(admitted(source), "{source}");
        for prefix_identifiers in [false, true] {
            let allocator = Allocator::new();
            let compile = |davinci_retained_lane| {
                compile_vapor(
                    &allocator,
                    source,
                    VaporCompilerOptions {
                        prefix_identifiers,
                        davinci_retained_lane,
                        ..Default::default()
                    },
                )
            };
            assert_eq!(
                compile(false).code,
                compile(true).code,
                "{source}: prefix={prefix_identifiers}"
            );
        }
    }
    // Inside an element only the node numbering differs (parent-first).
    assert!(admitted(
        r#"<div><MyComponent><template #item="{ data }"><b>{{ data }}</b></template></MyComponent><i>{{ z }}</i></div>"#
    ));
}

#[test]
fn plain_identifier_slot_params_name_the_props_object() {
    let allocator = Allocator::new();
    let source = r#"<MyComponent v-slot="p">{{ p.x }}</MyComponent>"#;
    for retained in [false, true] {
        let code = compile_vapor(
            &allocator,
            source,
            VaporCompilerOptions {
                davinci_retained_lane: retained,
                ..Default::default()
            },
        )
        .code;
        assert_eq!(
            [
                code.matches("(p) =>").count(),
                code.matches("_toDisplayString(p.x)").count()
            ],
            [1, 1],
            "{code}"
        );
        assert_eq!(code.matches("_slotProps").count(), 0, "{code}");
    }
}

#[test]
fn unsupported_slot_shapes_select_exact_legacy_reasons() {
    use LegacyReason::{Component, Element};
    for (source, reason) in [
        // Implicit default content beside named templates.
        (
            r#"<MyComp><template #head>H</template>body</MyComp>"#,
            Component,
        ),
        (
            r#"<MyComp><template #a>x</template><template #a>y</template></MyComp>"#,
            Component,
        ),
        (r#"<MyComp v-slot="{ a: b }">{{ b }}</MyComp>"#, Component),
        (r#"<MyComp v-slot="{ a = 1 }">{{ a }}</MyComp>"#, Component),
        (
            r#"<MyComp v-slot="{ ...rest }">{{ rest }}</MyComp>"#,
            Component,
        ),
        (r#"<MyComp v-slot="[a]">{{ a }}</MyComp>"#, Component),
        (r#"<MyComp v-slot="{ a, a }">{{ a }}</MyComp>"#, Component),
        // A template outside a component is not slot content.
        (r#"<div><template #a>x</template></div>"#, Element),
        (r#"<div><template><b>x</b></template></div>"#, Element),
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Legacy(actual) if actual == reason),
            "{source}: expected {reason:?}, got {status:?}"
        );
    }
}
