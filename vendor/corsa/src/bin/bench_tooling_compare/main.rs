mod args;
mod dataset;
mod measure;
mod process;
mod report;
mod runner;
mod stats;

use std::process::ExitCode;

use corsa::runtime::block_on;

fn main() -> ExitCode {
    let cli = match args::parse() {
        Ok(Some(cli)) => cli,
        Ok(None) => return ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}\nhelp: pass --help to see benchmark options");
            return ExitCode::FAILURE;
        }
    };
    match block_on(run(cli)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", error.diagnostic());
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: args::Cli) -> corsa::Result<()> {
    let datasets = dataset::load(&cli).await?;
    let results = runner::run(&cli, &datasets).await?;
    println!("corsa: {}", cli.corsa_path.display());
    println!("node: {}", cli.node_command);
    println!("iterations: {}", cli.iterations);
    println!("warmup_iterations: {}", cli.warmup_iterations);
    println!("timeout_ms: {}", cli.timeout_ms);
    println!();
    println!("dataset\tfiles\tbytes\tlines\tconfig");
    for dataset in &datasets {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            dataset.label,
            dataset.file_count,
            dataset.total_bytes,
            dataset.total_lines,
            dataset.config_path.display()
        );
    }
    println!();
    println!(
        "workload\tdataset\ttool\tsamples\tmedian_ms\tp95_ms\tp99_ms\tmean_ms\tstddev_ms\tcv_pct\tmin_ms\tmax_ms"
    );
    for row in &results {
        println!(
            "{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.2}\t{:.3}\t{:.3}",
            row.workload,
            row.dataset,
            row.tool,
            row.stats.sample_count(),
            row.stats.median_ms(),
            row.stats.p95_ms(),
            row.stats.p99_ms(),
            row.stats.mean_ms(),
            row.stats.stddev_ms(),
            row.stats.cv_percent(),
            row.stats.min_ms(),
            row.stats.max_ms()
        );
    }
    println!();
    println!("project_check vs corsa baseline");
    println!("dataset\ttool\tmedian_ms\tcorsa_median_ms\tvs_corsa_x");
    for line in report::project_check_vs_corsa_lines(&results) {
        println!("{line}");
    }
    println!();
    println!("project_check requested opponent comparison");
    println!(
        "dataset\ttool\topponent\ttool_median_ms\topponent_median_ms\ttool_vs_opponent_x\twinner"
    );
    for line in report::project_check_requested_opponent_lines(&results) {
        println!("{line}");
    }
    println!();
    println!("editor_workflow vs corsa CLI project_check baseline (not equivalent work)");
    println!("dataset\ttool\tmedian_ms\tcorsa_project_check_median_ms\tvs_corsa_x");
    for line in report::workflow_vs_corsa_lines(&results) {
        println!("{line}");
    }
    if let Some(path) = &cli.json_output_path {
        report::write(path, &cli, &datasets, &results)?;
    }
    Ok(())
}
