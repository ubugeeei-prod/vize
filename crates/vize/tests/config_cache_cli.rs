#![cfg(test)]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! Repeated CLI runs reuse an evaluated Vite config only while every input
//! the evaluation read is unchanged (#10783).

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, SystemTime},
};

use vize_l0::cstr;

const CONFIG: &str = r#"import { appendFileSync, existsSync } from "node:fs";
import { semi } from "./settings.mjs";
import { packaged } from "settings-package";

appendFileSync(new URL("./evaluations.log", import.meta.url), "x");
const flagged = existsSync(new URL("./semi.flag", import.meta.url));

export default {
  vue: {
    formatter: { semi: semi || packaged || flagged || process.env.VIZE_TEST_SEMI === "1" },
  },
};
"#;

const FORMATTED: &str = "Found 1 file(s)\n\nChecked 1 file(s)\n  1 file(s) already formatted\n";
const NEEDS_SEMICOLONS: &str = "Found 1 file(s)\nWould reformat: src/main.ts\n\nChecked 1 file(s)\n  1 file(s) would be reformatted\n";

struct Project {
    root: PathBuf,
}

impl Project {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir()
            .join(cstr!("vize-config-cache-cli-{}-{nonce}", std::process::id()).as_str());
        let project = Self { root };
        fs::create_dir_all(project.root.join("node_modules")).unwrap();
        project.write("package.json", r#"{ "type": "module" }"#);
        project.write("vite.config.mjs", CONFIG);
        project.write("settings.mjs", "export const semi = false;\n");
        project.write("src/main.ts", "export const value = 1\n");
        for name in ["a", "b"] {
            let semi = if name == "a" { "false" } else { "true" };
            project.write(
                cstr!("packages/{name}/package.json").as_str(),
                r#"{ "name": "settings-package", "type": "module", "exports": "./index.mjs" }"#,
            );
            project.write(
                cstr!("packages/{name}/index.mjs").as_str(),
                cstr!("export const packaged = {semi};\n").as_str(),
            );
        }
        project.link_package("a");
        project
    }

    /// Settled inputs are older than the cache's racy-timestamp window.
    fn write(&self, path: &str, content: &str) {
        self.write_recent(path, content);
        let old = SystemTime::now() - Duration::from_secs(30);
        fs::File::options()
            .write(true)
            .open(self.root.join(path))
            .unwrap()
            .set_modified(old)
            .unwrap();
    }

    fn write_recent(&self, path: &str, content: &str) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn link_package(&self, name: &str) {
        let link = self.root.join("node_modules/settings-package");
        let _ = fs::remove_file(&link);
        #[cfg(unix)]
        std::os::unix::fs::symlink(self.root.join("packages").join(name), link).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_dir(self.root.join("packages").join(name), link).unwrap();
    }

    /// Runs `fmt --check` and returns (formatted with semicolons?, evaluations so far).
    fn check(&self, env: &[(&str, &str)]) -> (bool, usize) {
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command
            .current_dir(&self.root)
            .args(["fmt", "--check", "src/main.ts"])
            .env_remove("VIZE_CONFIG_CACHE")
            .env_remove("VIZE_TEST_SEMI");
        for (name, value) in env {
            command.env(name, value);
        }
        let output = command.output().unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        // `export const value = 1` is already formatted only without semicolons.
        let semicolons = match (output.status.code(), stderr.as_ref()) {
            (Some(0), FORMATTED) => false,
            (Some(1), NEEDS_SEMICOLONS) => true,
            result => panic!("unexpected fmt result: {result:?}"),
        };
        let evaluations = fs::read_to_string(self.root.join("evaluations.log"))
            .unwrap_or_default()
            .len();
        (semicolons, evaluations)
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn assert_cache_file(root: &Path) {
    let files = fs::read_dir(root.join("node_modules/.vize/config-cache"))
        .unwrap()
        .count();
    assert_eq!(files, 1, "one cache file per config");
}

#[test]
fn fmt_reuses_config_evaluation_until_a_recorded_input_changes() {
    let project = Project::new();
    assert_eq!(project.check(&[]), (false, 1));
    assert_eq!(project.check(&[]), (false, 1), "unchanged inputs reuse");
    assert_cache_file(&project.root);

    // Each environment keeps its own entry while the files stay unchanged.
    assert_eq!(project.check(&[("VIZE_TEST_SEMI", "1")]), (true, 2), "env");
    assert_eq!(project.check(&[]), (false, 2));
    assert_eq!(project.check(&[("VIZE_TEST_SEMI", "1")]), (true, 2));
    assert_eq!(project.check(&[("VIZE_CONFIG_CACHE", "0")]), (false, 3));

    project.write("settings.mjs", "export const semi = true;\n");
    assert_eq!(project.check(&[]), (true, 4), "imported file");
    project.write("settings.mjs", "export const semi = false;\n");
    assert_eq!(project.check(&[]), (false, 5));

    project.write("semi.flag", "");
    assert_eq!(project.check(&[]), (true, 6), "probed file appeared");
    fs::remove_file(project.root.join("semi.flag")).unwrap();
    assert_eq!(project.check(&[]), (false, 7), "probed file removed");

    // The previous target stays untouched; only the bare import's route moved.
    project.link_package("b");
    assert_eq!(project.check(&[]), (true, 8), "relinked package");
    project.link_package("a");
    assert_eq!(project.check(&[]), (false, 9));
    assert_eq!(project.check(&[]), (false, 9));
    assert_cache_file(&project.root);

    // A just-written input may still change within its timestamp: not stored.
    project.write_recent("settings.mjs", "export const semi = false; // edited\n");
    assert_eq!(project.check(&[]), (false, 10));
    assert_eq!(project.check(&[]), (false, 11), "racy input is not reused");
    project.write("settings.mjs", "export const semi = false; // edited\n");
    assert_eq!(project.check(&[]), (false, 12));
    assert_eq!(project.check(&[]), (false, 12));
}
