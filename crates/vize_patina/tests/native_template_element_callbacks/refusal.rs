use super::support::*;
use vize_l0::cstr;
use vize_patina::{
    HelpLevel, Locale,
    native::{
        NativeLintRefusal,
        template::{
            NativeTemplateAttributeProfile as Profile, NativeTemplateLintRefusal as Refusal,
        },
    },
};

#[test]
fn incomplete_dynamic_lexer_heads_preclude_all_root_and_element_callbacks() {
    for source in ["<div :[name='v'/>", "<div :[keys[index]='v'/>"] {
        for locale in LOCALES {
            let log = events();
            let configured = linter(
                vec![Audit::new(&FIRST, "actual", Profile::Bindings, log.clone())],
                locale,
                HelpLevel::Full,
            );
            assert_eq!(
                configured
                    .lint_native_template(source, "exact.vue")
                    .unwrap_err(),
                Refusal::Recovered {
                    offset: source.find('=').unwrap() as u32
                }
            );
            assert!(
                log.lock().unwrap().is_empty(),
                "even pending root callbacks must not run"
            );
            let original = configured.lint_template(source, "exact.vue");
            let full = source
                .strip_prefix("<div ")
                .unwrap()
                .strip_suffix("/>")
                .unwrap();
            let mut expected = single_expected(&configured, source, "exact.vue", &[full]);
            let start = source.find('=').unwrap() as u32;
            expected["diagnostics"].as_array_mut().unwrap().push(serde_json::json!({
                "rule_name": "parser/template", "severity": "error",
                "message": "Dynamic directive argument is missing its closing `]`; inferred the argument end at the next tag boundary.",
                "start": start, "end": start + 1, "help": null, "labels": [], "fix": null,
            }));
            expected["error_count"] = 1.into();
            assert_eq!(complete(&original), expected);
            assert_eq!(
                cstr!("{original:#?}"),
                cstr!("{:#?}", configured.lint_template(source, "exact.vue")),
                "retain complete original error metadata/order, never partial native clean output"
            );
        }
    }
}

#[test]
fn empty_dynamic_and_object_bindings_remain_precisely_refused_under_the_wider_profile() {
    for (source, head) in [
        ("<div :[]='opaque'/>", ":[]"),
        ("<div v-bind='attrs'/>", "v-bind"),
    ] {
        for locale in LOCALES {
            let configured = linter(
                vec![Audit::new(&FIRST, "actual", Profile::Bindings, events())],
                locale,
                HelpLevel::Full,
            );
            assert_eq!(
                configured
                    .lint_native_template(source, "exact.vue")
                    .unwrap_err(),
                Refusal::Header(NativeLintRefusal::UnresolvedBinding {
                    span: span(source, head)
                })
            );
            original_single(
                &configured,
                source,
                "exact.vue",
                &[if head == ":[]" {
                    ":[]='opaque'"
                } else {
                    "v-bind='attrs'"
                }],
            );
        }
    }
}

#[test]
fn unsupported_event_and_structural_headers_cannot_publish_previously_pending_element_findings() {
    for (late, full) in [
        ("<span @click='handler'/>", "@click='handler'"),
        ("<span v-show='opaque'/>", "v-show='opaque'"),
    ] {
        let source = cstr!("<div id='first'/>{late}");
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
                Refusal::UnsupportedAttribute {
                    span: span(&source, full)
                }
            );
            assert_eq!(
                *log.lock().unwrap(),
                [
                    Event::Root("actual"),
                    element_event("actual", "div", 0, None),
                    static_event("actual", &source, 0, "id", "id='first'", Some("first"))
                ]
            );
            original_single(&configured, &source, "exact.vue", &["id='first'", full]);
        }
    }
}

#[test]
fn late_ordinary_comment_discards_root_and_attribute_results_without_rollback_of_rule_side_effects()
{
    let source = "<div id='first'/><section><span title='second'/><!-- ordinary --></section>";
    for locale in LOCALES {
        let log = events();
        let configured = linter(
            vec![Audit::new(&FIRST, "actual", Profile::Bindings, log.clone())],
            locale,
            HelpLevel::Full,
        );
        assert_eq!(
            configured
                .lint_native_template(source, "exact.vue")
                .unwrap_err(),
            Refusal::Comment {
                span: span(source, "<!-- ordinary -->")
            }
        );
        assert_eq!(
            *log.lock().unwrap(),
            [
                Event::Root("actual"),
                element_event("actual", "div", 0, None),
                static_event("actual", source, 0, "id", "id='first'", Some("first")),
                element_event("actual", "section", 1, None),
                element_event("actual", "span", 0, Some("section")),
                static_event(
                    "actual",
                    source,
                    0,
                    "title",
                    "title='second'",
                    Some("second")
                )
            ]
        );
        original_single(
            &configured,
            source,
            "exact.vue",
            &["id='first'", "title='second'"],
        );
    }
}

