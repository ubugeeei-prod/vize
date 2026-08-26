//! Regression tests for the LightningCSS engine panic defense (#3276, #3280,
//! #4961).
//!
//! The css_parse fuzz target found two upstream crash classes that still
//! reproduce on the latest released lightningcss/cssparser-color versions
//! (retirement tracked in #3295):
//!
//! - `Percentage::parse` hits `unreachable!()` for math functions in
//!   percentage-typed slots whose calc tree does not fold to a plain
//!   percentage. This fires in every profile, and the release profile builds
//!   with `panic = "abort"`, so it used to crash the shipped CLI and LSP on
//!   inputs as small as `lch(sign(-50%)`. The same `unreachable!()` also
//!   fires with no `%` at all when the tree folds to a bare number
//!   (`Calc::Number`) in a percentage-only slot — `text-size-adjust:asin(5)`,
//!   `font-stretch:calc(5)` (#4961).
//! - `hsl_to_rgb` debug-asserts hue into `[0, 1]`, so a non-finite hue
//!   crashes every build with debug assertions (tests, fuzzing).
//!
//! The defense is layered (`css::parser::engine_boundary`): a pre-parse value
//! guard rejects the crashing math-function shapes in color-function and
//! opacity contexts in every profile, and a `catch_unwind` boundary converts
//! the remaining engine panics into error results in unwinding profiles.
//! These tests pin the empirically-mapped panic matrix, the guard's
//! documented over-rejections, and the neighboring shapes that must keep
//! parsing, so an upstream bump that shifts the surface is visible.
#![cfg(feature = "native")]

use vize_atelier_sfc::{CssCompileOptions, compile_css, parse_css_ast};

/// The 39-byte css_parse fuzz artifact from #3276 (run 30147643440).
const LCH_SIGN_ARTIFACT: &str = "\n@t (lch(sign(-50%);\n  mask-clipatenv5,";

const GUARD_ERROR: &str = "CSS parse error: unsupported math function in a percentage context (upstream lightningcss defect; see vize issue #3295)";
const PARSE_BOUNDARY_ERROR: &str = "CSS parse error: the CSS engine hit an internal defect (upstream lightningcss panic; see vize issue #3295)";
const COMPILE_GUARD_ERROR: &str = "CSS parse error: unsupported math function in a percentage context (upstream lightningcss defect; see vize issue #3295)";

/// Math-function shapes in guarded contexts (color functions, opacity-family
/// declarations): every row panics inside `Percentage::parse` without the
/// guard, in every build profile, so the pre-parse guard must reject them
/// before the engine sees them — that is what keeps the `panic = "abort"`
/// release binaries alive.
#[test]
fn parse_css_ast_rejects_the_percentage_math_panic_matrix() {
    for source in [
        LCH_SIGN_ARTIFACT,
        "a{color:lch(sign(-50%) 0 0)}",
        "a{color:lch(0% abs(-50%) 0)}",
        "a{color:hsl(0 abs(-50%) 0%)}",
        "a{color:hwb(0 abs(-50%) 0%)}",
        "a{color:rgb(abs(-50%) 0 0)}",
        "a{color:hsl(0 0% 0% / abs(-50%))}",
        "a{color:hsl(0 mod(-50%, 2) 0%)}",
        "a{color:hsl(0 rem(-50%, 2) 0%)}",
        "a{color:hsl(0 round(-50%, 10) 0%)}",
        "a{color:hsl(0 round(nearest, -50%, 10) 0%)}",
        "a{color:hsl(0 min(-50%, 2) 0%)}",
        "a{color:hsl(0 max(-50%, 2) 0%)}",
        "a{color:hsl(0 clamp(1, 50%, 2) 0%)}",
        "a{color:hsl(0 hypot(-50%, 2) 0%)}",
        "a{color:hsl(0 calc(sign(-50%)) 0%)}",
        "a{color:hsl(0 abs(sign(-50%)) 0%)}",
        "a{color:hsl(0 min(abs(-50%), 60%) 0%)}",
        "a{opacity:sign(-50%)}",
        "a{opacity:abs(-50%)}",
        "a{opacity:calc(sign(-50%))}",
        "a{fill-opacity:abs(-50%)}",
    ] {
        let result = parse_css_ast(source, &CssCompileOptions::default());
        assert!(
            result.ast.is_none(),
            "expected guard rejection for {source:?}"
        );
        assert_eq!(result.errors, [GUARD_ERROR], "source {source:?}");
        assert_eq!(
            result.warnings,
            Vec::<vize_carton::String>::new(),
            "source {source:?}"
        );
    }
}

