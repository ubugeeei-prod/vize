#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/nuxt_cli.rs"]
mod nuxt_cli;
#[path = "support/nuxt_fifo.rs"]
mod nuxt_fifo;
#[path = "support/nuxt_stress.rs"]
mod nuxt_stress;

#[cfg(unix)]
#[path = "check_nuxt_tsconfig_isolation_cli/barrier.rs"]
mod barrier;

#[cfg(unix)]
mod unix {
    use std::{
        fs,
        path::{Path, PathBuf},
        process::{Command, Output, Stdio},
    };

    use super::{
        barrier::{await_phase, create_phase_barrier, release},
        corsa_requirement,
        nuxt_cli::resolve_test_corsa_path,
        nuxt_stress::required_iterations,
    };

    #[test]
    fn concurrent_projects_never_share_a_nuxt_checker_config() {
        let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path())
        else {
            return;
        };
        let case = tempfile::tempdir().unwrap();
        let shared_node_modules = case.path().join("shared-node_modules");
        fs::create_dir(&shared_node_modules).unwrap();
        let alpha = create_project(
            case.path(),
            &shared_node_modules,
            "alpha",
            "string",
            "\"alpha\"",
        );
        let bravo = create_project(case.path(), &shared_node_modules, "bravo", "number", "42");
        let legacy_wrapper = shared_node_modules.join(".vize/cli/tsconfig.nuxt-fallback.json");
        let legacy_before = fs::read(&legacy_wrapper).ok();
        let iterations = required_iterations();

        for iteration in 0..iterations {
            let barrier = case.path().join(format!("barrier-{iteration}"));
            let prepared_barrier = barrier.join("prepared");
            let active_barrier = barrier.join("active");
            create_phase_barrier(&prepared_barrier);
            create_phase_barrier(&active_barrier);

            let alpha_child = check_command(&alpha, &corsa_path, &barrier, "alpha", true, 1)
                .spawn()
                .unwrap();
            let bravo_child = check_command(&bravo, &corsa_path, &barrier, "bravo", false, 2)
                .spawn()
                .unwrap();
            let (alpha_child, bravo_child) =
                await_phase(&prepared_barrier, alpha_child, bravo_child);
            release(&prepared_barrier, "alpha");
            release(&prepared_barrier, "bravo");
            let (alpha_child, bravo_child) = await_phase(&active_barrier, alpha_child, bravo_child);
            if iteration + 1 == iterations {
                release(&active_barrier, "alpha");
                assert_clean(alpha_child.wait_with_output().unwrap(), "alpha", iteration);
                fs::remove_dir_all(&alpha).unwrap();
                release(&active_barrier, "bravo");
                assert_clean(bravo_child.wait_with_output().unwrap(), "bravo", iteration);
            } else {
                release(&active_barrier, "alpha");
                release(&active_barrier, "bravo");
                assert_clean(alpha_child.wait_with_output().unwrap(), "alpha", iteration);
                assert_clean(bravo_child.wait_with_output().unwrap(), "bravo", iteration);
            }
        }

