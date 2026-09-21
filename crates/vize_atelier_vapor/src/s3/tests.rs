use super::{
    LegacyReason, VaporS3BridgeOptions, VaporS3BridgeStatus, admit, lower_source_for_vapor,
    retained::Retained,
};
use vize_atelier_core::TemplateSyntaxMode;
use vize_carton::Allocator;
use vize_s3::{
    op::OpId,
    operand::{OperandRole, ValueKind},
};

pub(super) fn options() -> VaporS3BridgeOptions {
    VaporS3BridgeOptions {
        ssr: false,
        custom_renderer: false,
        experimental_in_tag_comments: false,
        experimental_patterned_template: false,
        template_syntax: TemplateSyntaxMode::Standard,
        has_custom_elements: false,
        prefixed_binding_metadata: false,
        retained_lane: false,
        inline: false,
    }
}

const SOURCE: &str =
    r#"<main class="shell"><button :disabled="locked" @click="save">{{ label }}</button></main>"#;

pub(super) fn generated(
    status: VaporS3BridgeStatus<'_>,
    allocator: &Allocator,
) -> vize_carton::String {
    let VaporS3BridgeStatus::Accepted(artifact) = status else {
        panic!("expected an executable artifact: {status:?}");
    };
    // The decoy source cannot provide any of the generated DOM or expressions.
    let ir = artifact.into_ir(allocator, "<aside>decoy</aside>", None);
    crate::generate::generate_vapor(&ir, None).code
}

#[test]
fn accepted_payload_outlives_surface_storage_and_supplies_generation() {
    let allocator = Allocator::new();
    let status = {
        let source = vize_carton::String::from(SOURCE);
        lower_source_for_vapor(&allocator, &source, options())
    };
    let code = generated(status, &allocator);
    assert!(
        code.contains("<main class=\\\"shell\\\"><button> </button></main>"),
        "{code}"
    );
    for value in ["_ctx.locked", "_ctx.save(e)", "_ctx.label"] {
        assert!(code.contains(value), "missing {value}: {code}");
    }
    assert!(!code.contains("decoy"));
}

fn lowered<'a>(allocator: &'a Allocator) -> vize_s2_to_s3::Lowered<'a> {
    lowered_source(allocator, SOURCE)
}

pub(super) fn lowered_source<'a>(
    allocator: &'a Allocator,
    source: &str,
) -> vize_s2_to_s3::Lowered<'a> {
    let scratch = Allocator::new();
    let (tree, errors) = vize_s1::parse(&scratch, source);
    let s2 = vize_s1_to_s2::lower(&scratch, &tree, &errors);
    vize_s2_to_s3::lower(allocator, &s2.root)
}

#[test]
fn graph_payload_mutations_change_generated_behavior() {
    let allocator = Allocator::new();
    let mut s3 = lowered(&allocator);
    for operand in &mut s3.program.operands {
        if operand.role == OperandRole::Text && operand.value.kind == ValueKind::Js {
            operand.value.text = "message";
        }
        if operand.role == OperandRole::Attribute && operand.name == Some("class") {
            operand.value.text = "changed";
        }
        if operand.role == OperandRole::Value && operand.value.text == "save" {
            operand.value.text = "submit";
        }
    }
    let code = generated(admit(s3, &Retained::new(&allocator)), &allocator);
    for value in ["changed", "_ctx.message", "_ctx.submit(e)"] {
        assert!(code.contains(value), "{code}");
    }
    for value in ["shell", "_ctx.label", "_ctx.save(e)"] {
        assert!(!code.contains(value), "stale payload {value}: {code}");
    }
}

#[test]
fn corrupt_graph_and_partition_are_rejected_instead_of_falling_back() {
    let allocator = Allocator::new();
    for mutation in 0..4 {
        let mut s3 = lowered(&allocator);
        match mutation {
            0 => s3.program.operands[0].op = OpId::new(999),
            1 => s3.partition.ops[1].op = s3.partition.ops[0].op,
            2 => s3.partition.ops[0].span.end += 1,
            _ => s3.partition.ops[0].kind = vize_s2_to_s3::PartitionKind::Dynamic,
        }
        assert!(
            matches!(
                admit(s3, &Retained::new(&allocator)),
                VaporS3BridgeStatus::Rejected(_)
            ),
            "mutation {mutation}"
        );
    }
}

