//! Small opt-in custody of actual normal-project/cache boundaries, without VTS clones.
use super::{Command, Path, PathBuf, Value, VirtualProject, fs, json};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicUsize, Ordering};

fn failure(errors: &mut Vec<Value>, operation: &str, path: &Path, error: impl std::fmt::Display) {
    errors.push(
        json!({"operation": operation, "path": path.to_string_lossy(), "error": error.to_string()}),
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

fn packet(command: &Command, project_root: &Path, virtual_root: &Path, config: &Path) -> Value {
    let mut errors = Vec::new();
    let program = Path::new(command.get_program());
    let binary = match program.canonicalize() {
        Ok(binary) => Some(binary),
        Err(error) => {
            failure(&mut errors, "canonicalize-native", program, error);
            None
        }
    };
    let native_sha = binary.as_ref().and_then(|binary| match fs::read(binary) {
        Ok(bytes) => Some(vize_l0::cstr!("{:x}", Sha256::digest(bytes))),
        Err(error) => {
            failure(&mut errors, "hash-native", binary, error);
            None
        }
    });
    let config_bytes = match fs::read_to_string(config) {
        Ok(bytes) => Some(bytes),
        Err(error) => {
            failure(&mut errors, "read-requested-config", config, error);
            None
        }
    };
    let config_sha = config_bytes
        .as_ref()
        .map(|bytes| vize_l0::cstr!("{:x}", Sha256::digest(bytes.as_bytes())));
    let project_root = path_value(project_root, &mut errors);
    let virtual_root = path_value(virtual_root, &mut errors);
    let config_path = path_value(config, &mut errors);
    let native_binary = binary.as_ref().map(|path| path_value(path, &mut errors));
    let cwd = command
        .get_current_dir()
        .map(|path| path_value(path, &mut errors));
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
    let packet = json!({
        "schema":"vize.cache-runtime-custody", "version":1, "metadataOnly":true,
        "route":"native-cli", "nativeBinary":native_binary, "nativeSha256":native_sha,
        "actualArgs":arguments, "actualCwd":cwd, "nativePid":null,
        "requestedConfigPath":config_path, "requestedConfigBytes":config_bytes,
        "requestedConfigSha256":config_sha,
        "projectRoot":project_root, "virtualRoot":virtual_root,
        "compiledSourceRevision":option_env!("VIZE_CACHE_OBSERVER_SOURCE_SHA"),
        "crateVersion":env!("CARGO_PKG_VERSION"),
        "authoredConfig":"not read: semantic root can differ from the authored config directory",
        "completion":"PENDING", "errors":errors,
        "unobserved":["native PID", "native engine normalized options", "native engine loaded source-file vector"],
    });
    packet
}

pub(super) fn requested(
    command: &Command,
    project: &VirtualProject,
    config: &Path,
    directory: &Path,
) -> Option<(PathBuf, Value)> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let packet = packet(
        command,
        project.project_root(),
        project.virtual_root(),
        config,
    );
    let path = directory.join(vize_l0::cstr!("cache-{sequence:03}-native-cli.json").as_str());
    if let Err(error) = fs::create_dir_all(directory).and_then(|()| {
        fs::write(
            &path,
            serde_json::to_vec_pretty(&packet).expect("JSON Value serialization"),
        )
    }) {
        eprintln!(
            "Vize cache custody unavailable at {}: {error}",
            path.display()
        );
        return None;
    }
    Some((path, packet))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(
            result["errors"]
                .as_array()
                .unwrap()
                .iter()
                .any(|error| error["operation"] == "read-requested-config")
        );
        assert!(result["requestedConfigBytes"].is_null());
        assert!(!result.as_object().unwrap().contains_key("generatedFiles"));
        assert!(
            !result
                .as_object()
                .unwrap()
                .contains_key("authoredConfigBytes")
        );
        assert_eq!(
            result["compiledSourceRevision"],
            json!(option_env!("VIZE_CACHE_OBSERVER_SOURCE_SHA"))
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
