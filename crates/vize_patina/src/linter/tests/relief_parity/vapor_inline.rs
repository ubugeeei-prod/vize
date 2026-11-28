//! The Vapor error keeps all packets while the shared rule gains TSRX support.

use super::{ReliefOnly, linter};
use crate::native::template::NativeTemplateLintRefusal;
use crate::rules::vapor::NoInlineTemplate;
use vize_atelier_jsx::JsxLang;

#[test]
fn vapor_inline_template_preserves_whole_template_sfc_jsx_and_tsx_results() {
    let actual = linter(Box::new(NoInlineTemplate));
    let legacy = linter(Box::new(ReliefOnly(Box::new(NoInlineTemplate))));
    for source in [
        r#"<div inline-template INLINE-TEMPLATE><span :inline-template="value"/></div>"#,
        r#"<MyComponent inline-template><template #default><p>日本語😀</p></template></MyComponent>"#,
        r#"<div v-pre inline-template :inline-template="value"/>"#,
        "<!-- eslint-disable-next-line vapor/no-inline-template -->\r\n<div inline-template/>\r\n<div inline-template/>",
    ] {
        assert_eq!(
            format!("{:?}", actual.lint_template(source, "Vapor.vue")),
            format!("{:?}", legacy.lint_template(source, "Vapor.vue")),
            "{source}"
        );
        let sfc =
            format!("<script setup>const s='日本語😀'</script>\r\n<template>{source}</template>");
        assert_eq!(
            format!("{:?}", actual.lint_sfc(&sfc, "Vapor.vue")),
            format!("{:?}", legacy.lint_sfc(&sfc, "Vapor.vue")),
            "{source}"
        );
    }
    for lang in [JsxLang::Jsx, JsxLang::Tsx] {
        for source in [
            r#"const A = () => <div inline-template INLINE-TEMPLATE inline-template={value}/>;"#,
            r#"const A = () => <Component render={<div inline-template/>}><p inline-template/><div custom:inline-template/></Component>;"#,
            r#"const A = () => <other:div inline-template/>;"#,
        ] {
            assert_eq!(
                format!("{:?}", actual.lint_jsx(source, "Vapor.tsx", lang)),
                format!("{:?}", legacy.lint_jsx(source, "Vapor.tsx", lang)),
                "{source}"
            );
        }
    }
}

#[test]
fn the_checked_native_callback_keeps_the_complete_legacy_packet() {
    let actual = linter(Box::new(NoInlineTemplate));
    for source in [
        r#"<div inline-template/>"#,
        r#"<div inline-template><p INLINE-TEMPLATE/></div>"#,
        r#"<div><p>日本語😀</p></div>"#,
    ] {
        let native = actual.lint_native_template(source, "Vapor.vue").unwrap();
        let legacy = actual.lint_template(source, "Vapor.vue");
        assert_eq!(format!("{native:?}"), format!("{legacy:?}"), "{source}");
    }
    assert_eq!(
        actual
            .lint_native_template("<div inline-template/>", "Vapor.vue")
            .unwrap()
            .error_count,
        1
    );
}

#[test]
fn the_native_callback_preserves_whole_input_refusal() {
    let actual = linter(Box::new(NoInlineTemplate));
    assert!(matches!(
        actual.lint_native_template("<div inline-template/><!-- comment -->", "Vapor.vue"),
        Err(NativeTemplateLintRefusal::Comment { .. })
    ));
    assert!(matches!(
        actual.lint_native_template("<div inline-template :id=\"value\"/>", "Vapor.vue"),
        Err(NativeTemplateLintRefusal::UnsupportedAttribute { .. })
    ));
}

#[test]
fn tsrx_reports_the_full_vapor_error_at_the_written_static_attribute() {
    let source = "export function Page() @{\n  <div inline-template/>\n}\n";
    let actual = linter(Box::new(NoInlineTemplate)).lint_tsrx(source, "Vapor.tsrx");
    let template =
        linter(Box::new(NoInlineTemplate)).lint_template("<div inline-template/>", "Vapor.tsrx");
    let offset = source.find("<div").unwrap() as u32;
    let mut expected = template;
    for diagnostic in &mut expected.diagnostics {
        diagnostic.start += offset;
        diagnostic.end += offset;
    }
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
    assert_eq!((actual.error_count, actual.warning_count), (1, 0));
}