#[test]
fn generic_graph_verification_does_not_imply_backend_admission() {
    let allocator = Allocator::new();
    let mut reordered = lowered(&allocator);
    reordered.program.edges.reverse();
    assert!(matches!(
        admit(reordered, &Retained::new(&allocator)),
        VaporS3BridgeStatus::Accepted(_)
    ));
    for mutation in 0..6 {
        let mut s3 = lowered(&allocator);
        match mutation {
            0 => {
                s3.program.operands.remove(0);
            }
            1 => {
                let value = s3.program.operands[0];
                s3.program.operands.push(value);
            }
            2 => {
                let value = s3
                    .program
                    .operands
                    .iter_mut()
                    .find(|v| v.role == OperandRole::Text)
                    .unwrap();
                value.value.kind = ValueKind::Opaque;
                value.value.qualifier = "compound";
            }
            3 => {
                s3.program.edges.pop();
            }
            4 => {
                let value = s3
                    .program
                    .operands
                    .iter_mut()
                    .find(|v| v.role == OperandRole::BindingKind)
                    .unwrap();
                value.value.text = "unmodeled";
            }
            _ => {
                // Preserve the length and individually valid edges while
                // replacing a required dependency with a duplicate.
                s3.program.edges[1] = s3.program.edges[0];
            }
        }
        assert!(vize_s3::verify::verify(&s3.program).is_empty());
        let status = admit(s3, &Retained::new(&allocator));
        if matches!(mutation, 0 | 1 | 3 | 5) {
            assert!(
                matches!(status, VaporS3BridgeStatus::Rejected(_)),
                "mutation {mutation}: {status:?}"
            );
        } else {
            assert!(
                matches!(status, VaporS3BridgeStatus::Legacy(_)),
                "mutation {mutation}: {status:?}"
            );
        }
    }
}

#[test]
fn unsupported_source_semantics_have_explicit_legacy_routes() {
    let allocator = Allocator::new();
    for source in [
        "<component :is=\"view\" />",
        "<template v-if=\"ok\" :key=\"k\"><div></div><div></div></template>",
        "<div v-for=\"x in (xs as any)\"></div>",
        "<div>{{ one as number }}</div>",
        "<div v-pre>{{ literal }}</div>",
        "<div v-once></div>",
        "<input v-model=\"text\" />",
        "<input v-model.lazy=\"text\" />",
        "<input type=\"checkbox\" v-model=\"checked\" />",
        "<div v-cloak></div>",
        "<div :[key]=\"value\"></div>",
        "<div ref=\"node\"></div>",
        "<div :style=\"s\" style=\"color: red\"></div>",
        "<div @click=\"a++; b++\"></div>",
        "<button @click=\"$event\"></button>",
        "<svg><circle /></svg>",
        "<table><tr><td>{{ value }}</td></tr></table>",
        "<div>&#10; text</div>",
        "<div title=\"&quot;\"></div>",
        "<Comp v-slot=\"{ p: q }\">{{ q }}</Comp>",
        "<Teleport to=\"body\"><div></div></Teleport>",
        "<div v-bind=\"props\"></div>",
        "<pre> text </pre>",
        // The legacy parser reports invalid self-closing HTML elements.
        "<div />",
        "<span v-if=\"ok\" />",
    ] {
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporS3BridgeStatus::Legacy(_)
            ),
            "{source}"
        );
    }
    assert!(matches!(
        lower_source_for_vapor(
            &allocator,
            SOURCE,
            VaporS3BridgeOptions {
                prefixed_binding_metadata: true,
                ..options()
            }
        ),
        VaporS3BridgeStatus::Legacy(LegacyReason::Options)
    ));
    // Baselines and A/B runs select the retained lane explicitly; the selection
    // is counted separately from unsupported options.
    assert!(matches!(
        lower_source_for_vapor(
            &allocator,
            SOURCE,
            VaporS3BridgeOptions {
                retained_lane: true,
                ..options()
            }
        ),
        VaporS3BridgeStatus::Legacy(LegacyReason::Selected)
    ));
}

#[test]
fn native_event_families_are_admitted() {
    let allocator = Allocator::new();
    let source = "<div @focus=\"save\" @change.once=\"save\" @custom-event.capture=\"save\"></div>";
    let status = lower_source_for_vapor(&allocator, source, options());
    assert!(
        matches!(status, VaporS3BridgeStatus::Accepted(_)),
        "{status:?}"
    );
    for directive in [
        "focus",
        "change.once",
        "custom-event.capture",
        "keydown.enter.stop",
        "click.once.capture.passive",
    ] {
        let source = vize_carton::cstr!("<div @{directive}=\"save\"></div>");
        let status = lower_source_for_vapor(&allocator, &source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
            "{directive}: {status:?}"
        );
    }
}

#[test]
fn adjacent_text_runs_are_admitted() {
    let allocator = Allocator::new();
    let source = "<div>hello {{ name }}!<span>{{ a }}{{ b }}</span>tail {{ end }}</div>";
    let status = lower_source_for_vapor(&allocator, source, options());
    assert!(
        matches!(status, VaporS3BridgeStatus::Accepted(_)),
        "{status:?}"
    );
}

#[test]
fn empty_static_text_cannot_shift_materialized_child_addresses() {
    let allocator = Allocator::new();
    let mut s3 = lowered_source(&allocator, "<div>prefix<span>{{ label }}</span></div>");
    let text = s3
        .program
        .operands
        .iter_mut()
        .find(|operand| {
            operand.role == OperandRole::Text && operand.value.kind == ValueKind::Literal
        })
        .unwrap();
    text.value.text = "";
    assert!(vize_s3::verify::verify(&s3.program).is_empty());
    // Empty HTML text produces no DOM node. Counting it as a child would make
    // the following dynamic span address the wrong browser node.
    assert!(matches!(
        admit(s3, &Retained::new(&allocator)),
        VaporS3BridgeStatus::Legacy(LegacyReason::ExpressionOrEncoding)
    ));
}

mod control;

mod attributes;
mod components;
mod parser_agreement;
mod slots;
mod templates;
