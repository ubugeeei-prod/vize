//! Regression for #3320: a `vize check` run with no explicit inputs must not
//! report an unrelated ancestor project's result as the working directory's
//! result.
//!
//! Project-root discovery walks up from the working directory until some
//! ancestor owns a `tsconfig.json`. When that project's program contains no
//! file under the working directory, the run used to type-check the ancestor's
//! files, find nothing wrong with them, and exit `0` — while the working
//! directory's own broken sources were never checked. The failure mode is a
//! silent false negative, so every layout below pins the *presence* of the
//! report, and the layouts that reach dependencies through links are pinned
//! alongside the plain one because the original report arrived through a
//! symlinked `node_modules`.

use vize_carton::cstr;

#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;

#[path = "support/default_run_root.rs"]
mod default_run_root;

use default_run_root::{
    ANCESTOR_TSCONFIG_OWNING_THE_APP, Layout, NodeModulesLayout, build_case, link_vue_packages,
    resolve_test_corsa_path, run_check, run_check_with_corsa, stderr_lines, symlink_path,
    unowned_error, unowned_error_for, workspace_vue_package,
};

/// The reported layout: an ancestor `tsconfig.json` that owns only its own
/// sources, and a nested app with no config of its own.
#[test]
fn default_run_outside_the_resolved_program_reports_the_wrong_root() {
    let case = build_case("plain", Layout::unowned());
    let output = run_check(&case.app, &[]);

    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, unowned_error(&case));
    assert_eq!(output.code, Some(2));

    case.cleanup();
}

/// `--format json` must not turn the wrong root into an empty-but-valid report.
#[test]
fn default_run_outside_the_resolved_program_writes_no_json_report() {
    let case = build_case("json", Layout::unowned());
    let output = run_check(&case.app, &["--format", "json"]);

    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, unowned_error(&case));
    assert_eq!(output.code, Some(2));

    case.cleanup();
}

/// `--quiet` suppresses progress output, never the wrong root.
#[test]
fn default_run_outside_the_resolved_program_reports_even_when_quiet() {
    let case = build_case("quiet", Layout::unowned());
    let output = run_check(&case.app, &["--quiet"]);

    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, unowned_error(&case));
    assert_eq!(output.code, Some(2));

    case.cleanup();
}

/// A real `node_modules` in the workspace: the same verdict as with none.
#[test]
fn default_run_outside_the_resolved_program_with_a_real_node_modules() {
    let case = build_case(
        "real-node-modules",
        Layout {
            node_modules: NodeModulesLayout::RealDirectory,
            ..Layout::unowned()
        },
    );
    let output = run_check(&case.app, &[]);

    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, unowned_error(&case));
    assert_eq!(output.code, Some(2));

    case.cleanup();
}

/// `node_modules` symlinked to a store outside the workspace — the layout the
/// issue was reported from. Discovery must reach the same verdict as with a real
/// directory, in the same run, with the same message.
#[test]
fn default_run_outside_the_resolved_program_with_a_symlinked_node_modules() {
    let case = build_case(
        "symlinked-node-modules",
        Layout {
            node_modules: NodeModulesLayout::SymlinkedStore,
            ..Layout::unowned()
        },
    );
    let output = run_check(&case.app, &[]);

    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, unowned_error(&case));
    assert_eq!(output.code, Some(2));

    case.cleanup();
}

/// pnpm's isolated linker: `node_modules` is real and the package inside it is
/// the link.
#[test]
fn default_run_outside_the_resolved_program_with_a_symlinked_package() {
    let case = build_case(
        "symlinked-package",
        Layout {
            node_modules: NodeModulesLayout::SymlinkedPackage,
            ..Layout::unowned()
        },
    );
    let output = run_check(&case.app, &[]);

    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, unowned_error(&case));
    assert_eq!(output.code, Some(2));

    case.cleanup();
}

/// A `package.json` in the app declares a package, not a type-checking program:
/// the ancestor project still owns none of its files, so the run is still
/// refused rather than silently reporting the ancestor's result.
#[test]
fn default_run_outside_the_resolved_program_with_a_package_boundary() {
    let case = build_case(
        "package-boundary",
        Layout {
            app_package_json: true,
            ..Layout::unowned()
        },
    );
    let output = run_check(&case.app, &[]);

    assert_eq!(output.stdout, "");
    assert_eq!(output.stderr, unowned_error(&case));
    assert_eq!(output.code, Some(2));

    case.cleanup();
}

