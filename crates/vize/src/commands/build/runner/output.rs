//! Deterministic build output planning and writing.

mod plan;

use std::{
    fs, io,
    io::Write as _,
    path::{Path, PathBuf},
};

use vize_s0::profiler::global_profiler;
use vize_s0::{String, profile};

use super::super::{
    OutputFormat, ScriptExtension,
    config::{CompileOutput, get_output_extension},
};

use plan::{OutputPath, destination, validate_output_paths};
pub(super) use plan::{PlannedInput, plan_inputs, preflight_outputs};

pub(super) struct CompiledBuildOutput<'a> {
    pub input: &'a PlannedInput,
    pub output: CompileOutput,
}

#[derive(Debug)]
pub(super) enum OutputError {
    CurrentDirectory(io::Error),
    NoCommonRoot,
    SourceOutsideRoot {
        source: PathBuf,
        root: PathBuf,
    },
    Collision {
        first_source: PathBuf,
        first_output: PathBuf,
        second_source: PathBuf,
        second_output: PathBuf,
    },
    CreateDirectory {
        path: PathBuf,
        source: io::Error,
    },
    Serialize {
        source_path: PathBuf,
        output_path: PathBuf,
        source: serde_json::Error,
    },
    Write {
        source_path: PathBuf,
        output_path: PathBuf,
        source: io::Error,
    },
}

impl std::fmt::Display for OutputError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CurrentDirectory(error) => {
                write!(
                    formatter,
                    "failed to resolve the current directory: {error}"
                )
            }
            Self::NoCommonRoot => write!(formatter, "input roots have no common ancestor"),
            Self::SourceOutsideRoot { source, root } => write!(
                formatter,
                "input {} is outside the common root {}",
                source.display(),
                root.display()
            ),
            Self::Collision {
                first_source,
                first_output,
                second_source,
                second_output,
            } => write!(
                formatter,
                "output collision: {} -> {} conflicts with {} -> {}",
                first_source.display(),
                first_output.display(),
                second_source.display(),
                second_output.display()
            ),
            Self::CreateDirectory { path, source } => write!(
                formatter,
                "failed to create output directory {}: {source}",
                path.display()
            ),
            Self::Serialize {
                source_path,
                output_path,
                source,
            } => write!(
                formatter,
                "failed to serialize {} -> {}: {source}",
                source_path.display(),
                output_path.display()
            ),
            Self::Write {
                source_path,
                output_path,
                source,
            } => write!(
                formatter,
                "failed to write {} -> {}: {source}",
                source_path.display(),
                output_path.display()
            ),
        }
    }
}

pub(super) fn write_outputs<'a>(
    outputs: impl IntoIterator<Item = CompiledBuildOutput<'a>>,
    output_directory: &Path,
    format: OutputFormat,
    script_extension: ScriptExtension,
) -> Result<(), OutputError> {
    let outputs: Vec<_> = outputs.into_iter().collect();
    let paths: Vec<_> = outputs
        .iter()
        .map(|compiled| {
            let extension = match format {
                OutputFormat::Js => {
                    get_output_extension(&compiled.output.script_lang, script_extension)
                }
                OutputFormat::Json => "json",
                OutputFormat::Stats => unreachable!(),
            };
            OutputPath {
                source: &compiled.input.source,
                output: destination(compiled.input, output_directory, extension),
            }
        })
        .collect();
    validate_output_paths(&paths, output_directory)?;

    // Serialize every output before mutating the destination tree. Besides
    // surfacing serialization failures, this keeps them atomic with preflight.
    let prepared: Result<Vec<_>, OutputError> = outputs
        .into_iter()
        .zip(paths)
        .map(|(compiled, path)| {
            let content: String = match format {
                OutputFormat::Js => compiled.output.code,
                OutputFormat::Json =>
                {
                    #[allow(clippy::disallowed_methods)]
                    serde_json::to_string_pretty(&compiled.output)
                        .map_err(|source| OutputError::Serialize {
                            source_path: compiled.input.source.clone(),
                            output_path: path.output.clone(),
                            source,
                        })?
                        .into()
                }
                OutputFormat::Stats => unreachable!(),
            };
            Ok((compiled.input, path.output, content))
        })
        .collect();
    let prepared = prepared?;

    let mut directories: Vec<_> = prepared
        .iter()
        .filter_map(|(_, output, _)| output.parent().map(Path::to_path_buf))
        .collect();
    directories.sort();
    directories.dedup();
    for directory in directories {
        match profile!(
            "cli.build.output.create_dir_all",
            fs::create_dir_all(&directory)
        ) {
            Ok(()) => global_profiler().record_fs_create_dir_all(),
            Err(source) => {
                global_profiler().record_fs_create_dir_all_failure();
                return Err(OutputError::CreateDirectory {
                    path: directory,
                    source,
                });
            }
        }
    }

    let stderr = io::stderr();
    let mut report = io::BufWriter::new(stderr.lock());
    let mut first_write_error = None;
    for (input, output, content) in prepared {
        let bytes = content.len();
        match profile!(
            "cli.build.output.write",
            fs::write(&output, content.as_str())
        ) {
            Ok(()) => {
                global_profiler().record_fs_write(bytes);
                let _ = writeln!(
                    report,
                    "Built: {} -> {}",
                    input.source.display(),
                    output.display()
                );
            }
            Err(source) => {
                global_profiler().record_fs_write_failure(bytes);
                let _ = writeln!(
                    report,
                    "Failed to write {} -> {}: {source}",
                    input.source.display(),
                    output.display()
                );
                let error = OutputError::Write {
                    source_path: input.source.clone(),
                    output_path: output,
                    source,
                };
                first_write_error.get_or_insert(error);
            }
        }
    }
    let _ = report.flush();
    first_write_error.map_or(Ok(()), Err)
}

#[cfg(test)]
mod tests;
