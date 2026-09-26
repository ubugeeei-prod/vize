//! Transition payloads are accepted only for the mounted JS-hook contract.

use super::{VaporS3BridgeStatus, lower_source_for_vapor, options};
use crate::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

const SOURCES: &[&str] = &[
    r#"<Transition :css="false"><p>{{ label }}</p></Transition>"#,
    r#"<Transition :css="false" @enter="enter" @leave="leave"><button v-if="show" @click="save">{{ label }}</button></Transition>"#,
    r#"<main><Transition :css="false" @before-enter="beforeEnter" @enter-cancelled="cancel"><div v-if="visible">{{ label }}</div></Transition><i>tail</i></main>"#,
    r#"<TransitionGroup tag="ul" :css="false" @enter="enter" @leave="leave"><li v-for="item in items" :key="item.id" :data-id="item.id">{{ item.text }}</li></TransitionGroup>"#,
    r#"<TransitionGroup tag="div" :css="false"><span v-for="item in items" :key="item.id">{{ item.text }}</span></TransitionGroup>"#,
];

#[test]
fn transition_codegen_preserves_retained_parser_and_output() {
    for source in SOURCES {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
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
            assert_eq!(native.error_messages, retained.error_messages, "{source}");
            assert!(
                native.error_messages.is_empty(),
                "{source}: {:?}",
                native.error_messages
            );
            assert_eq!(
                native.code, retained.code,
                "{source}: prefix={prefix_identifiers}"
            );
            assert!(
                native.code.contains("_createComponent(_VaporTransition"),
                "{}",
                native.code
            );
            assert!(!native.code.contains("_resolveComponent(\"Transition"));
        }
    }
}

#[test]
fn unproved_transition_shapes_remain_explicit_legacy() {
    for source in [
        r#"<Transition><p>CSS</p></Transition>"#,
        r#"<Transition :css="enabled"><p>dynamic CSS</p></Transition>"#,
        r#"<Transition :css="false" appear><p>appear</p></Transition>"#,
        r#"<Transition :css="false" mode="out-in"><p v-if="show">one</p><p v-else>two</p></Transition>"#,
        r#"<Transition :css="false"><p>one</p><p>two</p></Transition>"#,
        r#"<Transition :css="false"><Child /></Transition>"#,
        r#"<Transition :css="false"><p v-if="show">one</p><p v-else>two</p></Transition>"#,
        r#"<Transition :css="false"><div><p v-if="show">nested</p></div></Transition>"#,
        r#"<Transition :css="false"><div><Child /></div></Transition>"#,
        r#"<Transition :css="false"><div><input v-model="value" /></div></Transition>"#,
        r#"<Transition :css="false"><div><span v-html="html" /></div></Transition>"#,
        r#"<Transition :css="false"><p :key="id">keyed</p></Transition>"#,
        r#"<Transition :css="false"><template #default><p>slot</p></template></Transition>"#,
        r#"<Transition :css="false" :[name]="value"><p>computed</p></Transition>"#,
        r#"<Transition :css="false" v-bind="props"><p>spread</p></Transition>"#,
        r#"<transition :css="false"><p>alias</p></transition>"#,
        r#"<TransitionGroup :css="false"><li v-for="item in items">{{ item }}</li></TransitionGroup>"#,
        r#"<TransitionGroup tag="ul" :css="false"><li v-for="item in 3" :key="item">{{ item }}</li></TransitionGroup>"#,
        r#"<TransitionGroup :css="false"><li v-for="item in items" :key="item.id">{{ item.text }}</li></TransitionGroup>"#,
        r#"<TransitionGroup :css="false"><template v-for="item in items" :key="item.id"><li>{{ item.text }}</li></template></TransitionGroup>"#,
        r#"<TransitionGroup :css="false" :tag="tag"><li v-for="item in items" :key="item.id">{{ item.text }}</li></TransitionGroup>"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Legacy(_)),
            "{source}: {status:?}"
        );
    }
}

#[test]
fn transition_payload_owns_wrapper_identity_and_checks_css_mutation() {
    let allocator = Allocator::new();
    let source = r#"<TransitionGroup tag="ul" :css="false"><li v-for="item in items" :key="item.id">{{ item.text }}</li></TransitionGroup>"#;
    let scratch = Allocator::new();
    let (tree, errors) = vize_s1::parse(&scratch, source);
    let s2 = vize_s1_to_s2::lower(&scratch, &tree, &errors);
    let retained = crate::s3::retained::Retained::collect(&allocator, &s2.root);
    let mut s3 = vize_s2_to_s3::lower(&allocator, &s2.root);
    let tag = s3
        .program
        .operands
        .iter_mut()
        .find(|value| value.name == Some("tag"))
        .unwrap();
    tag.value.text = "div";
    let code = super::generated(crate::s3::admit(s3, &retained), &allocator);
    assert!(code.contains("tag: () => (\"div\")"), "{code}");
    let mut s3 = vize_s2_to_s3::lower(&allocator, &s2.root);
    let css = s3
        .program
        .operands
        .iter_mut()
        .find(|value| value.value.text == "false")
        .unwrap();
    css.value.text = "true";
    assert!(matches!(
        crate::s3::admit(s3, &retained),
        VaporS3BridgeStatus::Legacy(_)
    ));
}

#[test]
fn nested_transition_html_preserves_reviewed_generated_identity() {
    for source in [
        r#"<Transition :css="false"><div><span>{{ label }}</span></div></Transition>"#,
        r#"<main><Transition :css="false" @enter="enter" @leave="leave"><div v-if="show"><button @click="save">{{ label }}</button></div></Transition><i>tail</i></main>"#,
        r#"<TransitionGroup tag="ul" :css="false"><li v-for="item in items" :key="item.id"><span>{{ item.text }}</span></li></TransitionGroup>"#,
    ] {
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, source, options());
        assert!(
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
            "{source}: {status:?}"
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
            assert_eq!(native.error_messages, retained.error_messages, "{source}");
            assert!(
                native.error_messages.is_empty(),
                "{source}: {:?}",
                native.error_messages
            );
            assert_eq!(
                crate::tests_generated_identity::normalized(&native.code),
                crate::tests_generated_identity::normalized(&retained.code),
                "{source}: prefix={prefix_identifiers}"
            );
        }
    }
}

#[test]
fn nested_transition_descendant_depth_has_an_explicit_limit() {
    for (depth, accepted) in [(16, true), (17, false)] {
        let mut source = vize_carton::String::from("<Transition :css=\"false\"><div>");
        for _ in 1..depth {
            source.push_str("<span>");
        }
        source.push('x');
        for _ in 1..depth {
            source.push_str("</span>");
        }
        source.push_str("</div></Transition>");
        let allocator = Allocator::new();
        let status = lower_source_for_vapor(&allocator, &source, options());
        assert_eq!(
            matches!(status, VaporS3BridgeStatus::Accepted(_)),
            accepted,
            "depth={depth}: {status:?}"
        );
    }
}
