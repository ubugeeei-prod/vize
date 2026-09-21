//! Source-map provenance for `compile_sfc`'s module assembly (Davinci P3-9).
//!
//! Every helper here is inert unless the compile asked for a map: the traced
//! variants of each pass return byte-identical text, so turning maps on never
//! changes the module (TS-11).

use vize_carton::{String, ToCompactString, profile};

use crate::compile_script::artifacts::erase_artifact_macro_statements_traced;
use crate::compile_script::lazy_hydration::{
    LazyHydrationTransform, transform_lazy_hydration_macros,
};
use crate::compile_script::typescript::{
    transform_typescript_to_js, transform_typescript_to_js_traced,
};
use crate::module_map::{Runs, edit_runs, module_map_value};
use crate::rewrite_default::rewrite_default_traced;
use crate::types::{SfcDescriptor, SfcScriptBlock};

/// A script block's content after the lazy-hydration and artifact-macro
/// passes, and (when `trace`) that content's provenance in `.vue` offsets.
pub(super) fn prepared_script(
    block: &SfcScriptBlock<'_>,
    trace: bool,
) -> (Option<LazyHydrationTransform>, String, Option<Runs>) {
    let lazy = transform_lazy_hydration_macros(&block.content);
    let source = lazy.as_ref().map_or_else(
        || block.content.to_compact_string(),
        |lazy| lazy.code.clone(),
    );
    let block_runs = || Runs::identity(block.content.len()).offset_origin(block.loc.start);
    let mut runs = trace.then(|| match &lazy {
        Some(lazy) => lazy.runs.compose(&block_runs()),
        None => block_runs(),
    });
    let content = match erase_artifact_macro_statements_traced(&source) {
        Some((erased, erased_runs)) => {
            runs = runs.map(|runs| erased_runs.compose(&runs));
            erased
        }
        None => source,
    };
    (lazy, content, runs)
}

/// A `<script>`-only component's script: `export default` rewritten to
/// `const _sfc_main`, then TypeScript stripped when the source is TypeScript
/// and the output is not. `runs` (the content's provenance) is carried
/// through both passes.
pub(super) fn script_module(
    content: &str,
    runs: Option<Runs>,
    source_is_ts: bool,
    is_ts: bool,
) -> (String, Option<Runs>) {
    let (rewritten, _, rewrite_runs) = profile!(
        "atelier.sfc.normal_script.rewrite_default",
        rewrite_default_traced(content, "_sfc_main", source_is_ts)
    );
    let runs = runs.map(|runs| rewrite_runs.compose(&runs));
    if !source_is_ts || is_ts {
        return (rewritten, runs);
    }
    profile!(
        "atelier.sfc.normal_script.ts_to_js",
        match runs {
            Some(runs) => {
                let (js, js_runs) = transform_typescript_to_js_traced(&rewritten);
                (js, Some(js_runs.compose(&runs)))
            }
            None => (transform_typescript_to_js(&rewritten), None),
        }
    )
}

/// Carry `runs` through v-model demotion, which rewrote `const` to `let` at
/// each of `starts` in a text of `len` bytes.
pub(super) fn demoted(runs: Option<Runs>, len: usize, starts: &[usize]) -> Option<Runs> {
    let edits: Vec<_> = starts
        .iter()
        .map(|&start| (start, start + "const".len(), "let".len()))
        .collect();
    runs.map(|runs| edit_runs(len, &edits).compose(&runs))
}

/// The `SfcCompileResult.map` of `code`, given the provenance of the module
/// before `finalize_output_mode` (`output`, placed at byte `at`) and that
/// pass's own provenance (`rewrite`, `None` when it kept the module).
pub(super) fn module_map(
    code: &str,
    output: Option<Runs>,
    at: usize,
    rewrite: Option<Runs>,
    descriptor: &SfcDescriptor<'_>,
    filename: &str,
) -> Option<serde_json::Value> {
    let mut placed = Runs::default();
    placed.append(at, &output?);
    let provenance = match rewrite {
        Some(rewrite) => rewrite.compose(&placed),
        None => placed,
    };
    module_map_value(code, provenance, filename, &descriptor.source)
}
