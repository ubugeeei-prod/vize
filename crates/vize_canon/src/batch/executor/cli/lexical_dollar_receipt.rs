//! Lossless optional custody of the actual normal-batch native CLI invocation.

use super::VirtualProject;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

pub(super) fn requested(
    command: &Command,
    project: &VirtualProject,
    config: &Path,
) -> Option<(PathBuf, Value)> {
    let directory = std::env::var_os("VIZE_LEXICAL_DOLLAR_CAPTURE_DIR").map(PathBuf::from)?;
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let binary = Path::new(command.get_program()).canonicalize().unwrap();
    let bytes = fs::read(&binary).unwrap();
    let digest: vize_l0::String = Sha256::digest(bytes)
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect();
    let packet = json!({
        "route": "native-cli", "nativeBinary": binary, "nativeSha256": digest,
        "actualArgs": command.get_args().map(|arg|arg.to_str().expect("fixture argv UTF8")).collect::<Vec<_>>(),
        "actualCwd": command.get_current_dir(), "nativePid": null,
        "requestedConfigPath": config, "requestedConfigBytes": fs::read_to_string(config).unwrap(),
        "authoredConfigPath": project.project_root().join("tsconfig.json"),
        "authoredConfigBytes": fs::read_to_string(project.project_root().join("tsconfig.json")).unwrap(),
        "projectRoot": project.project_root(), "virtualRoot": project.virtual_root(),
        "generatedFiles": project.virtual_files_sorted().into_iter().map(|file|json!({
            "original":file.original_path,"virtual":file.virtual_path,
            "authoredSource":fs::read_to_string(&file.original_path).unwrap(),
            "code":file.content.as_str(),"sourceMap":vize_l0::cstr!("{:#?}",file.source_map),
        })).collect::<Vec<_>>(),
        "completion": "PENDING",
        "unobserved": ["native PID", "native engine normalized options", "native engine loaded source-file vector"],
    });
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join(vize_l0::cstr!("{sequence:03}-native-cli.json").as_str());
    fs::write(&path, serde_json::to_vec_pretty(&packet).unwrap()).unwrap();
    Some((path, packet))
}

pub(super) fn completed(receipt: Option<(PathBuf, Value)>, output: &std::io::Result<Output>) {
    let Some((path, mut packet)) = receipt else {
        return;
    };
    match output {
        Ok(value) => {
            packet["completion"] = json!("REAPED");
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
    fs::write(path, serde_json::to_vec_pretty(&packet).unwrap()).unwrap();
}
