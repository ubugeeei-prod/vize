//! Small opt-in custody of actual normal-project/cache boundaries, without VTS clones.
use super::{Command, Output, Path, PathBuf, VirtualProject};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

fn sha256_hex(bytes: &[u8]) -> vize_l0::String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect()
}

fn failure(errors: &mut Vec<Value>, operation: &str, path: &Path, error: impl std::fmt::Display) {
    errors.push(
        json!({"operation": operation, "path": path.to_string_lossy(), "error": vize_l0::cstr!("{error}")}),
    );
}

fn path_value(path: &Path, errors: &mut Vec<Value>) -> Value {
    match path.to_str() {
        Some(value) => json!(value),
        None => {
            failure(
                errors,
                "encode-path",
                path,
                "path is not UTF8; custody is unavailable",
            );
            Value::Null
        }
    }
}

fn absolute_path_value(path: &Path, errors: &mut Vec<Value>) -> Value {
    if !path.is_absolute() {
        failure(
            errors,
            "absolute-path",
            path,
            "relative boundary is unobserved",
        );
        return Value::Null;
    }
    path_value(path, errors)
}

fn write_once(path: &Path, packet: &Value) -> std::io::Result<()> {
    let bytes = serde_json::to_vec_pretty(packet).map_err(std::io::Error::other)?;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(&bytes)
}

fn packet(command: &Command, project_root: &Path, virtual_root: &Path, config: &Path) -> Value {
    let mut errors = Vec::new();
    let program = Path::new(command.get_program());
    let binary = if !program.is_absolute() {
        failure(
            &mut errors,
            "resolve-invoked-program",
            program,
            "relative program resolution is unobserved; this carrier requires an absolute program",
        );
        None
    } else {
        match program.canonicalize() {
            Ok(binary) => Some(binary),
            Err(error) => {
                failure(&mut errors, "canonicalize-invoked-program", program, error);
                None
            }
        }
    };
    let invoked_sha = binary.as_ref().and_then(|binary| match fs::read(binary) {
        Ok(bytes) => Some(sha256_hex(&bytes)),
        Err(error) => {
            failure(&mut errors, "hash-invoked-program", binary, error);
            None
        }
    });
    let config_bytes = match config.is_absolute().then(|| fs::read_to_string(config)) {
        Some(Ok(bytes)) => Some(bytes),
        Some(Err(error)) => {
            failure(&mut errors, "read-requested-config", config, error);
            None
        }
        None => None,
    };
    let config_sha = config_bytes
        .as_ref()
        .map(|bytes| sha256_hex(bytes.as_bytes()));
    let project_root = absolute_path_value(project_root, &mut errors);
    let virtual_root = absolute_path_value(virtual_root, &mut errors);
    let config_path = absolute_path_value(config, &mut errors);
    let invoked_program = path_value(program, &mut errors);
    let resolved_invoked_program = binary.as_ref().map(|path| path_value(path, &mut errors));
    let cwd = command
        .get_current_dir()
        .map(|path| absolute_path_value(path, &mut errors));
    let arguments: Vec<_> = command
        .get_args()
        .map(|arg| match arg.to_str() {
            Some(value) => json!(value),
            None => {
                errors.push(json!({"operation":"encode-argument","error":"argument is not UTF8"}));
                Value::Null
            }
        })
        .collect();
    let source = option_env!("VIZE_CACHE_OBSERVER_SOURCE_SHA");
    if !source.is_some_and(|sha| {
        sha.len() == 40
            && sha
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    }) {
        errors.push(json!({"operation":"compiled-source-seal","error":"expected exact lowercase40hex source revision"}));
    }
    json!({
        "schema":"vize.cache-runtime-custody", "version":2, "metadataOnly":true, "stage":"requested",
        "timingEligible":false, "observerCost":"untimed whole invoked-program hash and raw output JSON; CPU/RSS are perturbed",
        "route":"invoked-cli-process", "invokedProgram":invoked_program,
        "resolvedInvokedProgram":resolved_invoked_program, "invokedProgramSha256":invoked_sha,
        "actualArgs":arguments, "actualCwd":cwd, "invokedPid":null,
        "completionScope":"invoked process only; it may be a proxy",
        "requestedConfigPath":config_path, "requestedConfigBytes":config_bytes,
        "requestedConfigSha256":config_sha,
        "requestedConfigObservation":"pre-invocation read, not a post-execution reread",
        "projectRoot":project_root, "virtualRoot":virtual_root,
        "compiledSourceRevision":source,
        "crateVersion":env!("CARGO_PKG_VERSION"),
        "authoredConfig":"not read: semantic root can differ from the authored config directory",
        "completion":"PENDING", "errors":errors,
        "unobserved":["invoked PID", "descendant executable/hash/PID", "native engine normalized options",
            "native engine loaded source-file vector", "original tsconfig owner", "editor/context identity"],
    })
}

