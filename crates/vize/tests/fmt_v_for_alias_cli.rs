//! Actual CLI writes preserve complete v-for bindings and independent Vue observations.
#![cfg(feature = "glyph")]
use std::{error::Error, fs, path::Path, process::Command};

const OBSERVE: &str = r#"
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
const { observeSource, providerIdentities } = await import(pathToFileURL(process.argv[1]).href);
const before = await observeSource(readFileSync(process.argv[2], 'utf8'), 'original', [false, true]);
const after = await observeSource(readFileSync(process.argv[3], 'utf8'), 'formatted', [false, true]);
process.stdout.write(JSON.stringify({ before, after, providers: providerIdentities }) + '\n');
"#;

#[test]
fn complete_aliases_survive_five_cli_processes_and_stock_dom_ssr_per_case()
-> Result<(), Box<dyn Error>> {
    // Alias interiors are authored byte oracles. Only the outer parameter
    // separator gains a space; the independent stock Vue renderer supplies
    // complete before/after DOM, SSR, runtime text and call observations.
    let cases = [
        (
            "string-default",
            "({label = 'a,b'},index) in [{}]",
            "({label = 'a,b'}, index) in [{}]",
            "{{ label }}:{{ index }}",
            "a,b:0",
            false,
        ),
        (
            "array-default",
            "({items = ['a,b','c,d']},index) in [{}]",
            "({items = ['a,b','c,d']}, index) in [{}]",
            "{{ items[0] }}|{{ items[1] }}:{{ index }}",
            "a,b|c,d:0",
            false,
        ),
        (
            "object-default",
            "({entry = {left: 'a,b', right: 'c,d'}},index) in [{}]",
            "({entry = {left: 'a,b', right: 'c,d'}}, index) in [{}]",
            "{{ entry.left }}|{{ entry.right }}:{{ index }}",
            "a,b|c,d:0",
            false,
        ),
        (
            "nested-destructuring",
            "({entry: {label = 'a,b'} = {}},index) in [{}]",
            "({entry: {label = 'a,b'} = {}}, index) in [{}]",
            "{{ label }}:{{ index }}",
            "a,b:0",
            false,
        ),
        (
            "array-pattern",
            "([label = 'a,b'],index) in [[]]",
            "([label = 'a,b'], index) in [[]]",
            "{{ label }}:{{ index }}",
            "a,b:0",
            false,
        ),
        (
            "nested-call-default",
            "({label = ((left,right) => left + right)('a,b','c,d')},index) in [{}]",
            // The whole head exceeds the retained provider's existing syntax
            // budget. Preserve every authored byte on that explicit refusal.
            "({label = ((left,right) => left + right)('a,b','c,d')},index) in [{}]",
            "{{ label }}:{{ index }}",
            "a,bc,d:0",
            false,
        ),
        (
            "template-default",
            "({label = `a,b`},index) of [{}]",
            "({label = `a,b`}, index) of [{}]",
            "{{ label }}:{{ index }}",
            "a,b:0",
            false,
        ),
        (
            "regex-default",
            "({label = /a,b/.source},index) in [{}]",
            "({label = /a,b/.source}, index) in [{}]",
            "{{ label }}:{{ index }}",
            "a,b:0",
            false,
        ),
        (
            "plain-spacing",
            "(item,index) in ['a,b']",
            "(item, index) in ['a,b']",
            "{{ item }}:{{ index }}",
            "a,b:0",
            false,
        ),
        (
            "profile-parentheses",
            "{label = 'a,b'},index in [{}]",
            "({label = 'a,b'}, index) in [{}]",
            "{{ label }}:{{ index }}",
            "a,b:0",
            true,
        ),
    ];
    let mut commands = 0;
    for (id, aliases, normalized, content, text, parens) in cases {
        let project = tempfile::tempdir()?;
        let file = project.path().join("Example.vue");
        let original = project.path().join("Original.vue");
        let input = format!("<template><p v-for=\"{aliases}\">{content}</p></template>\n");
        let expected =
            format!("<template>\n  <p v-for=\"{normalized}\">{content}</p>\n</template>\n");
        fs::write(&file, &input)?;
        fs::write(&original, &input)?;
        for (pass, mode) in ["--check", "--write", "--write", "--write", "--check"]
            .into_iter()
            .enumerate()
        {
            let before = fs::read(&file)?;
            let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
            command.current_dir(project.path()).args([
                "fmt",
                mode,
                "--no-config",
                "--print-width",
                "500",
            ]);
            if parens {
                command.args(["--preset", "prettier"]);
            }
            let result = command.arg("Example.vue").output()?;
            let changed = pass < 2;
            let checked = mode == "--check";
            let detail = if changed {
                format!(
                    "{}: Example.vue\n",
                    if checked {
                        "Would reformat"
                    } else {
                        "Reformatted"
                    }
                )
            } else {
                String::new()
            };
            let action = if checked { "Checked" } else { "Formatted" };
            let summary = match (checked, changed) {
                (true, true) => "would be reformatted",
                (true, false) => "already formatted",
                (false, true) => "reformatted",
                (false, false) => "unchanged",
            };
            assert_eq!(
                result.status.code(),
                Some(i32::from(checked && changed)),
                "{id} pass{pass}"
            );
            assert_eq!(result.stdout, b"", "{id} pass{pass}");
            assert_eq!(
                result.stderr,
                format!("Found 1 file(s)\n{detail}\n{action} 1 file(s)\n  1 file(s) {summary}\n")
                    .as_bytes(),
                "{id} pass{pass}"
            );
            let after = fs::read(&file)?;
            if checked {
                assert_eq!(after, before, "{id}: check must not write");
            }
            if pass == 1 {
                assert_stock_semantics(id, &original, &file, text)?;
            }
            assert_eq!(
                after,
                if pass == 0 {
                    input.as_bytes()
                } else {
                    expected.as_bytes()
                },
                "{id} pass{pass}: whole bytes"
            );
            commands += 1;
        }
    }
    assert_eq!(commands, 50);
    Ok(())
}

