use super::{generated, lowered_source, options};
use crate::s3::{
    LegacyReason, VaporS3BridgeStatus, admit, lower_source_for_vapor, retained::Retained,
};
use vize_carton::Allocator;
use vize_s3::operand::OperandRole;

#[test]
fn component_outlet_and_fragment_shapes_are_admitted() {
    for source in [
        "<MyComp />",
        r#"<my-comp title="t" :count="n + 1" @change="save" @update-value="v = $event" />"#,
        r#"<div><MyComp :x="1">text {{ y }}<b v-if="ok">{{ z }}</b></MyComp><i>{{ k }}</i></div>"#,
        r#"<MyComp class="a" :class="cls" style="color: red" :style="s" />"#,
        "<slot />",
        r#"<div><slot name="row" id="r" :item="it">fallback {{ it }}</slot></div>"#,
        r#"<ul><li v-for="x in xs" :key="x.id"><Row :x="x" /></li></ul>"#,
        r#"<b>{{ a }}</b>text<i :title="c">x</i>"#,
        "{{ a }} and {{ b }}",
        "static root text",
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
fn unsupported_component_shapes_select_exact_legacy_reasons() {
    use LegacyReason::{Binding, Component, Operation};
    for (source, reason) in [
        // Slot shapes beyond static names and flat parameter patterns
        // (`tests/slots.rs`).
        (
            r#"<MyComp v-slot="{ item = 1 }">{{ item }}</MyComp>"#,
            Component,
        ),
        (
            r#"<MyComp><template #head>H</template>body</MyComp>"#,
            Component,
        ),
        (r#"<component :is="view" />"#, Component),
        (r#"<Teleport to="body"><div></div></Teleport>"#, Component),
        (r#"<KeepAlive><MyComp /></KeepAlive>"#, Component),
        (r#"<MyComp @change.once="save" />"#, Component),
        (r#"<MyComp :key="id" />"#, Component),
        (r#"<MyComp key="k" />"#, Component),
        (r#"<MyComp ref="c" />"#, Component),
        (r#"<MyComp v-show="ok" />"#, Component),
        (r#"<MyComp title="a" :title="b" />"#, Component),
        (r#"<slot :name="dynamic" />"#, Component),
        (r#"<slot @click="save" />"#, Component),
        (r#"<MyComp v-model="value" />"#, Binding),
        (r#"<MyComp v-bind="props" />"#, Operation),
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
fn component_payload_mutations_drive_generation() {
    let allocator = Allocator::new();
    let source = r#"<div><MyComp title="t" :count="n" @change="save">{{ label }}</MyComp><slot name="aside" :item="row" /></div>"#;
    let mut s3 = lowered_source(&allocator, source);
    for operand in &mut s3.program.operands {
        match (operand.role, operand.value.text) {
            (OperandRole::Tag, "MyComp") => operand.value.text = "Other",
            (OperandRole::Attribute, "t") => operand.value.text = "u",
            (OperandRole::Value, "n") => operand.value.text = "m",
            (OperandRole::Value, "save") => operand.value.text = "submit",
            (OperandRole::Name, "aside") => operand.value.text = "side",
            (OperandRole::Value, "row") => operand.value.text = "cell",
            _ => {}
        }
    }
    let code = generated(admit(s3, &Retained::new(&allocator)), &allocator);
    for expected in [
        "const _component_Other = _resolveComponent(\"Other\")",
        "title: () => (\"u\")",
        "count: () => (_ctx.m)",
        "onChange: () => (_ctx.submit)",
        "_createSlot(\"side\", { item: () => (_ctx.cell) })",
        "_toDisplayString(_ctx.label)",
    ] {
        assert!(code.contains(expected), "missing {expected}: {code}");
    }
    for stale in ["MyComp", "_ctx.n)", "_ctx.save", "aside", "_ctx.row"] {
        assert!(!code.contains(stale), "stale {stale}: {code}");
    }
}
