use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::vue::NoBooleanAttrValue;
use vize_atelier_jsx::JsxLang;
use vize_s0::Allocator;

fn run_over_template<R: MarkupRule>(rule: &R, source: &str) -> usize {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let parser = vize_armature::Parser::new(&allocator, source);
    let (root, _errors) = parser.parse();
    let document = MarkupDocument::new(&root, TemplateSyntax::Vue);

    let mut lint = LintContext::new(&allocator, source, "test.vue");
    let mut ctx = MarkupContext::new(&mut lint, &document);
    document.visit_with(rule, &mut ctx);
    lint.diagnostics().len()
}

fn run_over_jsx_lowered<R: MarkupRule>(rule: &R, source: &str) -> usize {
    let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let lowered =
        vize_atelier_jsx::lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);

    let mut total = 0;
    for lowered_root in &lowered.roots {
        let document = MarkupDocument::new(&lowered_root.root, TemplateSyntax::Vue);
        let mut lint = LintContext::new(&allocator, source, "test.jsx");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        document.visit_with(rule, &mut ctx);
        total += lint.diagnostics().len();
    }
    total
}

fn run_over_jsx_oxc<R: MarkupRule>(rule: &R, source: &str) -> usize {
    let oxc_allocator = oxc_allocator::Allocator::default();
    let parsed = vize_atelier_jsx::parse_module(&oxc_allocator, source, JsxLang::Jsx);
    let document = MarkupDocument::from_jsx(&parsed.program, TemplateSyntax::Vue, 0);

    let lint_allocator = Allocator::with_capacity(source.len() * 4 + 1024);
    let mut lint = LintContext::new(&lint_allocator, source, "test.jsx");
    let mut ctx = MarkupContext::new(&mut lint, &document);
    document.visit_with(rule, &mut ctx);
    lint.diagnostics().len()
}

#[test]
fn no_boolean_attr_value_template_boundaries() {
    let rule = NoBooleanAttrValue;
    for (source, expected, label) in [
        (
            r#"<input disabled />"#,
            0,
            "valueless boolean attr is already shorthand",
        ),
        (
            r#"<input checked />"#,
            0,
            "another valueless boolean attr is clean",
        ),
        (
            r#"<input type="text" />"#,
            0,
            "non-boolean static attrs stay clean",
        ),
        (
            r#"<input declare="declare" webkitdirectory="webkitdirectory" />"#,
            0,
            "attributes outside the Patina local boolean list stay clean",
        ),
        (
            r#"<td nowrap="nowrap"></td>"#,
            0,
            "legacy boolean attrs omitted from this rule stay clean",
        ),
        (
            r#"<input :disabled="isDisabled" />"#,
            0,
            "bound shorthand is dynamic and outside this rule",
        ),
        (
            r#"<input v-bind:disabled="isDisabled" />"#,
            0,
            "long-form dynamic binding stays clean",
        ),
        (
            r#"<input v-bind="{ disabled: true }" />"#,
            0,
            "object v-bind has no static attr argument",
        ),
        (
            r#"<my-button disabled="disabled" />"#,
            0,
            "unknown lowercase custom tag is not native",
        ),
        (
            r#"<MyButton hidden="hidden" />"#,
            0,
            "components stay skipped",
        ),
        (
            r#"<input DISABLED="disabled" />"#,
            0,
            "attribute names remain exact",
        ),
        (
            r#"<INPUT disabled="disabled" />"#,
            0,
            "native tag names remain exact for legacy HTML lookup",
        ),
        (
            r#"<input disabled="disabled" />"#,
            1,
            "explicit boolean attr value warns",
        ),
        (
            r#"<input disabled="" />"#,
            1,
            "empty explicit value still warns",
        ),
        (
            r#"<button disabled="true">Click</button>"#,
            1,
            "arbitrary explicit value warns",
        ),
        (
            r#"<input disabled="disabled" required="required" />"#,
            2,
            "each explicit boolean attr reports",
        ),
        (
            r#"<div hidden="hidden">text</div>"#,
            1,
            "global boolean attrs on native tags report",
        ),
        (
            r#"<svg hidden="hidden"></svg>"#,
            1,
            "global boolean attrs on native SVG tags report",
        ),
    ] {
        assert_eq!(
            run_over_template(&rule, source),
            expected,
            "template boundary changed for {label}"
        );
    }
}

#[test]
fn no_boolean_attr_value_jsx_top_level_direct_matches_lowered() {
    let rule = NoBooleanAttrValue;
    for (source, expected, label) in [
        (
            r#"const A = () => <input disabled />;"#,
            0,
            "valueless boolean attr is clean",
        ),
        (
            r#"const A = () => <input disabled="disabled" />;"#,
            1,
            "static explicit boolean attr value warns",
        ),
        (
            r#"const A = () => <input disabled="" />;"#,
            1,
            "empty static explicit value warns",
        ),
        (
            r#"const A = () => <button disabled="true">Click</button>;"#,
            1,
            "arbitrary string value warns",
        ),
        (
            r#"const A = () => <input disabled="disabled" required="required" />;"#,
            2,
            "multiple boolean attrs report independently",
        ),
        (
            r#"const A = () => <input type="text" />;"#,
            0,
            "non-boolean static attrs stay clean",
        ),
        (
            r#"const A = () => <input declare="declare" webkitdirectory="webkitdirectory" />;"#,
            0,
            "attributes outside the Patina local boolean list stay clean",
        ),
        (
            r#"const A = () => <td nowrap="nowrap" />;"#,
            0,
            "legacy boolean attrs omitted from this rule stay clean",
        ),
        (
            r#"const A = () => <input disabled={isDisabled} />;"#,
            0,
            "dynamic expression-valued JSX attr is outside this rule",
        ),
        (
            r#"const A = () => <input {...props} />;"#,
            0,
            "spread attributes do not expose a static boolean value",
        ),
        (
            r#"const A = () => <my-button disabled="disabled" />;"#,
            0,
            "unknown lowercase custom tag is not native",
        ),
        (
            r#"const A = () => <MyButton disabled="disabled" />;"#,
            0,
            "capitalized components stay skipped",
        ),
        (
            r#"const A = () => <INPUT disabled="disabled" />;"#,
            0,
            "uppercase intrinsic spelling is a component in JSX",
        ),
        (
            r#"const A = () => <Forms.input disabled="disabled" />;"#,
            0,
            "member components stay skipped even with native local name",
        ),
        (
            r#"const A = () => <input DISABLED="disabled" />;"#,
            0,
            "attribute names remain exact",
        ),
        (
            r#"const A = () => <input autoFocus="autoFocus" />;"#,
            0,
            "camelCase JSX attr names stay outside the lowercase legacy list",
        ),
        (
            r#"const A = () => <svg hidden="hidden" />;"#,
            1,
            "global boolean attrs on native SVG tags report",
        ),
        (
            r#"const A = () => <svg:input disabled="disabled" />;"#,
            0,
            "namespaced intrinsic local tag keeps the lowered fallback boundary",
        ),
        (
            r#"const A = () => <svg:circle hidden="hidden" />;"#,
            0,
            "namespaced SVG tags stay outside the lowered fallback boundary",
        ),
        (
            r#"const A = () => <input html:disabled="disabled" />;"#,
            0,
            "namespaced JSX attributes stay ignored",
        ),
    ] {
        let direct = run_over_jsx_oxc(&rule, source);
        assert_eq!(direct, expected, "JSX direct case failed: {label}");
        assert_eq!(
            direct,
            run_over_jsx_lowered(&rule, source),
            "JSX direct and lowered fallback diverged for {label}"
        );
    }
}
