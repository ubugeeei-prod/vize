use crate::context::LintContext;
use crate::ir::TemplateSyntax;
use crate::markup::{MarkupContext, MarkupDocument, MarkupRule};
use crate::rules::a11y::MediaHasCaption;
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
fn media_has_caption_template() {
    let rule = MediaHasCaption;
    assert_eq!(
        run_over_template(&rule, r#"<video src="movie.mp4"></video>"#),
        1,
        "template <video> without captions must warn through markup IR"
    );
    assert_eq!(
        run_over_template(&rule, r#"<audio src="podcast.mp3"></audio>"#),
        1,
        "template <audio> without captions must warn through markup IR"
    );
    assert_eq!(
        run_over_template(&rule, r#"<video src="movie.mp4" muted></video>"#),
        0,
        "static muted attribute keeps the legacy exemption"
    );
    assert_eq!(
        run_over_template(&rule, r#"<video src="movie.mp4" :muted="true"></video>"#),
        1,
        "bound muted does not satisfy the legacy static-attribute exemption"
    );
    assert_eq!(
        run_over_template(&rule, r#"<video src="movie.mp4" MUTED></video>"#),
        1,
        "muted attribute names remain exact and case-sensitive"
    );
    assert_eq!(
        run_over_template(&rule, r#"<video v-bind="{ muted: true }"></video>"#),
        1,
        "object v-bind does not satisfy the legacy static-muted check"
    );
    assert_eq!(
        run_over_template(
            &rule,
            r#"<video src="movie.mp4" aria-label="Movie clip"></video>"#
        ),
        0,
        "static aria-label provides an accessible name"
    );
    assert_eq!(
        run_over_template(
            &rule,
            r#"<audio src="podcast.mp3" :aria-labelledby="labelId"></audio>"#
        ),
        0,
        "static-arg bound aria-labelledby provides an accessible name"
    );
    assert_eq!(
        run_over_template(
            &rule,
            r#"<video><track kind="captions" src="captions.vtt" /></video>"#
        ),
        0,
        "direct static captions track is clean"
    );
    assert_eq!(
        run_over_template(&rule, r#"<video><track kind="descriptions" /></video>"#),
        0,
        "direct static descriptions track is clean"
    );
    assert_eq!(
        run_over_template(&rule, r#"<video><track :kind="'captions'" /></video>"#),
        1,
        "bound track kind remains outside the legacy static-value check"
    );
    assert_eq!(
        run_over_template(&rule, r#"<video><track KIND="captions" /></video>"#),
        1,
        "track kind attribute names remain exact and case-sensitive"
    );
    assert_eq!(
        run_over_template(&rule, r#"<video><track kind="Captions" /></video>"#),
        1,
        "track kind values remain exact and case-sensitive"
    );
    assert_eq!(
        run_over_template(&rule, r#"<video><track kind kind="captions" /></video>"#),
        1,
        "a valueless first kind attribute masks later duplicates like the legacy helper"
    );
    assert_eq!(
        run_over_template(
            &rule,
            r#"<video><span><track kind="captions" /></span></video>"#
        ),
        1,
        "nested track does not satisfy the direct-child legacy check"
    );
    assert_eq!(
        run_over_template(&rule, r#"<VideoPlayer src="movie.mp4"></VideoPlayer>"#),
        0,
        "components stay skipped"
    );
}

#[test]
fn media_has_caption_jsx_direct_matches_lowered_boundaries() {
    let rule = MediaHasCaption;
    for (source, expected, label) in [
        (
            r#"const A = () => <video src="movie.mp4" />;"#,
            1,
            "video without captions",
        ),
        (
            r#"const A = () => <audio src="podcast.mp3" />;"#,
            1,
            "audio without captions",
        ),
        (
            r#"const A = () => <video src="movie.mp4" muted />;"#,
            0,
            "boolean muted",
        ),
        (
            r#"const A = () => <video src="movie.mp4" muted={true} />;"#,
            1,
            "dynamic muted",
        ),
        (
            r#"const A = () => <video src="movie.mp4" aria-label="Movie clip" />;"#,
            0,
            "static aria-label",
        ),
        (
            r#"const A = () => <video src="movie.mp4" aria-label={label} />;"#,
            0,
            "dynamic aria-label",
        ),
        (
            r#"const A = () => <audio src="podcast.mp3" aria-labelledby={labelId} />;"#,
            0,
            "dynamic aria-labelledby",
        ),
        (
            r#"const A = () => <video><track kind="captions" /></video>;"#,
            0,
            "captions track",
        ),
        (
            r#"const A = () => <video><track kind="descriptions" /></video>;"#,
            0,
            "descriptions track",
        ),
        (
            r#"const A = () => <video><track kind={kind} /></video>;"#,
            1,
            "dynamic track kind",
        ),
        (
            r#"const A = () => <video><track KIND="captions" /></video>;"#,
            1,
            "case-sensitive track kind attr",
        ),
        (
            r#"const A = () => <video><track kind="Captions" /></video>;"#,
            1,
            "case-sensitive track kind value",
        ),
        (
            r#"const A = () => <video><track kind kind="captions" /></video>;"#,
            1,
            "first duplicate track kind wins",
        ),
        (
            r#"const A = () => <video><span><track kind="captions" /></span></video>;"#,
            1,
            "nested track",
        ),
        (
            r#"const A = () => <video><><track kind="captions" /></></video>;"#,
            0,
            "child fragments are transparent like JSX lowering",
        ),
        (
            r#"const A = () => <Video src="movie.mp4" />;"#,
            0,
            "component",
        ),
        (
            r#"const A = () => <Player.Video src="movie.mp4" />;"#,
            0,
            "member component",
        ),
        (
            r#"const A = () => <svg:video><track kind="captions" /></svg:video>;"#,
            0,
            "namespaced video",
        ),
        (
            r#"const A = () => <video><svg:track kind="captions" /></video>;"#,
            1,
            "namespaced track",
        ),
    ] {
        assert_eq!(
            run_over_jsx_lowered(&rule, source),
            expected,
            "lowered JSX boundary changed for {label}"
        );
        assert_eq!(
            run_over_jsx_oxc(&rule, source),
            expected,
            "direct JSX IR must match the lowered boundary for {label}"
        );
    }
}
