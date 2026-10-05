use super::*;
use vize_l0::cstr;

#[test]
fn clean_closed_dynamic_suffix_prefix_and_multiple_argument_overwrites_are_refused_before_element_dispatch()
 {
    for head in [
        ":[name]tail",
        ".[name]tail",
        "v-bind:[name]tail",
        ":pre[name]",
        ":[name][other]",
        "v-bind:[name][other]",
    ] {
        let source = cstr!("<div id='first'/><span {head}='opaque'/>");
        for locale in LOCALES {
            let log = events();
            let configured = linter(
                vec![Audit::new(&FIRST, "actual", Profile::Bindings, log.clone())],
                locale,
                HelpLevel::Full,
            );
            assert_eq!(
                configured
                    .lint_native_template(&source, "exact.vue")
                    .unwrap_err(),
                Refusal::Header(NativeLintRefusal::UnresolvedBinding {
                    span: span(&source, head)
                })
            );
            assert_eq!(
                *log.lock().unwrap(),
                [
                    Event::Root("actual"),
                    element_event("actual", "div", 0, None),
                    static_event("actual", &source, 0, "id", "id='first'", Some("first"))
                ]
            );
            original_single(
                &configured,
                &source,
                "exact.vue",
                &["id='first'", &cstr!("{head}='opaque'")],
            );
        }
    }
}

#[test]
fn empty_dynamic_shorthand_prop_and_full_arguments_remain_unresolved_with_full_original_metadata() {
    for head in [":[]", ".[]", "v-bind:[]"] {
        let source = cstr!("<div {head}='opaque'/>");
        for locale in LOCALES {
            let log = events();
            let configured = linter(
                vec![Audit::new(&FIRST, "actual", Profile::Bindings, log.clone())],
                locale,
                HelpLevel::Full,
            );
            assert_eq!(
                configured
                    .lint_native_template(&source, "exact.vue")
                    .unwrap_err(),
                Refusal::Header(NativeLintRefusal::UnresolvedBinding {
                    span: span(&source, head)
                })
            );
            assert_eq!(*log.lock().unwrap(), [Event::Root("actual")]);
            original_single(
                &configured,
                &source,
                "exact.vue",
                &[&cstr!("{head}='opaque'")],
            );
        }
    }
}
