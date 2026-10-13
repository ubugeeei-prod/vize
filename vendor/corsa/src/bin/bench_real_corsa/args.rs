use std::{env, path::PathBuf};

use corsa::{
    api::ApiMode,
    fast::{CompactString, SmallVec},
};

const HELP: &str = "\
usage: cargo run -p corsa --bin bench_real_corsa -- [options]

options:
  --corsa PATH              corsa executable (default: .cache/corsa)
  --dataset PATH           tsconfig path to benchmark (repeatable)
  --json-output PATH       write machine-readable benchmark JSON
  --profile                enable detailed per-phase profiling output
  --transport TRANSPORT    jsonrpc | msgpack | both (default: both)
  --cold-iterations N      cold benchmark iterations (default: 5)
  --warm-iterations N      warm benchmark iterations (default: 20)
  --help                   show this message
";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RunMode {
    Benchmark,
    Profiling,
}

#[derive(Clone, Debug)]
pub struct Cli {
    pub root_dir: PathBuf,
    pub corsa_path: PathBuf,
    pub dataset_paths: SmallVec<[PathBuf; 4]>,
    pub json_output_path: Option<PathBuf>,
    pub run_mode: RunMode,
    pub modes: SmallVec<[ApiMode; 2]>,
    pub cold_iterations: usize,
    pub warm_iterations: usize,
}

pub fn parse() -> Result<Option<Cli>, CompactString> {
    let root_dir = discover_root_dir()?;
    let mut corsa_path = default_corsa_path(&root_dir);
    let mut dataset_paths = SmallVec::<[PathBuf; 4]>::new();
    let mut json_output_path = None;
    let mut run_mode = RunMode::Benchmark;
    let mut modes = both_modes();
    let mut cold_iterations = 5_usize;
    let mut warm_iterations = 20_usize;
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
            "--dataset" => {
                dataset_paths.push(read_path(&mut args, &argument, &root_dir)?);
            }
            "--json-output" => {
                json_output_path = Some(read_path(&mut args, &argument, &root_dir)?);
            }
            "--profile" => {
                run_mode = RunMode::Profiling;
            }
            "--run-mode" => {
                run_mode = parse_run_mode(read_value(&mut args, &argument)?)?;
            }
            "--transport" | "--mode" => {
                modes = parse_transport(read_value(&mut args, &argument)?)?;
            }
            "--cold-iterations" => {
                cold_iterations = parse_usize(read_value(&mut args, &argument)?, &argument)?;
            }
            "--warm-iterations" => {
                warm_iterations = parse_usize(read_value(&mut args, &argument)?, &argument)?;
            }
            _ => {
                return Err(CompactString::from(format!("unknown option `{argument}`")));
            }
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
    if cold_iterations == 0 {
        return Err(CompactString::from(
            "--cold-iterations must be greater than 0",
        ));
    }
    if warm_iterations == 0 {
        return Err(CompactString::from(
            "--warm-iterations must be greater than 0",
        ));
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
        dataset_paths,
        json_output_path,
        run_mode,
        modes,
        cold_iterations,
        warm_iterations,
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

fn both_modes() -> SmallVec<[ApiMode; 2]> {
    let mut modes = SmallVec::<[ApiMode; 2]>::new();
    modes.push(ApiMode::AsyncJsonRpcStdio);
    modes.push(ApiMode::SyncMsgpackStdio);
    modes
}

fn default_corsa_path(root_dir: &std::path::Path) -> PathBuf {
    let candidates = [
        root_dir.join(".cache/corsa"),
        root_dir.join(".cache/corsa.exe"),
        root_dir.join("ref/corsa-upstream/.cache/corsa"),
        root_dir.join("ref/corsa-upstream/.cache/corsa.exe"),
        root_dir.join("origin/corsa-upstream/.cache/corsa"),
        root_dir.join("origin/corsa-upstream/.cache/corsa.exe"),
        root_dir.join("ref/corsa-upstream/built/local/corsa"),
        root_dir.join("ref/corsa-upstream/built/local/corsa.exe"),
        root_dir.join("origin/corsa-upstream/built/local/corsa"),
        root_dir.join("origin/corsa-upstream/built/local/corsa.exe"),
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
    let mut datasets = SmallVec::<[PathBuf; 4]>::new();
    for base in [
        root_dir.join("ref/corsa-upstream"),
        root_dir.join("origin/corsa-upstream"),
    ] {
        {
            let path = base.join("packages/typescript/tsconfig.json");
            if path.exists() {
                datasets.push(path);
            }
        }
        if !datasets.is_empty() {
            break;
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

fn parse_transport(value: CompactString) -> Result<SmallVec<[ApiMode; 2]>, CompactString> {
    match value.as_str() {
        "jsonrpc" => {
            let mut modes = SmallVec::<[ApiMode; 2]>::new();
            modes.push(ApiMode::AsyncJsonRpcStdio);
            Ok(modes)
        }
        "msgpack" => {
            let mut modes = SmallVec::<[ApiMode; 2]>::new();
            modes.push(ApiMode::SyncMsgpackStdio);
            Ok(modes)
        }
        "both" => Ok(both_modes()),
        _ => Err(CompactString::from(format!(
            "invalid transport `{value}`; expected jsonrpc, msgpack, or both"
        ))),
    }
}

fn parse_run_mode(value: CompactString) -> Result<RunMode, CompactString> {
    match value.as_str() {
        "benchmark" => Ok(RunMode::Benchmark),
        "profiling" => Ok(RunMode::Profiling),
        _ => Err(CompactString::from(format!(
            "invalid run mode `{value}`; expected benchmark or profiling"
        ))),
    }
}

fn parse_usize(value: CompactString, flag: &CompactString) -> Result<usize, CompactString> {
    value
        .parse::<usize>()
        .map_err(|_| CompactString::from(format!("`{flag}` must be an integer, got `{value}`")))
}

#[cfg(test)]
mod tests {
    use super::{RunMode, both_modes, parse_run_mode, parse_transport, parse_usize};
    use corsa::api::ApiMode;

    #[test]
    fn parse_transport_supports_both_variants() {
        assert_eq!(parse_transport("both".into()).unwrap(), both_modes());
        assert_eq!(
            parse_transport("jsonrpc".into()).unwrap().as_slice(),
            &[ApiMode::AsyncJsonRpcStdio]
        );
        assert_eq!(
            parse_transport("msgpack".into()).unwrap().as_slice(),
            &[ApiMode::SyncMsgpackStdio]
        );
    }

    #[test]
    fn parse_usize_rejects_invalid_numbers() {
        assert_eq!(parse_usize("42".into(), &"--n".into()).unwrap(), 42);
        assert!(parse_usize("nope".into(), &"--n".into()).is_err());
    }

    #[test]
    fn parse_run_mode_supports_profiling() {
        assert_eq!(
            parse_run_mode("benchmark".into()).unwrap(),
            RunMode::Benchmark
        );
        assert_eq!(
            parse_run_mode("profiling".into()).unwrap(),
            RunMode::Profiling
        );
        assert!(parse_run_mode("nope".into()).is_err());
    }
}
