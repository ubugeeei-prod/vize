//! Static force modifiers retain their runtime semantics in both emitters.
use super::{generated, options};
use crate::l3::{VaporL3BridgeStatus, lower_source_for_vapor};
use vize_carton::Allocator;

#[test]
fn static_force_modifiers_are_native_and_merge_in_authored_order() {
    for (source, calls) in [
        (
            r#"<input :value.attr="v">"#,
            ["_setAttr(n0, \"value\", _ctx.v)", ""],
        ),
        (
            r#"<div :title.prop="v"></div>"#,
            ["_setDOMProp(n0, \"title\", _ctx.v)", ""],
        ),
        (
            r#"<div :title.attr="v" v-bind="attrs" id="static"></div>"#,
            [
                "_setDynamicProps(n0, [{ \"^title\": _ctx.v }, _ctx.attrs, { id: \"static\" }])",
                "_template(\"<div></div>\", true)",
            ],
        ),
        (
            r#"<div id="before" :title.prop="v" v-bind="attrs"></div>"#,
            [
                "_setDynamicProps(n0, [{ id: \"before\", \".title\": _ctx.v }, _ctx.attrs])",
                "_template(\"<div></div>\", true)",
            ],
        ),
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporL3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
        );
        let native = generated(status, &allocator);
        for call in calls {
            assert!(native.contains(call), "{source}: {native}");
        }
        let retained = crate::compile_vapor(
            &allocator,
            source,
            crate::VaporCompilerOptions {
                prefix_identifiers: true,
                davinci_retained_lane: true,
                ..Default::default()
            },
        );
        assert!(
            retained.error_messages.is_empty(),
            "{:?}",
            retained.error_messages
        );
        assert_eq!(native, retained.code, "{source}");
    }
}

#[test]
fn computed_modifier_and_component_prop_boundaries_stay_explicit() {
    let allocator = Allocator::new();
    for source in [
        r#"<div :[key].attr="v"></div>"#,
        r#"<Child :title.attr="v" />"#,
    ] {
        assert!(
            matches!(
                lower_source_for_vapor(&allocator, source, options()),
                VaporL3BridgeStatus::Legacy(_)
            ),
            "{source}"
        );
    }
}
