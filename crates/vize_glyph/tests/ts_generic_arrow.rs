use oxc_span::{FileExtension, SourceType};
use vize_glyph::{
    Allocator, EndOfLine, FormatOptions, TrailingComma, format_script_with_source_type, format_sfc,
};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter-regressions/ts-generic-arrow-7965/Generic.vue.txt"
);
const PLAIN: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter-regressions/ts-generic-arrow-7965/plain.ts.txt"
);

fn sfc_fixed_point(source: &str, expected: &str, options: &FormatOptions) {
    let mut current = source.to_owned();
    for _ in 0..3 {
        let result = format_sfc(&current, options).unwrap();
        assert_eq!(result.code.as_str(), expected);
        assert_eq!(result.changed, current != expected);
        current = result.code.to_string();
    }
}

#[test]
fn complete_original_setup_ts_keeps_both_generic_parameters() {
    sfc_fixed_point(ORIGINAL, ORIGINAL, &FormatOptions::default());
}

#[test]
fn regular_ts_and_preexisting_disambiguation_commas_use_plain_ts_printing() {
    let regular = ORIGINAL.replace("<script setup", "<script");
    sfc_fixed_point(&regular, &regular, &FormatOptions::default());
    let with_commas = ORIGINAL
        .replace("<T>", "<T,>")
        .replace("<T = unknown>", "<T = unknown,>");
    sfc_fixed_point(&with_commas, ORIGINAL, &FormatOptions::default());
}

#[test]
fn declared_tsx_keeps_mandatory_commas_under_every_trailing_comma_option() {
    let tsx = ORIGINAL
        .replace("lang=\"ts\"", "lang=\"tsx\"")
        .replace("<T>", "<T,>")
        .replace("<T = unknown>", "<T = unknown,>");
    for trailing_comma in [TrailingComma::None, TrailingComma::Es5, TrailingComma::All] {
        let options = FormatOptions {
            trailing_comma,
            ..FormatOptions::default()
        };
        sfc_fixed_point(ORIGINAL, ORIGINAL, &options);
        sfc_fixed_point(&tsx, &tsx, &options);
    }
}

#[test]
fn crlf_auto_and_single_pass_keep_the_complete_ts_document() {
    let source = ORIGINAL.replace('\n', "\r\n");
    for end_of_line in [EndOfLine::Crlf, EndOfLine::Auto] {
        for skip_script_stabilization in [false, true] {
            let options = FormatOptions {
                end_of_line,
                skip_script_stabilization,
                ..FormatOptions::default()
            };
            sfc_fixed_point(&source, &source, &options);
        }
    }
}

#[test]
fn explicit_public_ts_and_tsx_paths_keep_their_whole_outputs() {
    let allocator = Allocator::default();
    let options = FormatOptions::default();
    let comma = PLAIN.replace("<T>", "<T,>");
    for (source, source_type, expected) in [
        (PLAIN, SourceType::from(FileExtension::Ts), PLAIN),
        (
            comma.as_str(),
            SourceType::from(FileExtension::Tsx),
            comma.as_str(),
        ),
        // Unknown-extension public callers retain their original formatter policy.
        (PLAIN, SourceType::ts(), comma.as_str()),
    ] {
        let actual = format_script_with_source_type(
            source,
            &options,
            &allocator,
            source_type.with_module(true),
        )
        .unwrap();
        assert_eq!(actual.as_str(), expected);
    }
}

#[test]
fn jsx_admission_and_ts_angle_assertions_keep_their_original_languages() {
    let allocator = Allocator::default();
    let options = FormatOptions::default();
    let jsx = "const card = <section />;\n";
    assert!(
        format_script_with_source_type(
            jsx,
            &options,
            &allocator,
            SourceType::from(FileExtension::Ts).with_module(true),
        )
        .is_err()
    );
    let actual = format_script_with_source_type(
        jsx,
        &options,
        &allocator,
        SourceType::from(FileExtension::Tsx).with_module(true),
    )
    .unwrap();
    assert_eq!(actual.as_str(), jsx);
    let source = "<script lang=\"ts\">\nconst asserted = <T>value;\n</script>\n";
    sfc_fixed_point(source, source, &options);
}

#[test]
fn constraints_and_multiple_parameters_keep_the_complete_shared_spelling() {
    let script = "const object = <T extends object>(value: T): T => value;\nconst pair = <T, U>(first: T, second: U): [T, U] => [first, second];\n";
    for opening in ["<script lang=\"ts\">\n", "<script lang=\"tsx\">\n"] {
        let source = [opening, script, "</script>\n"].concat();
        sfc_fixed_point(&source, &source, &FormatOptions::default());
    }
}