        assert_logical_and_physical_spelling_checks(case.path(), &bravo, &corsa_path);
        assert_missing_alias_reports_authored_ts2307(&bravo, &corsa_path);
        assert_eq!(
            fs::read(legacy_wrapper).ok(),
            legacy_before,
            "the check must not create or mutate generated state under shared node_modules"
        );
    }

    fn create_project(
        case: &Path,
        shared_node_modules: &Path,
        name: &str,
        expected: &str,
        value: &str,
    ) -> PathBuf {
        let root = case.join(name);
        fs::create_dir_all(root.join("src")).unwrap();
        std::os::unix::fs::symlink(shared_node_modules, root.join("node_modules")).unwrap();
        let dependencies = if name == "alpha" {
            r#""nuxt": "2.17.0", "@nuxt/bridge": "3.0.0""#
        } else {
            r#""nuxt": "3.0.0""#
        };
        write(
            &root.join("package.json"),
            &format!(
                r#"{{ "private": true, "name": "{name}", "dependencies": {{ {dependencies} }} }}"#
            ),
        );
        write(
            &root.join("nuxt.config.ts"),
            &format!("export default {{ srcDir: \"{name}\" }}\n"),
        );
        let vize_config = if name == "alpha" {
            r#"{ "vue": { "version": "2.7" }, "typeChecker": { "legacyVue2": true, "optionsApi": true } }"#
        } else {
            r#"{ "vue": { "version": "3" }, "typeChecker": { "optionsApi": true } }"#
        };
        write(&root.join("vize.config.json"), vize_config);
        if name == "alpha" {
            write(
                &root.join(".nuxt/tsconfig.json"),
                r##"{
  "compilerOptions": {
    "paths": { "~/*": ["../alpha/*"], "#bridge": ["types/bridge.d.ts"] }
  }
}"##,
            );
            write(
                &root.join(".nuxt/types/bridge.d.ts"),
                "export type BridgeToken = 'bridge-owned';\n",
            );
        }
        write(
            &root.join("tsconfig.json"),
            &format!(
                r##"{{
  "compilerOptions": {{
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  }},
  "include": ["src/**/*.vue", "{name}/**/*.vue"]
}}"##
            ),
        );
        let component = if name == "alpha" {
            "<script lang=\"ts\">\nexport default { props: { value: { type: String, required: true } } };\n</script>\n<template><div>{{ value }}</div></template>\n".to_owned()
        } else {
            format!(
                "<script setup lang=\"ts\">\ndefineProps<{{ value: {expected} }}>();\n</script>\n<template><div>{{{{ value }}}}</div></template>\n"
            )
        };
        write(&root.join(format!("{name}/Component.vue")), &component);
        let app = if name == "alpha" {
            format!(
                "<script lang=\"ts\">\nimport Component from \"~/Component.vue\";\nimport type {{ BridgeToken }} from \"#bridge\";\nexport default {{ components: {{ Component }}, data() {{ const value: {expected} = {value}; const bridge: BridgeToken = 'bridge-owned'; return {{ value, bridge }}; }} }};\n</script>\n<template><Component :value=\"value\" /></template>\n"
            )
        } else {
            format!(
                "<script setup lang=\"ts\">\nimport Component from \"~/Component.vue\";\nconst value: {expected} = {value};\n</script>\n<template><Component :value=\"value\" /></template>\n"
            )
        };
        write(&root.join("src/App.vue"), &app);
        root
    }

    fn check_command(
        project: &Path,
        corsa_path: &Path,
        barrier: &Path,
        participant: &str,
        explicit_tsconfig: bool,
        servers: usize,
    ) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
        command
            .current_dir(project)
            .env("CORSA_PATH", corsa_path)
            .env(
                "VIZE_TEST_NUXT_CONFIG_PREPARED_BARRIER",
                barrier.join("prepared"),
            )
            .env(
                "VIZE_TEST_NUXT_CONFIG_ACTIVE_BARRIER",
                barrier.join("active"),
            )
            .env("VIZE_TEST_NUXT_CONFIG_PARTICIPANT", participant)
            .args([
                "check",
                "src/App.vue",
                "--format",
                "json",
                "--servers",
                &servers.to_string(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if explicit_tsconfig {
            command.args(["--tsconfig", "tsconfig.json"]);
        }
        command
    }

    fn assert_clean(output: Output, project: &str, iteration: usize) {
        let stdout = std::string::String::from_utf8(output.stdout).unwrap();
        let stderr = std::string::String::from_utf8(output.stderr).unwrap();
        assert!(
            output.status.success(),
            "{project} iteration {iteration} failed\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
        let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        assert_eq!(value["errorCount"], 0, "{project}: {stdout}");
    }

    fn assert_missing_alias_reports_authored_ts2307(project: &Path, corsa_path: &Path) {
        write(
            &project.join("src/App.vue"),
            "<script setup lang=\"ts\">\nimport { typed } from \"~/missing\";\nvoid typed;\n</script>\n",
        );
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project)
            .env("CORSA_PATH", corsa_path)
            .args([
                "check",
                "src/App.vue",
                "--tsconfig",
                "tsconfig.json",
                "--format",
                "json",
            ])
            .output()
            .unwrap();
        let stdout = std::string::String::from_utf8(output.stdout).unwrap();
        assert!(!output.status.success(), "{stdout}");
        let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let file = value["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|file| {
                file["file"]
                    .as_str()
                    .is_some_and(|path| path.ends_with("src/App.vue"))
            })
            .expect("the authored Vue file must own the diagnostic");
        let diagnostic = file["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(serde_json::Value::as_str)
            .find(|diagnostic| diagnostic.contains("[TS2307]"))
            .unwrap_or_else(|| panic!("missing authored alias must report TS2307: {stdout}"));
        assert!(diagnostic.starts_with("error:2:"), "{diagnostic}");
        assert!(diagnostic.contains("Cannot find module '~/missing'"));
    }

    fn assert_logical_and_physical_spelling_checks(case: &Path, project: &Path, corsa_path: &Path) {
        let logical = case.join("logical-bravo");
        std::os::unix::fs::symlink(project, &logical).unwrap();
        let physical = fs::canonicalize(project).unwrap();
        assert_ne!(logical, physical);
        for (index, spelling) in [&logical, &physical].into_iter().enumerate() {
            let output = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(case)
                .env("CORSA_PATH", corsa_path)
                .arg("check")
                .arg(spelling.join("src/App.vue"))
                .arg("--tsconfig")
                .arg(spelling.join("tsconfig.json"))
                .arg("--config")
                .arg(spelling.join("vize.config.json"))
                .args(["--format", "json", "--servers", "1"])
                .output()
                .unwrap();
            assert_clean(output, "bravo-path-spelling", index);
        }
    }

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }
}
