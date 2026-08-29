use crate::linter::{LintResult, Linter};
use crate::rule::{Rule, RuleRegistry};
use crate::rules::a11y::MediaHasCaption;
use vize_atelier_jsx::JsxLang;

fn linter_with(rule: Box<dyn Rule>) -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(rule);
    Linter::with_registry(registry)
}

fn diagnostic_rules(result: &LintResult) -> Vec<&str> {
    result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.rule_name.as_ref())
        .collect()
}

#[test]
fn media_has_caption_fires_on_jsx_and_tsx_ir() {
    let linter = linter_with(Box::new(MediaHasCaption));
    let source = r#"const A = () => <video src="movie.mp4" />;"#;
    let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
    assert_eq!(
        result.warning_count, 1,
        "JSX <video> without captions must flag through the IR pass: {:?}",
        result.diagnostics
    );
    assert_eq!(diagnostic_rules(&result), vec!["a11y/media-has-caption"]);

    let diag = &result.diagnostics[0];
    let video_start = source.find("<video").unwrap() as u32;
    assert_eq!(
        diag.start, video_start,
        "range must start at the written JSX element"
    );

    let tsx = linter.lint_jsx(
        r#"const A = (): JSX.Element => <audio src="podcast.mp3" />;"#,
        "test.tsx",
        JsxLang::Tsx,
    );
    assert_eq!(
        diagnostic_rules(&tsx),
        vec!["a11y/media-has-caption"],
        "TSX <audio> without captions must also flag through the IR pass"
    );
}

#[test]
fn media_has_caption_preserves_legacy_jsx_boundaries() {
    let linter = linter_with(Box::new(MediaHasCaption));
    for source in [
        r#"const A = () => <video muted />;"#,
        r#"const A = () => <video aria-label={label} />;"#,
        r#"const A = () => <audio aria-labelledby={labelId} />;"#,
        r#"const A = () => <video><track kind="captions" /></video>;"#,
        r#"const A = () => <video><track kind="descriptions" /></video>;"#,
        r#"const A = () => <video><><track kind="captions" /></></video>;"#,
        r#"const A = () => <Video src="movie.mp4" />;"#,
        r#"const A = () => <Player.Video src="movie.mp4" />;"#,
        r#"const A = () => <svg:video><track kind="captions" /></svg:video>;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.warning_count, 0,
            "must stay clean for {source}: {:?}",
            result.diagnostics
        );
    }

    for source in [
        r#"const A = () => <video muted={true} />;"#,
        r#"const A = () => <video><track kind={kind} /></video>;"#,
        r#"const A = () => <video><track KIND="captions" /></video>;"#,
        r#"const A = () => <video><track kind="Captions" /></video>;"#,
        r#"const A = () => <video><track kind kind="captions" /></video>;"#,
        r#"const A = () => <video><span><track kind="captions" /></span></video>;"#,
        r#"const A = () => <video><svg:track kind="captions" /></video>;"#,
    ] {
        let result = linter.lint_jsx(source, "test.jsx", JsxLang::Jsx);
        assert_eq!(
            result.warning_count, 1,
            "must keep warning for {source}: {:?}",
            result.diagnostics
        );
    }
}

#[test]
fn migrated_media_has_caption_reports_once_not_per_backend() {
    let linter = linter_with(Box::new(MediaHasCaption));
    let result = linter.lint_jsx(
        r#"const A = () => <video src="movie.mp4" />;"#,
        "test.jsx",
        JsxLang::Jsx,
    );
    assert_eq!(
        result.diagnostics.len(),
        1,
        "a migrated media-has-caption rule must report once: {:?}",
        result.diagnostics
    );
}
