use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::{Duration, Instant};

use serde_json::json;

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";
const VUE_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_VUE";

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn copy_fixture(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "node_modules" {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_fixture(&source_path, &destination_path);
        } else {
            std::fs::copy(source_path, destination_path).unwrap();
        }
    }
}

fn install_packages(project_root: &Path) {
    let mapper_root = project_root.join("node_modules/vize");
    std::fs::create_dir_all(&mapper_root).unwrap();
    std::fs::write(
        mapper_root.join("package.json"),
        serde_json::to_vec_pretty(&json!({
            "name": "vize",
            "private": true,
            "typescript": {
                "contentMapper": {
                    "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                    "compilerOptions": ["noUnusedLocals"],
                },
            },
        }))
        .unwrap(),
    )
    .unwrap();

    let vue_source = std::env::var_os(VUE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let store = workspace_root().join("node_modules/.pnpm");
            let mut candidates = std::fs::read_dir(&store)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", store.display()))
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().starts_with("vue@3."))
                .map(|entry| entry.path().join("node_modules/vue"))
                .filter(|path| path.join("package.json").is_file())
                .collect::<Vec<_>>();
            candidates.sort();
            candidates.pop().unwrap_or_else(|| {
                panic!(
                    "no Vue 3 package found under {}; set {VUE_ENV}",
                    store.display()
                )
            })
        });
    assert!(vue_source.join("package.json").is_file());
    let vue_target = project_root.join("node_modules/vue");
    #[cfg(unix)]
    std::os::unix::fs::symlink(vue_source, vue_target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(vue_source, vue_target).unwrap();
}

struct WatchProcess {
    child: Child,
    output: String,
    receiver: Receiver<String>,
}

impl WatchProcess {
    fn spawn(tsgo: &Path, project_root: &Path, config: &str) -> Self {
        let mut child = Command::new(tsgo)
            .current_dir(project_root)
            .args([
                "--runExternalCode",
                "--watch",
                "-p",
                config,
                "--pretty",
                "false",
                "--watchFile",
                "FixedPollingInterval",
                "--watchDirectory",
                "FixedPollingInterval",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|error| panic!("failed to run {}: {error}", tsgo.display()));
        let stdout = child.stdout.take().expect("watch stdout");
        let stderr = child.stderr.take().expect("watch stderr");
        let (sender, receiver) = channel();
        for mut stream in [
            Box::new(stdout) as Box<dyn std::io::Read + Send>,
            Box::new(stderr) as Box<dyn std::io::Read + Send>,
        ] {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut buffer = [0; 4096];
                loop {
                    match stream.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(count) => {
                            let chunk = String::from_utf8_lossy(&buffer[..count]).into_owned();
                            if sender.send(chunk).is_err() {
                                break;
                            }
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
                        Err(_) => break,
                    }
                }
            });
        }
        Self {
            child,
            output: String::new(),
            receiver,
        }
    }

    fn wait_for(&mut self, start: usize, label: &str, predicate: impl Fn(&str) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(45);
        while Instant::now() < deadline {
            match self.receiver.recv_timeout(Duration::from_millis(200)) {
                Ok(chunk) => self.output.push_str(&chunk),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if predicate(&self.output[start..]) {
                return;
            }
            if let Some(status) = self.child.try_wait().unwrap() {
                panic!("watch exited before {label}: {status}\n{}", self.output);
            }
        }
        panic!("timed out waiting for {label}\n{}", self.output);
    }

    fn assert_running(&mut self) {
        assert!(
            self.child.try_wait().unwrap().is_none(),
            "watch exited after reaching idle state\n{}",
            self.output
        );
    }
}

impl Drop for WatchProcess {
    fn drop(&mut self) {
        if self.child.try_wait().unwrap().is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

#[test]
fn standard_tsgo_watch_enters_idle_with_authored_mapper_diagnostics() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper watch conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-watch-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_packages(project.path());

    let mut clean = WatchProcess::spawn(&tsgo, project.path(), "tsconfig.json");
    clean.wait_for(0, "clean watch idle state", |output| {
        output.contains("Found 0 errors")
            && output.contains("Watching for file changes")
            && !output.contains("error TS")
    });
    clean.assert_running();
    drop(clean);

    let mut broken = WatchProcess::spawn(&tsgo, project.path(), "tsconfig.error.json");
    broken.wait_for(0, "broken watch idle state", |output| {
        output.contains("errors/Broken.vue")
            && output.contains("TS2322")
            && output.contains("not assignable to type 'number'")
            && output.contains("src/Unused.vue")
            && output.contains("TS6133")
            && output.contains("Watching for file changes")
    });
    broken.assert_running();
}