pub(super) fn requested(
    command: &Command,
    project: &VirtualProject,
    config: &Path,
) -> Option<(PathBuf, Value)> {
    let requested_directory =
        std::env::var_os("VIZE_CACHE_CUSTODY_CAPTURE_DIR").map(PathBuf::from)?;
    static OWNER: OnceLock<Result<(PathBuf, PathBuf), vize_l0::String>> = OnceLock::new();
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let owner = OWNER.get_or_init(|| {
        if !requested_directory.is_absolute() {
            return Err(vize_l0::cstr!("capture directory must be absolute"));
        }
        fs::create_dir_all(&requested_directory)
            .map_err(|error| vize_l0::cstr!("create capture parent: {error}"))?;
        let base = requested_directory
            .canonicalize()
            .map_err(|error| vize_l0::cstr!("canonicalize capture parent: {error}"))?;
        let directory = base.join(vize_l0::cstr!("process-{}", std::process::id()).as_str());
        // Exclusive creation refuses a reused PID directory, including stale captures.
        fs::create_dir(&directory)
            .map_err(|error| vize_l0::cstr!("create fresh process capture: {error}"))?;
        Ok((base, directory))
    });
    let (base, directory) = match owner {
        Ok(owner) => owner,
        Err(error) => {
            eprintln!("Vize cache custody has no fresh process owner: {error}");
            return None;
        }
    };
    if requested_directory.canonicalize().ok().as_ref() != Some(base) {
        eprintln!("Vize cache custody directory changed within one process");
        return None;
    }
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let mut packet = packet(
        command,
        project.project_root(),
        project.virtual_root(),
        config,
    );
    packet["receiptOwnerPid"] = json!(std::process::id());
    packet["receiptSequence"] = json!(sequence);
    let path = directory.join(vize_l0::cstr!("cache-{sequence:03}-invoked-process").as_str());
    if let Err(error) = write_once(&path.with_extension("requested.json"), &packet) {
        eprintln!("Vize cache custody refuses requested receipt replacement: {error}");
        return None;
    }
    Some((path, packet))
}

