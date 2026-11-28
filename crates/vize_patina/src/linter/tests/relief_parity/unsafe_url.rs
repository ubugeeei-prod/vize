//! Whole packets retain the URL rule's authored ranges and parser admission.

use super::{ReliefOnly, linter};
use crate::diagnostic::LintDiagnostic;
use crate::ir::TemplateSyntax;
use crate::linter::LintResult;
use crate::markup::{MarkupContext, MarkupDocument};
use crate::rules::vue::NoUnsafeUrl;
use vize_atelier_jsx::JsxLang;
use vize_l0::Allocator;

const TEMPLATES: &[&str] = &[
    r#"<a href="javascript:alert(1)" :href="url" HREF="vbscript:run()" v-bind:HREF="url">日本語😀</a>"#,
    r#"<iframe src="java&#x0a;script:alert(1)" :src="url"/><object :data="url"/><form :action="url"><button :formaction="url"/></form>"#,
    r#"<div><a :href="safe(url)"/><a :href="tools.sanitizeUrl(url)"/><a :href="getUrl(url)"/></div>"#,
    r##"<div><a :href="`#${id}`"/><a :href="'#' + id"/><iframe :src="`#${id}`"/></div>"##,
    r#"<div><a :[href]="url"/><iframe :[src]="url"/><a :href/><a href/></div>"#,
    r#"<div><a v-pre :href="url" href="/safe"/><MyLink :href="url"/><my-link href="javascript:alert(1)"/><slot :data="url"/><img :src="url"/><video src="javascript:alert(1)"/></div>"#,
    r#"<svg><a xlink:href="javascript:alert(1)" :xlink:href="url"/></svg>"#,
    "<!-- eslint-disable-next-line vue/no-unsafe-url -->\r\n<a :href=\"url\"/>\r\n<a :href=\"other\"/>",
    "<!-- @vize:level off next-line vue/no-unsafe-url -->\n<a :href=\"url\"/>\n<iframe :src=\"other\"/>",
];

const JSX: &[&str] = &[
    r#"const A = () => <a href="javascript:alert(1)" href={url} HREF="vbscript:run()" HREF={url}>日本語😀</a>;"#,
    r#"const A = () => <><iframe src="java&#x0a;script:alert(1)" src={url}/><object data={url}/><form action={url}><button formaction={url}/></form></>;"#,
    r#"const A = () => <><a href={safe(url)}/><a href={tools.sanitizeUrl(url)}/><a href={getUrl(url)}/></>;"#,
    r##"const A = () => <><a href={`#${id}`}/><a href={'#' + id}/><iframe src={`#${id}`}/></>;"##,
    r#"const A = () => <><a href/><MyLink href={url}/><my-link href="javascript:alert(1)"/><slot data={url}/><img src={url}/><video src="javascript:alert(1)"/></>;"#,
    r#"const A = () => <><svg><a xlink:href="javascript:alert(1)" xlink:href={url}/></svg><a other:href={url}/><svg:a href={url}/><UI.a href={url}/></>;"#,
    r#"const A = () => <Component render={<a href="javascript:alert(1)"/>}><a href={url}/></Component>;"#,
    r#"const A = () => <div>{ok ? <a href={url}/> : <iframe src={url}/>}</div>;"#,
    "/* eslint-disable vue/no-unsafe-url */\nconst A = () => <a href={url}/>;",
];

#[test]
fn complete_template_sfc_jsx_and_tsx_results_match_the_legacy_dispatch() {
    let actual = linter(Box::new(NoUnsafeUrl));
    let legacy = linter(Box::new(ReliefOnly(Box::new(NoUnsafeUrl))));
    for source in TEMPLATES {
        assert_eq!(
            format!("{:?}", actual.lint_template(source, "Url.vue")),
            format!("{:?}", legacy.lint_template(source, "Url.vue")),
            "bare template: {source}"
        );
        let sfc = format!(
            "<script setup lang=\"ts\">const note = '日本語😀'</script>\r\n<template>{source}</template>"
        );
        assert_eq!(
            format!("{:?}", actual.lint_sfc(&sfc, "Url.vue")),
            format!("{:?}", legacy.lint_sfc(&sfc, "Url.vue")),
            "SFC: {source}"
        );
    }
    for lang in [JsxLang::Jsx, JsxLang::Tsx] {
        for source in JSX {
            assert_eq!(
                format!("{:?}", actual.lint_jsx(source, "Url.tsx", lang)),
                format!("{:?}", legacy.lint_jsx(source, "Url.tsx", lang)),
                "JSX/TSX: {source}"
            );
        }
    }
}

