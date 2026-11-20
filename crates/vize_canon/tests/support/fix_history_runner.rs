//! Execute the immutable diagnostic corpus through the production checker.
#![expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]
#![expect(clippy::disallowed_types, reason = "JSON fixtures use std strings")]

use super::fix_history_contract::{Diagnostic, Pack};
use super::fix_history_normalization::normalize_generated_check_ordinals;
use super::fix_history_paths::{link_vue, safe_relative};
use std::path::Path;
use vize_canon::{
    BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait, PackageResolutionMode,
    PackageRouteBinding, PackageRouteResolver, PackageSourceOptions,
    batch::{TsconfigOwnershipCache, TsconfigSourceKind},
    virtual_ts::VirtualTsOptions,
};

pub(crate) fn native_fixture_runtime_disabled() -> bool {
    std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some()
}

mod art_native_display;
mod capture;
mod required_listener_events;
pub(crate) use capture::WorkspaceCapturePolicy;

/// Hand-written packs whose original tests enabled the Options API checker.
/// Tool-written packs declare their options in the registry instead.
const AUTHORED_OPTIONS_API_PACKS: &[&str] = &[
    "options-api-any-instance",
    "options-api-reserved-component-reference",
    "required-default-props",
    "setup-custom-properties",
];

/// The checker and project options a pack must declare: a tool-written pack's
/// registry entry, else the hand-written packs' original options.
fn expected_options(
    root: &Path,
    name: &str,
) -> (Option<serde_json::Value>, Option<serde_json::Value>) {
    let registry: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            root.join("tests/_fixtures/differential/typechecker/fix-history-packs.json"),
        )
        .expect("the fix-history pack registry must exist"),
    )
    .expect("the fix-history pack registry must be JSON");
    if let Some(entry) = registry.get(name) {
        return (
            entry.get("checkerOptions").cloned(),
            entry.get("projectOptions").cloned(),
        );
    }
    let checker = if AUTHORED_OPTIONS_API_PACKS.contains(&name) {
        Some(serde_json::json!({"optionsApi": true}))
    } else if name == "jsx-prop-token-ranges-10494" {
        Some(serde_json::json!({"jsxTypecheck": true}))
    } else {
        None
    };
    let project =
        (name == "authored-unused-symbols").then(|| serde_json::json!({"vuePackage": "absent"}));
    (checker, project)
}

pub(crate) fn check_pack(
    name: &str,
    test: &str,
    regression: &str,
    historical_issue: Option<u32>,
    case_ids: &[&str],
) {
    check_pack_with_capture_policy(
        name,
        test,
        regression,
        historical_issue,
        case_ids,
        WorkspaceCapturePolicy { record: true },
    );
}

