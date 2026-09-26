//! Controlled whole-SFC selected/retained pairs using shipping adapter options.
//! I/O, admission counters and profile attribution stay outside timing windows.

mod davinci_production_perf {
    pub mod attribution;
    pub mod corpus;
    pub mod measure;
    pub mod shapes;
}

use davinci_production_perf::{attribution, corpus, measure, shapes::Shape};
use serde_json::json;
use std::{env, fs, path::PathBuf};

#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = env::args()
        .nth(1)
        .ok_or("usage: davinci_production_perf <output.json>")?;
    let corpus = corpus::load(&root.join("tests/_fixtures"))?;
    let attribution_only = env::args()
        .nth(2)
        .is_some_and(|arg| arg == "--attribution-only");
    if cfg!(feature = "davinci-production-profile") && !attribution_only {
        return Err("detailed attribution builds cannot produce timed acceptance samples".into());
    }
    let mut shapes = Vec::new();
    for shape in Shape::ALL {
        shapes.push(if attribution_only {
            attribution::run(&corpus, shape)?
        } else {
            measure::run(&corpus, shape)?
        });
    }
    let report = json!({
        "schema_version": 1,
        "mode": if attribution_only { "attribution-only (no acceptance timings)" } else { "controlled-pairs" },
        "head_sha": env::var("VIZE_BENCH_HEAD_SHA").unwrap_or_default(),
        "manifest_sha256": corpus::manifest_hash(&corpus),
        "files": corpus.len(),
        "profile": "ci-opt (opt-level=3, thin LTO, 16 codegen units)",
        "allocator": "mimalloc (same default as native CLI; allocation tracking disabled)",
        "features": if cfg!(feature = "davinci-production-profile") { "native,davinci-production-profile; no retained-AST differential dual-run" }
            else { "native,davinci-production-bench; no retained-AST differential dual-run" },
        "options": "P3-17 shipping adapter shapes; Standard syntax, default codegen, default DOM compiler options, script/style ids=fixture path; scoped styles inferred; inline DOM/Vapor, separate DOM module/SSR",
        "window": "parse_sfc + compile_sfc_for_adapter + result destruction; source I/O excluded; profiler disabled; retained scope changes outside timing",
        "sampling": "one warmup batch per lane; 9 samples, 5 corpus passes each; lane order alternates each sample; all/accepted/fallback/diagnostic/no-template/parse-error/routed-Vapor cohorts reported separately",
        "scope": "committed fixture SFCs, excluding _git, _git-worktrees and node_modules; this report does not establish universal speed or semantic parity",
        "shapes": shapes,
    });
    fs::write(output, serde_json::to_vec_pretty(&report)?)?;
    Ok(())
}
