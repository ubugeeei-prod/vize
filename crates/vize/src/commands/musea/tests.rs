use super::{ServeArgs, VITE_BIN_NAMES, create_serve_plan, resolve_vite_binary};
use std::fs;
use std::path::{Path, PathBuf};

fn write_vite_bin(root: &Path) -> PathBuf {
    let bin_dir = root.join("node_modules").join(".bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let vite_bin = bin_dir.join(VITE_BIN_NAMES[0]);
    fs::write(&vite_bin, "").unwrap();
    vite_bin
}

fn write_musea_vite_setup(root: &Path) {
    fs::write(
        root.join("package.json"),
        r#"{
  "devDependencies": {
    "@vizejs/vite-plugin-musea": "0.236.0"
  }
}"#,
    )
    .unwrap();
    fs::write(
        root.join("vite.config.ts"),
        r#"import { musea } from "@vizejs/vite-plugin-musea";

export default {
  plugins: [musea()],
};
"#,
    )
    .unwrap();
}

#[test]
fn resolves_vite_binary_from_project_ancestors() {
    let temp = tempfile::tempdir().unwrap();
    let vite_bin = write_vite_bin(temp.path());
    let nested = temp.path().join("packages").join("app");
    fs::create_dir_all(&nested).unwrap();

    assert_eq!(resolve_vite_binary(&nested), Some(vite_bin));
}

#[test]
fn serve_plan_defaults_to_vite_dev_with_gallery_route() {
    let temp = tempfile::tempdir().unwrap();
    let vite_bin = write_vite_bin(temp.path());
    write_musea_vite_setup(temp.path());

    let plan = create_serve_plan(
        &ServeArgs {
            open: true,
            ..ServeArgs::default()
        },
        temp.path(),
    )
    .unwrap();

    assert_eq!(plan.program, vite_bin);
    assert_eq!(
        plan.args,
        [
            "dev",
            "--host",
            "localhost",
            "--port",
            "6006",
            "--open",
            "/__musea__"
        ]
    );
}

#[test]
fn serve_plan_rejects_missing_musea_vite_plugin_setup() {
    let temp = tempfile::tempdir().unwrap();
    write_vite_bin(temp.path());
    fs::write(
        temp.path().join("vite.config.ts"),
        "export default { plugins: [] }",
    )
    .unwrap();

    let error = create_serve_plan(&ServeArgs::default(), temp.path()).unwrap_err();

    assert!(error.contains("Musea is not configured for this Vite project"));
    assert!(error.contains("@vizejs/vite-plugin-musea"));
    assert!(error.contains("dependency: missing"));
    assert!(error.contains("does not import or call musea()"));
}

#[test]
fn serve_plan_runs_static_build_with_musea_environment() {
    let temp = tempfile::tempdir().unwrap();
    let vite_bin = write_vite_bin(temp.path());
    write_musea_vite_setup(temp.path());

    let plan = create_serve_plan(
        &ServeArgs {
            build: true,
            open: true,
            strict_port: true,
            ..ServeArgs::default()
        },
        temp.path(),
    )
    .unwrap();

    assert_eq!(plan.program, vite_bin);
    assert_eq!(plan.args, ["build"]);
    assert_eq!(plan.env, [("VIZE_MUSEA_STATIC_BUILD".into(), "1".into())]);
}

#[test]
fn serve_plan_supports_strict_port_alias_for_vite() {
    let temp = tempfile::tempdir().unwrap();
    let vite_bin = write_vite_bin(temp.path());
    write_musea_vite_setup(temp.path());

    let plan = create_serve_plan(
        &ServeArgs {
            strict_port: true,
            ..ServeArgs::default()
        },
        temp.path(),
    )
    .unwrap();

    assert_eq!(plan.program, vite_bin);
    assert_eq!(
        plan.args,
        [
            "dev",
            "--host",
            "localhost",
            "--port",
            "6006",
            "--strictPort"
        ]
    );
}

#[test]
fn serve_plan_rejects_nuxt_project_without_direct_vite() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("nuxt.config.ts"), "export default {}").unwrap();
    fs::write(
        temp.path().join("package.json"),
        r#"{
  "dependencies": {
    "@vizejs/nuxt": "0.162.0",
    "nuxt": "4.3.1"
  }
}"#,
    )
    .unwrap();

    let error = create_serve_plan(
        &ServeArgs {
            build: true,
            ..ServeArgs::default()
        },
        temp.path(),
    )
    .unwrap_err();

    assert!(error.contains("detected a Nuxt project"));
    assert!(error.contains("standalone `vize musea` command only runs direct Vite projects"));
    assert!(error.contains("nuxi build"));
    assert!(error.contains("/__musea__/"));
}

#[test]
fn serve_plan_rejects_silent_stories_option() {
    let temp = tempfile::tempdir().unwrap();
    write_vite_bin(temp.path());

    let error = create_serve_plan(
        &ServeArgs {
            stories: Some(PathBuf::from("stories")),
            ..ServeArgs::default()
        },
        temp.path(),
    )
    .unwrap_err();

    assert!(error.contains("--stories is not supported"));
}
