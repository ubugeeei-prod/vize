//! Fixture loading and Vize-side compilation for the `@vue/babel-plugin-jsx`
//! differential oracle (see `../babel_compat_oracle.rs`).

#![allow(dead_code)]

mod verdicts;

pub use verdicts::VERDICTS;

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use serde::Deserialize;
use vize_atelier_jsx::{
    BabelJsxOptions, JsxCompatMode, JsxCompileConfig, JsxLang, compile_jsx_with_babel_pragma,
};
use vize_carton::Bump;

/// The relationship between Vize's default output and babel's for one case.
///
/// This is a *semantic* verdict, not a byte comparison: Vize emits through
/// `vize_atelier_dom` (block tree, hoisting, lowered control flow) while babel
/// emits bare `createVNode` calls, so identical bytes are never the goal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Vize's Babel-compat output is semantically equivalent to babel's: it mounts
    /// to the same DOM and patches the same way, even where the emitted call shape
    /// differs.
    Equivalent,
    /// Vize's Babel-compat output still differs observably from babel's. The
    /// payload names the remaining difference.
    Divergent(&'static str),
    /// Vize does not implement this case at all yet. The payload names the
    /// reason and the tracking issue.
    Deferred(&'static str),
}

impl Verdict {
    /// The declared verdict for a corpus case id.
    ///
    /// Panics when the id has no entry: the verdict table and the corpus are
    /// asserted to be in exact correspondence by
    /// `babel_recording_is_complete`, so a missing entry means a corpus row was
    /// added without classifying it — which repo policy treats as a
    /// non-existent edge case.
    pub fn for_case(id: &str) -> Self {
        VERDICTS
            .iter()
            .find(|(case_id, _)| *case_id == id)
            .map(|(_, verdict)| *verdict)
            .unwrap_or_else(|| panic!("corpus case {id} has no verdict entry"))
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Equivalent => write!(f, "equivalent"),
            Self::Divergent(reason) => write!(f, "divergent: {reason}"),
            Self::Deferred(reason) => write!(f, "deferred: {reason}"),
        }
    }
}

/// The `fixtures/corpus.json` manifest.
#[derive(Deserialize)]
pub struct Corpus {
    pub oracle: OraclePin,
    pub cases: Vec<Case>,
}

/// The pinned oracle toolchain recorded in the corpus manifest.
#[derive(Deserialize)]
pub struct OraclePin {
    pub plugin: std::string::String,
    #[serde(rename = "pluginVersion")]
    pub plugin_version: std::string::String,
}

/// One corpus case: a JSX/TSX input plus the babel options it is compiled with.
#[derive(Deserialize)]
pub struct Case {
    pub id: std::string::String,
    pub category: std::string::String,
    pub lang: std::string::String,
    pub source: std::string::String,
    #[serde(rename = "babelOptions", default)]
    pub babel_options: BTreeMap<std::string::String, serde_json::Value>,
}

impl Case {
    /// The case's language as a Vize [`JsxLang`].
    pub fn jsx_lang(&self) -> JsxLang {
        match self.lang.as_str() {
            "jsx" => JsxLang::Jsx,
            "tsx" => JsxLang::Tsx,
            other => panic!("unknown corpus lang {other}"),
        }
    }

    /// The babel options for this case, rendered deterministically for the
    /// snapshot (`BTreeMap` keeps key order stable).
    pub fn babel_options_display(&self) -> std::string::String {
        if self.babel_options.is_empty() {
            return "(defaults)".into();
        }
        serde_json::to_string(&self.babel_options).expect("options serialize")
    }
}

/// The `fixtures/babel-output.json` recording.
#[derive(Deserialize)]
pub struct Recording {
    #[serde(rename = "pluginVersion")]
    pub plugin_version: std::string::String,
    pub cases: BTreeMap<std::string::String, RecordedOutput>,
}

/// Babel's recorded result for one case: emitted code, or the error it raised.
#[derive(Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum RecordedOutput {
    /// Babel transformed the input; payload is the emitted module.
    Ok { code: std::string::String },
    /// Babel rejected the input; payload is its error message.
    Error { message: std::string::String },
}

impl RecordedOutput {
    /// Snapshot rendering: the emitted code, or the rejection message.
    pub fn display(&self) -> std::string::String {
        match self {
            Self::Ok { code } => code.clone(),
            Self::Error { message } => format!("<error> {message}"),
        }
    }
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("babel_compat")
        .join("fixtures")
}

/// Load `fixtures/corpus.json`.
pub fn load_corpus() -> Corpus {
    let path = fixture_dir().join("corpus.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

/// Load `fixtures/babel-output.json`.
pub fn load_recording() -> Recording {
    let path = fixture_dir().join("babel-output.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|error| panic!("parse {}: {error}", path.display()))
}

/// Compile one corpus case with Vize's opt-in Babel compatibility mode and
/// render the result for the snapshot.
///
/// This uses [`compile_jsx`] + `module_code()` rather than the lower-level
/// per-component render code, because that is the module Vize's bundler
/// bindings actually emit — the same unit babel emits — so the two sides of the
/// snapshot are directly comparable.
///
/// Diagnostics are rendered rather than asserted away: a compat-mode oracle has
/// to show what Vize does with inputs babel rejects (and vice versa), so a
/// diagnosing case is a legitimate, recordable outcome.
pub fn vize_vdom_output(case: &Case) -> std::string::String {
    let bump = Bump::new();
    let out = compile_jsx_with_babel_pragma(
        &bump,
        &case.source,
        case.jsx_lang(),
        &JsxCompileConfig {
            compat: JsxCompatMode::Babel,
            ..Default::default()
        },
        &BabelJsxOptions {
            transform_on: case
                .babel_options
                .get("transformOn")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or_default(),
        },
        case.babel_options
            .get("pragma")
            .and_then(serde_json::Value::as_str),
    );

    let mut rendered = std::string::String::new();
    for diagnostic in &out.diagnostics {
        rendered.push_str(&format!(
            "<{:?}> {}\n",
            diagnostic.severity, diagnostic.message
        ));
    }
    let module = out.module_code();
    let module = module.trim_end();
    if module.is_empty() {
        rendered.push_str("(no render root found)");
    } else {
        rendered.push_str(module);
    }
    rendered
}
