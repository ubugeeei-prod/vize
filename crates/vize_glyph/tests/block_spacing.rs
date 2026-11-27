//! `blockSpacing` chooses the line breaks between top-level SFC blocks:
//! `blank-line` writes exactly one blank line, `preserve` keeps an authored
//! blank line (at most one) and adds none, as Prettier does.
use vize_glyph::{BlockSpacing, EndOfLine, FormatOptions, FormatPreset, format_sfc};

fn assert_fixed_point(input: &str, expected: &str, options: &FormatOptions) {
    let mut current: vize_l0::String = input.into();
    for pass in 0..3 {
        let result = format_sfc(&current, options).expect("format");
        assert_eq!(result.code.as_str(), expected, "pass {pass}");
        assert_eq!(result.changed, current.as_str() != expected, "pass {pass}");
        current = result.code;
    }
}

const INPUT: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-prettier-block-spacing/App.vue.txt"
);
const PRETTIER: &str = include_str!(
    "../../../tests/_fixtures/differential/formatter/sfc-prettier-block-spacing/reference.expected.txt"
);

const BLANK_LINES: &str = r#"<template>
  <p>{{ note }}</p>
</template>

<script setup lang="ts">
const note = "";
</script>

<style scoped>
p {
  color: red;
}
</style>

<!-- How the note reads. -->
<docs>
A note.
</docs>

<docs>
Another note.
</docs>
"#;

#[test]
fn prettier_preset_keeps_authored_block_spacing() {
    // Prettier 3.10.1 output for the fixture input.
    let options = FormatOptions::from_preset(FormatPreset::Prettier);
    assert_eq!(options.block_spacing, BlockSpacing::Preserve);
    assert_fixed_point(INPUT, PRETTIER, &options);

    // The reported file, already formatted by Prettier, stays unchanged.
    let reported = "<template>\n  <p>{{ note }}</p>\n</template>\n<script setup lang=\"ts\">\nconst note = \"\";\n</script>\n<style scoped>\np {\n  color: red;\n}\n</style>\n";
    assert_fixed_point(reported, reported, &options);
}

#[test]
fn default_block_spacing_still_writes_one_blank_line() {
    let options = FormatOptions {
        sort_blocks: false,
        ..FormatOptions::default()
    };
    assert_eq!(options.block_spacing, BlockSpacing::BlankLine);
    assert_fixed_point(INPUT, BLANK_LINES, &options);
    assert_fixed_point(PRETTIER, BLANK_LINES, &options);
}

#[test]
fn block_spacing_overrides_either_preset() {
    let mut preserve = FormatOptions::from_preset(FormatPreset::Vize);
    preserve.sort_blocks = false;
    preserve.block_spacing = BlockSpacing::Preserve;
    assert_fixed_point(INPUT, PRETTIER, &preserve);

    let mut blank_line = FormatOptions::from_preset(FormatPreset::Prettier);
    blank_line.block_spacing = BlockSpacing::BlankLine;
    assert_fixed_point(INPUT, BLANK_LINES, &blank_line);
}

#[test]
fn config_block_spacing_applies_after_the_preset() {
    let profile = |json: &str| {
        serde_json::from_str::<vize_l0::config::ConfigDocument>(json)
            .map(|document| FormatOptions::from_profile(document.formatter_profile()))
    };
    let preset = profile(r#"{"formatter":{"preset":"prettier"}}"#).unwrap();
    assert_eq!(preset.unwrap().block_spacing, BlockSpacing::Preserve);
    let explicit =
        profile(r#"{"formatter":{"preset":"prettier","blockSpacing":"blank-line"}}"#).unwrap();
    assert_eq!(explicit.unwrap().block_spacing, BlockSpacing::BlankLine);
    let preserve = profile(r#"{"formatter":{"blockSpacing":"preserve"}}"#).unwrap();
    assert_eq!(preserve.unwrap().block_spacing, BlockSpacing::Preserve);
    assert!(profile(r#"{"formatter":{"blockSpacing":"blank"}}"#).is_err());
}

#[test]
fn preserved_block_spacing_keeps_crlf_and_prologue_separators() {
    let options = FormatOptions {
        end_of_line: EndOfLine::Crlf,
        ..FormatOptions::from_preset(FormatPreset::Prettier)
    };
    assert_fixed_point(
        &INPUT.replace('\n', "\r\n"),
        &PRETTIER.replace('\n', "\r\n"),
        &options,
    );

    let options = FormatOptions::from_preset(FormatPreset::Prettier);
    let prologue = "<!-- a -->\n\n\n<!-- b -->\n<template>\n  <p />\n</template>\n<!-- c -->\n";
    assert_fixed_point(
        prologue,
        "<!-- a -->\n\n<!-- b -->\n<template>\n  <p />\n</template>\n<!-- c -->\n",
        &options,
    );
}

#[test]
fn a_block_sorted_off_the_source_start_takes_a_blank_line() {
    // Only sorting can move the first source block after another one; it has
    // no authored separator, so it takes the blank line.
    let options = FormatOptions {
        sort_blocks: true,
        ..FormatOptions::from_preset(FormatPreset::Prettier)
    };
    let source = "<template>\n  <p />\n</template>\n<script setup>\nconst a = 1;\n</script>\n";
    assert_fixed_point(
        source,
        "<script setup>\nconst a = 1;\n</script>\n\n<template>\n  <p />\n</template>\n",
        &options,
    );
}
