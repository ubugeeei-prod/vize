//! Tests for template compilation utilities that are shared by SFC assembly.

use super::string_tracking::{
    StringTrackState, count_braces_outside_strings, count_braces_with_state,
    count_delims_with_state,
};
use super::vapor::{add_scope_id_to_template, transform_vapor_template_output};
use super::{TemplateBlockCompileContext, compile_template_block};
use crate::types::{BindingMetadata, BlockLocation, SfcTemplateBlock, TemplateCompileOptions};
use std::borrow::Cow;

fn template(content: &'static str) -> SfcTemplateBlock<'static> {
    SfcTemplateBlock {
        content: Cow::Borrowed(content),
        loc: BlockLocation {
            start: 0,
            end: 0,
            tag_start: 0,
            tag_end: 0,
            start_line: 1,
            start_column: 1,
            end_line: 1,
            end_column: 1,
        },
        lang: None,
        src: None,
        attrs: Default::default(),
    }
}

#[test]
fn test_add_scope_id_to_template() {
    let input = r#"const t0 = _template("<div class='container'>Hello</div>")"#;
    let result = add_scope_id_to_template(input, "data-v-abc123");
    insta::assert_snapshot!(result.as_str());
}

#[test]
fn test_transform_vapor_template_output_current_render_format() {
    let vapor_code = r#"import { template as _template } from 'vue/vapor';
const t0 = _template("<div> </div>", true)

export function render(_ctx) {
  const n0 = t0()
  return n0
}"#;

    let result =
        transform_vapor_template_output(vapor_code, None, &template("<div>{{ msg }}</div>"), None)
            .expect("current Vapor output should be transformed");

    insta::assert_snapshot!(result.as_str());
}

#[test]
fn count_braces_ignores_string_like_braces() {
    assert_eq!(count_braces_outside_strings("{ a: 1 }"), 0);
    assert_eq!(count_braces_outside_strings("{"), 1);
    assert_eq!(count_braces_outside_strings("}"), -1);
    assert_eq!(
        count_braces_outside_strings("_toDisplayString(isArray.value ? ']' : '}')"),
        0
    );
    assert_eq!(count_braces_outside_strings(r#"var x = "{";"#), 0);
    assert_eq!(count_braces_outside_strings("var x = `{`;"), 0);
    assert_eq!(count_braces_outside_strings(r"var x = '\'' + '}'"), 0);
}

#[test]
fn count_braces_tracks_multiline_template_expression_state() {
    let mut state = StringTrackState::default();

    let count1 = count_braces_with_state(
        r#"}, _toDisplayString(`${t("key")}: v${ver.major}.${"#,
        &mut state,
    );
    assert_eq!(count1, -1);
    assert!(
        !state.template_expr_brace_stack.is_empty(),
        "line 1 should leave the scanner inside a template expression"
    );

    let count2 = count_braces_with_state("            ver.minor", &mut state);
    assert_eq!(count2, 0);

    let count3 = count_braces_with_state(
        r##"          }`) + "\n      ", 1 /* TEXT */)))"##,
        &mut state,
    );
    assert_eq!(count3, 0);
    assert!(!state.in_string);
    assert!(state.template_expr_brace_stack.is_empty());
    assert_eq!(count1 + count2 + count3, -1);
}

#[test]
fn count_braces_tracks_nested_template_literals() {
    let cases = [
        r#"x = `outer ${`inner ${x}`} end`"#,
        r#"x = `${items.map(x => ({ name: x })).join()}`"#,
        r#"if (x) { var s = "}" + '{' }"#,
    ];

    for case in cases {
        let mut state = StringTrackState::default();
        let count = count_braces_with_state(case, &mut state);
        assert_eq!(count, 0, "case should be balanced: {case}");
        assert!(!state.in_string, "case should close strings: {case}");
    }
}

#[test]
fn count_braces_tracks_state_across_many_lines() {
    let mut state = StringTrackState::default();
    let mut total = 0;
    for line in [
        "function render() {",
        r#"  return _toDisplayString(`${fn({"#,
        "    key: val,",
        "    nested: {",
        "      deep: true",
        "    }",
        r#"  })}`)"#,
        "}",
    ] {
        total += count_braces_with_state(line, &mut state);
    }
    assert_eq!(total, 0);
    assert!(!state.in_string);
    assert!(state.template_expr_brace_stack.is_empty());
}

#[test]
fn count_delims_tracks_multiline_object_literal() {
    let mut state = StringTrackState::default();
    let mut depth = 0;
    depth += count_delims_with_state("const _hoisted_1 = { style: {", &mut state);
    assert_eq!(depth, 2);
    depth += count_delims_with_state("  position: 'absolute',", &mut state);
    assert_eq!(depth, 2);
    depth += count_delims_with_state("  content: '({[',", &mut state);
    assert_eq!(depth, 2, "delimiters inside strings must not affect depth");
    depth += count_delims_with_state("} }", &mut state);
    assert_eq!(depth, 0);
}

#[test]
fn dom_inline_parts_are_sliced_from_sections_for_template_matrix() {
    let templates = [
        ("<div>{{ msg }}</div>", "_toDisplayString"),
        (
            "<div><img style=\"position: absolute; top: 0\" alt=\"x\"></div>",
            "_createElementVNode",
        ),
        (
            "<MyWidget v-focus>{{ count + 1 }}</MyWidget>",
            "_withDirectives",
        ),
        (
            "<div v-if=\"shown\">a</div>\n<span v-else>b</span>",
            "_ctx.shown",
        ),
        ("<header>h</header>\n<footer>f</footer>", "_Fragment"),
        (
            "<ul><li v-for=\"item in items\" :key=\"item.id\">{{ item.name }}</li></ul>",
            "_renderList",
        ),
        ("<slot name=\"body\" :row=\"row\" />", "_renderSlot"),
        (
            "<input v-model=\"text\" @keyup.enter=\"submit($event)\">",
            "_vModelText",
        ),
        ("hello", "\"hello\""),
    ];

    for (source, expected_body_fragment) in templates {
        let bindings = BindingMetadata::default();
        let result = compile_template_block(
            &template(source),
            &TemplateCompileOptions::default(),
            TemplateBlockCompileContext {
                scope_id: "abc123",
                apply_scope_id: false,
                has_scoped: true,
                is_ts: false,
                inline: true,
                component_name: Some("TestComp"),
                bindings: Some(&bindings),
                croquis: None,
            },
            vize_atelier_core::TemplateSyntaxMode::Standard,
        )
        .expect("template should compile");

        assert!(
            result.sections.is_some(),
            "DOM output must record fine sections for template:\n{source}\n\n{}",
            result.code
        );
        let parts = result
            .body_parts_for_inline()
            .expect("sectioned DOM output should slice inline parts");

        assert_eq!(parts.render_fn_name, "render");
        assert!(
            parts.imports.contains("from 'vue'") || parts.imports.contains("from \"vue\""),
            "imports should come from the recorded imports section:\n{}",
            parts.imports
        );
        assert!(
            parts.render_body.contains(expected_body_fragment),
            "render body should contain `{expected_body_fragment}` for template:\n{source}\n\nbody:\n{}",
            parts.render_body
        );
    }
}
