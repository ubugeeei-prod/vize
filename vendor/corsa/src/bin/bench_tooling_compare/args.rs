use std::{env, path::PathBuf};

use corsa::fast::{CompactString, SmallVec};

const HELP: &str = "\
usage: cargo run -p corsa --bin bench_tooling_compare -- [options]

options:
  --corsa PATH                corsa executable (default: .cache/corsa)
  --node CMD                 node executable or command name (default: node)
  --dataset PATH             tsconfig path to benchmark (repeatable)
  --json-output PATH         write machine-readable benchmark JSON
  --allow-partial-failures    skip failed benchmark rows and continue
  --suite SUITE              project-check | workflow | both (default: both)
  --iterations N             timed iterations per row (default: 10)
  --warmup-iterations N      untimed warmup iterations (default: 2)
  --timeout-ms N             per-process timeout in milliseconds (default: 60000)
  --help                     show this message
";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Suite {
    ProjectCheck,
    Workflow,
}

#[derive(Clone, Debug)]
pub struct Cli {
    pub root_dir: PathBuf,
    pub corsa_path: PathBuf,
    pub node_command: CompactString,
    pub dataset_paths: SmallVec<[PathBuf; 4]>,
    pub json_output_path: Option<PathBuf>,
    pub allow_partial_failures: bool,
    pub suites: SmallVec<[Suite; 2]>,
    pub iterations: usize,
    pub warmup_iterations: usize,
    pub timeout_ms: u64,
}

pub fn parse() -> Result<Option<Cli>, CompactString> {
    let root_dir = discover_root_dir()?;
    let mut corsa_path = default_corsa_path(&root_dir);
    let mut node_command = CompactString::from("node");
    let mut dataset_paths = SmallVec::<[PathBuf; 4]>::new();
    let mut json_output_path = None;
    let mut allow_partial_failures = false;
    let mut suites = both_suites();
    let mut iterations = 10_usize;
    let mut warmup_iterations = 2_usize;
    let mut timeout_ms = 60_000_u64;
    let mut args = env::args_os().skip(1);
    while let Some(argument) = args.next() {
        let argument = CompactString::from(argument.to_string_lossy().as_ref());
        match argument.as_str() {
            "--help" | "-h" => {
                println!("{HELP}");
                return Ok(None);
            }
            "--corsa" => {
                corsa_path = read_path(&mut args, &argument, &root_dir)?;
            }
            "--node" => {
                node_command = read_value(&mut args, &argument)?;
            }
            "--dataset" => {
                dataset_paths.push(read_path(&mut args, &argument, &root_dir)?);
            }
            "--json-output" => {
                json_output_path = Some(read_path(&mut args, &argument, &root_dir)?);
            }
            "--allow-partial-failures" => {
                allow_partial_failures = true;
            }
            "--suite" => {
                suites = parse_suite(read_value(&mut args, &argument)?)?;
            }
            "--iterations" => {
                iterations = parse_usize(read_value(&mut args, &argument)?, &argument)?;
            }
            "--warmup-iterations" => {
                warmup_iterations = parse_usize(read_value(&mut args, &argument)?, &argument)?;
            }
            "--timeout-ms" => {
                timeout_ms = parse_u64(read_value(&mut args, &argument)?, &argument)?;
            }
            _ => return Err(CompactString::from(format!("unknown option `{argument}`"))),
        }
    }
    if dataset_paths.is_empty() {
        dataset_paths = default_datasets(&root_dir);
    }
    if dataset_paths.is_empty() {
        return Err(CompactString::from(
            "no datasets found; pass --dataset PATH explicitly",
        ));
    }
    if iterations == 0 {
        return Err(CompactString::from("--iterations must be greater than 0"));
    }
    if !corsa_path.exists() {
        return Err(CompactString::from(format!(
            "corsa executable does not exist: {}",
            corsa_path.display()
        )));
    }
    Ok(Some(Cli {
        root_dir,
        corsa_path,
        node_command,
        dataset_paths,
        json_output_path,
        allow_partial_failures,
        suites,
        iterations,
        warmup_iterations,
        timeout_ms,
    }))
}

fn discover_root_dir() -> Result<PathBuf, CompactString> {
    let cwd = env::current_dir().map_err(|error| CompactString::from(error.to_string()))?;
    for candidate in cwd.ancestors() {
        if candidate.join("pnpm-workspace.yaml").exists()
            && candidate.join("vite.config.ts").exists()
        {
            return Ok(candidate.to_path_buf());
        }
    }
    Ok(cwd)
}

