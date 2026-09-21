use super::{generated, lowered_source, options};
use crate::s3::{
    LegacyReason, VaporS3BridgeStatus, admit, lower_source_for_vapor, retained::Retained,
};
use vize_carton::Allocator;
use vize_s3::operand::{OperandRole, ValueKind};

const BRANCHES: &str =
    r#"<div><span v-if="a">A</span><b v-else-if="b">{{ label }}</b><i v-else>C</i></div>"#;
const LOOP: &str = r#"<ul><li v-for="(item, name, position) in items" :key="item.id" :title="name">{{ position }}</li></ul>"#;

#[test]
fn control_flow_shapes_are_admitted() {
    for source in [
        BRANCHES,
        LOOP,
        r#"<div v-if="ready">root</div>"#,
        r#"<div v-if="a">A</div><div v-else>B</div>"#,
        r#"<span v-for="n in 3">{{ n }}</span>"#,
        r#"<section><div v-for="row in rows" :key="row.id"><b v-if="row.open">{{ row.label }}</b><ul><li v-for="cell in row.cells">{{ cell }}</li></ul></div></section>"#,
        r#"<main><button v-for="item in items" :key="item" @click="save">{{ item }}</button></main>"#,
        // A loop body is its own template; DOM insertion never reparses it.
        r#"<ul><li v-for="group in groups"><ul><li v-for="item in group.items">{{ item }}</li></ul></li></ul>"#,
        // Compound conditions, sources and keys consume retained ASTs.
        r#"<div><span v-if="a.b() && !c">x</span><i v-else-if="n > 1">y</i></div>"#,
        r#"<ul><li v-for="x in xs.slice(1)" :key="x.id + 1">{{ x.a + x.b }}</li></ul>"#,
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
fn unsupported_control_flow_selects_exact_legacy_reasons() {
    use LegacyReason::{Binding, ControlFlow, ExpressionOrEncoding, Structure};
    for (source, reason) in [
        // S2 keeps template wrappers and carrier branch keys as side facts.
        (
            r#"<div><template v-if="a"><span>x</span></template></div>"#,
            ControlFlow,
        ),
        (
            r#"<div><template v-for="x in xs"><span>{{ x }}</span></template></div>"#,
            ControlFlow,
        ),
        (
            r#"<div><template v-for="x in xs" :key="x.id"><span>{{ x }}</span></template></div>"#,
            ControlFlow,
        ),
        (
            r#"<div><span v-if="a" :key="k">x</span></div>"#,
            ControlFlow,
        ),
        (
            r#"<ul><li v-if="ok" v-for="x in xs">{{ x }}</li></ul>"#,
            ControlFlow,
        ),
        (
            r#"<ul><li v-for="{ id } in xs">{{ id }}</li></ul>"#,
            ControlFlow,
        ),
        (
            r#"<ul><li v-for="(x, x) in xs">{{ x }}</li></ul>"#,
            ControlFlow,
        ),
        (
            r#"<ul><li v-for="_x in xs">{{ _x }}</li></ul>"#,
            ControlFlow,
        ),
        (
            r#"<ul><li v-for="Math in xs">{{ Math }}</li></ul>"#,
            ControlFlow,
        ),
        (
            r#"<div><span v-if="a as boolean">x</span></div>"#,
            ExpressionOrEncoding,
        ),
        (
            r#"<div><span v-if="$props.a">x</span></div>"#,
            ExpressionOrEncoding,
        ),
        (
            r#"<ul><li v-for="x in (xs as any[])">{{ x }}</li></ul>"#,
            ExpressionOrEncoding,
        ),
        (
            r#"<ul><li v-for="x in xs" :key="x.id as string">{{ x }}</li></ul>"#,
            ExpressionOrEncoding,
        ),
        (r#"<div><span :key="k">x</span></div>"#, Binding),
        (
            r#"<ul><li v-for="x in xs" key="static">{{ x }}</li></ul>"#,
            Binding,
        ),
        (r#"<ul><li><span><li>x</li></span></li></ul>"#, Structure),
        (r#"<ul><li><b><li>y</li></b></li></ul>"#, Structure),
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
fn control_flow_payload_mutations_drive_generation() {
    let allocator = Allocator::new();
    let mut s3 = lowered_source(&allocator, LOOP);
    for operand in &mut s3.program.operands {
        match (operand.role, operand.value.text) {
            (OperandRole::ForSource, "items") => operand.value.text = "rows",
            (OperandRole::Value, "item.id") => operand.value.text = "item.uid",
            _ => {}
        }
    }
    let code = generated(admit(s3, &Retained::new(&allocator)), &allocator);
    for expected in [
        "_createFor(() => (_ctx.rows), (_for_item0, _for_key0, _for_index0) => {",
        "}, (item, name, position) => (item.uid))",
        "_setProp(n4, \"title\", _for_key0.value)",
        "_toDisplayString(_for_index0.value)",
    ] {
        assert!(code.contains(expected), "missing {expected}: {code}");
    }
    assert!(
        !code.contains("items") && !code.contains("item.id"),
        "{code}"
    );

    let mut s3 = lowered_source(&allocator, BRANCHES);
    for operand in &mut s3.program.operands {
        if operand.role == OperandRole::Condition && operand.value.text == "b" {
            operand.value.text = "changed";
        }
    }
    let code = generated(admit(s3, &Retained::new(&allocator)), &allocator);
    assert!(
        code.contains("_createIf(() => (_ctx.a)")
            && code.contains("_createIf(() => (_ctx.changed)")
            && !code.contains("_ctx.b)"),
        "{code}"
    );
}

#[test]
fn corrupt_control_flow_graphs_are_rejected_instead_of_falling_back() {
    let allocator = Allocator::new();
    for mutation in 0..5 {
        let mut s3 = lowered_source(&allocator, if mutation < 3 { BRANCHES } else { LOOP });
        let operands = &mut s3.program.operands;
        let find = |role: OperandRole, text: &str, operands: &[vize_s3::operand::Operand<'_>]| {
            operands
                .iter()
                .position(|operand| operand.role == role && operand.value.text == text)
                .unwrap()
        };
        match mutation {
            // A branch region is no longer accounted for by a condition.
            0 => {
                let index = find(OperandRole::Condition, "b", operands);
                operands.remove(index);
            }
            // Two conditions claim one branch.
            1 => {
                let value = operands[find(OperandRole::Condition, "b", operands)];
                operands.push(value);
            }
            // The unconditional branch is no longer trailing.
            2 => {
                let index = find(OperandRole::Condition, "a", operands);
                operands[index].value.kind = ValueKind::Absent;
                operands[index].value.text = "";
            }
            // A loop loses one binding position.
            3 => {
                let index = find(OperandRole::ForKey, "name", operands);
                operands.remove(index);
            }
            // A loop body element loses its inherited dynamic partition.
            _ => {
                let body = s3
                    .program
                    .ops
                    .iter()
                    .position(|op| op.kind == vize_s3::op::OpKind::InsertNode && op.span.start == 4)
                    .unwrap();
                s3.program.ops[body].effect = None;
                s3.partition.ops[body].kind = vize_s2_to_s3::PartitionKind::Static;
            }
        }
        assert!(
            vize_s3::verify::verify(&s3.program).is_empty(),
            "mutation {mutation} must pass the generic verifier"
        );
        let status = admit(s3, &Retained::new(&allocator));
        assert!(
            matches!(status, VaporS3BridgeStatus::Rejected(_)),
            "mutation {mutation}: {status:?}"
        );
    }
}
