use std::{
    fs,
    future::Future,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use corsa::{
    CorsaError, Result,
    api::{ApiClient, ApiMode, ApiSpawnConfig, SymbolHandle, UpdateSnapshotParams},
    fast::{CompactString, SmallVec},
};
use serde_json::json;

use crate::{
    args::{Cli, Suite},
    dataset::DatasetCase,
    measure::measure_with_warmup,
    process::run_command,
    stats::Stats,
};

#[derive(Clone, Debug)]
pub struct ToolRow {
    pub workload: CompactString,
    pub dataset: CompactString,
    pub tool: CompactString,
    pub stats: Stats,
}

struct ToolSupport {
    workspace_root: PathBuf,
    corsa_upstream_root: PathBuf,
    node_command: CompactString,
    tsc_script: PathBuf,
    eslint_script: PathBuf,
    eslint_config: PathBuf,
    oxlint_script: PathBuf,
    corsa_oxlint_config: PathBuf,
    tsgolint_script: PathBuf,
}

struct OverlayConfig {
    dir: OverlayDir,
    path: PathBuf,
}

struct OverlayDir {
    path: PathBuf,
}

struct WorkflowSession {
    client: ApiClient,
    snapshot: corsa::api::ManagedSnapshot,
    project: corsa::api::ProjectHandle,
    file: CompactString,
    target: BenchTarget,
}

struct BenchTarget {
    position: u32,
    symbol: SymbolHandle,
}

pub async fn run(cli: &Cli, datasets: &[DatasetCase]) -> Result<SmallVec<[ToolRow; 32]>> {
    let mut rows = SmallVec::<[ToolRow; 32]>::new();
    let support = ToolSupport::discover(cli)?;
    let run_project_check = cli.suites.contains(&Suite::ProjectCheck);
    let run_workflow = cli.suites.contains(&Suite::Workflow);
    for dataset in datasets {
        let overlay = OverlayConfig::create(cli, dataset)?;
        if run_project_check {
            rows.extend(run_project_check_suite(cli, dataset, &support, &overlay).await?);
        }
        if run_workflow {
            rows.extend(run_workflow_suite(cli, dataset).await?);
        }
    }
    Ok(rows)
}

async fn run_project_check_suite(
    cli: &Cli,
    dataset: &DatasetCase,
    support: &ToolSupport,
    overlay: &OverlayConfig,
) -> Result<SmallVec<[ToolRow; 16]>> {
    let mut rows = SmallVec::<[ToolRow; 16]>::new();
    let timeout = Duration::from_millis(cli.timeout_ms);
    if let Some(stats) = measure_row(cli.allow_partial_failures, "project_check:tsc", || {
        measure_with_warmup(cli.warmup_iterations, cli.iterations, || async {
            let mut command = tsc_command(support, overlay);
            run_command(&mut command, timeout, &[0], "tsc")
        })
    })
    .await?
    {
        rows.push(row("project_check", dataset, "tsc", stats));
    }
    if let Some(stats) = measure_row(cli.allow_partial_failures, "project_check:corsa", || {
        measure_with_warmup(cli.warmup_iterations, cli.iterations, || async {
            let mut command = corsa_command(cli, support, overlay);
            run_command(&mut command, timeout, &[0], "corsa")
        })
    })
    .await?
    {
        rows.push(row("project_check", dataset, "corsa", stats));
    }
    if let Some(stats) = measure_row(
        cli.allow_partial_failures,
        "project_check:typescript-eslint",
        || {
            measure_with_warmup(cli.warmup_iterations, cli.iterations, || async {
                let mut command = eslint_command(dataset, support, overlay);
                run_command(&mut command, timeout, &[0, 1], "typescript-eslint")
            })
        },
    )
    .await?
    {
        rows.push(row("project_check", dataset, "typescript-eslint", stats));
    }
    if let Some(stats) = measure_row(
        cli.allow_partial_failures,
        "project_check:corsa-oxlint",
        || {
            measure_with_warmup(cli.warmup_iterations, cli.iterations, || async {
                let mut command = corsa_oxlint_command(cli, dataset, support, overlay);
                run_command(&mut command, timeout, &[0, 1], "corsa-oxlint")
            })
        },
    )
    .await?
    {
        rows.push(row("project_check", dataset, "corsa-oxlint", stats));
    }
    if let Some(stats) = measure_row(
        cli.allow_partial_failures,
        "project_check:oxlint-bare",
        || {
            measure_with_warmup(cli.warmup_iterations, cli.iterations, || async {
                let mut command = oxlint_bare_command(dataset, support);
                run_command(&mut command, timeout, &[0, 1], "oxlint-bare")
            })
        },
    )
    .await?
    {
        rows.push(row("project_check", dataset, "oxlint-bare", stats));
    }
    if let Some(stats) = measure_row(cli.allow_partial_failures, "project_check:tsgolint", || {
        measure_with_warmup(cli.warmup_iterations, cli.iterations, || async {
            let mut command = tsgolint_command(support, overlay);
            run_command(&mut command, timeout, &[0, 1], "tsgolint")
        })
    })
    .await?
    {
        rows.push(row("project_check", dataset, "tsgolint", stats));
    }
    Ok(rows)
}

async fn run_workflow_suite(cli: &Cli, dataset: &DatasetCase) -> Result<SmallVec<[ToolRow; 4]>> {
    let mut rows = SmallVec::<[ToolRow; 4]>::new();
    if let Some(stats) = measure_row(
        cli.allow_partial_failures,
        "editor_workflow:corsa-msgpack-cold",
        || workflow_cold(cli, dataset),
    )
    .await?
    {
        rows.push(row("editor_workflow", dataset, "corsa-msgpack-cold", stats));
    }
    if let Some(stats) = measure_row(
        cli.allow_partial_failures,
        "editor_workflow:corsa-msgpack-warm",
        || workflow_warm(cli, dataset),
    )
    .await?
    {
        rows.push(row("editor_workflow", dataset, "corsa-msgpack-warm", stats));
    }
    Ok(rows)
}

fn row(workload: &str, dataset: &DatasetCase, tool: &str, stats: Stats) -> ToolRow {
    ToolRow {
        workload: CompactString::from(workload),
        dataset: dataset.label.clone(),
        tool: CompactString::from(tool),
        stats,
    }
}

async fn measure_row<T, Fut>(
    allow_partial_failures: bool,
    label: &str,
    make_stats: impl FnOnce() -> Fut,
) -> Result<Option<T>>
where
    Fut: Future<Output = Result<T>>,
{
    match make_stats().await {
        Ok(stats) => Ok(Some(stats)),
        Err(error) if allow_partial_failures => {
            eprintln!(
                "warning: skipping {label} benchmark\n{}",
                error.diagnostic()
            );
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

fn tsc_command(support: &ToolSupport, overlay: &OverlayConfig) -> Command {
    let mut command = Command::new(support.node_command.as_str());
    command
        .current_dir(&support.corsa_upstream_root)
        .arg(&support.tsc_script)
        .arg("--pretty")
        .arg("false")
        .arg("--noEmit")
        .arg("-p")
        .arg(&overlay.path);
    command
}

fn corsa_command(cli: &Cli, support: &ToolSupport, overlay: &OverlayConfig) -> Command {
    let mut command = Command::new(&cli.corsa_path);
    command
        .current_dir(&support.corsa_upstream_root)
        .arg("--pretty")
        .arg("false")
        .arg("--noEmit")
        .arg("-p")
        .arg(&overlay.path);
    command
}

fn eslint_command(
    dataset: &DatasetCase,
    support: &ToolSupport,
    overlay: &OverlayConfig,
) -> Command {
    let mut command = Command::new(support.node_command.as_str());
    command
        .current_dir(&support.workspace_root)
        .arg(&support.eslint_script)
        .arg("--config")
        .arg(&support.eslint_config)
        .arg("--no-config-lookup")
        .env("CORSA_RS_BENCH_TSCONFIG", &overlay.path);
    for file in &dataset.source_files {
        command.arg(file.as_str());
    }
    command
}

fn corsa_oxlint_command(
    cli: &Cli,
    dataset: &DatasetCase,
    support: &ToolSupport,
    overlay: &OverlayConfig,
) -> Command {
    let mut command = Command::new(support.node_command.as_str());
    command
        .current_dir(&support.workspace_root)
        .arg(&support.oxlint_script)
        .arg("--config")
        .arg(&support.corsa_oxlint_config)
        .arg("--disable-nested-config")
        .arg("--silent")
        .env("CORSA_RS_BENCH_TSCONFIG", &overlay.path)
        .env("CORSA_RS_BENCH_EXECUTABLE", &cli.corsa_path)
        .env("CORSA_RS_BENCH_ROOT", &support.workspace_root);
    for file in &dataset.source_files {
        command.arg(file.as_str());
    }
    command
}

fn oxlint_bare_command(dataset: &DatasetCase, support: &ToolSupport) -> Command {
    let mut command = Command::new(support.node_command.as_str());
    command
        .current_dir(&support.workspace_root)
        .arg(&support.oxlint_script)
        .arg("--disable-nested-config")
        .arg("--silent");
    for file in &dataset.source_files {
        command.arg(file.as_str());
    }
    command
}

fn tsgolint_command(support: &ToolSupport, overlay: &OverlayConfig) -> Command {
    let mut command = Command::new(support.node_command.as_str());
    command
        .current_dir(&overlay.dir.path)
        .arg(&support.tsgolint_script)
        .arg("--tsconfig")
        .arg(&overlay.path);
    command
}

async fn workflow_cold(cli: &Cli, dataset: &DatasetCase) -> Result<Stats> {
    measure_with_warmup(0, cli.iterations, || async {
        let session = open_workflow_session(cli, dataset).await?;
        let workflow = run_editor_workflow(&session).await;
        let cleanup = close_workflow_session(session).await;
        workflow?;
        cleanup
    })
    .await
}

async fn workflow_warm(cli: &Cli, dataset: &DatasetCase) -> Result<Stats> {
    let session = open_workflow_session(cli, dataset).await?;
    let measured = measure_with_warmup(cli.warmup_iterations, cli.iterations, || async {
        run_editor_workflow(&session).await
    })
    .await;
    let cleanup = close_workflow_session(session).await;
    match (measured, cleanup) {
        (Ok(stats), Ok(())) => Ok(stats),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error),
    }
}

async fn open_workflow_session(cli: &Cli, dataset: &DatasetCase) -> Result<WorkflowSession> {
    let client = ApiClient::spawn(
        ApiSpawnConfig::new(&cli.corsa_path)
            .with_cwd(&cli.root_dir)
            .with_mode(ApiMode::SyncMsgpackStdio),
    )
    .await?;
    let snapshot = client
        .update_snapshot(UpdateSnapshotParams {
            open_project: Some(dataset.config_wire.to_string()),
            file_changes: None,
            overlay_changes: None,
        })
        .await?;
    let project = snapshot
        .projects
        .first()
        .ok_or_else(|| {
            CorsaError::Protocol(CompactString::from(format!(
                "benchmark snapshot did not open a project for {}",
                dataset.config_wire
            )))
        })?
        .id
        .clone();
    let target =
        discover_bench_target(&client, &snapshot, &project, dataset.primary_file.as_str()).await?;
    Ok(WorkflowSession {
        client,
        snapshot,
        project,
        file: dataset.primary_file.clone(),
        target,
    })
}

async fn run_editor_workflow(session: &WorkflowSession) -> Result<()> {
    let _ = session
        .client
        .get_default_project_for_file(session.snapshot.handle.clone(), session.file.as_str())
        .await?;
    let _ = session
        .client
        .get_source_file(
            session.snapshot.handle.clone(),
            session.project.clone(),
            session.file.as_str(),
        )
        .await?;
    let _ = session
        .client
        .get_symbol_at_position(
            session.snapshot.handle.clone(),
            session.project.clone(),
            session.file.as_str(),
            session.target.position,
        )
        .await?;
    let _ = session
        .client
        .get_type_of_symbol(
            session.snapshot.handle.clone(),
            session.project.clone(),
            session.target.symbol.clone(),
        )
        .await?;
    let ty = session
        .client
        .get_type_at_position(
            session.snapshot.handle.clone(),
            session.project.clone(),
            session.file.as_str(),
            session.target.position,
        )
        .await?
        .ok_or(CorsaError::Protocol(
            "workflow target no longer resolves to a type".into(),
        ))?;
    let _ = session
        .client
        .type_to_string(
            session.snapshot.handle.clone(),
            session.project.clone(),
            ty.id,
            None,
            None,
        )
        .await?;
    Ok(())
}

async fn close_workflow_session(session: WorkflowSession) -> Result<()> {
    let release = session.snapshot.release().await;
    let close = session.client.close().await;
    release?;
    close
}

async fn discover_bench_target(
    client: &ApiClient,
    snapshot: &corsa::api::ManagedSnapshot,
    project: &corsa::api::ProjectHandle,
    file: &str,
) -> Result<BenchTarget> {
    let source = client
        .get_source_file(snapshot.handle.clone(), project.clone(), file)
        .await?
        .ok_or(CorsaError::Protocol(
            "benchmark dataset is missing its primary file".into(),
        ))?;
    let text = String::from_utf8_lossy(source.as_bytes());
    for (position, token) in identifier_positions(text.as_ref()) {
        if token.len() <= 1 || is_noise_identifier(token) {
            continue;
        }
        if let Some(symbol) = client
            .get_symbol_at_position(snapshot.handle.clone(), project.clone(), file, position)
            .await?
        {
            return Ok(BenchTarget {
                position,
                symbol: symbol.id,
            });
        }
    }
    Err(CorsaError::Protocol(
        "failed to discover a benchmarkable symbol in the primary file".into(),
    ))
}

fn identifier_positions(text: &str) -> impl Iterator<Item = (u32, &str)> {
    let mut items = SmallVec::<[(u32, &str); 128]>::new();
    let bytes = text.as_bytes();
    let mut index = 0_usize;
    while index < bytes.len() {
        if !is_identifier_start(bytes[index]) {
            index += 1;
            continue;
        }
        let start = index;
        index += 1;
        while index < bytes.len() && is_identifier_continue(bytes[index]) {
            index += 1;
        }
        items.push((
            u32::try_from(start).unwrap_or(u32::MAX),
            &text[start..index],
        ));
    }
    items.into_iter()
}

fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$')
}

fn is_identifier_continue(byte: u8) -> bool {
    is_identifier_start(byte) || byte.is_ascii_digit()
}

fn is_noise_identifier(token: &str) -> bool {
    matches!(
        token,
        "const"
            | "let"
            | "var"
            | "function"
            | "class"
            | "interface"
            | "type"
            | "import"
            | "export"
            | "from"
            | "return"
            | "if"
            | "else"
            | "for"
            | "while"
            | "switch"
            | "case"
            | "default"
            | "extends"
            | "implements"
            | "new"
            | "true"
            | "false"
            | "null"
            | "undefined"
    )
}

impl ToolSupport {
    fn discover(cli: &Cli) -> Result<Self> {
        let workspace_root = cli.root_dir.clone();
        let corsa_upstream_root = workspace_root.join("ref/corsa-upstream");
        let tsc_script = corsa_upstream_root.join("node_modules/typescript/bin/tsc");
        if !tsc_script.exists() {
            return Err(CorsaError::Protocol(CompactString::from(
                "missing ref/corsa-upstream/node_modules/typescript/bin/tsc; run `vp run -w bench_tooling_setup` first",
            )));
        }
        let cli_compare_root = workspace_root.join("bench/cli_compare");
        let eslint_script = cli_compare_root.join("node_modules/eslint/bin/eslint.js");
        if !eslint_script.exists() {
            return Err(CorsaError::Protocol(CompactString::from(
                "missing bench/cli_compare/node_modules/eslint/bin/eslint.js; run `vp run -w bench_tooling_setup` first",
            )));
        }
        let eslint_config = cli_compare_root.join("eslint.config.mjs");
        let oxlint_script =
            workspace_root.join("src/bindings/nodejs/corsa_oxlint/node_modules/oxlint/bin/oxlint");
        if !oxlint_script.exists() {
            return Err(CorsaError::Protocol(CompactString::from(
                "missing src/bindings/nodejs/corsa_oxlint/node_modules/oxlint/bin/oxlint; run `vp install` first",
            )));
        }
        let corsa_oxlint_config = cli_compare_root.join("corsa-oxlint.config.mjs");
        if !corsa_oxlint_config.exists() {
            return Err(CorsaError::Protocol(CompactString::from(
                "missing bench/cli_compare/corsa-oxlint.config.mjs",
            )));
        }
        let corsa_oxlint_rules =
            workspace_root.join("src/bindings/nodejs/corsa_oxlint/dist/rules/index.js");
        if !corsa_oxlint_rules.exists() {
            return Err(CorsaError::Protocol(CompactString::from(
                "missing src/bindings/nodejs/corsa_oxlint/dist/rules/index.js; run `vp run -w build_corsa_oxlint` first",
            )));
        }
        let tsgolint_script = cli_compare_root.join("node_modules/oxlint-tsgolint/bin/tsgolint.js");
        if !tsgolint_script.exists() {
            return Err(CorsaError::Protocol(CompactString::from(
                "missing bench/cli_compare/node_modules/oxlint-tsgolint/bin/tsgolint.js; run `vp run -w bench_tooling_setup` first",
            )));
        }
        Ok(Self {
            workspace_root,
            corsa_upstream_root,
            node_command: cli.node_command.clone(),
            tsc_script,
            eslint_script,
            eslint_config,
            oxlint_script,
            corsa_oxlint_config,
            tsgolint_script,
        })
    }
}

impl OverlayConfig {
    fn create(cli: &Cli, dataset: &DatasetCase) -> Result<Self> {
        let dir = OverlayDir::create(&cli.root_dir)?;
        let path = dir.path.join(format!("{}.json", dataset.label.as_str()));
        let extends = relative_path(&dir.path, &dataset.config_path);
        fs::write(
            &path,
            serde_json::to_vec_pretty(&json!({
                "extends": extends,
                "compilerOptions": {
                    "customConditions": ["@typescript/source"]
                }
            }))?,
        )?;
        fs::write(
            dir.path.join(".oxlintrc.json"),
            serde_json::to_vec_pretty(&json!({
                "options": {
                    "typeAware": true,
                },
                "rules": {
                    "typescript/await-thenable": "error",
                    "typescript/no-array-delete": "error",
                    "typescript/no-base-to-string": "error",
                    "typescript/no-floating-promises": "error",
                    "typescript/no-for-in-array": "error",
                    "typescript/no-implied-eval": "error",
                    "typescript/no-mixed-enums": "error",
                    "typescript/no-unsafe-assignment": "error",
                    "typescript/no-unsafe-return": "error",
                    "typescript/no-unsafe-unary-minus": "error",
                    "typescript/only-throw-error": "error",
                    "typescript/prefer-find": "error",
                    "typescript/prefer-includes": "error",
                    "typescript/prefer-promise-reject-errors": "error",
                    "typescript/prefer-regexp-exec": "error",
                    "typescript/prefer-string-starts-ends-with": "error",
                    "typescript/require-array-sort-compare": "error",
                    "typescript/restrict-plus-operands": "error",
                    "typescript/use-unknown-in-catch-callback-variable": "error",
                }
            }))?,
        )?;
        Ok(Self { dir, path })
    }
}

impl OverlayDir {
    fn create(root_dir: &Path) -> Result<Self> {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos())
            .unwrap_or(0);
        let path = root_dir
            .join("ref/corsa-upstream/.cache/bench_tooling_compare")
            .join(format!("overlay-{}-{suffix}", std::process::id()));
        fs::create_dir_all(&path)?;
        Ok(Self { path })
    }
}

impl Drop for OverlayDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn relative_path(from_dir: &Path, to: &Path) -> PathBuf {
    let from_components = from_dir.components().collect::<Vec<_>>();
    let to_components = to.components().collect::<Vec<_>>();
    let mut shared = 0_usize;
    while shared < from_components.len()
        && shared < to_components.len()
        && from_components[shared] == to_components[shared]
    {
        shared += 1;
    }
    let mut path = PathBuf::new();
    for _ in shared..from_components.len() {
        path.push("..");
    }
    for component in &to_components[shared..] {
        path.push(component.as_os_str());
    }
    path
}
