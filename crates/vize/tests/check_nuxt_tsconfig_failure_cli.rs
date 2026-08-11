#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/nuxt_cli.rs"]
mod nuxt_cli;
#[path = "support/nuxt_fifo.rs"]
mod nuxt_fifo;

#[cfg(unix)]
mod unix {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::{Command, Stdio},
        sync::{Mutex, MutexGuard, OnceLock, PoisonError},
        time::{Duration, Instant},
    };

    use super::{corsa_requirement, nuxt_cli::resolve_test_corsa_path, nuxt_fifo::create_fifo};

    #[test]
    fn failed_corsa_start_leaves_no_pending_config_or_dependency_state() {
        let _serial = failure_test_lock();
        let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path())
        else {
            return;
        };
        let case = tempfile::tempdir().unwrap();
        let cache_home = case.path().join("cache-home");
        let shared_node_modules = case.path().join("shared-node_modules");
        fs::create_dir_all(&cache_home).unwrap();
        fs::create_dir(&shared_node_modules).unwrap();
        let project = case.path().join("nuxt-project");
        fs::create_dir_all(project.join("src")).unwrap();
        std::os::unix::fs::symlink(&shared_node_modules, project.join("node_modules")).unwrap();
        write(
            &project.join("package.json"),
            r#"{ "private": true, "dependencies": { "nuxt": "3.0.0" } }"#,
        );
        write(&project.join("nuxt.config.ts"), "export default {}\n");
        write(
            &project.join("tsconfig.json"),
            r#"{
  "compilerOptions": {
    "strict": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "paths": { "~/*": ["src/*"] }
  }
}"#,
        );
        write(
            &project.join("src/value.ts"),
            "export const value: string = 'owned';\n",
        );
        write(
            &project.join("src/App.vue"),
            "<script setup lang=\"ts\">import { value } from '~/value'; void value;</script>\n",
        );
        let dependency_before = snapshot_tree(&shared_node_modules.join(".vize/cli"));
        let transient_root = transient_root(&cache_home);
        let transient_before = transient_configs(&transient_root);

        let failed = check(&project, Path::new("/usr/bin/false"), &cache_home);
        assert!(!failed.status.success(), "a non-LSP Corsa binary must fail");
        let transient_after = transient_configs(&transient_root);
        assert!(
            transient_after
                .iter()
                .all(|path| transient_before.contains(path)),
            "failed Corsa startup left new pending files or leases: {transient_after:?}"
        );
        assert_eq!(
            snapshot_tree(&shared_node_modules.join(".vize/cli")),
            dependency_before,
            "failed startup must not mutate generated state under shared node_modules"
        );

        let recovered = check(&project, &corsa_path, &cache_home);
        assert!(
            recovered.status.success(),
            "a failed child must not poison the next checker\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&recovered.stdout),
            String::from_utf8_lossy(&recovered.stderr)
        );
        assert_killed_active_checker_does_not_poison_cache(
            &project,
            &corsa_path,
            &cache_home,
            &transient_root,
            &transient_before,
        );

        write(
            &project.join("src/App.vue"),
            "<script setup lang=\"ts\">import { missing } from '~/missing'; void missing;</script>\n",
        );
        let diagnostic_failure = check(&project, &corsa_path, &cache_home);
        assert!(!diagnostic_failure.status.success());
        assert!(String::from_utf8_lossy(&diagnostic_failure.stdout).contains("[TS2307]"));
        let diagnostic_transients = transient_configs(&transient_root);
        assert!(
            diagnostic_transients
                .iter()
                .all(|path| transient_before.contains(path)),
            "diagnostic exit skipped a config lease destructor: {diagnostic_transients:?}"
        );
    }

    #[test]
    fn later_out_of_root_program_drops_an_earlier_nuxt_config_lease() {
        let _serial = failure_test_lock();
        let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path())
        else {
            return;
        };
        let case = tempfile::tempdir().unwrap();
        let cache_home = case.path().join("cache-home");
        let shared_node_modules = case.path().join("shared-node_modules");
        fs::create_dir_all(&cache_home).unwrap();
        fs::create_dir(&shared_node_modules).unwrap();
        let project = case.path().join("nuxt-workspace");
        let inside = project.join("internal/src/App.vue");
        let outside_root = case.path().join("outside");
        let outside = outside_root.join("src/App.vue");
        fs::create_dir_all(inside.parent().unwrap()).unwrap();
        fs::create_dir_all(outside.parent().unwrap()).unwrap();
        std::os::unix::fs::symlink(&shared_node_modules, project.join("node_modules")).unwrap();
        write(
            &project.join("package.json"),
            r#"{ "private": true, "dependencies": { "nuxt": "3.0.0" } }"#,
        );
        write(
            &project.join("nuxt.config.ts"),
            "export default { srcDir: 'internal/src' };\n",
        );
        write(
            &project.join("tsconfig.json"),
            r#"{ "files": [], "references": [{ "path": "./internal" }, { "path": "../outside" }] }"#,
        );
        for root in [project.join("internal"), outside_root] {
            write(
                &root.join("tsconfig.json"),
                r#"{
  "compilerOptions": { "strict": true, "module": "ESNext", "moduleResolution": "bundler" },
  "include": ["src/**/*.vue"]
}"#,
            );
        }
        write(
            &project.join("internal/src/value.ts"),
            "export const value = 1;\n",
        );
        write(
            &inside,
            "<script setup lang=\"ts\">import { value } from '~/value'; void value;</script>\n",
        );
        write(
            &outside,
            "<script setup lang=\"ts\">const value = 1; void value;</script>\n",
        );
        let transient_root = transient_root(&cache_home);
        let transient_before = transient_configs(&transient_root);

        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(&project)
            .env("CORSA_PATH", corsa_path)
            .env("HOME", &cache_home)
            .env("XDG_CACHE_HOME", cache_home.join("cache"))
            .arg("check")
            .arg("--tsconfig")
            .arg("tsconfig.json")
            .arg("internal/src/App.vue")
            .arg(&outside)
            .args(["--format", "json", "--no-config"])
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success(),
            "stdout:\n{}\nstderr:\n{stderr}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(
            stderr.contains("Building Corsa virtual project"),
            "the first Nuxt program did not acquire a config lease: {stderr}"
        );
        assert!(
            stderr.contains("outside project root"),
            "missing later-program validation error: {stderr}"
        );
        let transient_after = transient_configs(&transient_root);
        assert!(
            transient_after
                .iter()
                .all(|path| transient_before.contains(path)),
            "a later program validation exit leaked an earlier config lease: {transient_after:?}"
        );
    }

    fn assert_killed_active_checker_does_not_poison_cache(
        project: &Path,
        corsa_path: &Path,
        cache_home: &Path,
        transient_root: &Path,
        transient_before: &[PathBuf],
    ) {
        let barrier = project.parent().unwrap().join("active-cancellation");
        fs::create_dir(&barrier).unwrap();
        fs::write(barrier.join("ready"), []).unwrap();
        create_fifo(&barrier.join("release-cancelled"));
        let mut child = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project)
            .env("CORSA_PATH", corsa_path)
            .env("HOME", cache_home)
            .env("XDG_CACHE_HOME", cache_home.join("cache"))
            .env("VIZE_TEST_NUXT_CONFIG_ACTIVE_BARRIER", &barrier)
            .env("VIZE_TEST_NUXT_CONFIG_PARTICIPANT", "cancelled")
            .args([
                "check",
                "src/App.vue",
                "--tsconfig",
                "tsconfig.json",
                "--format",
                "json",
                "--no-config",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        let ready = barrier.join("ready");
        let ready_metadata = fs::symlink_metadata(&ready).unwrap();
        assert!(
            ready_metadata.is_file() && !ready_metadata.file_type().is_symlink(),
            "the cancellation barrier must use a regular ready file"
        );
        loop {
            if fs::metadata(&ready).unwrap().len() == 1 {
                break;
            }
            assert!(
                child.try_wait().unwrap().is_none(),
                "checker exited before reaching the active Corsa barrier"
            );
            assert!(
                Instant::now() < deadline,
                "checker did not reach the active Corsa barrier within 30s"
            );
            std::thread::park_timeout(Duration::from_millis(10));
        }
        child.kill().unwrap();
        child.wait().unwrap();

        let recovered = check(project, corsa_path, cache_home);
        assert!(
            recovered.status.success(),
            "a killed active checker poisoned its immutable config\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&recovered.stdout),
            String::from_utf8_lossy(&recovered.stderr)
        );
        let transient_after = transient_configs(transient_root);
        assert!(
            transient_after
                .iter()
                .all(|path| transient_before.contains(path)),
            "a killed checker left a live cache reader: {transient_after:?}"
        );
    }

    fn check(project: &Path, corsa_path: &Path, cache_home: &Path) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project)
            .env("CORSA_PATH", corsa_path)
            .env("HOME", cache_home)
            .env("XDG_CACHE_HOME", cache_home.join("cache"))
            .args([
                "check",
                "src/App.vue",
                "--tsconfig",
                "tsconfig.json",
                "--format",
                "json",
                "--no-config",
            ])
            .output()
            .unwrap()
    }

    fn transient_configs(root: &Path) -> Vec<PathBuf> {
        snapshot_tree(root)
            .into_iter()
            .filter_map(|(path, _)| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| {
                        name.ends_with(".pending")
                            || name.starts_with(".lease-")
                            || name == ".deleting"
                    })
                    .then_some(path)
            })
            .collect()
    }

    fn transient_root(cache_home: &Path) -> PathBuf {
        #[cfg(target_os = "macos")]
        let root = cache_home.join("Library/Caches");
        #[cfg(not(target_os = "macos"))]
        let root = cache_home.join("cache");
        root.join("vize/check/nuxt/v2")
    }

    fn snapshot_tree(root: &Path) -> Vec<(PathBuf, Option<Vec<u8>>)> {
        fn visit(root: &Path, path: &Path, entries: &mut Vec<(PathBuf, Option<Vec<u8>>)>) {
            let Ok(children) = fs::read_dir(path) else {
                return;
            };
            for child in children.filter_map(Result::ok) {
                let child_path = child.path();
                let relative = child_path.strip_prefix(root).unwrap().to_path_buf();
                if child.file_type().is_ok_and(|kind| kind.is_dir()) {
                    entries.push((relative, None));
                    visit(root, &child_path, entries);
                } else {
                    entries.push((relative, fs::read(child_path).ok()));
                }
            }
        }
        let mut entries = Vec::new();
        visit(root, root, &mut entries);
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        entries
    }

    fn write(path: &Path, content: &str) {
        fs::write(path, content).unwrap();
    }

    fn failure_test_lock() -> MutexGuard<'static, ()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }
}
