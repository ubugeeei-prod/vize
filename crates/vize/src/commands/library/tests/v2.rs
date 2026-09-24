//! `init`, `add`, `outdated`, and third-party namespaces.

use std::fs;

use vize_s0::String;

use super::fixture::{Item, Project, ui_v1, ui_v2, write_registry_as};
use super::{ID_TS, lockfile};

const BUTTON: &str = "export const button = \"acme\";\n";

fn acme(dir: &std::path::Path) {
    write_registry_as(
        dir,
        "@acme/ui",
        "ui",
        "1.0.0",
        &[Item::new("button", &[("button/button.ts", BUTTON)], &[])],
    );
}

#[test]
fn init_detects_the_layout_and_writes_the_lib_section() {
    let project = Project::new();
    fs::create_dir_all(project.path("src")).unwrap();
    project.write(
        "tsconfig.json",
        r#"{ "compilerOptions": { "strict": true } }"#,
    );

    let preview = project.json(&[], &["init", "--dry-run"]);
    assert_eq!(preview["action"], "create");
    assert_eq!(preview["sourceDir"], "src");
    assert_eq!(preview["lib"]["uiDir"], "src/components/vize");
    assert!(
        preview["hints"][0]
            .as_str()
            .unwrap()
            .contains("allowImportingTsExtensions")
    );
    assert!(!project.path("vize.config.json").exists());

    project.run(&["init", "--ui-dir", "src/ui"]).unwrap();
    let config: serde_json::Value =
        serde_json::from_str(&project.read("vize.config.json")).unwrap();
    assert_eq!(config["lib"]["uiDir"], "src/ui");
    assert_eq!(config["lib"]["composableDir"], "src/composables/vize");

    let again = project.json(&[], &["init"]);
    assert_eq!(again["action"], "unchanged");
}

#[test]
fn init_appends_to_existing_json_config_and_defers_on_ts_config() {
    let project = Project::new();
    project.write(
        "vize.config.json",
        "{\n  \"formatter\": { \"semi\": false }\n}\n",
    );
    project.run(&["init"]).unwrap();
    let text = project.read("vize.config.json");
    assert!(
        text.starts_with("{\n  \"formatter\": { \"semi\": false },\n  \"lib\": {"),
        "{text}"
    );

    let ts = Project::new();
    ts.write("vize.config.ts", "export default {};\n");
    let report = ts.json(&[], &["init"]);
    assert_eq!(report["action"], "manual");
    assert!(
        report["hints"][0]
            .as_str()
            .unwrap()
            .contains("uiDir: \"src/components/vize\"")
    );
    assert_eq!(ts.read("vize.config.ts"), "export default {};\n");
}

#[cfg(unix)]
#[test]
fn init_rejects_symlinked_config_outside_project() {
    use std::os::unix::fs::symlink;

    let project = Project::new();
    let outside = tempfile::tempdir().unwrap();
    let target = outside.path().join("vize.config.json");
    let original = "{ \"formatter\": { \"semi\": false } }\n";
    fs::write(&target, original).unwrap();
    symlink(&target, project.path("vize.config.json")).unwrap();

    let error = project.run(&["init", "--force"]).unwrap_err();
    assert!(error.message().contains("symbolic link"), "{error}");
    assert_eq!(fs::read_to_string(target).unwrap(), original);
}

#[test]
fn add_is_a_shadcn_compatible_alias_of_pull() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));

    project
        .run_with(&["ui"], &["add", "id", "-y", "--path", "src/vendor"])
        .unwrap();

    assert!(project.path("src/vendor/foundations/id/id.ts").is_file());
}

#[test]
fn outdated_compares_lockfile_versions_with_the_registry() {
    let project = Project::new();
    ui_v1(&project.registry("v1"));
    ui_v2(&project.registry("v2"));
    project.run_with(&["v1"], &["pull", "rating"]).unwrap();

    let report = project.json(&["v2"], &["--offline", "outdated"]);
    let items = report["items"].as_array().unwrap();
    let rating = items
        .iter()
        .find(|item| item["item"] == "ui:rating")
        .unwrap();
    assert_eq!(rating["current"], "1.0.0");
    assert_eq!(rating["wanted"], "2.0.0");
    assert_eq!(rating["state"], "update-available");
    let id = items.iter().find(|item| item["item"] == "ui:id").unwrap();
    assert_eq!(id["state"], "up-to-date");

    let table = project
        .run_with(&["v2"], &["--offline", "outdated"])
        .unwrap();
    assert!(table.contains("ui:rating"), "{table}");
    assert!(!table.contains("ui:id "), "{table}");
    let fresh = project
        .run_with(&["v1"], &["--offline", "outdated"])
        .unwrap();
    assert!(fresh.contains("are up to date"), "{fresh}");
}

