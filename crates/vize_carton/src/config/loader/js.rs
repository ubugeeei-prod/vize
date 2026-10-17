//! JavaScript and TypeScript config evaluation.
//!
//! Native CLI commands support the existing Vite JS/TS configuration. Rather
//! than embed a JS runtime, the loader shells out to the local Node runtime,
//! loads the config with the project's Vite bundle loader when installed,
//! calls function exports with the default env, and
//! returns JSON for Rust deserialization.

use std::{
    io::{Error as IoError, ErrorKind},
    path::{Path, PathBuf},
    process::Command,
};

use crate::config::ConfigDocument;

use super::BeforeConfigRead;

/// Evaluate a JS-like config file through Node and deserialize the result.
pub(super) fn parse_js_config_observed(
    path: &Path,
    observed: &mut Vec<PathBuf>,
    complete: &mut bool,
    before_read: &mut BeforeConfigRead<'_>,
) -> Result<ConfigDocument, Box<dyn std::error::Error>> {
    let EvaluatedConfig {
        path: config_path,
        json,
        dependencies,
        policy,
    } = evaluate_js_config_with_dependencies(path, observed, complete, before_read)?;

    // Public Vite settings permit flat entry arrays. Normalize only that shape
    // with the same pure Rust projection used by vize/config; ordinary objects
    // and dedicated config files retain their existing deserialization path.
    if config_path.file_name().is_some_and(|name| {
        super::discovery::CONFIG_FILE_NAMES
            .iter()
            .any(|candidate| name == *candidate)
    }) && json.iter().find(|byte| !byte.is_ascii_whitespace()) == Some(&b'[')
    {
        let value = serde_json::from_slice(&json)?;
        let normalized = crate::config::normalize_public_config_value(value)
            .map_err(|error| IoError::new(ErrorKind::InvalidData, error))?;
        let mut document: ConfigDocument = serde_json::from_value(normalized.clone())?;
        document.set_evaluated_policy(normalized);
        document.set_config_dependencies(dependencies);
        return Ok(document);
    }
    let mut document = super::parse::document_from_json(&json, &config_path)?;
    document.set_evaluated_policy(policy);
    document.set_config_dependencies(dependencies);
    Ok(document)
}

struct EvaluatedConfig {
    path: PathBuf,
    json: Vec<u8>,
    policy: serde_json::Value,
    dependencies: Vec<std::string::String>,
}

fn evaluate_js_config_with_dependencies(
    path: &Path,
    observed: &mut Vec<PathBuf>,
    complete: &mut bool,
    before_read: &mut BeforeConfigRead<'_>,
) -> Result<EvaluatedConfig, Box<dyn std::error::Error>> {
    *complete = false;
    let config_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    if let Some(observe) = before_read.as_deref_mut() {
        observe(std::slice::from_ref(&config_path));
    }
    let config_path = config_path.canonicalize().unwrap_or(config_path);
    if let Some(observe) = before_read.as_deref_mut() {
        observe(std::slice::from_ref(&config_path));
    }
    observed.push(config_path.clone());
    let script = [
        include_str!("vite-runtime.mjs"),
        include_str!("module-runtime.mjs"),
        r#"
import { dirname } from "node:path";

const configPath = process.argv[1];
const observing = process.argv[2] === "--observe-config-inputs";
const acknowledgements = observing ? process.stdin[Symbol.asyncIterator]() : undefined;
const beforeRead = observing ? async (paths) => {
  const payload = Buffer.from(JSON.stringify(paths));
  if (payload.length > 65536) throw new Error("Config observation frame exceeds 64 KiB");
  const frame = Buffer.concat([Buffer.from(`B${payload.length}\n`), payload]);
  await new Promise((resolve, reject) => process.stdout.write(frame, (error) => error ? reject(error) : resolve()));
  const { value: ack, done } = await acknowledgements.next();
  if (done || ack.length !== 1 || ack[0] !== 6) {
    throw new Error("Config observation acknowledgement ended");
  }
} : undefined;
console.log = (...args) => console.error(...args);
let attemptedDependencies = [configPath];
let attemptedComplete = false;
let evaluated;
try {
const { value: config, dependencies } = await importConfigModuleWithDependencies(
  configPath, undefined, undefined, async (module) => {
    const exported = module.default ?? module;
    return isViteConfigFile(configPath)
      ? await resolveViteConfigExport(exported, undefined, dirname(configPath))
      : typeof exported === "function"
      ? await exported({ mode: "development", command: "serve" })
      : exported;
  },
  (paths, complete) => { attemptedDependencies = paths; attemptedComplete = complete; },
  beforeRead,
);
evaluated = { config: config ?? {}, dependencies, watchDependencies: attemptedDependencies, watchDependenciesComplete: true };
} finally {
  if (observing) process.stdout.write("R\n");
  process.stdout.write(JSON.stringify(evaluated ?? { dependencies: attemptedDependencies, watchDependenciesComplete: attemptedComplete }));
}
"#,
    ]
    .concat();
    let mut command = Command::new("node");
    command
        .arg("--input-type=module")
        .arg("-e")
        .arg(script)
        .arg(&config_path)
        .current_dir(config_path.parent().unwrap_or_else(|| Path::new(".")));
    let output = if let Some(observe) = before_read.as_deref_mut() {
        super::js_observation::evaluate_observed(&mut command, observe)?
    } else {
        command.output()?
    };

    #[derive(serde::Deserialize)]
    struct Evaluation {
        #[serde(default)]
        config: serde_json::Value,
        #[serde(default, rename = "watchDependenciesComplete")]
        complete: bool,
        #[serde(default, rename = "watchDependencies")]
        watch_dependencies: Vec<PathBuf>,
        dependencies: Vec<std::string::String>,
    }
    let evaluated = serde_json::from_slice::<Evaluation>(&output.stdout);
    if let Ok(evaluated) = &evaluated {
        *complete = evaluated.complete;
        observed.extend(evaluated.watch_dependencies.iter().cloned());
        observed.extend(evaluated.dependencies.iter().map(PathBuf::from));
    }

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Box::new(IoError::new(
            ErrorKind::InvalidData,
            crate::cstr!("node failed to load config: {}", stderr.trim()).to_string(),
        )));
    }

    let evaluated = evaluated?;
    Ok(EvaluatedConfig {
        path: config_path,
        json: serde_json::to_vec(&evaluated.config)?,
        policy: evaluated.config,
        dependencies: evaluated.dependencies,
    })
}