pub(super) fn completed(receipt: Option<(PathBuf, Value)>, output: &std::io::Result<Output>) {
    let Some((path, mut packet)) = receipt else {
        return;
    };
    packet["stage"] = json!("completed");
    match output {
        Ok(value) => {
            packet["completion"] = json!("INVOKED_PROCESS_REAPED");
            packet["status"] = json!(vize_l0::cstr!("{}", value.status));
            packet["exitCode"] = json!(value.status.code());
            packet["success"] = json!(value.status.success());
            packet["rawStdoutBytes"] = json!(value.stdout);
            packet["rawStderrBytes"] = json!(value.stderr);
        }
        Err(error) => {
            packet["completion"] = json!("IO_ERROR");
            packet["ioError"] = json!(vize_l0::cstr!("{error}"));
        }
    }
    if let Err(error) = write_once(&path.with_extension("completed.json"), &packet) {
        eprintln!("Vize cache custody refuses completed receipt replacement: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has_error(packet: &Value, operation: &str) -> bool {
        let errors = packet["errors"].as_array().unwrap();
        errors.iter().any(|error| error["operation"] == operation)
    }

    #[test]
    fn outside_missing_config_is_explicit_and_never_reads_an_authored_root_config() {
        let mut command = Command::new("/vize-nonexistent-native-custody-control");
        command.args(["--incremental", "--project", "/outside/requested.json"]);
        command.current_dir("/cache-owner");
        let result = packet(
            &command,
            Path::new("/authored-project"),
            Path::new("/cache-owner"),
            Path::new("/outside/requested.json"),
        );
        assert_eq!(result["projectRoot"], "/authored-project");
        assert_eq!(result["virtualRoot"], "/cache-owner");
        assert_eq!(result["actualCwd"], "/cache-owner");
        assert_eq!(
            result["actualArgs"],
            json!(["--incremental", "--project", "/outside/requested.json"])
        );
        assert!(has_error(&result, "read-requested-config"));
        assert!(result["requestedConfigBytes"].is_null());
        let fields = result.as_object().unwrap();
        assert!(!fields.contains_key("generatedFiles"));
        assert!(!fields.contains_key("authoredConfigBytes"));
        assert_eq!(
            result["compiledSourceRevision"],
            json!(option_env!("VIZE_CACHE_OBSERVER_SOURCE_SHA"))
        );
    }

    #[test]
    fn digest_is_exact_lowercase_sha256_hex() {
        assert_eq!(
            sha256_hex(b"abc").as_str(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn relative_invoked_program_is_unknown_instead_of_guessed_from_parent_cwd() {
        let command = Command::new("relative-corsa");
        let result = packet(
            &command,
            Path::new("/authored"),
            Path::new("/cache"),
            Path::new("/missing.json"),
        );
        assert!(result["resolvedInvokedProgram"].is_null());
        assert!(result["invokedProgramSha256"].is_null());
        assert_eq!(result["invokedProgram"], "relative-corsa");
        assert!(has_error(&result, "resolve-invoked-program"));
        assert_eq!(
            result["completionScope"],
            "invoked process only; it may be a proxy"
        );
    }

    #[test]
    fn relative_boundaries_are_unknown_and_never_read_from_parent_cwd() {
        let command = Command::new("/missing-native");
        let result = packet(
            &command,
            Path::new("authored"),
            Path::new("cache"),
            Path::new("config.json"),
        );
        for key in [
            "projectRoot",
            "virtualRoot",
            "requestedConfigPath",
            "requestedConfigBytes",
        ] {
            assert!(result[key].is_null(), "{key}");
        }
        assert!(
            result["errors"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|error| error["operation"] == "absolute-path")
                .count()
                == 3
        );
    }

    #[test]
    fn requested_and_completed_receipts_are_exclusive_and_preserve_pre_invocation_bytes() {
        let root = tempfile::tempdir().unwrap();
        let base = root.path().join("cache-000");
        let requested = base.with_extension("requested.json");
        let completed_path = base.with_extension("completed.json");
        let packet = json!({"stage":"requested", "completion":"PENDING"});
        write_once(&requested, &packet).unwrap();
        let bytes = fs::read(&requested).unwrap();
        let error = Err(std::io::Error::other("intentional launch failure"));
        completed(Some((base.clone(), packet.clone())), &error);
        assert_eq!(fs::read(&requested).unwrap(), bytes);
        let after: Value = serde_json::from_slice(&fs::read(&completed_path).unwrap()).unwrap();
        assert_eq!(after["completion"], "IO_ERROR");
        assert_eq!(after["stage"], "completed");
        assert_eq!(
            write_once(&requested, &packet).unwrap_err().kind(),
            std::io::ErrorKind::AlreadyExists
        );
        assert_eq!(
            write_once(&completed_path, &packet).unwrap_err().kind(),
            std::io::ErrorKind::AlreadyExists
        );
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_boundaries_are_unknown_with_an_error_instead_of_lossy_ownership() {
        use std::os::unix::ffi::OsStringExt;
        let path = PathBuf::from(std::ffi::OsString::from_vec(vec![b'/', 0xff]));
        let mut errors = Vec::new();
        assert!(path_value(&path, &mut errors).is_null());
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0]["operation"], "encode-path");
    }
}