#[test]
fn true_late_global_suppression_keeps_full_original_empty_and_native_typed_refusal() {
    let source = "<div id='first'/><!-- eslint-disable -->";
    for locale in LOCALES {
        let log = events();
        let configured = linter(
            vec![Audit::new(&FIRST, "actual", Profile::Bindings, log.clone())],
            locale,
            HelpLevel::Full,
        );
        assert_eq!(
            complete(&configured.lint_template(source, "exact.vue")),
            serde_json::json!({
                "filename": "exact.vue", "diagnostics": [], "error_count": 0, "warning_count": 0,
            })
        );
        assert_eq!(
            configured
                .lint_native_template(source, "exact.vue")
                .unwrap_err(),
            Refusal::Comment {
                span: span(source, "<!-- eslint-disable -->")
            }
        );
        assert_eq!(
            *log.lock().unwrap(),
            [
                Event::Root("actual"),
                element_event("actual", "div", 0, None),
                static_event("actual", source, 0, "id", "id='first'", Some("first"))
            ]
        );
    }
}

#[test]
fn late_original_interpolation_refuses_after_authentic_parent_attributes() {
    let source = "<div :title='opaque'><span id='literal'/>{{ value }}</div>";
    for locale in LOCALES {
        let log = events();
        let configured = linter(
            vec![Audit::new(&FIRST, "actual", Profile::Bindings, log.clone())],
            locale,
            HelpLevel::Full,
        );
        assert_eq!(
            configured
                .lint_native_template(source, "exact.vue")
                .unwrap_err(),
            Refusal::Interpolation {
                span: span(source, "{{ value }}")
            }
        );
        assert_eq!(
            *log.lock().unwrap(),
            [
                Event::Root("actual"),
                element_event("actual", "div", 0, None),
                binding_event(
                    "actual",
                    source,
                    0,
                    ":title",
                    ":title='opaque'",
                    "opaque",
                    vize_l1::markup::ArgSyntax::Static(span(source, "title"))
                ),
                element_event("actual", "span", 0, Some("div")),
                static_event("actual", source, 0, "id", "id='literal'", Some("literal"))
            ]
        );
        original_single(
            &configured,
            source,
            "exact.vue",
            &[":title='opaque'", "id='literal'"],
        );
    }
}

#[test]
fn late_foreign_or_component_owner_is_not_dispatched_despite_a_checked_literal_header() {
    for (source, opening) in [
        (
            "<div id='first'/><Foo title='second'/>",
            "<Foo title='second'/>",
        ),
        (
            "<div id='first'/><svg title='second'></svg>",
            "<svg title='second'>",
        ),
    ] {
        let log = events();
        let configured = linter(
            vec![Audit::new(&FIRST, "actual", Profile::Bindings, log.clone())],
            Locale::En,
            HelpLevel::Full,
        );
        assert_eq!(
            configured
                .lint_native_template(source, "exact.vue")
                .unwrap_err(),
            Refusal::UnsupportedContext {
                span: span(source, opening)
            }
        );
        assert_eq!(
            *log.lock().unwrap(),
            [
                Event::Root("actual"),
                element_event("actual", "div", 0, None),
                static_event("actual", source, 0, "id", "id='first'", Some("first"))
            ]
        );
        original_single(
            &configured,
            source,
            "exact.vue",
            &["id='first'", "title='second'"],
        );
    }
}

#[test]
fn callback_error_discards_its_own_pending_warning_and_stops_later_actual_instances() {
    let source = "<div id='literal'><span title='child'/></div>";
    let log = events();
    let mut first = Audit::new(&FIRST, "first", Profile::Bindings, log.clone());
    first.fail = true;
    let configured = linter(
        vec![
            first,
            Audit::new(&SECOND, "second", Profile::Bindings, log.clone()),
        ],
        Locale::En,
        HelpLevel::Full,
    );
    assert_eq!(
        configured
            .lint_native_template(source, "exact.vue")
            .unwrap_err(),
        Refusal::UnsupportedAttribute {
            span: span(source, "id='literal'")
        }
    );
    assert_eq!(
        *log.lock().unwrap(),
        [
            Event::Root("first"),
            Event::Root("second"),
            element_event("first", "div", 0, None),
            static_event("first", source, 0, "id", "id='literal'", Some("literal"))
        ]
    );
}

mod geometry;
mod parser;
