//! Strict real dependency setup for the original held-Hover process fixture.

use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::json;
use vize_carton::corsa_resolver::discover_corsa_in_ancestors;

pub const ORIGINAL: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/canceled-native-hover/SlotAuthoring.vue.txt"
);
pub const SYNTAX: &str = "<template>\n  <p>ready</p>\n</template>\n";
const CONFIG: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/canceled-native-hover/tsconfig.json"
);
const PROXY: &str = include_str!(
    "../../../../tests/_fixtures/differential/lsp/canceled-native-hover/held-native.js"
);

pub struct Project(tempfile::TempDir);
impl Project {
    pub fn new(source: &str) -> Self {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        let runtime = discover_corsa_in_ancestors(workspace)
            .expect("held hover requires the actual workspace native runtime");
        let vue = workspace
            .join("playground/node_modules/vue")
            .canonicalize()
            .expect("held hover requires the frozen Playground Vue dependency");
        let cases = workspace.join("target/vize-tests/tests");
        std::fs::create_dir_all(&cases).unwrap();
        let root = tempfile::Builder::new()
            .prefix("lsp-held-hover-")
            .tempdir_in(cases)
            .unwrap();
        std::fs::create_dir(root.path().join("src")).unwrap();
        std::fs::create_dir(root.path().join("node_modules")).unwrap();
        std::os::unix::fs::symlink(vue, root.path().join("node_modules/vue")).unwrap();
        std::fs::write(root.path().join("src/SlotAuthoring.vue"), source).unwrap();
        std::fs::write(root.path().join("src/Syntax.vue"), SYNTAX).unwrap();
        std::fs::write(root.path().join("tsconfig.json"), CONFIG).unwrap();
        let gate = root.path().join("gate");
        std::fs::create_dir(&gate).unwrap();
        // The repository is ESM; this copied native launcher uses CommonJS.
        std::fs::write(gate.join("package.json"), r#"{"type":"commonjs"}"#).unwrap();
        let proxy = gate.join("tsgo.js");
        std::fs::write(&proxy, PROXY).unwrap();
        std::fs::set_permissions(&proxy, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(
            gate.join("gate.json"),
            serde_json::to_vec(&json!({"runtime":runtime,"root":gate})).unwrap(),
        )
        .unwrap();
        std::fs::write(
            root.path().join("vize.config.json"),
            serde_json::to_vec(&json!({
                "typeChecker":{"corsaPath":proxy,"checkFallthroughAttrs":false},
                "lsp":{"typecheck":true,"lint":false,"hover":true}
            }))
            .unwrap(),
        )
        .unwrap();
        Self(root)
    }

    pub fn root(&self) -> &Path {
        self.0.path()
    }
    pub fn gate(&self) -> std::path::PathBuf {
        self.root().join("gate")
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        // A proxy terminated by the owning server cannot catch SIGKILL; always
        // reap its recorded real children rather than leaking a native process.
        if let Ok(pids) = std::fs::read_to_string(self.gate().join("native.pids")) {
            let _ = std::process::Command::new("kill")
                .args(pids.lines())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
    }
}

pub fn await_file(path: &Path) {
    let started = Instant::now();
    while !path.is_file() {
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "real native gate never entered: {}",
            path.display()
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}
