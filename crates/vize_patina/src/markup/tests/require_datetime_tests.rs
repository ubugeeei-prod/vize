use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::html::RequireDatetime;
use vize_atelier_jsx::JsxLang;
use vize_s0::Allocator;

fn run_over_template<R: MarkupRule>(rule: &R, source: &str) -> Vec<LintDiagnostic> {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let parser = vize_armature::Parser::new(&allocator, source);
    let (root, _errors) = parser.parse();
    let document = MarkupDocument::new(&root, TemplateSyntax::Vue);

    let mut lint = LintContext::new(&allocator, source, "test.vue");
    let mut ctx = MarkupContext::new(&mut lint, &document);
    document.visit_with(rule, &mut ctx);
    lint.diagnostics().to_vec()
}

fn run_over_jsx_lowered<R: MarkupRule>(rule: &R, source: &str) -> Vec<LintDiagnostic> {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let lowered =
        vize_atelier_jsx::lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);

    let mut diagnostics = Vec::new();
    for lowered_root in &lowered.roots {
        let document = MarkupDocument::new(&lowered_root.root, TemplateSyntax::Vue);
        let mut lint = LintContext::new(&allocator, source, "test.jsx");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        document.visit_with(rule, &mut ctx);
        diagnostics.extend_from_slice(lint.diagnostics());
    }
    diagnostics
}

fn diagnostic_ranges(diagnostics: &[LintDiagnostic]) -> Vec<(u32, u32)> {
    diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.start, diagnostic.end))
        .collect()
}

#[test]
fn require_datetime_template_attr_and_text_boundaries() {
    let rule = RequireDatetime;
    for (source, expected, label) in [
        (
            r#"<time datetime="2024-12-25">Christmas</time>"#,
            0,
            "static datetime attribute",
        ),
        (
            r#"<time :datetime="date">Christmas</time>"#,
            0,
            "v-bind datetime attribute",
        ),
        (
            r#"<time v-bind:datetime="date">Christmas</time>"#,
            0,
            "longhand v-bind datetime attribute",
        ),
        (
            r#"<time :[datetime]="date">Christmas</time>"#,
            0,
            "dynamic arg content still matches legacy simple expression",
        ),
        (
            r#"<time v-bind="attrs">Christmas</time>"#,
            1,
            "argument-less v-bind does not prove datetime exists",
        ),
        (
            r#"<time DATETIME="2024-12-25">Christmas</time>"#,
            1,
            "attribute matching is case-sensitive",
        ),
        (r#"<time>2024-12-25</time>"#, 0, "valid date text"),
        (r#"<time>2024-12-25T10:30</time>"#, 0, "valid datetime text"),
        (r#"<time>PT1H30M</time>"#, 0, "valid duration text"),
        (
            r#"<time>last Tuesday</time>"#,
            1,
            "human-readable text requires datetime",
        ),
        (r#"<time></time>"#, 1, "empty text requires datetime"),
        (
            r#"<time> </time>"#,
            1,
            "whitespace-only text requires datetime",
        ),
        (
            r#"<time>{{ formattedDate }}</time>"#,
            0,
            "direct interpolation is dynamic content",
        ),
        (
            r#"<time>last {{ unit }}</time>"#,
            0,
            "mixed direct text and interpolation is dynamic content",
        ),
        (
            r#"<time><span>2024-12-25</span></time>"#,
            1,
            "nested text is ignored by the legacy direct text scan",
        ),
        (
            r#"<time>2024<span>x</span>-12-25</time>"#,
            0,
            "direct text concatenates around ignored nested element text",
        ),
        (
            r#"<Time>Christmas</Time>"#,
            0,
            "component-like uppercase tag is not lowercase time",
        ),
        (
            r#"<time-clock>Christmas</time-clock>"#,
            0,
            "custom element tag is not lowercase time",
        ),
    ] {
        let diagnostics = run_over_template(&rule, source);
        assert_eq!(
            diagnostics.len(),
            expected,
            "template boundary changed for {label}: {diagnostics:?}"
        );
    }
}

#[test]
fn require_datetime_template_reports_on_time_element() {
    let rule = RequireDatetime;
    let source = r#"<section><time>Christmas</time></section>"#;
    let diagnostics = run_over_template(&rule, source);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");

    let start = source.find("<time>").unwrap() as u32;
    assert_eq!(
        diagnostic_ranges(&diagnostics),
        vec![(start, start + "<time>".len() as u32)]
    );
    assert_eq!(diagnostics[0].rule_name, "html/require-datetime");
    assert_eq!(diagnostics[0].severity, Severity::Warning);
    assert!(
        diagnostics[0].help.is_some(),
        "diagnostic should carry help text: {:?}",
        diagnostics[0]
    );
    assert!(
        diagnostics[0].fix.is_none(),
        "require-datetime is not auto-fixable: {:?}",
        diagnostics[0]
    );
}

#[test]
fn require_datetime_jsx_lowered_attr_and_text_boundaries() {
    let rule = RequireDatetime;
    for (source, expected, label) in [
        (
            r#"const A = () => <time datetime="2024-12-25">Christmas</time>;"#,
            0,
            "static datetime attribute",
        ),
        (
            r#"const A = () => <time datetime={date}>Christmas</time>;"#,
            0,
            "dynamic datetime attribute",
        ),
        (
            r#"const A = () => <time v-bind:datetime={date}>Christmas</time>;"#,
            0,
            "JSX directive spelling lowers to v-bind datetime",
        ),
        (
            r#"const A = () => <time {...attrs}>Christmas</time>;"#,
            1,
            "spread attributes do not prove datetime exists",
        ),
        (
            r#"const A = () => <time dateTime="2024-12-25">Christmas</time>;"#,
            1,
            "camel-cased dateTime does not match legacy lowercase datetime",
        ),
        (
            r#"const A = () => <time html:datetime="2024-12-25">Christmas</time>;"#,
            1,
            "namespaced datetime attribute does not match lowercase datetime",
        ),
        (
            r#"const A = () => <time>{'2024-12-25'}</time>;"#,
            0,
            "static string expression lowers to text",
        ),
        (
            r#"const A = () => <time>{'Christmas'}</time>;"#,
            1,
            "invalid static string expression lowers to text",
        ),
        (
            r#"const A = () => <time>{formattedDate}</time>;"#,
            0,
            "expression child lowers to interpolation",
        ),
        (
            r#"const A = () => <time>last {unit}</time>;"#,
            0,
            "mixed JSX text and expression is dynamic content",
        ),
        (
            r#"const A = () => <time><span>2024-12-25</span></time>;"#,
            1,
            "nested text is ignored by the legacy direct text scan",
        ),
        (
            r#"const A = () => <time><>{'Christmas'}</></time>;"#,
            1,
            "plain fragments are transparent through JSX lowering",
        ),
        (
            r#"const A = () => <time><>{'2024-12-25'}</></time>;"#,
            0,
            "plain fragments preserve valid lowered text",
        ),
        (
            r#"const A = () => <Time>Christmas</Time>;"#,
            0,
            "component tag is not lowercase time",
        ),
        (
            r#"const A = () => <Foo.time>Christmas</Foo.time>;"#,
            0,
            "member component tag is not lowercase time",
        ),
        (
            r#"const A = () => <time-clock>Christmas</time-clock>;"#,
            0,
            "custom element tag is not lowercase time",
        ),
    ] {
        let diagnostics = run_over_jsx_lowered(&rule, source);
        assert_eq!(
            diagnostics.len(),
            expected,
            "lowered JSX boundary changed for {label}: {diagnostics:?}"
        );
    }
}