/// Dedicated admission executes every assertion; only the selected canonical
/// executable owns the full-workspace observation receipt.
pub(crate) fn check_pack_with_capture_policy(
    name: &str,
    test: &str,
    regression: &str,
    historical_issue: Option<u32>,
    case_ids: &[&str],
    capture_policy: WorkspaceCapturePolicy,
) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixtures = root
        .join("tests/_fixtures/differential/typechecker")
        .join(name);
    let fixture_pack =
        std::fs::read(fixtures.join("cases.json")).expect("the fixture pack must exist");
    let pack: Pack = serde_json::from_slice(&fixture_pack)
        .expect("the fixture pack must have the expected schema");
    assert_eq!(pack.version, 1);
    assert_eq!(pack.issue, 6879);
    assert_eq!(pack.historical_issue, historical_issue);
    assert_eq!(
        pack.historical_issues.as_deref(),
        (name == "runtime-props-report-10100-10101").then_some(&[10100, 10101][..]),
        "every historical issue must retain exact fixture custody"
    );
    assert_eq!(pack.regression_commit, regression);
    // Packs captured from their own fix commit need no entry here.
    assert_eq!(
        pack.source_revision,
        match name {
            "import-literal-geometry-10762" | "escaped-import-literals-10763" => {
                "62652a4d4029fa11016dfa1ac8c24669b42c8d7a"
            }
            "package-target-deletion-refresh" => "13fa41356d075047a229f580014b637d3bbeb57f",
            "valueless-directive-import-3140" => "88e8500c1e5f1b29d80f42c8ca065cc9cbd56899",
            "component-event-tuples"
            | "component-listener-prop-duplicate-10397"
            | "event-handler-narrowing"
            | "required-props-edges"
            | "reserved-expression-shapes"
            | "template-definite-assignment"
            | "unicode-reserved-props" => "9aaa1fe458a09e0d0c6604dc8835ccf7c737d943",
            _ => regression,
        }
    );
    assert_eq!(pack.diagnostic_contract.position_base, 1);
    assert_eq!(pack.diagnostic_contract.positions, "authored UTF16 start");
    assert_eq!(
        pack.diagnostic_contract.comparison,
        "all production diagnostics in returned order"
    );
    assert_eq!(
        pack.diagnostic_contract.missing_fields,
        ["end", "relatedInformation", "raw backend diagnostics"]
    );
    assert_eq!(pack.diagnostic_contract.native, "unsupported");
    assert_eq!(pack.diagnostic_contract.required_tier, "T1");
    let raw_pack: serde_json::Value =
        serde_json::from_slice(&fixture_pack).expect("the fixture pack must be JSON");
    let (checker_options, project_options) = expected_options(&root, name);
    assert_eq!(
        raw_pack.get("checkerOptions"),
        checker_options.as_ref(),
        "the original explicit checker options must be retained"
    );
    assert_eq!(
        pack.cases
            .iter()
            .map(|case| case.id.as_str())
            .collect::<Vec<_>>(),
        case_ids,
        "missing, duplicated or reordered cases must not pass"
    );
    let vue = std::fs::canonicalize(root.join("tests/node_modules/vue"))
        .expect("the pinned real Vue package must be installed");
    assert_eq!(
        pack.project_options, project_options,
        "the original project options must be retained"
    );
    let mut capture = capture_policy.capture(name, test);
    for case in pack.cases {
        let project = tempfile::tempdir().expect("a project must be created");
        let uses_router_provider = matches!(
            name,
            "experimental-router-import-10520" | "content-mapper-experimental-router-10526"
        );
        let project_root = if name == "literal-reference-paths-10551" {
            super::fix_history_reference_paths::project_root(project.path())
        } else if uses_router_provider {
            super::fix_history_router_provider::prepare(&root, project.path(), &fixtures)
        } else {
            std::fs::canonicalize(project.path()).expect("project identity must resolve")
        };
        if pack.project_options.is_none() && !uses_router_provider {
            std::fs::create_dir(project_root.join("node_modules"))
                .expect("node_modules must be created");
            link_vue(&vue, &project_root.join("node_modules/vue"));
            if name == "empty-generic-public-contract-10540" {
                let helpers = std::fs::canonicalize(
                    root.join("tests/node_modules/vue-component-type-helpers"),
                )
                .expect("the pinned real public component type helpers must be installed");
                link_vue(
                    &helpers,
                    &project_root.join("node_modules/vue-component-type-helpers"),
                );
            }
        }
        for input in &case.inputs {
            // A Vue stub may only be shipped when the real package is not
            // linked; writing it through the link would overwrite the
            // installed package for every later pack.
            let ships_vue = input.file.split('/').take(2).eq(["node_modules", "vue"]);
            assert!(
                pack.project_options.is_some() || !ships_vue,
                "{}",
                input.file
            );
            let file = safe_relative(&input.file);
            let source = safe_relative(&input.source);
            let destination = project_root.join(file);
            std::fs::create_dir_all(destination.parent().expect("an input must have a parent"))
                .expect("input parents must be created");
            std::fs::copy(fixtures.join(source), destination)
                .expect("exact fixture bytes must copy");
        }
        if pack.project_options.is_some() {
            // Only the pack's own carriers (a historical Vue stub) may exist.
            let ships_packages = case
                .inputs
                .iter()
                .any(|input| input.file.split('/').next() == Some("node_modules"));
            assert!(ships_packages || !project_root.join("node_modules").exists());
            assert!(!project_root.join("node_modules/vue").is_symlink());
        }
        if name == "experimental-router-props-10579" {
            super::fix_history_router_provider::install_source_build(&root, &project_root);
        }
        if name == "workspace-alias-cache-10063" {
            for (source, destination) in [
                ("packages/b", "packages/a/node_modules/@demo/b"),
                ("packages/c", "packages/b/node_modules/@demo/c"),
            ] {
                let destination = project_root.join(destination);
                std::fs::create_dir_all(
                    destination
                        .parent()
                        .expect("a workspace link must have a parent"),
                )
                .expect("workspace link parents must be created");
                link_vue(&project_root.join(source), &destination);
            }
            let check = |config: &str| {
                let mut checker = BatchTypeChecker::with_options(
                    &project_root,
                    BatchTypeCheckerOptions {
                        tsconfig_path: Some(project_root.join(config)),
                        ..Default::default()
                    },
                )
                .expect("the package selection must start");
                checker
                    .scan_project()
                    .expect("the package selection must scan");
                checker
                    .check_project()
                    .expect("the package selection must finish")
            };
            if case.id == "repaired-private-package-identity" {
                // The exact shared-divergent-shared mutation runs in the
                // pnpm workspace route oracle. This pinned corpus retains the
                // private-brand sources independently of that filesystem
                // lifecycle harness.
                let direct = project_root.join("packages/a/node_modules/@demo/c");
                std::fs::create_dir_all(direct.parent().expect("a direct link has a parent"))
                    .expect("the direct link parent must be created");
                link_vue(&project_root.join("packages/c"), &direct);
                assert!(check("tsconfig.a.json").diagnostics.is_empty());
            } else {
                // #10063 is a hard materialization error before diagnostics.
                // Both orderings must complete and preserve the empty authored
                // result; the complete contract is compared again below.
                assert!(check("tsconfig.b.json").diagnostics.is_empty());
                assert!(check("tsconfig.a.json").diagnostics.is_empty());
            }
        }
        // Workspace packages `vize check` sees through node_modules links.
        for link in pack
            .checker_options
            .iter()
            .flat_map(|options| &options.workspace_links)
        {
            let destination = project_root.join(safe_relative(&link.link));
            std::fs::create_dir_all(destination.parent().expect("a link must have a parent"))
                .expect("workspace link parents must be created");
            link_vue(
                &project_root.join(safe_relative(&link.package)),
                &destination,
            );
        }
        if name == "workspace-package-import-alias-10175" {
            for (source, destination) in [
                ("packages/ui", "node_modules/@demo/ui"),
                ("packages/core", "packages/ui/node_modules/@demo/core"),
            ] {
                let destination = project_root.join(destination);
                std::fs::create_dir_all(
                    destination
                        .parent()
                        .expect("a workspace link must have a parent"),
                )
                .expect("workspace link parents must be created");
                link_vue(&project_root.join(source), &destination);
            }
        }
        let options = BatchTypeCheckerOptions {
            virtual_ts_options: if name == "literal-reference-paths-10551" {
                super::fix_history_reference_paths::options(&project_root)
            } else if let Some(options) = &pack.checker_options {
                VirtualTsOptions {
                    strict_instance_globals: options.strict_instance_globals,
                    auto_import_stubs: options
                        .auto_import_stubs
                        .iter()
                        .map(|stub| stub.as_str().into())
                        .collect(),
                    ..Default::default()
                }
            } else {
                Default::default()
            },
            tsconfig_path: match name {
                "public-instance-ts-directives-10984" => {
                    Some(project_root.join("test-workspace/tsc/#3488/tsconfig.json"))
                }
                "workspace-alias-cache-10063" => Some(project_root.join("tsconfig.a.json")),
                "import-literal-geometry-10762" => {
                    Some(project_root.join("packages/web/tsconfig.app.json"))
                }
                // `vize check` and the editor check a page with its owning
                // referenced project.
                "tsconfig-literal-brackets-10819" => {
                    let page = case
                        .inputs
                        .iter()
                        .find(|input| input.file.ends_with(".vue"))
                        .expect("the case must name its page");
                    Some(
                        TsconfigOwnershipCache::default().effective_config_for_source(
                            &project_root.join("tsconfig.json"),
                            &project_root.join(safe_relative(&page.file)),
                            TsconfigSourceKind::Typed,
                        ),
                    )
                }
                "relative-sibling-storage-10934" => {
                    Some(project_root.join("apps/app/tsconfig.json"))
                }
                _ => None,
            },
        };
        let mut checker =
            BatchTypeChecker::with_options(&project_root, options).expect("the checker must start");
        if name == "relative-sibling-storage-10934" {
            checker
                .set_persistent_storage_owner(
                    &project_root.join("apps/app"),
                    Some(&project_root.join("apps/app/tsconfig.json")),
                )
                .expect("the package must select its private persistent storage owner");
        }
        let routes: Vec<(&str, &str)> = if name == "workspace-package-import-alias-10175" {
            vec![
                ("src/main.ts", "#app"),
                ("src/main.ts", "@demo/ui"),
                ("packages/ui/src/MyButton.vue", "@demo/core"),
            ]
        } else {
            pack.checker_options
                .as_ref()
                .map(|options| {
                    options
                        .package_routes
                        .iter()
                        .map(|route| (route.importer.as_str(), route.specifier.as_str()))
                        .collect()
                })
                .unwrap_or_default()
        };
        if !routes.is_empty() {
            install_package_routes(&mut checker, &project_root, &routes);
        }
        if pack
            .checker_options
            .as_ref()
            .is_some_and(|options| options.options_api)
        {
            checker.enable_options_api();
        }
        if pack
            .checker_options
            .as_ref()
            .is_some_and(|options| options.jsx_typecheck)
        {
            checker.enable_jsx_typecheck();
        }
        if pack
            .checker_options
            .as_ref()
            .is_some_and(|options| options.legacy_vue2)
        {
            checker.enable_legacy_vue2();
        }
        if let Some(checks) = pack
            .checker_options
            .as_ref()
            .and_then(|options| options.virtual_ts_checks.as_ref())
        {
            checker.set_virtual_ts_checks(checks.props, checks.template_bindings, checks.emits);
        }
        checker.scan_project().expect("the project must scan");
        let result = checker
            .check_project()
            .expect("the production checker must finish");
        let actual: Vec<Diagnostic> = result
            .diagnostics
            .iter()
            .map(|diagnostic| Diagnostic {
                file: diagnostic
                    .file
                    .strip_prefix(&project_root)
                    .expect("every diagnostic must belong to this exact project")
                    .to_string_lossy()
                    .replace('\\', "/"),
                line: diagnostic.line + 1,
                column: diagnostic.column + 1,
                severity: diagnostic.severity,
                code: diagnostic.code,
                message: String::from(diagnostic.message.as_str()),
            })
            .collect();
        let (compared_actual, compared_expected) = if name == "required-props-edges" {
            (
                normalize_generated_check_ordinals(&actual),
                normalize_generated_check_ordinals(&case.diagnostics),
            )
        } else if name == "art-variant-implicit-component-9854" {
            (
                actual.clone(),
                art_native_display::expected(&fixtures, &fixture_pack, &case),
            )
        } else if name == "component-listener-prop-duplicate-10397" {
            (
                actual.clone(),
                required_listener_events::expected(&fixtures, &fixture_pack, &case),
            )
        } else {
            (actual.clone(), case.diagnostics.clone())
        };
        assert_eq!(
            compared_actual, compared_expected,
            "complete diagnostic list for {}",
            case.id
        );
        if let Some(capture) = &mut capture {
            capture.record(case.id, &case.inputs, &project_root, actual, &result);
        }
    }
    if let Some(capture) = capture {
        capture.finish(name, test, &fixture_pack);
    }
}

/// Install the package routes `vize check` resolves for each importer.
fn install_package_routes(
    checker: &mut BatchTypeChecker,
    project_root: &Path,
    routes: &[(&str, &str)],
) {
    let mut resolver = PackageRouteResolver::default();
    let bindings = routes
        .iter()
        .map(|&(importer, specifier)| {
            let importer = project_root.join(importer);
            let (context, mut context_inputs) = resolver.resolution_context(
                &importer,
                PackageResolutionMode::Import,
                Some("bundler"),
                Some("esnext"),
                std::iter::empty::<&str>(),
            );
            let (route, mut invalidation_paths) = resolver
                .lookup_with_context(
                    importer.parent().expect("the importer must have a parent"),
                    specifier,
                    PackageSourceOptions::new(false, false),
                    context.clone(),
                )
                .into_parts();
            invalidation_paths.append(&mut context_inputs);
            PackageRouteBinding {
                importer_path: importer.clone(),
                specifier: specifier.into(),
                occurrence_mode: PackageResolutionMode::Import,
                context,
                route: Some(route.expect("the fixture package route must resolve")),
                invalidation_paths,
                shared_invalidation_paths: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    checker.set_package_route_resolver(resolver);
    checker.set_package_routes(bindings);
}