#[test]
fn pulls_from_a_local_namespace_registry() {
    let project = Project::new();
    acme(&project.path("registries/acme"));
    project.write(
        "vize.config.json",
        r#"{ "lib": { "registries": { "@acme": { "source": "./registries/acme", "dir": "src/acme" } } } }"#,
    );

    let listed = project.json(&[], &["--offline", "list", "--kind", "@acme"]);
    assert_eq!(listed["items"][0]["source"], "@acme");

    project.run(&["--offline", "pull", "@acme/button"]).unwrap();
    assert_eq!(project.read("src/acme/button/button.ts"), BUTTON);
    let lock = lockfile(&project);
    let button = lock.get("@acme", "button").unwrap();
    assert_eq!(button.package, "@acme/ui");
    assert_eq!(button.label(), "@acme/button");

    let status = project.run(&["--offline", "status"]).unwrap();
    assert!(status.contains("@acme/button 1.0.0 clean"), "{status}");
    project
        .run(&["--offline", "remove", "@acme/button"])
        .unwrap();
    assert!(!project.path("src/acme/button").exists());
}

#[test]
fn rejects_unknown_namespaces_and_invalid_third_party_registries() {
    let project = Project::new();
    acme(&project.path("registries/acme"));
    project.write(
        "vize.config.json",
        r#"{ "lib": { "registries": { "@acme": "./registries/acme" } } }"#,
    );
    let unknown = project
        .run(&["--offline", "pull", "@other/button"])
        .unwrap_err();
    assert!(
        unknown
            .message()
            .contains("unknown registry namespace @other"),
        "{unknown}"
    );

    let manifest_path = project.path("registries/acme/registry.json");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    fs::write(
        &manifest_path,
        manifest.replace("\"role\": \"module\"", "\"role\": \"binary\""),
    )
    .unwrap();
    let role = project
        .run(&["--offline", "pull", "@acme/button"])
        .unwrap_err();
    assert!(role.message().contains("unknown file role"), "{role}");

    fs::write(
        &manifest_path,
        manifest.replace("\"size\"", "\"extra\": 1, \"size\""),
    )
    .unwrap();
    let extra = project
        .run(&["--offline", "pull", "@acme/button"])
        .unwrap_err();
    assert!(extra.message().contains("unknown field"), "{extra}");

    fs::write(
        &manifest_path,
        manifest.replace(
            "\"registryDependencies\": []",
            "\"registryDependencies\": [\"ghost\"]",
        ),
    )
    .unwrap();
    let ghost = project
        .run(&["--offline", "pull", "@acme/button"])
        .unwrap_err();
    assert!(
        ghost
            .message()
            .contains("unknown registry dependency ghost"),
        "{ghost}"
    );

    let bad_key = Project::new();
    bad_key.write(
        "vize.config.json",
        r#"{ "lib": { "registries": { "acme": "./x" } } }"#,
    );
    assert!(
        bad_key
            .run(&["list"])
            .unwrap_err()
            .message()
            .contains("must be an @namespace")
    );
}