fn assert_stock_semantics(
    id: &str,
    original: &Path,
    formatted: &Path,
    expected_text: &str,
) -> Result<(), Box<dyn Error>> {
    let helper = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/tooling/support/formatter-expression-width-semantics.mjs");
    let result = Command::new("node")
        .args(["--input-type=module", "-e", OBSERVE])
        .arg(helper)
        .arg(original)
        .arg(formatted)
        .output()?;
    if let Some(root) = std::env::var_os("VIZE_TEST_FIX_HISTORY_CAPTURE_DIR") {
        let evidence = Path::new(&root).join("glyph-v-for-alias").join(id);
        fs::create_dir_all(&evidence)?;
        fs::copy(original, evidence.join("Original.vue"))?;
        fs::copy(formatted, evidence.join("Formatted.vue"))?;
        fs::write(evidence.join("stock-stdout.json"), &result.stdout)?;
        fs::write(evidence.join("stock-stderr.txt"), &result.stderr)?;
    }
    assert_eq!(result.status.code(), Some(0), "{id}: {result:?}");
    assert_eq!(result.stderr, b"", "{id}: stock reference stderr");
    let packet: serde_json::Value = serde_json::from_slice(&result.stdout)?;
    let before = packet["before"]["observations"]
        .as_array()
        .ok_or("missing original observations")?;
    let after = packet["after"]["observations"]
        .as_array()
        .ok_or("missing formatted observations")?;
    assert_eq!(before.len(), 2, "{id}");
    assert_eq!(
        after, before,
        "{id}: complete stock DOM/SSR/runtime/calls changed"
    );
    for observation in before {
        assert_eq!(
            observation["beforeClick"]["textContent"], expected_text,
            "{id}: original value"
        );
        assert_eq!(
            observation["afterClick"]["textContent"], expected_text,
            "{id}: original value"
        );
    }
    Ok(())
}
