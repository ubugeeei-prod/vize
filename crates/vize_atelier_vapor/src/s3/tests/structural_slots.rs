//! Native checked structural slots agree with the retained compatibility lane.

use super::{generated, lowered_source, options};
use crate::s3::{VaporS3BridgeStatus, lower_source_for_vapor, retained::Retained};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

pub(crate) const SOURCES: &[&str] = &[
    r#"<Child><template #one v-if="enabled">A</template></Child>"#,
    r#"<Child><template #one v-if="enabled">A</template><template #two v-else>B</template></Child>"#,
    r#"<Child><template #[names[selected]] v-if="enabled">A</template><template #two v-else-if="second">B</template><template #one v-else>C</template></Child>"#,
    r#"<Child><template #fixed>F</template><template #[name]="p" v-if="flags.on"><b :title="p.x" @click="record(p.x)">{{p.x}}</b></template></Child>"#,
    r#"<Child><template v-for="item in items" #[item.name]><b>{{item.label}}</b></template></Child>"#,
    r#"<Child><template v-for="(item, key) in items" #[item.name]="p"><b>{{item.label}}:{{key}}:{{p.x}}</b></template></Child>"#,
    r#"<Child><template v-for="(item, key, index) in items" #[item.name]="{ value }"><button @click="record(item.label)">{{item.label}}:{{key}}:{{index}}:{{value}}</button></template></Child>"#,
    r#"<Child><template v-for="item in items" #[item.name]="item"><b>{{item.x}}</b></template></Child>"#,
];

#[test]
fn checked_structural_slots_match_retained_code() {
    for source in SOURCES {
        let allocator = Allocator::new();
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporS3BridgeStatus::Accepted(_)
            ),
            "{source}"
        );
        for prefix_identifiers in [false, true] {
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
            let native = compile(false);
            let retained = compile(true);
            assert!(
                native.error_messages.is_empty(),
                "{source}: {:?}",
                native.error_messages
            );
            assert!(
                retained.error_messages.is_empty(),
                "{source}: {:?}",
                retained.error_messages
            );
            assert_eq!(
                native.code, retained.code,
                "{source}: prefix={prefix_identifiers}"
            );
        }
    }
}

#[test]
fn ordinary_default_slot_controls_keep_their_dom_contract() {
    for source in [
        r#"<Child v-slot="{ blocked }"><p v-if="blocked">waiting</p><span v-else>ready</span></Child>"#,
        r#"<Child><p v-if="enabled">A</p><span v-else>B</span></Child>"#,
        r#"<Child><p v-for="item in items">{{item}}</p></Child>"#,
        r#"<Child><Nested v-if="enabled" v-slot="p"><b>{{p.x}}</b></Nested></Child>"#,
        r#"<Child><Nested v-slot="p"><b>{{p.x}}</b></Nested></Child>"#,
    ] {
        let allocator = Allocator::new();
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporS3BridgeStatus::Accepted(_)
            ),
            "{source}"
        );
        let native = compile_vapor(&allocator, source, VaporCompilerOptions::default());
        let retained = compile_vapor(
            &allocator,
            source,
            VaporCompilerOptions {
                davinci_retained_lane: true,
                ..Default::default()
            },
        );
        assert!(
            native.error_messages.is_empty(),
            "{source}: {:?}",
            native.error_messages
        );
        assert_eq!(native.code, retained.code, "{source}");
        assert!(!native.code.contains("createForSlots"), "{source}");
    }
}

#[test]
fn conditional_slot_condition_is_owned_by_the_checked_payload() {
    let allocator = Allocator::new();
    let mut lowered = lowered_source(&allocator, SOURCES[0]);
    let condition = lowered
        .program
        .operands
        .iter_mut()
        .find(|operand| operand.role == vize_s3::operand::OperandRole::Condition)
        .unwrap();
    condition.value.text = "replacement";
    let code = generated(
        crate::s3::admit(lowered, &Retained::new(&allocator)),
        &allocator,
    );
    assert!(code.contains("_ctx.replacement ?"), "{code}");
    assert!(!code.contains("_ctx.enabled"), "{code}");
}

#[test]
fn loop_slot_source_is_owned_by_the_checked_payload() {
    let allocator = Allocator::new();
    let source = r#"<Child><template v-for="item in items" #[item]>{{item}}</template></Child>"#;
    let mut lowered = lowered_source(&allocator, source);
    let source = lowered
        .program
        .operands
        .iter_mut()
        .find(|operand| operand.role == vize_s3::operand::OperandRole::ForSource)
        .unwrap();
    source.value.text = "replacement";
    let code = generated(
        crate::s3::admit(lowered, &Retained::new(&allocator)),
        &allocator,
    );
    assert!(
        code.contains("_createForSlots(() => (_ctx.replacement)"),
        "{code}"
    );
    assert!(code.contains("(item) => (item)"), "{code}");
    assert!(!code.contains("_ctx.items"), "{code}");
}

#[test]
fn unsupported_structural_slot_shapes_stay_legacy() {
    for source in [
        r#"<Child><template v-if="enabled"><template #one>A</template></template></Child>"#,
        r#"<Child><template #one v-if="enabled">A</template><template v-else><template #two>B</template></template></Child>"#,
        r#"<Child><template v-for="item in items"><template #[item.name]>A</template></template></Child>"#,
        r#"<Child><template v-for="item in items" :key="item.id" #[item.name]>A</template></Child>"#,
        r#"<Child><template #one v-if="on">A</template>implicit</Child>"#,
        r#"<Child v-slot="p"><template #one v-if="on">A</template></Child>"#,
        r#"<Child><template v-if="on"><template #one>A</template><template #two>B</template></template></Child>"#,
        r#"<Child><template v-for="{name} in items" #[name]>A</template></Child>"#,
        r#"<Child><template v-for="item in items" v-if="on" #[item.name]>A</template></Child>"#,
        r#"<Child><template #one v-if="on" v-slot="{ a: b }">{{b}}</template></Child>"#,
        r#"<KeepAlive><template #default v-if="on"><Child /></template></KeepAlive>"#,
    ] {
        let allocator = Allocator::new();
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporS3BridgeStatus::Legacy(_)
            ),
            "{source}"
        );
    }
}
