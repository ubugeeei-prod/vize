use crate::context::LintContext;
use crate::diagnostic::LintDiagnostic;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::html::NoDuplicateDt;
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
fn no_duplicate_dt_template_direct_child_terms() {
    let rule = NoDuplicateDt;
    for (source, expected, label) in [
        (
            r#"<dl><dt>A</dt><dd>def A</dd><dt>B</dt><dd>def B</dd></dl>"#,
            0,
            "unique terms",
        ),
        (
            r#"<dl><dt>A</dt><dd>def 1</dd><dt>A</dt><dd>def 2</dd></dl>"#,
            1,
            "one duplicate term",
        ),
        (
            r#"<dl><dt>X</dt><dd>1</dd><dt>X</dt><dd>2</dd><dt>X</dt><dd>3</dd></dl>"#,
            2,
            "every repeated term after the first reports",
        ),
        (
            r#"<dl><dt>  A  </dt><dt>A</dt></dl>"#,
            1,
            "term text is trimmed before comparison",
        ),
        (
            r#"<dl><dt>A</dt><dt>A </dt><dt> A</dt></dl>"#,
            2,
            "trimmed repeats after the first each report",
        ),
        (
            r#"<dl><dt>A{{ suffix }}</dt><dt>A</dt></dl>"#,
            1,
            "only direct text nodes form the term",
        ),
        (
            r#"<dl><dt><span>A</span></dt><dt>A</dt><dt>A</dt></dl>"#,
            1,
            "nested text is ignored before duplicate comparison",
        ),
        (
            r#"<dl><dt> </dt><dt></dt><dt>A</dt></dl>"#,
            0,
            "empty normalized terms are ignored",
        ),
        (
            r#"<dl><dt>A</dt><dt>a</dt></dl>"#,
            0,
            "term comparison is case-sensitive",
        ),
        (
            r#"<dl><dt>A B</dt><dt>A  B</dt></dl>"#,
            1,
            "parser-normalized inner whitespace participates in comparison",
        ),
        (
            r#"<dl><div><dt>A</dt></div><dt>A</dt></dl>"#,
            0,
            "nested dt elements are not dl direct children",
        ),
        (
            r#"<MyDl><dt>A</dt><dt>A</dt></MyDl>"#,
            0,
            "component parents are skipped",
        ),
        (
            r#"<MyDl><dl><dt>A</dt><dt>A</dt></dl></MyDl>"#,
            1,
            "native dl descendants under components still run",
        ),
        (
            r#"<DL><dt>A</dt><dt>A</dt></DL>"#,
            0,
            "dl tag names are case-sensitive",
        ),
        (
            r#"<dl><DT>A</DT><dt>A</dt></dl>"#,
            0,
            "dt tag names are case-sensitive",
        ),
        (
            r#"<dl><dt>A</dt><dt>A</dt></dl><dl><dt>A</dt><dt>A</dt></dl>"#,
            2,
            "separate dl elements do not share state",
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
fn no_duplicate_dt_template_reports_on_duplicate_dt_element() {
    let rule = NoDuplicateDt;
    let source = r#"<dl><dt>A</dt><dd>def 1</dd><dt>A</dt><dd>def 2</dd></dl>"#;
    let diagnostics = run_over_template(&rule, source);
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");

    let duplicate_start = source.rfind("<dt>").unwrap() as u32;
    assert_eq!(
        diagnostic_ranges(&diagnostics),
        vec![(duplicate_start, duplicate_start + "<dt>".len() as u32)]
    );
}

#[test]
fn no_duplicate_dt_template_reports_multi_term_duplicates_in_order() {
    let rule = NoDuplicateDt;
    let source = r#"<dl><dt>A</dt><dt>B</dt><dt>A</dt><dt>B</dt></dl>"#;
    let diagnostics = run_over_template(&rule, source);
    assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");

    let expected: Vec<(u32, u32)> = source
        .match_indices("<dt>")
        .skip(2)
        .map(|(offset, term)| {
            let start = offset as u32;
            (start, start + term.len() as u32)
        })
        .collect();
    assert_eq!(diagnostic_ranges(&diagnostics), expected);
    assert!(
        diagnostics[0].message.contains("A"),
        "first duplicate message should name term A: {:?}",
        diagnostics[0]
    );
    assert!(
        diagnostics[1].message.contains("B"),
        "second duplicate message should name term B: {:?}",
        diagnostics[1]
    );
}

#[test]
fn no_duplicate_dt_jsx_lowered_matches_legacy_fallback_boundaries() {
    let rule = NoDuplicateDt;
    for (source, expected, label) in [
        (
            r#"const A = () => <dl><dt>A</dt><dd>def 1</dd><dt>A</dt></dl>;"#,
            1,
            "one duplicate term",
        ),
        (
            r#"const A = () => <dl><dt>{'A'}</dt><dt>A</dt></dl>;"#,
            1,
            "static string expression text still follows JSX lowering",
        ),
        (
            r#"const A = () => <dl><dt>A{'!'}</dt><dt>A</dt></dl>;"#,
            0,
            "JSX string children around expression content follow lowering",
        ),
        (
            r#"const A = () => <dl><><dt>A</dt><dt>A</dt></></dl>;"#,
            1,
            "plain fragments are transparent through JSX lowering",
        ),
        (
            r#"const A = () => <dl>{cond && <dt>A</dt>}<dt>A</dt></dl>;"#,
            0,
            "conditional dt is not a direct dl child in the legacy fallback",
        ),
        (
            r#"const A = () => <dl>{items.map(() => <dt>A</dt>)}<dt>A</dt></dl>;"#,
            0,
            "mapped dt is not a direct dl child in the legacy fallback",
        ),
        (
            r#"const A = () => <Dl><dt>A</dt><dt>A</dt></Dl>;"#,
            0,
            "component parents are skipped",
        ),
        (
            r#"const A = () => <dl><DT>A</DT><dt>A</dt></dl>;"#,
            0,
            "uppercase DT stays outside exact lowercase dt matching",
        ),
        (
            r#"const A = () => <svg:dl><dt>A</dt><dt>A</dt></svg:dl>;"#,
            0,
            "namespaced dl stays outside exact lowercase dl matching",
        ),
        (
            r#"const A = () => <dl><svg:dt>A</svg:dt><dt>A</dt></dl>;"#,
            0,
            "namespaced dt stays outside exact lowercase dt matching",
        ),
        (
            r#"const A = () => <Lists.dl><dt>A</dt><dt>A</dt></Lists.dl>;"#,
            0,
            "member dl stays outside exact lowercase dl matching",
        ),
        (
            r#"const A = () => <dl><Terms.dt>A</Terms.dt><dt>A</dt></dl>;"#,
            0,
            "member dt stays outside exact lowercase dt matching",
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
