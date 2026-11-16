//! The command-line check projection with its trivia collapsed (`noEmit`)
//! against the same projection whole, for every committed Vue fixture: oxc
//! reads the same tokens, line breaks and `@ts-…` directives from both, and
//! every span and directive says the same about them.
#![cfg(test)]

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_parser::config::TokensParserConfig;
use oxc_span::SourceType;
use serde_json::{Value, json};
use vize_l0::cstr;

fn vue_fixtures(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            // Submodule checkouts and installed packages are not fixtures.
            if !path.ends_with("_git") && !path.ends_with("node_modules") {
                vue_fixtures(&path, found);
            }
        } else if path.extension().is_some_and(|extension| extension == "vue") {
            found.push(path);
        }
    }
}

fn frame(input: &mut Vec<u8>, value: &Value) {
    let body = serde_json::to_vec(value).unwrap();
    input.extend_from_slice(cstr!("Content-Length: {}\r\n\r\n", body.len()).as_bytes());
    input.extend_from_slice(&body);
}

/// Every reply of one mapper process, by request id.
fn exchange(input: Vec<u8>) -> BTreeMap<u64, Value> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_vize"))
        .arg("content-mapper")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let writer = std::thread::spawn(move || stdin.write_all(&input).unwrap());
    let mut output = Vec::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_end(&mut output)
        .unwrap();
    writer.join().unwrap();
    assert!(child.wait().unwrap().success());
    let mut replies = BTreeMap::new();
    let mut rest = output.as_slice();
    while !rest.is_empty() {
        let header = rest.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
        let length: usize = std::str::from_utf8(rest.get(..header).unwrap_or_default())
            .unwrap()
            .strip_prefix("Content-Length: ")
            .unwrap()
            .parse()
            .unwrap();
        let reply: Value = serde_json::from_slice(&rest[header + 4..][..length]).unwrap();
        replies.insert(reply["id"].as_u64().unwrap(), reply);
        rest = &rest[header + 4 + length..];
    }
    replies
}

/// Each token's text, whether a line break precedes it, and the `@ts-…`
/// directive comment before it, if any.
fn tokens<'a>(text: &'a str, extension: &str) -> Vec<(&'a str, bool, Option<&'static str>)> {
    let allocator = Allocator::default();
    let source_type = match extension {
        ".tsx" => SourceType::tsx(),
        ".jsx" => SourceType::jsx(),
        ".js" => SourceType::mjs(),
        _ => SourceType::ts(),
    };
    let parsed = Parser::new(&allocator, text, source_type)
        .with_config(TokensParserConfig)
        .parse();
    let mut comments = parsed.program.comments.iter().peekable();
    parsed
        .tokens
        .iter()
        .map(|token| {
            let mut directive = None;
            while let Some(comment) = comments.next_if(|c| c.span.end <= token.start()) {
                let body = text.get(comment.span.start as usize..comment.span.end as usize);
                let body = body.unwrap_or_default();
                let body = body.trim_start_matches(['/', '*']).trim_start();
                for name in ["@ts-ignore", "@ts-expect-error", "@ts-nocheck"] {
                    if body.starts_with(name) {
                        directive = Some(name);
                    }
                }
            }
            let token_text = text.get(token.start() as usize..token.end() as usize);
            let token_text = token_text.unwrap_or_default();
            (token_text, token.is_on_new_line(), directive)
        })
        .collect()
}

fn directives(result: &Value) -> Vec<[&Value; 4]> {
    array(&result["diagnosticDirectives"]["directives"])
        .iter()
        .map(|d| [&d[0], &d[1], &d[4], &d[5]])
        .collect()
}

fn array(value: &Value) -> &[Value] {
    value.as_array().map_or(&[], Vec::as_slice)
}

#[test]
fn collapsed_trivia_leaves_every_fixture_projection_the_same_to_typescript() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures");
    let mut fixtures = Vec::new();
    vue_fixtures(&root, &mut fixtures);
    fixtures.sort();
    assert!(fixtures.len() > 300, "{} fixtures", fixtures.len());

    let directory = tempfile::tempdir().unwrap();
    let config = directory.path().join("tsconfig.json");
    std::fs::write(&config, "{}").unwrap();
    let mut input = Vec::new();
    frame(
        &mut input,
        &json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize",
            "params": { "positionEncodings": ["utf-8"] } }),
    );
    for (id, handle, no_emit) in [(2, "whole", false), (3, "compact", true)] {
        let mut options = json!({ "configFilePath": config, "project": config, "strict": true });
        if no_emit {
            options["noEmit"] = json!(true);
        }
        frame(
            &mut input,
            &json!({ "jsonrpc": "2.0", "id": id, "method": "openProject", "params": {
                "configFileName": config, "projectHandle": handle,
                "options": null, "compilerOptions": options } }),
        );
    }
    for (index, path) in fixtures.iter().enumerate() {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        for (offset, handle) in [(0, "whole"), (1, "compact")] {
            frame(
                &mut input,
                &json!({ "jsonrpc": "2.0", "id": 10 + 2 * index + offset, "method": "transform",
                    "params": { "fileName": path, "content": content, "projectHandle": handle } }),
            );
        }
    }
    let replies = exchange(input);

    let mut compacted = 0;
    for (index, path) in fixtures.iter().enumerate() {
        let id = 10 + 2 * index as u64;
        let (whole, compact) = (&replies[&id]["result"], &replies[&(id + 1)]["result"]);
        let (Some(before), Some(after)) = (whole["text"].as_str(), compact["text"].as_str()) else {
            assert_eq!(whole, compact, "{}", path.display());
            continue;
        };
        if before == after {
            continue;
        }
        compacted += 1;
        let extension = whole["extension"].as_str().unwrap_or(".ts");
        assert_eq!(
            tokens(after, extension),
            tokens(before, extension),
            "{}:\n{after}",
            path.display()
        );
        assert_eq!(
            whole["diagnostics"],
            compact["diagnostics"],
            "{}",
            path.display()
        );
        let (spans, moved) = (array(&whole["mappings"]), array(&compact["mappings"]));
        assert_eq!(spans.len(), moved.len(), "{}", path.display());
        for (span, moved) in spans.iter().zip(moved) {
            let field = |span: &Value, at: usize| span[at].as_u64().unwrap() as usize;
            let (start, length, moved_start) = (field(span, 0), field(span, 1), field(moved, 0));
            assert_eq!(array(span)[1..], array(moved)[1..], "{}", path.display());
            assert_eq!(
                before.get(start..start + length),
                after.get(moved_start..moved_start + length),
                "{} {span}",
                path.display()
            );
        }
        // Ranges move with the text; the rest of each directive stays.
        assert_eq!(directives(whole), directives(compact), "{}", path.display());
    }
    assert!(
        compacted > fixtures.len() / 2,
        "{compacted} of {} fixtures compacted",
        fixtures.len()
    );
}
