//! Complete raw process and separate mapper-protocol companion receipts.
#![expect(
    clippy::unwrap_used,
    reason = "post-capture fixture evidence persistence"
)]

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use vize_l0::{String, cstr};

pub(super) fn hash(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(64);
    for byte in Sha256::digest(bytes) {
        result.push_str(cstr!("{byte:02x}").as_str());
    }
    result
}

pub(super) fn save(path: &Path, packet: &Value) {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    serde_json::to_writer(&mut file, packet).unwrap();
    file.flush().unwrap();
    file.sync_all().unwrap();
}

pub(super) fn process(command: &mut Command, input: Option<&[u8]>) -> Value {
    command
        .stdin(if input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            return json!({"status":null,"signal":null,"stdoutBytes":null,"stderrBytes":null,"stdoutUtf8":null,"stderrUtf8":null,"error":cstr!("{error}"),"stage":"spawn"});
        }
    };
    let pid = child.id();
    let input_error = input.and_then(|bytes| {
        child.stdin.take().map_or_else(
            || Some(String::from("missing piped stdin")),
            |mut stdin| stdin.write_all(bytes).err().map(|error| cstr!("{error}")),
        )
    });
    match child.wait_with_output() {
        Ok(output) => {
            #[cfg(unix)]
            let signal = {
                use std::os::unix::process::ExitStatusExt;
                output.status.signal()
            };
            #[cfg(not(unix))]
            let signal: Option<i32> = None;
            json!({"status":output.status.code(),"osStatus":cstr!("{}", output.status),"signal":signal,"pid":pid,
                "stdoutBytes":output.stdout,"stderrBytes":output.stderr,
                "stdoutUtf8":std::str::from_utf8(&output.stdout).ok(),"stderrUtf8":std::str::from_utf8(&output.stderr).ok(),
                "stdoutUtf8Error":std::str::from_utf8(&output.stdout).err().map(|error| cstr!("{error}")),
                "stderrUtf8Error":std::str::from_utf8(&output.stderr).err().map(|error| cstr!("{error}")),
                "error":input_error,"stage":"wait-complete"})
        }
        Err(error) => {
            json!({"status":null,"signal":null,"pid":pid,"stdoutBytes":null,"stderrBytes":null,"error":cstr!("{error}"),"inputError":input_error,"stage":"wait"})
        }
    }
}

fn image(path: &Path) -> Value {
    match std::fs::canonicalize(path)
        .and_then(|physical| std::fs::read(&physical).map(|bytes| (physical, bytes)))
    {
        Ok((physical, bytes)) => {
            json!({"requestedPath":path,"physicalPath":physical,"bytes":bytes.len(),"sha256":hash(&bytes)})
        }
        Err(error) => json!({"requestedPath":path,"error":cstr!("{error}")}),
    }
}

pub(super) fn provider(tsgo: &Path, project: &Path) -> Value {
    let vue_manifest = project.join("node_modules/vue/package.json");
    let vue_bytes = std::fs::read(&vue_manifest);
    json!({
        "checkerImage":image(tsgo),
        "checkerVersionCompanion":process(Command::new(tsgo).current_dir(project).arg("--version"), None),
        "sourceCliImage":image(Path::new(env!("CARGO_BIN_EXE_vize"))),
        "sourceCliVersionCompanion":process(Command::new(env!("CARGO_BIN_EXE_vize")).current_dir(project).arg("--version"), None),
        "vueManifestImage":image(&vue_manifest),"vueManifestBytes":vue_bytes.as_ref().ok(),
        "vueManifestError":vue_bytes.err().map(|error|cstr!("{error}")),
        "sourceMapperManifest":std::fs::read_to_string(project.join("node_modules/vize/package.json")).ok(),
        "githubSource":std::env::var("GITHUB_SHA").ok(),
        "githubRun":std::env::var("GITHUB_RUN_ID").ok(),
        "githubAttempt":std::env::var("GITHUB_RUN_ATTEMPT").ok(),
        "scope":"Source CLI mapper manifest plus actual installed Vue; reported npm/Windows/Node tuple remains separate.",
    })
}

fn decode_frames(bytes: &[u8]) -> Result<Vec<Value>, String> {
    let mut at = 0;
    let mut responses = Vec::new();
    while at < bytes.len() {
        let end = bytes[at..]
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .ok_or_else(|| String::from("missing frame header terminator"))?
            + at;
        let header = std::str::from_utf8(&bytes[at..end]).map_err(|error| cstr!("{error}"))?;
        let length = header
            .strip_prefix("Content-Length: ")
            .ok_or_else(|| String::from("unknown frame header"))?
            .parse::<usize>()
            .map_err(|error| cstr!("{error}"))?;
        let start = end + 4;
        let body = bytes
            .get(start..start + length)
            .ok_or_else(|| String::from("truncated response body"))?;
        responses.push(serde_json::from_slice(body).map_err(|error| cstr!("{error}"))?);
        at = start + length;
    }
    Ok(responses)
}

pub(super) fn mapper_companion(project: &Path, files: &BTreeMap<String, String>) -> Value {
    let config: Value = match serde_json::from_str(&files["tsconfig.json"]) {
        Ok(config) => config,
        Err(error) => {
            return json!({"protocolError":cstr!("{error}"),"process":null,"responses":null});
        }
    };
    let mut requests = vec![
        json!({"jsonrpc":"2.0","id":0,"method":"initialize","params":{"positionEncodings":["utf-8"]}}),
        json!({"jsonrpc":"2.0","id":1,"method":"openProject","params":{"projectHandle":"reported-dynamic-targets","configFileName":project.join("tsconfig.json"),"options":{},"compilerOptions":config["compilerOptions"]}}),
    ];
    for (name, text) in files.iter().filter(|(name, _)| name.ends_with(".vue")) {
        requests.push(json!({"jsonrpc":"2.0","id":requests.len(),"method":"transform","params":{"projectHandle":"reported-dynamic-targets","fileName":project.join(name.as_str()),"content":text}}));
    }
    let mut input = Vec::new();
    for request in &requests {
        let body = serde_json::to_vec(request).unwrap();
        write!(&mut input, "Content-Length: {}\r\n\r\n", body.len()).unwrap();
        input.extend_from_slice(&body);
    }
    let output = process(
        Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project)
            .arg("content-mapper"),
        Some(&input),
    );
    let bytes = output["stdoutBytes"].as_array().and_then(|bytes| {
        bytes
            .iter()
            .map(|byte| byte.as_u64().and_then(|byte| u8::try_from(byte).ok()))
            .collect::<Option<Vec<_>>>()
    });
    let decoded = bytes.as_deref().map(decode_frames);
    let (responses, error) = match decoded {
        Some(Ok(responses))
            if responses.len() == requests.len()
                && responses
                    .iter()
                    .zip(&requests)
                    .all(|(response, request)| response["id"] == request["id"]) =>
        {
            (Some(responses), None)
        }
        Some(Ok(responses)) => (
            Some(responses),
            Some(String::from("response count/order differs from requests")),
        ),
        Some(Err(error)) => (None, Some(error)),
        None => (None, Some(String::from("no complete stdout bytes"))),
    };
    json!({"scope":"Separate source-CLI protocol companion, not the native checker's mapper PID or opened project.","requests":requests,"stdinBytes":input,"process":output,"responses":responses,"protocolError":error})
}