fn both_suites() -> SmallVec<[Suite; 2]> {
    let mut suites = SmallVec::<[Suite; 2]>::new();
    suites.push(Suite::ProjectCheck);
    suites.push(Suite::Workflow);
    suites
}

fn default_corsa_path(root_dir: &std::path::Path) -> PathBuf {
    let candidates = [
        root_dir.join(".cache/corsa"),
        root_dir.join(".cache/corsa.exe"),
        root_dir.join("ref/corsa-upstream/.cache/corsa"),
        root_dir.join("ref/corsa-upstream/.cache/corsa.exe"),
        root_dir.join("ref/corsa-upstream/built/local/corsa"),
        root_dir.join("ref/corsa-upstream/built/local/corsa.exe"),
    ];
    for candidate in candidates {
        if candidate.exists() {
            return candidate;
        }
    }
    root_dir.join(if cfg!(windows) {
        ".cache/corsa.exe"
    } else {
        ".cache/corsa"
    })
}

fn default_datasets(root_dir: &std::path::Path) -> SmallVec<[PathBuf; 4]> {
    let base = root_dir.join("ref/corsa-upstream");
    let candidates = [base.join("packages/typescript/tsconfig.json")];
    let mut datasets = SmallVec::<[PathBuf; 4]>::new();
    for path in candidates {
        if path.exists() {
            datasets.push(path);
        }
    }
    datasets
}

fn read_path(
    args: &mut impl Iterator<Item = std::ffi::OsString>,
    flag: &CompactString,
    root_dir: &std::path::Path,
) -> Result<PathBuf, CompactString> {
    let value = PathBuf::from(read_value(args, flag)?.as_str());
    if value.is_absolute() {
        Ok(value)
    } else {
        Ok(root_dir.join(value))
    }
}

fn read_value(
    args: &mut impl Iterator<Item = std::ffi::OsString>,
    flag: &CompactString,
) -> Result<CompactString, CompactString> {
    let Some(value) = args.next() else {
        return Err(CompactString::from(format!("missing value for `{flag}`")));
    };
    Ok(CompactString::from(value.to_string_lossy().as_ref()))
}

fn parse_suite(value: CompactString) -> Result<SmallVec<[Suite; 2]>, CompactString> {
    match value.as_str() {
        "project-check" => {
            let mut suites = SmallVec::<[Suite; 2]>::new();
            suites.push(Suite::ProjectCheck);
            Ok(suites)
        }
        "workflow" => {
            let mut suites = SmallVec::<[Suite; 2]>::new();
            suites.push(Suite::Workflow);
            Ok(suites)
        }
        "both" => Ok(both_suites()),
        _ => Err(CompactString::from(format!(
            "invalid suite `{value}`; expected project-check, workflow, or both"
        ))),
    }
}

fn parse_usize(value: CompactString, flag: &CompactString) -> Result<usize, CompactString> {
    value
        .parse::<usize>()
        .map_err(|_| CompactString::from(format!("`{flag}` must be an integer, got `{value}`")))
}

fn parse_u64(value: CompactString, flag: &CompactString) -> Result<u64, CompactString> {
    value
        .parse::<u64>()
        .map_err(|_| CompactString::from(format!("`{flag}` must be an integer, got `{value}`")))
}

#[cfg(test)]
mod tests {
    use super::{Suite, both_suites, parse_suite, parse_u64, parse_usize};

    #[test]
    fn parse_suite_supports_all_variants() {
        assert_eq!(parse_suite("both".into()).unwrap(), both_suites());
        assert_eq!(
            parse_suite("project-check".into()).unwrap().as_slice(),
            &[Suite::ProjectCheck]
        );
        assert_eq!(
            parse_suite("workflow".into()).unwrap().as_slice(),
            &[Suite::Workflow]
        );
    }

    #[test]
    fn numeric_parsers_reject_invalid_values() {
        assert_eq!(
            parse_usize("10".into(), &"--iterations".into()).unwrap(),
            10
        );
        assert_eq!(parse_u64("20".into(), &"--timeout-ms".into()).unwrap(), 20);
        assert!(parse_usize("nope".into(), &"--iterations".into()).is_err());
        assert!(parse_u64("nope".into(), &"--timeout-ms".into()).is_err());
    }
}
