//! Executable canary consumer for the shared Atlas compilation.

use clap::{Args, ValueEnum};
use serde::Serialize;
use std::{fs, path::PathBuf};
use vize_atlas::{ExecutionOutcome, ProductId, ProductRequest, ProductStatus};
use vize_carton::{String, ToCompactString, cstr};

use crate::artifact_graph::{
    VizeGraphConfig, analysis_roots, compiler_roots, create_compilation, project_roots,
};

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum GraphTarget {
    #[default]
    Dom,
    Ssr,
    Vapor,
}

#[derive(Debug, Args)]
#[allow(clippy::disallowed_types)]
pub struct GraphArgs {
    /// SFC, JSX, or TSX sources to execute in one compilation snapshot.
    #[arg(required = true)]
    sources: Vec<PathBuf>,

    /// Compiler backend root requested alongside lint and typecheck roots.
    #[arg(long, value_enum, default_value = "dom")]
    target: GraphTarget,

    /// Omit the compiler root.
    #[arg(long)]
    no_compiler: bool,

    /// Omit the Patina semantic report root.
    #[arg(long)]
    no_lint: bool,

    /// Omit the Canon semantic Virtual TS root.
    #[arg(long)]
    no_typecheck: bool,

    /// Also request opt-in cross-file Croquis aggregation for the first source.
    #[arg(long)]
    project: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphReport {
    schema: &'static str,
    version: u8,
    snapshot_source_count: usize,
    files: Vec<GraphFileReport>,
    counters: Vec<GraphCounterReport>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphFileReport {
    path: String,
    source_id: u64,
    source_revision: u64,
    roots: Vec<&'static str>,
    products: Vec<GraphProductReport>,
    observations: usize,
    compiler_bytes: Option<usize>,
    vapor_blocks: Option<usize>,
    lint_diagnostics: Option<usize>,
    virtual_ts_bytes: Option<usize>,
    flow_reachable_blocks: Option<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphProductReport {
    source_id: u64,
    product: &'static str,
    provider: &'static str,
    status: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GraphCounterReport {
    product: &'static str,
    queries: u64,
    executions: u64,
    cache_hits: u64,
}

pub fn run(args: GraphArgs) {
    match execute(&args).and_then(|report| {
        serde_json::to_string_pretty(&report).map_err(|error| error.to_compact_string())
    }) {
        Ok(report) => println!("{report}"),
        Err(error) => {
            eprintln!("Artifact graph execution failed: {error}");
            std::process::exit(1);
        }
    }
}

fn execute(args: &GraphArgs) -> Result<GraphReport, String> {
    let mut compilation = create_compilation(VizeGraphConfig::default())
        .map_err(|error| error.to_compact_string())?;
    let mut registered = Vec::with_capacity(args.sources.len());
    for path in &args.sources {
        let text = fs::read_to_string(path)
            .map_err(|error| cstr!("failed to read {}: {error}", path.display()))?;
        let name = path.to_string_lossy().as_ref().to_compact_string();
        let source = compilation
            .add_source(name.clone(), text)
            .map_err(|error| error.to_compact_string())?;
        registered.push((name, source));
    }

    let snapshot = compilation.snapshot();
    let mut execution = snapshot.fork();
    let roots = requested_roots(args);
    if roots.is_empty() && !args.project {
        return Err("at least one compiler, lint, typecheck, or project root is required".into());
    }

    let mut files = Vec::with_capacity(registered.len());
    let mut observed_products = Vec::new();
    for (index, (path, source)) in registered.into_iter().enumerate() {
        let mut source_roots = roots.clone();
        if args.project && index == 0 {
            source_roots.extend(project_roots(true));
        }
        if source_roots.is_empty() {
            continue;
        }
        let plan = snapshot
            .plan(source, source_roots)
            .map_err(|error| error.to_compact_string())?;
        let outcome = execution
            .execute(plan)
            .map_err(|error| error.to_compact_string())?;
        observed_products.extend_from_slice(outcome.plan().products());
        files.push(file_report(path, &outcome)?);
    }
    observed_products.sort_by_key(|product| product.name());
    observed_products.dedup();
    let counters = observed_products
        .into_iter()
        .map(|product| {
            let counter = execution.counters().for_id(product);
            GraphCounterReport {
                product: product.name(),
                queries: counter.queries(),
                executions: counter.executions(),
                cache_hits: counter.cache_hits(),
            }
        })
        .collect();

    Ok(GraphReport {
        schema: "vize.artifact-graph.execution",
        version: 1,
        snapshot_source_count: snapshot.sources().len(),
        files,
        counters,
    })
}

fn requested_roots(args: &GraphArgs) -> Vec<ProductId> {
    let (dom, ssr, vapor) = match args.target {
        GraphTarget::Dom => (!args.no_compiler, false, false),
        GraphTarget::Ssr => (false, !args.no_compiler, false),
        GraphTarget::Vapor => (false, false, !args.no_compiler),
    };
    let mut roots = compiler_roots(dom, ssr, vapor);
    roots.extend(analysis_roots(!args.no_lint, !args.no_typecheck));
    roots
}

fn file_report(path: String, outcome: &ExecutionOutcome) -> Result<GraphFileReport, String> {
    let plan = outcome.plan();
    let products = plan
        .requests()
        .iter()
        .map(|request| product_report(*request, outcome))
        .collect::<Result<Vec<_>, _>>()?;
    let (compiler_bytes, vapor_blocks) = compiler_output_size(outcome)?;
    let lint_diagnostics = outcome
        .get::<vize_patina::PatinaSemanticReportProduct>()
        .map_err(|error| error.to_compact_string())?
        .map(|report| report.diagnostics.len());
    let virtual_ts = outcome
        .get::<vize_canon::CanonSemanticVirtualTsProduct>()
        .map_err(|error| error.to_compact_string())?;

    Ok(GraphFileReport {
        path,
        source_id: outcome.source().get(),
        source_revision: outcome.source_revision().get(),
        roots: plan.roots().iter().map(|product| product.name()).collect(),
        products,
        observations: outcome.observations().len(),
        compiler_bytes,
        vapor_blocks,
        lint_diagnostics,
        virtual_ts_bytes: virtual_ts.as_ref().map(|output| output.code.len()),
        flow_reachable_blocks: virtual_ts
            .as_ref()
            .map(|output| output.reachable_block_count),
    })
}

fn product_report(
    request: ProductRequest,
    outcome: &ExecutionOutcome,
) -> Result<GraphProductReport, String> {
    let provider = outcome
        .plan()
        .provider_for_request(request)
        .ok_or_else(|| cstr!("plan has no provider for {request}"))?;
    let status = match outcome.status_for_request(request) {
        Some(ProductStatus::Executed) => "executed",
        Some(ProductStatus::CacheHit) => "cache-hit",
        Some(ProductStatus::Pruned) => "pruned",
        None => return Err(cstr!("execution has no status for {request}")),
    };
    Ok(GraphProductReport {
        source_id: request.source().get(),
        product: request.product().name(),
        provider: provider.name(),
        status,
    })
}

fn compiler_output_size(
    outcome: &ExecutionOutcome,
) -> Result<(Option<usize>, Option<usize>), String> {
    if let Some(output) = outcome
        .get::<vize_atelier_dom::DomOutputProduct>()
        .map_err(|error| error.to_compact_string())?
    {
        return Ok((Some(output.code.len()), None));
    }
    if let Some(output) = outcome
        .get::<vize_atelier_ssr::SsrOutputProduct>()
        .map_err(|error| error.to_compact_string())?
    {
        return Ok((Some(output.code.len()), None));
    }
    Ok((
        None,
        outcome
            .get::<vize_atelier_vapor::VaporPlanProduct>()
            .map_err(|error| error.to_compact_string())?
            .map(|output| output.blocks().len()),
    ))
}