/// The guard is slot-blind inside a color function and span-blind around
/// nested `sign`/`abs`, so two shapes that parse on today's engine are
/// rejected with it. Pinning them keeps the trade-off explicit; if a future
/// slot-aware guard un-rejects them, this test is the place that changes.
#[test]
fn parse_css_ast_documents_the_guard_over_rejections() {
    for source in [
        // hue slot is not percentage-typed; the engine parses this today
        "a{color:hsl(sign(-50%) 0% 0%)}",
        // the % product rescues the type upstream, but the nested sign( is
        // still rejected by the span scan
        "a{color:hsl(0 calc(10% * sign(-50%)) 0%)}",
    ] {
        let result = parse_css_ast(source, &CssCompileOptions::default());
        assert!(result.ast.is_none(), "expected rejection for {source:?}");
        assert_eq!(result.errors, [GUARD_ERROR], "source {source:?}");
    }
}

/// Percentage-typed slots outside the guarded contexts still reach the
/// engine; in unwinding profiles (this test binary) the `catch_unwind`
/// boundary must convert the upstream panic into an error result. Release
/// binaries still abort on these — #3295 tracks the upstream fix.
#[test]
fn parse_css_ast_reports_under_guarded_engine_panics_as_errors() {
    let source = "a{border-image-slice:abs(-50%)}";
    let result = parse_css_ast(source, &CssCompileOptions::default());
    assert!(result.ast.is_none());
    assert_eq!(result.errors, [PARSE_BOUNDARY_ERROR]);
}

/// The 66-byte css_parse fuzz artifact from #4961 (run 32932217923) plus the
/// minimal shapes of its class: a math function whose calc tree folds to a
/// bare number (`Calc::Number`, NaN included — `asin(5)`) in a
/// percentage-only slot trips the `Percentage::parse` `unreachable!()` with
/// no `%` involved. The percentage-only routes are the `text-size-adjust`
/// and `font-stretch` longhands and `@property` `<percentage>` initial-value
/// parsing; in unwinding profiles (this test binary) the `catch_unwind`
/// boundary must convert the panic into an error result. Release binaries
/// still abort on these — #3295 tracks the upstream fix.
#[test]
fn parse_css_ast_reports_number_folding_math_in_percentage_slots_as_errors() {
    const NUMBER_FOLD_ARTIFACT: &str = "losrasinable-p {\n t>px`5;  text-size-adjust:asin(5\u{c})\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}UUUrath";
    // Pin the fixture size: the artifact must stay byte-for-byte the one the
    // fuzzer produced.
    assert_eq!(
        NUMBER_FOLD_ARTIFACT.len(),
        66,
        "fuzz artifact must stay verbatim"
    );
    for source in [
        NUMBER_FOLD_ARTIFACT,
        "a{text-size-adjust:asin(5)}",
        "a{text-size-adjust:calc(5)}",
        "a{text-size-adjust:calc(1 + 1)}",
        "a{text-size-adjust:sqrt(2)}",
        "a{text-size-adjust:sign(5)}",
        "a{font-stretch:calc(5)}",
        "@property --x{syntax:\"<percentage>\";inherits:false;initial-value:calc(5);}",
    ] {
        let result = parse_css_ast(source, &CssCompileOptions::default());
        assert!(
            result.ast.is_none(),
            "expected boundary error for {source:?}"
        );
        assert_eq!(result.errors, [PARSE_BOUNDARY_ERROR], "source {source:?}");
    }
}

