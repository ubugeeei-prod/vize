//! `<style lang="scss|sass|less|stylus">` for `vize build` (#8905–#8907).
//!
//! Vue scopes a preprocessor block after the project's own preprocessor ran.
//! The CLI does the same: one Node process per build compiles every such
//! block with the sass, less or stylus package resolved from the SFC's
//! directory, and the compile then treats the block as plain CSS.

use std::{
    borrow::Cow,
    io::Write as _,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use vize_atelier_sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};
use vize_l0::{FxHashMap, String, cstr};

const PREPROCESSOR_LANGS: [&str; 5] = ["scss", "sass", "less", "stylus", "styl"];

/// One block's preprocessing result.
pub(super) enum StyleOutcome {
    Css(String),
    /// A preprocessor error, already positioned in the `.vue` file.
    Failed(String),
    /// No Node or no installed preprocessor; the block stays as written.
    Unavailable(String),
}

/// Results by SFC path and style block index.
#[derive(Default)]
pub(super) struct PreprocessedStyles {
    by_file: FxHashMap<PathBuf, FxHashMap<usize, StyleOutcome>>,
}

#[derive(Serialize)]
struct Job<'a> {
    file: &'a Path,
    lang: &'a str,
    source: &'a str,
}

#[derive(Deserialize)]
struct JobResult {
    css: Option<String>,
    missing: Option<String>,
    error: Option<JobError>,
}

#[derive(Deserialize)]
struct JobError {
    message: String,
    line: Option<usize>,
    column: Option<usize>,
}

struct Pending {
    file: PathBuf,
    /// Node resolves the preprocessor package from here.
    absolute: PathBuf,
    index: usize,
    lang: String,
    source: String,
    /// 1-based line of the block's first content character.
    first_line: usize,
}

impl PreprocessedStyles {
    /// Preprocess every block of `files` that needs it, with one Node call.
    pub(super) fn run<'p>(files: impl Iterator<Item = &'p PathBuf>) -> Self {
        let files: Vec<&PathBuf> = files.collect();
        let pending: Vec<Pending> = files
            .par_iter()
            .flat_map_iter(|path| pending_blocks(path))
            .collect();
        let mut preprocessed = Self::default();
        if pending.is_empty() {
            return preprocessed;
        }
        let results = run_node(&pending);
        for (index, block) in pending.into_iter().enumerate() {
            let outcome = match results.as_ref().map(|results| results.get(index)) {
                Ok(Some(result)) => outcome(&block, result),
                Ok(None) => StyleOutcome::Unavailable(cstr!(
                    "<style lang=\"{}\"> was not preprocessed: no result from Node",
                    block.lang
                )),
                Err(reason) => StyleOutcome::Unavailable(cstr!(
                    "<style lang=\"{}\"> was not preprocessed: {reason}",
                    block.lang
                )),
            };
            preprocessed
                .by_file
                .entry(block.file)
                .or_default()
                .insert(block.index, outcome);
        }
        preprocessed
    }

    /// Swap preprocessed CSS into `descriptor`; returns warnings, or the first error.
    pub(super) fn apply(
        &self,
        path: &Path,
        descriptor: &mut SfcDescriptor<'_>,
    ) -> Result<Vec<String>, String> {
        let Some(outcomes) = self.by_file.get(path) else {
            return Ok(Vec::new());
        };
        let mut warnings = Vec::new();
        for (index, style) in descriptor.styles.iter_mut().enumerate() {
            match outcomes.get(&index) {
                Some(StyleOutcome::Css(css)) => {
                    style.content = Cow::Owned(css.as_str().into());
                    style.lang = Some(Cow::Borrowed("css"));
                }
                Some(StyleOutcome::Failed(error)) => return Err(error.clone()),
                Some(StyleOutcome::Unavailable(warning)) => warnings.push(warning.clone()),
                None => {}
            }
        }
        Ok(warnings)
    }
}

fn pending_blocks(path: &PathBuf) -> Vec<Pending> {
    let Ok(source) = vize_carton::source_io::read_to_string(path) else {
        return Vec::new();
    };
    // Cheap pre-check: most SFCs have no preprocessor block.
    if !PREPROCESSOR_LANGS
        .iter()
        .any(|lang| source.contains(cstr!("lang=\"{lang}\"").as_str()))
    {
        return Vec::new();
    }
    let Ok(descriptor) = parse_sfc(&source, SfcParseOptions::default()) else {
        return Vec::new();
    };
    descriptor
        .styles
        .iter()
        .enumerate()
        .filter_map(|(index, style)| {
            let lang = style.lang.as_deref()?;
            (style.src.is_none() && PREPROCESSOR_LANGS.contains(&lang)).then(|| Pending {
                file: path.clone(),
                absolute: std::path::absolute(path).unwrap_or_else(|_| path.clone()),
                index,
                lang: String::from(lang),
                source: String::from(style.content.as_ref()),
                first_line: source
                    .get(..style.loc.start)
                    .map_or(1, |head| head.matches('\n').count() + 1),
            })
        })
        .collect()
}

fn run_node(pending: &[Pending]) -> Result<Vec<JobResult>, String> {
    let jobs: Vec<Job<'_>> = pending
        .iter()
        .map(|block| Job {
            file: &block.absolute,
            lang: &block.lang,
            source: &block.source,
        })
        .collect();
    let input = serde_json::to_vec(&jobs).map_err(|error| cstr!("{error}"))?;
    let mut child = Command::new("node")
        .arg("--input-type=module")
        .arg("-e")
        .arg(include_str!("preprocess.mjs"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| cstr!("could not start node ({error})"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&input)
            .map_err(|error| cstr!("could not write to node ({error})"))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| cstr!("node failed ({error})"))?;
    if !output.status.success() {
        return Err(cstr!(
            "node failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout).map_err(|error| cstr!("{error}"))
}

fn outcome(block: &Pending, result: &JobResult) -> StyleOutcome {
    if let Some(css) = &result.css {
        return StyleOutcome::Css(css.clone());
    }
    if let Some(error) = &result.error {
        let place = match (error.line, error.column) {
            (Some(line), Some(column)) => cstr!(
                "{}:{}:{}",
                block.file.display(),
                block.first_line + line - 1,
                column
            ),
            _ => cstr!("{}", block.file.display()),
        };
        return StyleOutcome::Failed(cstr!("{place}: {} error: {}", block.lang, error.message));
    }
    StyleOutcome::Unavailable(cstr!(
        "<style lang=\"{}\"> was not preprocessed: install {} in the project",
        block.lang,
        result.missing.as_deref().unwrap_or("its preprocessor")
    ))
}
