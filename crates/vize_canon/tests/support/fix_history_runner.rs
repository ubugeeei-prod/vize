//! Execute the immutable diagnostic corpus through the production checker.
#![expect(clippy::expect_used, reason = "fixture assertions fail by panicking")]
#![expect(clippy::disallowed_types, reason = "JSON fixtures use std strings")]

use super::fix_history_contract::{Diagnostic, Pack};
use super::fix_history_normalization::normalize_generated_check_ordinals;
use super::fix_history_observation as observation;
use super::fix_history_paths::{link_vue, safe_relative};
use std::path::Path;
use vize_canon::{
    BatchTypeChecker, BatchTypeCheckerOptions, BatchTypeCheckerTrait, PackageResolutionMode,
    PackageRouteBinding, PackageRouteResolver, PackageSourceOptions,
};

pub(crate) fn check_pack(
    name: &str,
    test: &str,
    regression: &str,
    historical_issue: Option<u32>,
    case_ids: &[&str],
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
    assert_eq!(
        pack.source_revision,
        match name {
            "import-literal-geometry-10762" | "escaped-import-literals-10763" => {
                "62652a4d4029fa11016dfa1ac8c24669b42c8d7a"
            }
            "package-target-deletion-refresh" => "13fa41356d075047a229f580014b637d3bbeb57f",
            "template-context-undefined-10667"
            | "contextual-prop-anchors-10661"
            | "typed-import-meta"
            | "slot-outlet-key"
            | "options-api-any-instance"
            | "authored-unused-symbols"
            | "split-script-original"
            | "v-for-source-original"
            | "v-for-component-source-original"
            | "declared-prop-names-original"
            | "template-suppression-original"
            | "native-pointer-events-original"
            | "cache-maintenance-9860"
            | "workspace-alias-cache-10063"
            | "workspace-package-import-alias-10175"
            | "nuxt-unknown-template-options-10013"
            | "reactive-props-destructure-10008"
            | "generic-fallthrough-spread-9956"
            | "optional-fallthrough-spread-10170"
            | "misskey-generated-inference-9516"
            | "component-v-bind-duplicate-diagnostic-10069"
            | "inferred-project-tsconfig-9718"
            | "content-mapper-global-component-10185"
            | "allow-importing-ts-extensions-9890"
            | "nuxt-plugin-injection-types"
            | "listener-attribute-parity-9852"
            | "dynamic-component-union-listener-10026"
            | "native-inline-event-handler-9835"
            | "art-variant-implicit-component-9854"
            | "runtime-props-report-10100-10101"
            | "component-render-h-props-9855"
            | "component-listener-alias-scope-10112"
            | "content-mapper-duplicate-attributes-10184"
            | "content-mapper-musea-art-10186"
            | "content-mapper-nuxt-emit-10195"
            | "incomplete-script-setup-recovery-10200"
            | "dynamic-directive-expression-mapping-10182"
            | "content-mapper-template-expression-syntax"
            | "content-mapper-const-enum-import"
            | "content-mapper-diagnostics-10187"
            | "content-mapper-javascript-checkjs-10217"
            | "content-mapper-dynamic-template-ref-10198"
            | "content-mapper-type-only-import-use"
            | "content-mapper-unknown-options-10218"
            | "content-mapper-dynamic-template-ref-union-10251"
            | "content-mapper-dynamic-template-ref-scope-10251"
            | "content-mapper-props-binding-10196"
            | "strict-fallthrough-default-7856"
            | "content-mapper-multiline-callback-10191"
            | "content-mapper-define-slots-typeof-10197"
            | "content-mapper-nested-vfor-guard-10216"
            | "content-mapper-filename-globals-10190"
            | "native-expect-error-10413"
            | "generic-constructor-slots-10416"
            | "long-prop-kebab-case-10423"
            | "pug-refusal-10430"
            | "portable-pug-10434"
            | "jsx-prop-token-ranges-10494"
            | "quoted-generic-parameters-10540"
            | "planned-generic-syntax-route-10598"
            | "unicode-prop-bindings-10573"
            | "experimental-router-import-10520"
            | "literal-reference-paths-10551"
            | "content-mapper-experimental-router-10526"
            | "narrowed-slot-host-guard-10660"
            | "callback-render-position-10738"
            | "generic-slot-outlet-10883"
            | "empty-generic-public-contract-10540"
            | "component-input-diagnostic-owners-10607"
            | "content-mapper-merged-class-10788"
            | "content-mapper-slot-host-callbacks-10774"
            | "native-completion-order-10747"
            | "global-scope-build-info-10819"
            | "reopen-repaired-diagnostics-10730"
            | "editor-alias-import-extension-10753"
            | "options-api-reserved-component-reference"
            | "empty-v-slot-expression" => regression,
            "valueless-directive-import-3140" => {
                "88e8500c1e5f1b29d80f42c8ca065cc9cbd56899"
            }
            _ => "9aaa1fe458a09e0d0c6604dc8835ccf7c737d943",
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
    assert_eq!(
        pack.checker_options
            .as_ref()
            .map(|options| options.options_api),
        matches!(
            name,
            "options-api-any-instance"
                | "options-api-reserved-component-reference"
                | "jsx-prop-token-ranges-10494"
        )
        .then_some(matches!(
            name,
            "options-api-any-instance" | "options-api-reserved-component-reference"
        )),
        "the original explicit checker options must be retained"
    );
    assert_eq!(
        pack.checker_options
            .as_ref()
            .map(|options| options.jsx_typecheck),
        matches!(
            name,
            "options-api-any-instance"
                | "options-api-reserved-component-reference"
                | "jsx-prop-token-ranges-10494"
        )
        .then_some(name == "jsx-prop-token-ranges-10494"),
        "the authored JSX lowering option must be retained"
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
        pack.project_options,
        (name == "authored-unused-symbols").then(|| serde_json::json!({"vuePackage":"absent"}))
    );
    let mut capture = observation::Capture::from_environment();
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
            let file = safe_relative(&input.file);
            let source = safe_relative(&input.source);
            let destination = project_root.join(file);
            std::fs::create_dir_all(destination.parent().expect("an input must have a parent"))
                .expect("input parents must be created");
            std::fs::copy(fixtures.join(source), destination)
                .expect("exact fixture bytes must copy");
        }
        if pack.project_options.is_some() {
            assert!(!project_root.join("node_modules").exists());
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
            } else {
                Default::default()
            },
            tsconfig_path: match name {
                "workspace-alias-cache-10063" => Some(project_root.join("tsconfig.a.json")),
                "import-literal-geometry-10762" => {
                    Some(project_root.join("packages/web/tsconfig.app.json"))
                }
                _ => None,
            },
        };
        let mut checker =
            BatchTypeChecker::with_options(&project_root, options).expect("the checker must start");
        if name == "workspace-package-import-alias-10175" {
            let mut resolver = PackageRouteResolver::default();
            let bindings = [
                ("src/main.ts", "#app"),
                ("src/main.ts", "@demo/ui"),
                ("packages/ui/src/MyButton.vue", "@demo/core"),
            ]
            .into_iter()
            .map(|(importer, specifier)| {
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
