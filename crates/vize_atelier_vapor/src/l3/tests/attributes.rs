use super::{generated, lowered_source, options};
use crate::l3::{
    LegacyReason, VaporL3BridgeStatus, admit, lower_source_for_vapor, retained::Retained,
};
use vize_carton::Allocator;
use vize_l3::operand::OperandRole;

/// A concatenation that starts and ends with a quote is not one string
/// literal: its references still resolve (the generator once passed it
/// through, leaving `value` unprefixed on the native lane only).
#[test]
fn quote_bounded_concatenations_resolve_their_references() {
    for source in [
        r#"<p>{{ '[' + value + ']' }}</p>"#,
        r#"<p :title="'a' + b + 'c'">x</p>"#,
        r#"<p>{{ "x" + n + "y" }} {{ 'it\'s' }}</p>"#,
    ] {
        for prefix_identifiers in [false, true] {
            let compile = |davinci_retained_lane| {
                let allocator = Allocator::new();
                crate::compile_vapor(
                    &allocator,
                    source,
                    crate::VaporCompilerOptions {
                        prefix_identifiers,
                        davinci_retained_lane,
                        ..Default::default()
                    },
                )
                .code
            };
            assert_eq!(compile(false), compile(true), "{source}");
        }
    }
}

#[test]
fn expression_and_content_directive_shapes_are_admitted() {
    for source in [
        r#"<div :title="a + b" :class="{ on: active, [name]: flag }">{{ list.map(x => x.id).join(',') }}</div>"#,
        r#"<div>Total: {{ count * 2 }} of {{ max ?? 10 }}</div>"#,
        r#"<button @click="save(id)" @keydown.enter="count++" @focus="() => focus(1)">Go</button>"#,
        r#"<div class="base" :class="[size, { active }]" :style="{ color }"></div>"#,
        r#"<div v-show="visible && ready"><span v-text="label"></span><i v-html="markup"></i></div>"#,
        r#"<input v-show="open" type="checkbox">"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
    }
}

#[test]
fn simple_once_element_uses_native_one_shot_operations() {
    for (source, effects) in [
        (r#"<div v-once :title="tip">{{ msg }}</div>"#, 0),
        (
            r#"<main><div v-once :title="tip">{{ msg }}</div><span>{{ live }}</span></main>"#,
            1,
        ),
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
        for prefix_identifiers in [false, true] {
            let compile = |davinci_retained_lane| {
                let allocator = Allocator::new();
                crate::compile_vapor(
                    &allocator,
                    source,
                    crate::VaporCompilerOptions {
                        prefix_identifiers,
                        davinci_retained_lane,
                        ..Default::default()
                    },
                )
                .code
            };
            let native = compile(false);
            if effects == 0 {
                assert_eq!(native, compile(true), "{source}");
            }
            assert_eq!(
                native.matches("_renderEffect(() =>").count(),
                effects,
                "{native}"
            );
        }
    }
}

#[test]
fn once_plain_descendants_are_mount_only() {
    for (source, effects) in [
        (
            r#"<div v-once><span :title="tip">{{ value }}</span></div>"#,
            0,
        ),
        (
            r#"<main><div v-once><span><b :title="tip">{{ value }}</b></span></div><i>{{ live }}</i></main>"#,
            1,
        ),
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
        for prefix_identifiers in [false, true] {
            let compiled = crate::compile_vapor(
                &allocator,
                source,
                crate::VaporCompilerOptions {
                    prefix_identifiers,
                    ..Default::default()
                },
            );
            assert!(compiled.error_messages.is_empty(), "{source}");
            assert_eq!(
                compiled.code.matches("_renderEffect(() =>").count(),
                effects,
                "{source}: {}",
                compiled.code
            );
        }
    }
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta records complete generated modules and template lists"
)]
fn static_and_bound_styles_merge_in_authored_order() {
    let mut outputs = Vec::new();
    for source in [
        r#"<div style="color: red;" :style="theme"></div>"#,
        r#"<div :style="theme" style="color: red;"></div>"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
        let compiled = crate::compile_vapor(
            &allocator,
            source,
            crate::VaporCompilerOptions {
                prefix_identifiers: true,
                ..Default::default()
            },
        );
        assert!(
            compiled.error_messages.is_empty(),
            "{source}: {:?}",
            compiled.error_messages
        );
        outputs.push((source, compiled.code, compiled.templates));
    }
    insta::assert_debug_snapshot!("authored_style_order", outputs);
}

