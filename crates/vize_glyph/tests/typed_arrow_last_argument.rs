//! #7868: typed last-argument arrows retain the same call grouping as untyped arrows.
use oxc_span::{FileExtension, SourceType};
use vize_glyph::{Allocator, EndOfLine, FormatOptions, format_script_with_source_type, format_sfc};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/a.ts.txt"
);
const ORIGINAL_CRLF: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/a.ts.crlf.txt"
);
const EXPECTED: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/a.reference.expected.txt"
);
const SFC: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/App.vue.txt"
);
const SFC_CRLF: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/App.vue.crlf.txt"
);
const SFC_EXPECTED: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/typed-arrow-last-argument/reference.expected.txt"
);

fn script_fixed_point(
    source: &str,
    expected: &str,
    options: &FormatOptions,
    extension: FileExtension,
) {
    let mut current = vize_l0::String::from(source);
    for _ in 0..3 {
        let allocator = Allocator::default();
        current = format_script_with_source_type(
            &current,
            options,
            &allocator,
            SourceType::from(extension).with_module(true),
        )
        .unwrap();
        assert_eq!(current.as_str(), expected);
    }
}

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
fn complete_original_lf_and_crlf_programs_hug_the_typed_arrow_through_three_passes() {
    assert_eq!(ORIGINAL.len(), 191);
    assert_eq!(ORIGINAL_CRLF.len(), 193);
    for source in [ORIGINAL, ORIGINAL_CRLF] {
        script_fixed_point(
            source,
            EXPECTED,
            &FormatOptions::default(),
            FileExtension::Ts,
        );
    }
    let windows_expected = EXPECTED.replace('\n', "\r\n");
    for end_of_line in [EndOfLine::Crlf, EndOfLine::Auto] {
        let options = FormatOptions {
            end_of_line,
            ..FormatOptions::default()
        };
        script_fixed_point(
            ORIGINAL_CRLF,
            &windows_expected,
            &options,
            FileExtension::Ts,
        );
    }
    let untyped = ORIGINAL.replace(": Row =>", " =>");
    let untyped_expected = EXPECTED.replace(": Row =>", " =>");
    script_fixed_point(
        &untyped,
        &untyped_expected,
        &FormatOptions::default(),
        FileExtension::Ts,
    );
}

#[test]
fn complete_script_and_setup_carriers_keep_whole_outputs_and_changed_flags() {
    for source in [SFC, SFC_CRLF] {
        sfc_fixed_point(source, SFC_EXPECTED, &FormatOptions::default());
    }
    let setup = SFC.replace("<script lang", "<script setup lang");
    let setup_expected = SFC_EXPECTED.replace("<script lang", "<script setup lang");
    sfc_fixed_point(&setup, &setup_expected, &FormatOptions::default());
    let windows_expected = SFC_EXPECTED.replace('\n', "\r\n");
    for end_of_line in [EndOfLine::Crlf, EndOfLine::Auto] {
        let options = FormatOptions {
            end_of_line,
            ..FormatOptions::default()
        };
        sfc_fixed_point(SFC_CRLF, &windows_expected, &options);
    }
}

#[test]
fn typed_bodies_and_comments_keep_complete_independent_reference_outputs() {
    // Complete references independently checked with Prettier 3.9.6, width 100.
    for (source, expected, extension) in [
        (
            "const rows = values.map((value): Readonly<Row> => ({ id: value, name: \"name\", label: \"label\", enabled: true }));\n",
            "const rows = values.map((value): Readonly<Row> => ({\n  id: value,\n  name: \"name\",\n  label: \"label\",\n  enabled: true,\n}));\n",
            FileExtension::Ts,
        ),
        (
            "const rows = values.map((value): Row[] => [value, otherValue, thirdValue, fourthValue]);\n",
            "const rows = values.map((value): Row[] => [value, otherValue, thirdValue, fourthValue]);\n",
            FileExtension::Ts,
        ),
        (
            "const rows = values.map((value): Row => {\n  // retain body comment\n  return { id: value, name: \"name\", label: \"label\", enabled: true };\n});\n",
            "const rows = values.map((value): Row => {\n  // retain body comment\n  return { id: value, name: \"name\", label: \"label\", enabled: true };\n});\n",
            FileExtension::Ts,
        ),
        (
            "const scalar = values.map((value): Row => value);\n",
            "const scalar = values.map((value): Row => value);\n",
            FileExtension::Ts,
        ),
        (
            "const empty = consume((): Row => {});\n",
            "const empty = consume((): Row => {});\n",
            FileExtension::Ts,
        ),
        (
            "const rows = values.map((value): Row | null => ({ id: value, name: \"name\", label: \"label\", enabled: true }));\n",
            "const rows = values.map((value): Row | null => ({\n  id: value,\n  name: \"name\",\n  label: \"label\",\n  enabled: true,\n}));\n",
            FileExtension::Ts,
        ),
        (
            "const render = values.map((value): Element => <RowView value={value} label=\"label\" enabled={true} />);\n",
            "const render = values.map((value): Element => (\n  <RowView value={value} label=\"label\" enabled={true} />\n));\n",
            FileExtension::Tsx,
        ),
    ] {
        script_fixed_point(source, expected, &FormatOptions::default(), extension);
    }
}
