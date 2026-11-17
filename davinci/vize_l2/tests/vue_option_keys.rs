//! Native top-level option keys of a normal `<script>`'s options object
//! equal Croquis' options descriptor keys on the repository corpus (#6844).
//! Croquis is only this test's oracle.

#[expect(dead_code, reason = "this test reads the corpus, not setup facts")]
mod support {
    pub mod vue_options;
    pub mod vue_setup;
}

use support::vue_options::observe;
use support::vue_setup::{corpus, slice, text};
use vize_l0::{String, cstr};
use vize_l2::framework::vue::ComponentShape;

/// `(name, start, end)` of each option key, script-relative.
type KeyRow = (String, u32, u32);

#[test]
fn every_corpus_option_key_matches_the_croquis_descriptor() {
    let (root, files) = corpus();
    let (mut compared, mut keys) = (0usize, 0usize);
    let mut mismatches = Vec::new();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let Some((content, base, Ok(facts))) = observe(&source) else {
            continue;
        };
        let native: Option<Vec<KeyRow>> = facts
            .component()
            .filter(|component| component.shape == ComponentShape::Options)
            .map(|component| {
                component
                    .option_keys
                    .iter()
                    .map(|key| {
                        let name = slice(&content, base, key.name).unwrap_or_default();
                        (name, key.key.start - base, key.key.end - base)
                    })
                    .collect()
            });
        let oracle: Option<Vec<KeyRow>> =
            vize_croquis::script_parser::parse_script_with_options_and_jsx(
                &content,
                vize_croquis::script_parser::ScriptParserOptions {
                    options_api: true,
                    legacy_vue2: false,
                },
                false,
            )
            .into_croquis()
            .options_descriptor
            .map(|descriptor| {
                descriptor
                    .option_keys
                    .iter()
                    .map(|key| (String::from(key.name.as_str()), key.start, key.end))
                    .collect()
            });
        compared += usize::from(native.is_some());
        keys += native.as_ref().map_or(0, Vec::len);
        if native != oracle {
            let relative = file.strip_prefix(&root).unwrap_or(file).display();
            mismatches.push(cstr!(
                "{relative}\n  native:  {native:?}\n  croquis: {oracle:?}"
            ));
        }
    }
    eprintln!("compared {compared} options objects ({keys} option keys)");
    assert!(
        compared >= 80,
        "corpus shrank to {compared} options objects"
    );
    assert_eq!(mismatches, Vec::<String>::new());
}

#[test]
fn option_keys_list_every_static_key_verbatim() {
    let source = "<script lang=\"ts\">export default { name: 'X', 'compnents': {}, [k]: 1, ...rest, setup() {}, render() {} }</script>";
    let (content, base, facts) = observe(source).expect("observed");
    let facts = facts.expect("complete");
    let component = facts.component().expect("component");
    let keys = component
        .option_keys
        .iter()
        .map(|key| {
            (
                slice(&content, base, key.name).unwrap_or_default(),
                slice(&content, base, key.key).unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        keys,
        [
            (text("name"), text("name")),
            (text("compnents"), text("'compnents'")),
            (text("setup"), text("setup")),
            (text("render"), text("render")),
        ]
    );
}