/// Official Vapor binds a bare attribute as `""`. Style keeps authored order
/// on the native lane; class, and every retained merge, stays static-first.
#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta records both complete generation lanes for every option"
)]
fn valueless_static_class_and_style_merge_as_empty_strings() {
    let mut outputs = Vec::new();
    for (source, equal) in [
        (r#"<div style :style="s"></div>"#, true),
        (r#"<div :style="s" style></div>"#, false),
        (r#"<div class :class="c"></div>"#, true),
        (r#"<div :class="c" class></div>"#, true),
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
        for prefix_identifiers in [false, true] {
            let compile = |davinci_retained_lane| {
                crate::compile_vapor(
                    &allocator,
                    source,
                    crate::VaporCompilerOptions {
                        prefix_identifiers,
                        davinci_retained_lane,
                        ..Default::default()
                    },
                )
            };
            let native = compile(false);
            let retained = compile(true);
            assert!(
                native.error_messages.is_empty() && retained.error_messages.is_empty(),
                "{source}: {:?} {:?}",
                native.error_messages,
                retained.error_messages
            );
            if equal {
                assert_eq!(native.code, retained.code, "{source}");
                assert_eq!(native.templates, retained.templates, "{source}");
            } else {
                assert_ne!(native.code, retained.code, "{source}");
            }
            // Keep both complete modules and their static DOM lists. In
            // particular, only native style follows the authored order.
            outputs.push((
                source,
                prefix_identifiers,
                native.code,
                native.templates,
                retained.code,
                retained.templates,
            ));
        }
    }
    insta::assert_debug_snapshot!("valueless_static_attributes", outputs);
}

#[test]
fn cloak_is_a_native_one_shot_directive_in_static_and_branch_regions() {
    for source in [
        r#"<div v-cloak></div>"#,
        r#"<div v-cloak>{{ label }}</div>"#,
        r#"<main><div v-if="open" v-cloak :title="tip">{{ label }}</div><p>tail</p></main>"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
        for prefix_identifiers in [false, true] {
            let compile = |davinci_retained_lane| {
                crate::compile_vapor(
                    &allocator,
                    source,
                    crate::VaporCompilerOptions {
                        prefix_identifiers,
                        davinci_retained_lane,
                        ..Default::default()
                    },
                )
            };
            let native = compile(false);
            let retained = compile(true);
            assert_eq!(native.templates, retained.templates, "{source}");
            for code in [native.code, retained.code] {
                assert_eq!(
                    code.matches(".removeAttribute(\"v-cloak\")").count(),
                    1,
                    "{source}: {code}"
                );
            }
        }
    }
}

#[test]
fn cloak_schema_rejects_an_unmatched_directive_kind() {
    let allocator = Allocator::new();
    let mut s3 = lowered_source(&allocator, r#"<div v-cloak>{{ label }}</div>"#);
    let kind = s3
        .program
        .operands
        .iter_mut()
        .find(|operand| operand.role == OperandRole::BindingKind)
        .expect("cloak binding kind");
    kind.value.text = "vue.show";
    let status = admit(s3, &Retained::new(&allocator));
    assert!(
        matches!(status, VaporL3BridgeStatus::Rejected(_)),
        "{status:?}"
    );
}

#[test]
fn unsupported_attribute_shapes_select_exact_legacy_reasons() {
    use LegacyReason::{Binding, ExpressionOrEncoding, Operation, Structure};
    for (source, reason) in [
        (r#"<div v-html="markup"><b>child</b></div>"#, Structure),
        (r#"<div v-text="label">child</div>"#, Structure),
        (r#"<div :class="a" :class="b"></div>"#, Binding),
        (r#"<input :type="kind" v-model="value">"#, Binding),
        (r#"<div v-focus="value"></div>"#, Operation),
        (
            r#"<div v-once><span v-if="open">{{ value }}</span></div>"#,
            Operation,
        ),
        (r#"<div v-once><Foo :value="value" /></div>"#, Operation),
        (
            r#"<div v-once><span v-once>{{ value }}</span></div>"#,
            Operation,
        ),
        (r#"<div v-if="open" v-once>{{ value }}</div>"#, Operation),
        (
            r#"<div v-once v-show="visible">{{ value }}</div>"#,
            Operation,
        ),
        (
            r#"<button @click="a++; b++">x</button>"#,
            ExpressionOrEncoding,
        ),
        (
            r#"<div :title="value as string"></div>"#,
            ExpressionOrEncoding,
        ),
        (r#"<div :title="$attrs.title"></div>"#, ExpressionOrEncoding),
        (r#"<div :title="_ctx.title"></div>"#, ExpressionOrEncoding),
        (
            r#"<button @click="$event.target"></button>"#,
            ExpressionOrEncoding,
        ),
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Legacy(actual) if actual == reason),
            "{source}: expected {reason:?}, got {status:?}"
        );
    }
}

#[test]
#[expect(
    clippy::disallowed_macros,
    reason = "insta records complete original and mutated operand modules"
)]
fn retained_asts_never_describe_stale_operand_text() {
    let source = r#"<div :title="a + b">{{ c * d }}</div>"#;
    let lower = |allocator: &Allocator, text: Option<&'static str>| {
        let scratch = Allocator::new();
        let (tree, errors) = vize_l1::markup::parse(&scratch, source);
        let s2 = vize_l1_to_l2::lower(&scratch, &tree, &errors);
        let mut s3 = vize_l2_to_l3::lower(allocator, &s2.root);
        let retained = Retained::collect(allocator, &s2.root);
        if let Some(text) = text {
            let value = s3
                .program
                .operands
                .iter_mut()
                .find(|operand| operand.role == OperandRole::Value)
                .unwrap();
            value.value.text = text;
        }
        let status = admit(s3, &retained);
        match status {
            VaporL3BridgeStatus::Accepted(_) => generated(status, allocator),
            other => vize_carton::cstr!("{other:?}"),
        }
    };
    let allocator = Allocator::new();
    let code = lower(&allocator, None);
    // Replacing the operand text orphans the retained parse: the artifact is
    // refused instead of emitting either the stale tree or reparsed text.
    assert_eq!(
        lower(&allocator, Some("a - b")),
        "Legacy(ExpressionOrEncoding)"
    );
    // A direct reference needs no AST and drives generation from the payload.
    let changed = lower(&allocator, Some("changed"));
    insta::assert_debug_snapshot!("retained_operand_text", (code, changed));
}
