use super::options;
use crate::s3::{LegacyReason, VaporS3BridgeStatus, lower_source_for_vapor};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

/// Template carriers, fragment bodies and nested control flow the native lane
/// admits; each is also pinned byte-for-byte against the retained lane.
const ADMITTED: &[&str] = &[
    r#"<div><template v-if="ok"><b>{{ a }}</b><i>x</i></template><span v-else>no</span></div>"#,
    r#"<ul><template v-for="item in items" :key="item.id"><li>{{ item.a }}</li><li>{{ item.b }}</li></template></ul>"#,
    r#"<div><template v-if="ok"><b>{{ a }}</b></template></div>"#,
    r#"<template v-if="ok"><b>{{ a }}</b><i>x</i></template><template v-else>none</template>"#,
    r#"<ul><template v-for="(item, i) in items" :key="item.id + i"><li>{{ i }}</li><li>{{ item }}</li></template></ul>"#,
    r#"<ul><template v-for="item in items"><li>{{ item }}</li></template></ul>"#,
    r#"<ul><li v-if="ok" v-for="x in xs">{{ x }}</li></ul>"#,
    r#"<div><template v-if="ok"><span v-for="x in xs">{{ x }}</span></template></div>"#,
    r#"<div><template v-for="x in xs" :key="x"><span v-if="x.on">{{ x.a }}</span><i>{{ x.b }}</i></template></div>"#,
    r#"<div><template v-for="x in xs" :key="x.id"><Comp :x="x" /><slot :x="x"></slot></template></div>"#,
    r#"<div><template v-if="a"><template v-if="b"><i>ab</i></template></template></div>"#,
    r#"<main><template v-for="n in 3" :key="n">{{ n }}<br></template></main>"#,
    // Text bodies keep one node per part, as the retained lane emits them.
    r#"<main><template v-if="on">on {{ n }}</template><template v-else>off</template></main>"#,
    r#"<main><template v-for="x in xs">item {{ x }}</template></main>"#,
    r#"<main><template v-if="on">{{ a }}{{ b }}</template></main>"#,
];

#[test]
fn template_carriers_and_fragment_bodies_match_the_retained_lane() {
    for source in ADMITTED {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
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
        assert_eq!(native.code, retained.code, "{source}");
    }
}

#[test]
fn template_wrapper_facts_select_exact_legacy_reasons() {
    use LegacyReason::{Binding, ControlFlow, ExpressionOrEncoding, SurfaceSemantics};
    for (source, reason) in [
        // Branch keys, cached chains, and loop wrapper attributes or static
        // keys are wrapper facts the native payload does not carry.
        (
            r#"<div><template v-if="a" :key="k"><i>x</i></template></div>"#,
            ControlFlow,
        ),
        (
            r#"<div><template v-if="a" key="k"><i>x</i></template></div>"#,
            ControlFlow,
        ),
        (
            r#"<div><template v-once v-if="a"><i>x</i></template></div>"#,
            ControlFlow,
        ),
        (
            r#"<ul><template v-for="x in xs" key="k"><li>{{ x }}</li></template></ul>"#,
            ControlFlow,
        ),
        (
            r#"<ul><template v-for="x in xs" :key><li>{{ x }}</li></template></ul>"#,
            ControlFlow,
        ),
        (
            r#"<ul><template v-for="x in xs" class="c"><li>{{ x }}</li></template></ul>"#,
            ControlFlow,
        ),
        (
            r#"<ul><template v-for="x in xs" data-a="1"><li>{{ x }}</li></template></ul>"#,
            ControlFlow,
        ),
        // Empty bodies render nothing and stay on the legacy lane.
        (r#"<div><template v-if="a"></template></div>"#, ControlFlow),
        (
            r#"<ul><template v-for="x in xs"></template></ul>"#,
            ControlFlow,
        ),
        // A template loop is keyed on its wrapper, never on a body element.
        (
            r#"<ul><template v-for="x in xs" :key="x.id"><li :key="x.id">{{ x }}</li></template></ul>"#,
            Binding,
        ),
        (
            r#"<ul><template v-for="x in xs" :key="x.id as any"><li>{{ x }}</li></template></ul>"#,
            ExpressionOrEncoding,
        ),
        // Other `<template v-if>` attributes are recorded S2 drops.
        (
            r#"<div><template v-if="a" title="t"><i>x</i></template></div>"#,
            SurfaceSemantics,
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
