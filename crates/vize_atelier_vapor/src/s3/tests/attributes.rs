use super::{generated, options};
use crate::s3::{
    LegacyReason, VaporS3BridgeStatus, admit, lower_source_for_vapor, retained::Retained,
};
use vize_carton::Allocator;
use vize_s3::operand::OperandRole;

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
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
    }
}

#[test]
fn unsupported_attribute_shapes_select_exact_legacy_reasons() {
    use LegacyReason::{Binding, ExpressionOrEncoding, Operation, Structure};
    for (source, reason) in [
        (r#"<div v-html="markup"><b>child</b></div>"#, Structure),
        (r#"<div v-text="label">child</div>"#, Structure),
        (r#"<div style="color: red" :style="s"></div>"#, Binding),
        (r#"<div class :class="c"></div>"#, Binding),
        (r#"<div :class="a" :class="b"></div>"#, Binding),
        (r#"<input v-model="value">"#, Binding),
        (r#"<div v-focus="value"></div>"#, Operation),
        (r#"<div v-once>{{ value }}</div>"#, Operation),
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
            matches!(status, VaporS3BridgeStatus::Legacy(actual) if actual == reason),
            "{source}: expected {reason:?}, got {status:?}"
        );
    }
}

#[test]
fn retained_asts_never_describe_stale_operand_text() {
    let source = r#"<div :title="a + b">{{ c * d }}</div>"#;
    let lower = |allocator: &Allocator, text: Option<&'static str>| {
        let scratch = Allocator::new();
        let (tree, errors) = vize_s1::parse(&scratch, source);
        let s2 = vize_s1_to_s2::lower(&scratch, &tree, &errors);
        let mut s3 = vize_s2_to_s3::lower(allocator, &s2.root);
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
            VaporS3BridgeStatus::Accepted(_) => generated(status, allocator),
            other => vize_carton::cstr!("{other:?}"),
        }
    };
    let allocator = Allocator::new();
    let code = lower(&allocator, None);
    assert!(
        code.contains("_setProp(n0, \"title\", _ctx.a + _ctx.b)"),
        "{code}"
    );
    assert!(code.contains("_toDisplayString(_ctx.c * _ctx.d)"), "{code}");
    // Replacing the operand text orphans the retained parse: the artifact is
    // refused instead of emitting either the stale tree or reparsed text.
    assert_eq!(
        lower(&allocator, Some("a - b")),
        "Legacy(ExpressionOrEncoding)"
    );
    // A direct reference needs no AST and drives generation from the payload.
    let code = lower(&allocator, Some("changed"));
    assert!(
        code.contains("_setProp(n0, \"title\", _ctx.changed)"),
        "{code}"
    );
}
