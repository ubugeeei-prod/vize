//! n8n's original multiline prop and independently authored JS boundary laws.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::expect_used,
    clippy::panic,
    reason = "whole-source, whole-module and executable compiler regression oracles"
)]

use sha2::{Digest, Sha256};
use std::{
    io::Write,
    process::{Command, Stdio},
};
use vize_atelier_dom::{DomCompilerOptions, compile_template_legacy_with_options};
use vize_l0::Allocator;
use vize_l1_to_l2::{DomEmitOptions, LegacyCaps, emit_dom_source_with_options};

const ORIGINAL: &str = include_str!(
    "../../../tests/_fixtures/differential/compiler/n8n-prop-comments/RunDataJsonActions.vue.txt"
);

fn legacy(source: &str, is_ts: bool) -> String {
    let allocator = Allocator::new();
    let (_, errors, output) = compile_template_legacy_with_options(
        &allocator,
        source,
        DomCompilerOptions {
            is_ts,
            ..Default::default()
        },
    );
    assert!(errors.is_empty(), "{errors:?}");
    format!("{}\n{}", output.preamble, output.code)
}

fn current(source: &str, is_ts: bool) -> String {
    let allocator = Allocator::new();
    emit_dom_source_with_options(
        &allocator,
        source,
        LegacyCaps::VUE3,
        &DomEmitOptions {
            is_ts,
            ..DomEmitOptions::DEFAULT
        },
    )
    .expect("complete authored module emits")
    .assembled()
    .to_string()
}

#[test]
fn complete_original_n8n_template_keeps_the_legacy_module_bytes() {
    assert_eq!(
        Sha256::digest(ORIGINAL.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        "168e35ff13edb4dbce5d0a9e2ee12a5ff55baf830c00ffa7c270c8a1601c9b9b"
    );
    let parsed = vize_atelier_sfc::parse_sfc(ORIGINAL, Default::default())
        .expect("complete unchanged original n8n SFC");
    let template = parsed.template.as_ref().expect("original template");
    for is_ts in [false, true] {
        assert_eq!(
            current(template.content.as_ref(), is_ts),
            legacy(template.content.as_ref(), is_ts),
            "complete original: is_ts={is_ts}"
        );
    }
}

fn execute_props(module: &str) {
    let mut child = Command::new("node")
        .arg("-e")
        .arg(
            r#"
const assert = require('node:assert/strict');
let code = '';
process.stdin.setEncoding('utf8');
process.stdin.on('data', chunk => code += chunk);
process.stdin.on('end', () => {
  const vnode = (_type, props) => props;
  const Vue = { openBlock() {}, resolveComponent: name => name,
    createBlock: vnode, createVNode: vnode, createElementBlock: vnode,
    createElementVNode: vnode };
  const render = new Function('Vue', 'isInPopOutWindow', code + '\nreturn render;');
  for (const value of [false, true]) {
    const output = render(Vue, value)({}, [], {}, {}, {}, {});
    assert.deepEqual(output, { teleported: !value, after: 42 });
  }
  process.stdout.write('ok\n');
});
"#,
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node executes the complete emitted module");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(module.as_bytes())
        .expect("module bytes");
    let output = child.wait_with_output().expect("Node exits");
    assert!(
        output.status.success(),
        "status={}\nstdout={}\nstderr={}\n{module}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.stdout, b"ok\n");
}

#[test]
fn authored_prop_comments_keep_newline_bytes_and_safe_single_line_values() {
    for (name, expression) in [
        ("lf", "\n  !isInPopOutWindow // note\n  "),
        ("crlf", "\r\n\t!isInPopOutWindow // note\r\n\t"),
        ("cr", "\r\t!isInPopOutWindow // note\r\t"),
        (
            "block-delimiter-in-line-comment",
            "\n !isInPopOutWindow // */\n ",
        ),
        ("single-line", "!isInPopOutWindow // note"),
        ("single-line-with-spaces", "!isInPopOutWindow // note  "),
        (
            "quoted-slashes",
            "!isInPopOutWindow && 'https://x//y' === 'https://x//y'",
        ),
        ("regex-slashes", r"!isInPopOutWindow && /\/\//.test('//')"),
        ("existing-block", "!isInPopOutWindow /* // untouched */"),
        ("lf-block", "\n  !isInPopOutWindow /* note */\n  "),
        ("crlf-block", "\r\n\t!isInPopOutWindow /* note */\r\n\t"),
        ("lf-internal-line", "\n (!isInPopOutWindow // note\n )\n "),
        (
            "crlf-internal-line",
            "\r\n\t(!isInPopOutWindow // note\r\n )\r\n\t",
        ),
        (
            "lf-quoted-slashes",
            "\n !isInPopOutWindow && 'https://x//y' === 'https://x//y'\n ",
        ),
        (
            "crlf-regex-slashes",
            "\r\n !isInPopOutWindow && /\\/\\//.test('//')\r\n ",
        ),
    ] {
        let source = format!("<Probe :teleported=\"{expression}\" :after=\"42\" />");
        for is_ts in [false, true] {
            let current = current(&source, is_ts);
            assert_eq!(current, legacy(&source, is_ts), "{name}: is_ts={is_ts}");
            execute_props(&current);
        }
    }
}