/// The whole workspace reached through a symlinked path: the working directory
/// is spelled through the link while collected inputs carry the resolved
/// spelling. The verdict must be about ownership, not spelling — so this is
/// still the unowned case, reported against the resolved spelling.
#[test]
fn default_run_through_a_symlinked_workspace_path_reports_the_wrong_root() {
    let case = build_case("linked-workspace", Layout::unowned());
    let link = case.workspace.with_extension("link");
    let _ = std::fs::remove_file(&link);
    symlink_path(&case.workspace, &link).unwrap();
    let linked_app = link.join("nested/app");
    let output = run_check(&linked_app, &[]);

    assert_eq!(output.stdout, "");
    assert_eq!(
        output.stderr,
        unowned_error_for(&case.workspace, &case.app, 3)
    );
    assert_eq!(output.code, Some(2));

    let _ = std::fs::remove_file(&link);
    case.cleanup();
}

/// The negative direction. An ancestor `tsconfig.json` that *does* own the app's
/// sources is legitimate ancestor discovery: the run proceeds, reports the
/// app's real error, and surfaces the wider scope instead of failing.
#[test]
fn default_run_inside_an_owning_ancestor_program_checks_the_app_and_notes_the_scope() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        eprintln!("skipping owning-ancestor case: tsgo not found");
        return;
    };
    let Some(vue_package) = workspace_vue_package() else {
        eprintln!("skipping owning-ancestor case: workspace Vue package missing");
        return;
    };
    let case = build_case(
        "owning-ancestor",
        Layout {
            tsconfig: ANCESTOR_TSCONFIG_OWNING_THE_APP,
            node_modules: NodeModulesLayout::RealDirectory,
            ..Layout::unowned()
        },
    );
    link_vue_packages(&vue_package, &case.workspace.join("node_modules")).unwrap();

    let output = run_check_with_corsa(&case.app, &["--format", "json"], Some(&corsa_path));

    assert_eq!(
        stderr_lines(&output.stderr),
        vec![
            cstr!(
                "vize check: the project root resolved to `{}`, above the working directory \
                 `{}`; 3 of 4 checked files are outside `{}`.",
                case.workspace.display(),
                case.app.display(),
                case.app.display()
            ),
            cstr!(
                "Building Corsa virtual project for 4 files under {}...",
                case.workspace.display()
            ),
            cstr!("Running Corsa diagnostics for 4 files..."),
        ]
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::json!({
            "files": [
                { "file": case.workspace.join("helpers/h1.ts"), "diagnostics": [] },
                { "file": case.workspace.join("helpers/h2.ts"), "diagnostics": [] },
                { "file": case.workspace.join("helpers/h3.ts"), "diagnostics": [] },
                {
                    "file": "src/Broken.vue",
                    "diagnostics": [
                        "error:2:7 [TS2322] Type 'string' is not assignable to type 'number'."
                    ]
                },
            ],
            "programs": [
                {
                    "root": case.workspace,
                    "tsconfig": case.workspace.join("tsconfig.json"),
                    "files": [
                        case.workspace.join("helpers/h1.ts"),
                        case.workspace.join("helpers/h2.ts"),
                        case.workspace.join("helpers/h3.ts"),
                        "src/Broken.vue",
                    ]
                }
            ],
            "errorCount": 1,
            "warningCount": 0,
            "fileCount": 4
        })
    );
    assert_eq!(output.code, Some(1));

    case.cleanup();
}

/// The app's own `tsconfig.json` ends the walk-up at the nearest project: no
/// scope note, and the app's error is reported.
#[test]
fn default_run_with_a_local_tsconfig_stays_in_the_app() {
    let Some(corsa_path) = corsa_requirement::required_or_skip(resolve_test_corsa_path()) else {
        eprintln!("skipping local-tsconfig case: tsgo not found");
        return;
    };
    let Some(vue_package) = workspace_vue_package() else {
        eprintln!("skipping local-tsconfig case: workspace Vue package missing");
        return;
    };
    let case = build_case(
        "local-tsconfig",
        Layout {
            node_modules: NodeModulesLayout::RealDirectory,
            app_tsconfig: true,
            ..Layout::unowned()
        },
    );
    link_vue_packages(&vue_package, &case.workspace.join("node_modules")).unwrap();

    let output = run_check_with_corsa(&case.app, &["--format", "json"], Some(&corsa_path));

    assert_eq!(
        stderr_lines(&output.stderr),
        vec![
            cstr!(
                "Building Corsa virtual project for 1 files under {}...",
                case.app.display()
            ),
            cstr!("Running Corsa diagnostics for 1 files..."),
        ]
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&output.stdout).unwrap(),
        serde_json::json!({
            "files": [
                {
                    "file": "src/Broken.vue",
                    "diagnostics": [
                        "error:2:7 [TS2322] Type 'string' is not assignable to type 'number'."
                    ]
                },
            ],
            "programs": [
                {
                    "root": ".",
                    "tsconfig": "tsconfig.json",
                    "files": ["src/Broken.vue"]
                }
            ],
            "errorCount": 1,
            "warningCount": 0,
            "fileCount": 1
        })
    );
    assert_eq!(output.code, Some(1));

    case.cleanup();
}
