use super::{FormatOptionsNapi, apply_options, format_sfc_napi};
use vize_glyph::{ArrowParens, AttributeSortOrder, BlockSpacing, QuoteProps, TrailingComma};

#[test]
fn native_sfc_sorting_uses_the_authored_full_reference() {
    let source = include_str!("../../../../vize_glyph/tests/fixtures/sort-imports/UserCard.vue");
    let expected =
        include_str!("../../../../vize_glyph/tests/fixtures/sort-imports/UserCard.sorted.vue");
    let output = format_sfc_napi(
        source.into(),
        Some(FormatOptionsNapi {
            sort_imports: Some(serde_json::json!({})),
            ..Default::default()
        }),
    )
    .expect("native formatting");
    assert_eq!(output.code, expected);
    assert!(output.changed);
}

#[test]
fn native_normalization_options_override_presets_in_both_directions() {
    let enabled = apply_options(FormatOptionsNapi {
        normalize_css_quotes: Some(true),
        normalize_inline_styles: Some(true),
        normalize_v_for_parens: Some(true),
        self_close_void_elements: Some(true),
        reindent_preprocessor_styles: Some(true),
        ..Default::default()
    })
    .expect("native options");
    assert!(enabled.normalize_css_quotes);
    assert!(enabled.normalize_inline_styles);
    assert!(enabled.normalize_v_for_parens);
    assert!(enabled.self_close_void_elements);
    assert!(enabled.reindent_preprocessor_styles);

    let disabled = apply_options(FormatOptionsNapi {
        preset: Some("prettier".into()),
        normalize_css_quotes: Some(false),
        normalize_inline_styles: Some(false),
        normalize_v_for_parens: Some(false),
        self_close_void_elements: Some(false),
        reindent_preprocessor_styles: Some(false),
        ..Default::default()
    })
    .expect("native preset override");
    assert!(!disabled.normalize_css_quotes);
    assert!(!disabled.normalize_inline_styles);
    assert!(!disabled.normalize_v_for_parens);
    assert!(!disabled.self_close_void_elements);
    assert!(!disabled.reindent_preprocessor_styles);
}

#[test]
fn native_sfc_body_indentation_is_an_explicit_opt_in() {
    let defaults = apply_options(FormatOptionsNapi::default()).expect("native defaults");
    assert!(!defaults.vue_indent_script_and_style);

    let indented = apply_options(FormatOptionsNapi {
        preset: Some("vize".into()),
        vue_indent_script_and_style: Some(true),
        ..Default::default()
    })
    .expect("native indentation option");
    assert!(indented.vue_indent_script_and_style);
}

#[test]
fn native_attribute_order_is_typed_and_overrides_presets() {
    let options = apply_options(FormatOptionsNapi {
        preset: Some("vize".into()),
        attribute_sort_order: Some("as-written".into()),
        ..Default::default()
    })
    .expect("native attribute order option");
    assert_eq!(options.attribute_sort_order, AttributeSortOrder::AsWritten);

    let error = apply_options(FormatOptionsNapi {
        attribute_sort_order: Some("source".into()),
        ..Default::default()
    })
    .expect_err("unknown attribute order policy");
    assert_eq!(error.status, napi::Status::InvalidArg);
    assert_eq!(
        error.reason,
        "Unknown attributeSortOrder; expected 'alphabetical' or 'as-written'"
    );
}

#[test]
fn native_script_quote_options_are_typed_and_independent() {
    let options = apply_options(FormatOptionsNapi {
        single_quote: Some(false),
        jsx_single_quote: Some(true),
        quote_props: Some("preserve".into()),
        ..Default::default()
    })
    .expect("native script quote options");
    assert!(!options.single_quote);
    assert!(options.jsx_single_quote);
    assert_eq!(options.quote_props, QuoteProps::Preserve);

    let error = apply_options(FormatOptionsNapi {
        quote_props: Some("keep".into()),
        ..Default::default()
    })
    .expect_err("unknown quote policy");
    assert_eq!(error.status, napi::Status::InvalidArg);
    assert_eq!(
        error.reason,
        "Unknown quoteProps; expected 'as-needed', 'consistent', or 'preserve'"
    );
}

#[test]
fn native_trailing_comma_option_is_typed_and_overrides_presets() {
    let options = apply_options(FormatOptionsNapi {
        preset: Some("prettier".into()),
        trailing_comma: Some("none".into()),
        ..Default::default()
    })
    .expect("native trailing comma option");
    assert_eq!(options.trailing_comma, TrailingComma::None);

    let error = apply_options(FormatOptionsNapi {
        trailing_comma: Some("never".into()),
        ..Default::default()
    })
    .expect_err("unknown trailing comma policy");
    assert_eq!(error.status, napi::Status::InvalidArg);
    assert_eq!(
        error.reason,
        "Unknown trailingComma; expected 'all', 'es5', or 'none'"
    );
}

#[test]
fn native_arrow_parens_option_is_typed_and_overrides_presets() {
    let options = apply_options(FormatOptionsNapi {
        preset: Some("vize".into()),
        arrow_parens: Some("avoid".into()),
        ..Default::default()
    })
    .expect("native arrow parentheses option");
    assert_eq!(options.arrow_parens, ArrowParens::Avoid);

    let error = apply_options(FormatOptionsNapi {
        arrow_parens: Some("preserve".into()),
        ..Default::default()
    })
    .expect_err("unknown arrow parentheses policy");
    assert_eq!(error.status, napi::Status::InvalidArg);
    assert_eq!(
        error.reason,
        "Unknown arrowParens; expected 'always' or 'avoid'"
    );
}

#[test]
fn native_bracket_layout_options_override_presets() {
    let options = apply_options(FormatOptionsNapi {
        preset: Some("compact".into()),
        bracket_spacing: Some(false),
        bracket_same_line: Some(false),
        ..Default::default()
    })
    .expect("native bracket layout options");
    assert!(!options.bracket_spacing);
    assert!(!options.bracket_same_line);
}

#[test]
fn native_block_order_option_overrides_presets() {
    let preserve = apply_options(FormatOptionsNapi {
        preset: Some("vize".into()),
        sort_blocks: Some(false),
        ..Default::default()
    })
    .expect("native authored block order");
    assert!(!preserve.sort_blocks);

    let canonical = apply_options(FormatOptionsNapi {
        preset: Some("compact".into()),
        sort_blocks: Some(true),
        ..Default::default()
    })
    .expect("native canonical block order");
    assert!(canonical.sort_blocks);
}

#[test]
fn native_block_spacing_option_overrides_presets() {
    let prettier = apply_options(FormatOptionsNapi {
        preset: Some("prettier".into()),
        ..Default::default()
    })
    .expect("native prettier preset");
    assert_eq!(prettier.block_spacing, BlockSpacing::Preserve);

    let blank_line = apply_options(FormatOptionsNapi {
        preset: Some("prettier".into()),
        block_spacing: Some("blank-line".into()),
        ..Default::default()
    })
    .expect("native blank-line block spacing");
    assert_eq!(blank_line.block_spacing, BlockSpacing::BlankLine);

    let preserve = apply_options(FormatOptionsNapi {
        block_spacing: Some("preserve".into()),
        ..Default::default()
    })
    .expect("native preserved block spacing");
    assert_eq!(preserve.block_spacing, BlockSpacing::Preserve);

    assert!(
        apply_options(FormatOptionsNapi {
            block_spacing: Some("blank".into()),
            ..Default::default()
        })
        .is_err()
    );
}