/// A non-finite hue trips an upstream `debug_assert`, so the boundary reports
/// an error in debug-assertion builds; in release builds the parse currently
/// succeeds with a folded garbage value. Either way the process must survive
/// and the boundary error must not appear outside the panic path.
#[test]
fn parse_css_ast_survives_non_finite_hue() {
    for source in [
        "a{color:hsl(1e40 0 0)}",
        "a{color:hsl(1111111111111111111111111111111111111111111111111111112 0 0)}",
        "a{color:hsl(calc(1e20 * 1e20) 0% 0%)}",
    ] {
        let result = parse_css_ast(source, &CssCompileOptions::default());
        if cfg!(debug_assertions) {
            assert!(
                result.ast.is_none(),
                "expected boundary error for {source:?}"
            );
            assert_eq!(result.errors, [PARSE_BOUNDARY_ERROR], "source {source:?}");
        } else {
            assert_eq!(
                result.errors,
                Vec::<vize_carton::String>::new(),
                "source {source:?}"
            );
        }
    }
}

/// `compile_css` shares the guard: the authored source passes through next to
/// the error exactly like the existing parse-error path, so SFC compilation
/// degrades to a diagnostic instead of killing the process.
#[test]
fn compile_css_rejects_guarded_shapes_and_passes_the_source_through() {
    let css = "a{color:lch(sign(-50%) 0 0)}";
    let result = compile_css(css, &CssCompileOptions::default());
    assert_eq!(result.code, css);
    assert!(result.map.is_none());
    assert_eq!(result.errors, [COMPILE_GUARD_ERROR]);
    assert!(result.exports.is_none());
}

/// The panic matrix's nearest working neighbors: type-consistent argument
/// lists inside guarded contexts, and the same functions outside them. These
/// parse today and the guard must never widen enough to break them.
#[test]
fn parse_css_ast_keeps_accepting_resolvable_math_functions() {
    for source in [
        "a{color:hsl(0 mod(50%, 7%) 0%)}",
        "a{color:hsl(0 rem(50%, 7%) 0%)}",
        "a{color:hsl(0 round(-50%, 10%) 0%)}",
        "a{color:hsl(0 min(10%, 20%) 0%)}",
        "a{color:lch(sign(-5) 0 0)}",
        "a{color:hsl(0 50% 50%)}",
        "a{width:sign(-50%)}",
        "a{width:abs(-50%)}",
        "a{width:calc(10px * sign(-50%))}",
        "a{width:min(10%, 20%)}",
        "a{width:min(100%, 40rem)}",
        "a{opacity:calc(50% * 2)}",
        "a{opacity:min(10%, 90%)}",
        "a{transition:opacity 0.3s, min-width 1s}",
        "a{--sign:1}b{width:var(--sign)}",
        "a{background-image:linear-gradient(red abs(-50%), blue)}",
        "a{content:\"sign(-50%) in a string\"}",
        "a{font-size:abs(-50%)}",
        // #4961 neighbors: percentage-folding trees, keywords, and plain
        // percentages in the percentage-only slots parse fine, as do
        // number-folding trees in number- and angle-typed slots.
        "a{text-size-adjust:50%}",
        "a{text-size-adjust:auto}",
        "a{text-size-adjust:calc(50% + 10%)}",
        "a{font-stretch:condensed}",
        "a{font-stretch:calc(50% + 10%)}",
        "a{rotate:asin(0.5)}",
        "a{line-height:calc(exp(1))}",
        "a{opacity:calc(sqrt(0.25))}",
        "a{width:calc(pow(2, 3) * 1px)}",
    ] {
        let result = parse_css_ast(source, &CssCompileOptions::default());
        assert!(result.ast.is_some(), "expected clean parse for {source:?}");
        assert_eq!(
            result.errors,
            Vec::<vize_carton::String>::new(),
            "source {source:?}"
        );
    }
}