#[test]
fn the_only_migrated_rule_keeps_lowering_only_parser_diagnostics() {
    let source = r#"const A = () => <other:div><a href={url}/></other:div>;"#;
    let legacy = linter(Box::new(ReliefOnly(Box::new(NoUnsafeUrl)))).lint_jsx(
        source,
        "Url.jsx",
        JsxLang::Jsx,
    );
    assert!(
        legacy
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.rule_name == "parser/jsx"),
        "the fixture must exercise lowering-only parser diagnostics"
    );
    let actual = linter(Box::new(NoUnsafeUrl)).lint_jsx(source, "Url.jsx", JsxLang::Jsx);
    assert_eq!(format!("{actual:?}"), format!("{legacy:?}"));
}

#[test]
fn direct_oxc_markup_preserves_namespaces_sanitizers_and_attribute_subtree_boundaries() {
    let source = r#"const A = () => <Component render={<a href="javascript:alert(1)"/>}><a href={url}/><a href={safeUrl(url)}/><a other:href={url}/><a xlink:href={url}/></Component>;"#;
    let allocator = Allocator::new();
    let parsed = vize_atelier_jsx::parse_module(allocator.as_oxc(), source, JsxLang::Jsx);
    let document = MarkupDocument::from_jsx(&parsed.program, TemplateSyntax::Vue, 0);
    let mut lint = crate::LintContext::new(&allocator, source, "Url.jsx");
    document.visit_with(&NoUnsafeUrl, &mut MarkupContext::new(&mut lint, &document));
    let legacy = linter(Box::new(ReliefOnly(Box::new(NoUnsafeUrl)))).lint_jsx(
        source,
        "Url.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        format!("{:?}", lint.diagnostics()),
        format!("{:?}", legacy.diagnostics)
    );
    assert_eq!((lint.error_count(), lint.warning_count()), (0, 2));
}

#[test]
fn tsrx_publishes_the_same_complete_static_and_dynamic_url_packets() {
    let source = r#"export function Links({ url }: { url: string }) @{
  <div>
    <a href="javascript:alert(1)">unsafe</a>
    <iframe src={url}/>
    <a href={safeUrl(url)}>safe</a>
    <a href={'#' + url}>hash</a>
    <img src={url}/>
    <Link href={url}/>
  </div>
}
"#;
    let range = |text: &str| {
        let start = source.find(text).unwrap() as u32;
        (start, start + text.len() as u32)
    };
    let (start, end) = range("href=\"javascript:alert(1)\"");
    let static_url = LintDiagnostic::warn(
        "vue/no-unsafe-url",
        "Static URL attribute uses a potentially unsafe protocol or active data URL",
        start,
        end,
    ).with_help("Use http(s), relative URLs, or a non-executable data URL. Avoid javascript:, vbscript:, text/html data URLs, and SVG data URLs");
    let (start, end) = range("src={url}");
    let dynamic_url = LintDiagnostic::warn(
        "vue/no-unsafe-url",
        "Dynamic :src binding may be vulnerable to XSS via javascript: protocol",
        start,
        end,
    )
    .with_help("Ensure URLs are sanitized before binding to prevent XSS via javascript: protocol");
    let expected = LintResult {
        filename: "Url.tsrx".into(),
        diagnostics: vec![static_url, dynamic_url],
        error_count: 0,
        warning_count: 2,
    };
    let actual = linter(Box::new(NoUnsafeUrl)).lint_tsrx(source, "Url.tsrx");
    assert_eq!(format!("{actual:?}"), format!("{expected:?}"));
}