#[test]
fn namespaced_items_cannot_overwrite_files_owned_by_other_items() {
    let project = Project::new();
    ui_v1(&project.registry("ui"));
    write_registry_as(
        &project.path("registries/evil"),
        "@evil/ui",
        "ui",
        "1.0.0",
        &[Item::new(
            "id",
            &[("foundations/id/id.ts", "export const hijacked = true;\n")],
            &[],
        )],
    );
    project.write(
        "vize.config.json",
        r#"{ "lib": { "registries": { "@evil": { "source": "./registries/evil", "dir": "src/components/vize" } } } }"#,
    );
    project.run_with(&["ui"], &["pull", "ui:id"]).unwrap();

    let error = project
        .run_with(&["ui"], &["pull", "@evil/id", "--overwrite"])
        .unwrap_err();

    assert!(
        error.message().contains("belongs to pulled item ui:id"),
        "{error}"
    );
    assert_eq!(project.read(ID_TS), super::fixture::ID_V1);

    // A fresh lockfile cannot expose collisions between two sources planned
    // in the same invocation; reject the entire plan before any write.
    let fresh = Project::new();
    ui_v1(&fresh.registry("ui"));
    write_registry_as(
        &fresh.path("registries/evil"),
        "@evil/ui",
        "ui",
        "1.0.0",
        &[Item::new(
            "id",
            &[("foundations/id/id.ts", "export const hijacked = true;\n")],
            &[],
        )],
    );
    fresh.write(
        "vize.config.json",
        r#"{ "lib": { "registries": { "@evil": { "source": "./registries/evil", "dir": "src/components/vize" } } } }"#,
    );
    let error = fresh
        .run_with(&["ui"], &["pull", "ui:id", "@evil/id", "--overwrite"])
        .unwrap_err();
    assert!(error.message().contains("both target"), "{error}");
    assert!(!fresh.path(ID_TS).exists());
    assert!(!fresh.path("vize-lib.lock.json").exists());
}

#[cfg(unix)]
fn fake_tool(project: &Project, name: &str, body: &str) -> String {
    use std::os::unix::fs::PermissionsExt;
    let path = project.registry(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, vize_s0::cstr!("#!/bin/sh\n{body}").as_str()).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    String::from(path.to_str().unwrap())
}

#[cfg(unix)]
#[test]
fn pulls_from_npm_and_https_namespaces_with_verified_hashes() {
    let project = Project::new();
    acme(&project.registry("packed/package/registry"));
    acme(&project.registry("served"));
    let npm = fake_tool(
        &project,
        "fake-npm.sh",
        &vize_s0::cstr!(
            "if [ \"$1\" = view ]; then echo 1.1.0; exit 0; fi\ntar -czf \"$4/acme-ui-1.0.0.tgz\" -C '{}' package\n",
            project.registry("packed").display()
        ),
    );
    // curl ... --output <target> <url>: serve https://acme.test/r/<path> from registries/served.
    let served = project.registry("served");
    let curl = fake_tool(
        &project,
        "fake-curl.sh",
        &vize_s0::cstr!(
            "case \" $* \" in *\" --proto-redir =https \"*) ;; *) exit 97 ;; esac\nfor a; do last=\"$a\"; done\nwhile [ \"$1\" != --output ]; do shift; done\ncp \"{}/${{last#https://acme.test/r/}}\" \"$2\"\n",
            served.display()
        ),
    );
    project.write(
        "vize.config.json",
        r#"{ "lib": { "registries": {
            "@npmco": { "source": "npm:@acme/ui@1.0.0", "dir": "src/npmco" },
            "@web": { "source": "https://acme.test/r/registry.json", "dir": "src/web" }
        } } }"#,
    );

    project
        .run(&[
            "--npm",
            &npm,
            "--curl",
            &curl,
            "pull",
            "@npmco/button",
            "@web/button",
        ])
        .unwrap();

    assert_eq!(project.read("src/npmco/button/button.ts"), BUTTON);
    assert_eq!(project.read("src/web/button/button.ts"), BUTTON);
    let report = project.json(&[], &["--npm", &npm, "--curl", &curl, "outdated"]);
    let npmco = report["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["item"] == "@npmco/button")
        .unwrap()
        .clone();
    assert_eq!(npmco["latest"], "1.1.0");
    assert_eq!(npmco["state"], "newer-release");

    // A served file that no longer matches its digest is rejected.
    fs::write(
        project.registry("served/files/button/button.ts"),
        "tampered",
    )
    .unwrap();
    fs::remove_file(project.path("src/web/button/button.ts")).unwrap();
    let error = project
        .run(&["--curl", &curl, "update", "@web/button"])
        .unwrap_err();
    assert!(
        error
            .message()
            .contains("does not match its registry digest"),
        "{error}"
    );
}
