//! P5-8: `vize check` diagnostics match with and without check-server, and the
//! second check through the server does not initialize the TypeScript project.
#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![cfg(unix)]

use std::{
    io::{BufRead, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use serde_json::{Value, json};
use vize_s0::{
    corsa_resolver::{CorsaResolveRequest, resolve_corsa_executable},
    cstr,
};

#[path = "support/vue_stub.rs"]
mod vue_stub;

const SOURCE: &str = "<script setup lang=\"ts\">\nconst count: string = 0;\n</script>\n";
const EXPECTED: &str = "error:2:7 [TS2322] Type 'number' is not assignable to type 'string'.";

#[test]
fn check_diagnostics_match_with_and_without_the_server_and_the_second_run_does_not_init() {
    let Some(corsa_path) = resolve_corsa() else {
        return;
    };
    let project = create_project();
    let socket_name = cstr!("vize-p5-8b-{}-reuse.sock", std::process::id());
    let socket = std::env::temp_dir().join(socket_name.as_str());
    let _ = std::fs::remove_file(&socket);
    let mut server = CheckServer::spawn(&project, &corsa_path, &socket);

    let idle = rpc(
        &socket,
        json!({"jsonrpc":"2.0","id":1,"method":"session-stats"}),
    );
    assert_eq!(
        idle["result"],
        json!({"projectInits": 0, "liveSessions": 0})
    );

    let direct = run_check(&project, &corsa_path, None);
    let first = run_check(&project, &corsa_path, Some(&socket));
    assert_eq!(diagnostics(&direct), vec![json!(EXPECTED)]);
    assert_eq!(diagnostics(&first), vec![json!(EXPECTED)]);

    let warmed = rpc(
        &socket,
        json!({"jsonrpc":"2.0","id":2,"method":"session-stats"}),
    );
    let inits = warmed["result"]["projectInits"].as_u64().unwrap();
    assert!(inits >= 1, "the first check initializes, got {warmed}");
    assert_eq!(warmed["result"]["liveSessions"], 1);

    let second = run_check(&project, &corsa_path, Some(&socket));
    assert_eq!(diagnostics(&second), vec![json!(EXPECTED)]);
    let again = rpc(
        &socket,
        json!({"jsonrpc":"2.0","id":3,"method":"session-stats"}),
    );
    assert_eq!(again["result"]["projectInits"], inits);
    assert_eq!(again["result"]["liveSessions"], 1);

    server.stop();
    let _ = std::fs::remove_file(&socket);
    let _ = std::fs::remove_dir_all(project);
}

struct CheckServer {
    child: Option<Child>,
}

impl CheckServer {
    fn spawn(project: &Path, corsa_path: &Path, socket: &Path) -> Self {
        let stderr = std::fs::File::create(project.join("server.stderr")).unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_vize"))
            .args([
                "check-server",
                "--socket",
                socket.to_str().unwrap(),
                "--corsa-path",
                corsa_path.to_str().unwrap(),
                "--working-dir",
                project.to_str().unwrap(),
            ])
            .stderr(Stdio::from(stderr))
            .spawn()
            .unwrap();
        let mut server = Self { child: Some(child) };
        let deadline = Instant::now() + Duration::from_secs(15);
        while !socket.exists() {
            if Instant::now() > deadline || server.exited() {
                panic!(
                    "check-server did not listen on {}\n{}",
                    socket.display(),
                    std::fs::read_to_string(project.join("server.stderr")).unwrap_or_default()
                );
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        server
    }

    fn exited(&mut self) -> bool {
        self.child.as_mut().unwrap().try_wait().unwrap().is_some()
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for CheckServer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn rpc(socket: &Path, request: Value) -> Value {
    let mut stream = std::os::unix::net::UnixStream::connect(socket).unwrap();
    writeln!(stream, "{request}").unwrap();
    stream.flush().unwrap();
    let mut line = std::string::String::new();
    std::io::BufReader::new(stream)
        .read_line(&mut line)
        .unwrap();
    serde_json::from_str(&line).unwrap_or_else(|error| panic!("{error}: {line}"))
}

fn run_check(project: &Path, corsa_path: &Path, socket: Option<&Path>) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(project)
        .env("NO_COLOR", "1")
        .env("CORSA_PATH", corsa_path)
        .args(["check", "src/App.vue", "--format", "json", "--corsa-path"])
        .arg(corsa_path);
    if let Some(socket) = socket {
        command.arg("--socket").arg(socket);
    }
    let output = command.output().unwrap();
    assert!(
        !output.stdout.is_empty(),
        "no stdout\nstderr:\n{}",
        std::str::from_utf8(&output.stderr).unwrap_or("")
    );
    output
}

fn diagnostics(output: &std::process::Output) -> Vec<Value> {
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let stderr = std::str::from_utf8(&output.stderr).unwrap();
    let json: Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|error| panic!("{error}\nstdout:\n{stdout}\nstderr:\n{stderr}"));
    assert_eq!(json["errorCount"], 1, "{json}");
    assert_eq!(json["warningCount"], 0, "{json}");
    json["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|file| file["diagnostics"].as_array().unwrap().iter().cloned())
        .collect()
}

fn resolve_corsa() -> Option<PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    let explicit = std::env::var_os("CORSA_PATH").map(PathBuf::from);
    resolve_corsa_executable(CorsaResolveRequest {
        explicit_path: explicit.as_deref(),
        project_root: Some(workspace_root()),
    })
    .ok()
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
}

fn create_project() -> PathBuf {
    let project_name = cstr!("check-server-reuse-{}", std::process::id());
    let project = workspace_root()
        .join("target/vize-tests/tests")
        .join(project_name.as_str());
    let _ = std::fs::remove_dir_all(&project);
    std::fs::create_dir_all(project.join("src")).unwrap();
    vue_stub::install_vue_jsx_type_stub(&project);
    std::fs::write(
        project.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}"#,
    )
    .unwrap();
    std::fs::write(project.join("src/App.vue"), SOURCE).unwrap();
    project
}
