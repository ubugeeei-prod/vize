#![cfg(test)]
#![expect(
    clippy::disallowed_types,
    reason = "whole CLI fixtures use std strings"
)]

#[path = "support/vite_config.rs"]
mod vite_config;
use std::{fs, path::Path, process::Command};
use vize_atelier_core::parser::with_vue2_preserve_whitespace;
use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};

const SOURCE: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/whitespace-vue2-preserve-8259/App.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/whitespace-vue2-preserve-8259/vize.config.json.txt"
);
const VUE27_RENDER: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/whitespace-vue2-preserve-8259/vue27-preserve-render.txt"
);

#[test]
fn configured_vue2_preserve_matches_vue_2_7_text_runs() {
    let built = build(false);
    // Every interpolated text run, in order, as Vue 2.7.16 `preserve` emits it
    // (`_vm._s(_vm.x)` spelled `_toDisplayString(x)` in Vue 3 codegen).
    assert_eq!(interpolated_runs(&built), vue2_interpolated_runs());
    assert_eq!(built, compiler_code(false, false));
}

#[test]
fn configured_vue2_preserve_reaches_ssr_and_vapor() {
    for (ssr, vapor) in [(true, false), (false, true)] {
        let built = build_with(ssr, vapor);
        assert_eq!(built, compiler_code(ssr, vapor), "ssr={ssr}, vapor={vapor}");
    }
}

#[test]
fn vue2_preserve_keeps_removed_comment_and_named_slot_boundaries() {
    let comment_in_text = compiler_code_for(
        "<template><p>\n  {{ a }}\n  <!-- explain -->\n  <b>x</b>\n</p></template>",
        false,
        false,
    );
    assert_eq!(
        interpolated_runs(&comment_in_text),
        [r#""\n  " + _toDisplayString(_ctx.a) + "\n  ""#.to_owned()],
        "{comment_in_text}",
    );
    let comment_first = compiler_code_for(
        "<template><span>\n  <!-- keep -->\n  {{ kind }}</span></template>",
        false,
        false,
    );
    assert_eq!(
        interpolated_runs(&comment_first),
        [r#""\n  " + _toDisplayString(_ctx.kind)"#.to_owned()],
        "{comment_first}",
    );
    assert_eq!(
        comment_first.matches(r#"\n  \n  "#).count(),
        0,
        "{comment_first}",
    );

    let slot_tail = compiler_code_for(
        "<template><MyCard>\n  <span>body</span>\n  <template #footer>f</template>\n</MyCard></template>",
        false,
        false,
    );
    let default = slot_tail
        .split("default: _withCtx(() => [")
        .nth(1)
        .and_then(|tail| tail.split("]),").next())
        .expect("default slot body");
    assert_eq!(
        default.matches(r#"_createTextVNode(" ")"#).count(),
        0,
        "{slot_tail}",
    );
}

#[test]
fn vue2_preserve_matches_vue2_comment_and_named_slot_edges() {
    let between = compiler_code_for(
        "<template><p>\n  <b>a</b>\n  <!-- c -->\n  <i>b</i>\n</p></template>",
        false,
        false,
    );
    assert_eq!(text_vnode_arguments(&between), [""]);

    let trailing = compiler_code_for(
        "<template><div>\n  <b>x</b>\n  <!-- c -->\n</div></template>",
        false,
        false,
    );
    assert_eq!(text_vnode_arguments(&trailing), [] as [&str; 0]);

    let named_slot = compiler_code_for(
        "<template><MyCard>\n  {{ a }}\n  <template #footer>f</template>\n</MyCard></template>",
        false,
        false,
    );
    assert_eq!(
        interpolated_runs(&named_slot),
        [r#""\n  " + _toDisplayString(_ctx.a) + "\n  ""#.to_owned()]
    );
}

fn vue2_interpolated_runs() -> Vec<String> {
    VUE27_RENDER
        .split("_vm._v(")
        .skip(1)
        .filter_map(|tail| {
            let run = tail.get(..closing_paren(tail)?)?;
            run.contains("_vm._s(")
                .then(|| run.replace("_vm._s(_vm.", "_toDisplayString("))
        })
        .collect()
}

const DISPLAY: &str = "_toDisplayString(";

/// `"lead" + _toDisplayString(x) + "trail"` spans; bare `_toDisplayString(x)`
/// when no literal is joined to it.
fn interpolated_runs(code: &str) -> Vec<String> {
    let mut runs = Vec::new();
    let mut from = 0;
    while let Some(start) = code
        .get(from..)
        .and_then(|rest| rest.find(DISPLAY))
        .map(|offset| from + offset)
    {
        let args = start + DISPLAY.len();
        let Some(call_end) = code
            .get(args..)
            .and_then(closing_paren)
            .map(|close| args + close + 1)
        else {
            break;
        };
        let begin = code
            .get(..start)
            .and_then(|head| head.strip_suffix("\" + "))
            .and_then(|head| head.rfind('"'))
            .unwrap_or(start);
        let end = code
            .get(call_end..)
            .and_then(|tail| tail.strip_prefix(" + \""))
            .and_then(|tail| tail.find('"'))
            .map_or(call_end, |quote| call_end + " + \"".len() + quote + 1);
        runs.push(code.get(begin..end).unwrap_or_default().to_owned());
        from = end;
    }
    runs
}

fn text_vnode_arguments(code: &str) -> Vec<&str> {
    const CALL: &str = "_createTextVNode(";
    let mut arguments = Vec::new();
    let mut from = 0;
    while let Some(start) = code
        .get(from..)
        .and_then(|rest| rest.find(CALL))
        .map(|offset| from + offset + CALL.len())
    {
        let Some(end) = code
            .get(start..)
            .and_then(closing_paren)
            .map(|offset| start + offset)
        else {
            break;
        };
        arguments.push(code.get(start..end).unwrap_or_default());
        from = end + 1;
    }
    arguments
}

fn closing_paren(text: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, char) in text.char_indices() {
        match char {
            '(' => depth += 1,
            ')' if depth == 0 => return Some(index),
            ')' => depth -= 1,
            _ => {}
        }
    }
    None
}

fn build(ssr: bool) -> String {
    build_with(ssr, false)
}

fn build_with(ssr: bool, vapor: bool) -> String {
    let project = tempfile::tempdir().unwrap();
    fs::write(project.path().join("App.vue"), SOURCE).unwrap();
    fs::write(
        project.path().join("vite.config.mjs"),
        vite_config::source(CONFIG),
    )
    .unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(project.path())
        .env("RAYON_NUM_THREADS", "1")
        .args([
            "build",
            "--slow-threshold",
            "600000",
            "-o",
            "out",
            "-f",
            "js",
            "App.vue",
        ]);
    if ssr {
        command.arg("--ssr");
    }
    if vapor {
        command.arg("--vapor");
    }
    let output = command.output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read_to_string(Path::new(project.path()).join("out/App.js")).unwrap()
}

fn compiler_code(ssr: bool, vapor: bool) -> String {
    compiler_code_for(SOURCE, ssr, vapor)
}

fn compiler_code_for(source: &str, ssr: bool, vapor: bool) -> String {
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "App.vue".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let mut options = SfcCompileOptions::default();
    options.parse.filename = "App.vue".into();
    options.script.id = Some("App.vue".into());
    options.template.id = Some("App.vue".into());
    options.template.ssr = ssr;
    options.style.id = "App.vue".into();
    options.vapor = vapor;
    let result = with_vue2_preserve_whitespace(|| compile_sfc(&descriptor, options)).unwrap();
    assert!(result.errors.is_empty());
    result.code.as_str().into()
}