/// The css_parse fuzz artifact from #3303 (run 30335797526, found on
/// 4d619ee70 before the engine boundary landed): 1045 bytes of mangled
/// stylesheet that drives a non-finite hue into the upstream
/// `hsl_to_rgb` `debug_assert` — a plain finite negative hue like
/// `hsla(-842 ...)` normalizes fine, so the trigger hides in the mangled
/// body. With the boundary in place the parse must survive in every profile;
/// the exact result stays profile-dependent because the panic is an upstream
/// `debug_assert`.
#[test]
fn parse_css_ast_survives_the_hue_assert_artifact() {
    const ARTIFACT: &str = "\n.t {\n  \n.comglsdhriight {\n  --code;\n  box-shadow: inset 0 \u{11}U\u{7}\u{0}\u{0}\u{0}.primeighlio\n\u{4}UUUUurlh255p\u{1}\u{0}\u{0}\u{0}\u{0}52:, .(255p\u{1}\u{0}\u{0}\u{0}\u{0}\u{0}2rad, 2primeighlig\n\u{4}UUUUUU hsla(-842 5 2\n\u{4}UUUUhsl, 2\n\u{4}UUUUUUU.primeighlig\n\u{4}UUUUUU hsla(-842 5 1\n\u{4}UUUUhsla(-555 52:, 2\n\u{4}UUUUUUUa(-555 52:.l {\n m:\u{4},\nU U2px(-3ody[da\u{0}\u{0}\u{0}B\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}a-them=\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}] .codsohighlight :deUUUUep(.codel\\\\\\fff 2\n\u{4}UUUUhsl, 2\n\u{4}UUUUUUU.primeighlig\n\u{4}UUUUUU hsla(-8555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555555542 5 1\n\u{4}UUUUhsla(-555 52:, 2\n\u{4}UUUUprimeighlig\n\u{4}UUUUUU hsla(-842 5 2\n\u{4}UUUUhsl, 2\n\u{4}UUUUUUU.primeighlig\n\u{4}UUUUUU hsla(-842 5 1\n\u{4}UUUUhsla(-555 52:, 2\n\u{4}UUUUUUUa(-555 52:.l {\n m:\u{4},\nU U2px(-3ody[da\u{0}\u{0}\u{0}B\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}a-them=\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}] .codsohighlight :deUUUUep(.codel\\\\\\fff 2\n\u{4}UUUUhsl, 2\n\u{4}UUUUUUU.primeighlig\n\u{4}UUUUUU hsla(-842 5 1\n\u{4}UUUUhsla(-555 52:, 2\n\u{4}UUUUUUUa(-555 52:.l {\n m:\u{4},\nU U2px(-3ody[daUUUa(-555 52:.l {\n m:\u{4},\nU U2px(-3ody[da\u{0}\u{0}\u{0}B\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}a-them=\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}] .codsohighlight :deUUUUep(.codel\\\\\\fff\\ffyininset \u{11}p x0UU cd);\n  or: \u{0}\u{0}0ar(et--\n";
    // Pin the fixture size: the artifact must stay byte-for-byte the one the
    // fuzzer produced, so an accidental truncation cannot quietly turn this
    // into a weaker input that no longer reaches the assert.
    assert_eq!(ARTIFACT.len(), 1045, "fuzz artifact must stay verbatim");
    let result = parse_css_ast(ARTIFACT, &CssCompileOptions::default());
    if cfg!(debug_assertions) {
        assert!(result.ast.is_none());
        assert_eq!(result.errors, [PARSE_BOUNDARY_ERROR]);
    } else {
        assert!(
            !result
                .errors
                .contains(&vize_carton::String::from(PARSE_BOUNDARY_ERROR))
        );
    }

    // A plain finite negative hue is not the crasher: hue normalization
    // handles it, so this must keep parsing cleanly in every profile.
    let negative_hue = "a{color:hsla(-842 5 2 / 1)}";
    let negative_hue_result = parse_css_ast(negative_hue, &CssCompileOptions::default());
    assert!(negative_hue_result.ast.is_some());
    assert_eq!(
        negative_hue_result.errors,
        Vec::<vize_carton::String>::new()
    );
}
