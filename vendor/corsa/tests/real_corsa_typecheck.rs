mod support;

use std::{collections::BTreeSet, fs, process::Command};

use corsa::{
    CorsaError,
    api::{
        ApiClient, ApiMode, ApiSpawnConfig, FileDiagnosticsResponse, ProjectHandle, SnapshotHandle,
        UpdateSnapshotParams,
    },
    runtime::block_on,
};
use lsp_types::{Diagnostic, NumberOrString};
use tempfile::tempdir;

#[test]
fn real_corsa_reports_actual_type_errors() {
    let Some(binary) = support::resolved_real_corsa_binary() else {
        return;
    };
    let project = fixture(&[
        (
            "tsconfig.json",
            r#"{
  "compilerOptions": {
    "strict": true,
    "noEmit": true
  },
  "include": ["src/**/*.ts"]
}"#,
        ),
        (
            "src/index.ts",
            r#"const amount: number = "oops";
export const fixed = amount.toFixed(2);
"#,
        ),
    ]);
    let output = Command::new(binary)
        .arg("--pretty")
        .arg("false")
        .arg("-p")
        .arg(project.path().join("tsconfig.json"))
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "expected corsa to reject the fixture"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains(
            "src/index.ts(1,7): error TS2322: Type 'string' is not assignable to type 'number'."
        ),
        "unexpected compiler output: {stdout}"
    );
}

#[test]
fn real_corsa_infers_contextual_and_generic_types() {
    block_on(async {
        let Some(binary) = support::resolved_real_corsa_binary() else {
            return;
        };
        let project = fixture(&[
            (
                "tsconfig.json",
                r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "noEmit": true
  },
  "include": ["src/**/*.ts"]
}"#,
            ),
            (
                "src/index.ts",
                r#"const numbers = [1, 2, 3];
const inferred = numbers.map((value) => value.toFixed(2));
"#,
            ),
        ]);
        let file = project.path().join("src/index.ts");
        let file_text = fs::read_to_string(&file).unwrap();
        let file_wire = file.display().to_string();
        let config_wire = project.path().join("tsconfig.json").display().to_string();
        let value_pos = u32::try_from(file_text.find("(value) =>").unwrap() + 1).unwrap();
        let inferred_pos = u32::try_from(file_text.find("inferred =").unwrap() + 1).unwrap();

        for mode in [ApiMode::SyncMsgpackStdio, ApiMode::AsyncJsonRpcStdio] {
            let client = ApiClient::spawn(
                ApiSpawnConfig::new(binary.clone())
                    .with_mode(mode)
                    .with_cwd(project.path()),
            )
            .await
            .unwrap();
            let snapshot = client
                .update_snapshot(UpdateSnapshotParams {
                    open_project: Some(config_wire.clone()),
                    file_changes: None,
                    overlay_changes: None,
                })
                .await
                .unwrap();
            let project = snapshot.projects[0].id.clone();

            let value_type = client
                .get_type_at_position(
                    snapshot.handle.clone(),
                    project.clone(),
                    file_wire.as_str(),
                    value_pos,
                )
                .await
                .unwrap()
                .unwrap();
            let value_rendered = client
                .type_to_string(
                    snapshot.handle.clone(),
                    project.clone(),
                    value_type.id,
                    None,
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                value_rendered, "number",
                "unexpected callback parameter type for {mode:?}"
            );

            let inferred_type = client
                .get_type_at_position(
                    snapshot.handle.clone(),
                    project.clone(),
                    file_wire.as_str(),
                    inferred_pos,
                )
                .await
                .unwrap()
                .unwrap();
            let inferred_rendered = client
                .type_to_string(
                    snapshot.handle.clone(),
                    project.clone(),
                    inferred_type.id,
                    None,
                    None,
                )
                .await
                .unwrap();
            assert_eq!(
                inferred_rendered, "string[]",
                "unexpected inferred array type for {mode:?}"
            );

            snapshot.release().await.unwrap();
            client.close().await.unwrap();
        }
    });
}

#[test]
fn real_corsa_handles_monorepo_project_references_and_strict_config() {
    block_on(async {
        let Some(binary) = support::resolved_real_corsa_binary() else {
            return;
        };
        let project = fixture(&[
            (
                "tsconfig.json",
                r#"{
  "files": [],
  "references": [
    { "path": "./packages/core" },
    { "path": "./packages/config" },
    { "path": "./packages/app" }
  ]
}"#,
            ),
            (
                "tsconfig.base.json",
                r#"{
  "compilerOptions": {
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "exactOptionalPropertyTypes": true,
    "noUncheckedIndexedAccess": true,
    "composite": true,
    "declaration": true,
    "declarationMap": true,
    "skipLibCheck": true,
    "paths": {
      "@repo/core": ["./packages/core/dist/index.d.ts"],
      "@repo/config/*": ["./packages/config/dist/*"]
    }
  }
}"#,
            ),
            (
                "packages/core/tsconfig.json",
                r#"{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": {
    "rootDir": "src",
    "outDir": "dist",
    "tsBuildInfoFile": "dist/.tsbuildinfo"
  },
  "include": ["src/**/*.ts"]
}"#,
            ),
            (
                "packages/core/src/index.ts",
                r#"export type UserId = string & { readonly __brand: "UserId" };

export type Result<T> =
  | { ok: true; value: T }
  | { ok: false; reason: string };

export interface UserProfile {
  displayName?: string;
  email?: `${string}@${string}`;
}

export interface User {
  id: UserId;
  profile?: UserProfile;
}

export function defineUser<T extends User>(user: T): T {
  return user;
}

export function makeResult<T>(value: T): Result<T> {
  return { ok: true, value };
}
"#,
            ),
            (
                "packages/config/tsconfig.json",
                r#"{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": {
    "rootDir": "src",
    "outDir": "dist",
    "tsBuildInfoFile": "dist/.tsbuildinfo"
  },
  "include": ["src/**/*.ts"]
}"#,
            ),
            (
                "packages/config/src/settings.ts",
                r#"export const settings = {
  retries: 3,
  endpoints: ["https://api.example.test"]
} as const;

export const featureFlags = {
  billing: true,
  betaSearch: false
} as const;
"#,
            ),
            (
                "packages/app/tsconfig.json",
                r#"{
  "extends": "../../tsconfig.base.json",
  "compilerOptions": {
    "rootDir": "src",
    "outDir": "dist",
    "tsBuildInfoFile": "dist/.tsbuildinfo",
    "noEmit": true
  },
  "references": [
    { "path": "../core" },
    { "path": "../config" }
  ],
  "include": ["src/**/*.ts"]
}"#,
            ),
            (
                "packages/app/tsconfig.emit.json",
                r#"{
  "extends": "./tsconfig.json",
  "compilerOptions": {
    "noEmit": false,
    "emitDeclarationOnly": true,
    "outDir": "dist-types",
    "tsBuildInfoFile": "dist-types/.tsbuildinfo"
  },
  "include": ["src/good.ts"]
}"#,
            ),
            (
                "packages/app/src/good.ts",
                r#"import { defineUser, makeResult } from "@repo/core";
import type { Result, UserId } from "@repo/core";
import { featureFlags, settings } from "@repo/config/settings";

const typedUser = defineUser({
  id: "u_1" as UserId,
  profile: { displayName: "Ada" },
});

export const label = typedUser.profile?.displayName ?? "anonymous";
export const retryLiteral = settings.retries;
export const billingEnabled = featureFlags.billing;
export const result: Result<number> = makeResult(42);
"#,
            ),
            (
                "packages/app/src/bad.ts",
                r#"import { defineUser } from "@repo/core";
import type { Result } from "@repo/core";
import { settings } from "@repo/config/settings";

export const invalidUser = defineUser({
  id: 42,
  profile: { displayName: "Grace" },
});

export const wrongRetry: 4 = settings.retries;
export const wrongResult: Result<number> = { ok: true, value: "nope" };
export const exactOptional: { label?: string } = { label: undefined };

const names = ["Ada"];
export const firstName: string = names[1];
"#,
            ),
        ]);

        let app_config = project.path().join("packages/app/tsconfig.json");
        let app_config_wire = app_config.display().to_string();
        let good_file = project.path().join("packages/app/src/good.ts");
        let good_file_wire = good_file.display().to_string();
        let good_text = fs::read_to_string(&good_file).unwrap();
        let bad_file = project.path().join("packages/app/src/bad.ts");
        let bad_file_wire = bad_file.display().to_string();

        assert_monorepo_declaration_emit(&binary, project.path());
        assert_monorepo_typecheck_errors(&binary, &app_config);

        for mode in [ApiMode::SyncMsgpackStdio, ApiMode::AsyncJsonRpcStdio] {
            let client = ApiClient::spawn(
                ApiSpawnConfig::new(binary.clone())
                    .with_mode(mode)
                    .with_cwd(project.path()),
            )
            .await
            .unwrap();

            let app_config_response = client
                .parse_config_file(app_config_wire.as_str())
                .await
                .unwrap();
            assert!(
                app_config_response
                    .file_names
                    .iter()
                    .any(|file| file.ends_with("packages/app/src/good.ts")),
                "app tsconfig did not include the valid app file for {mode:?}: {app_config_response:?}"
            );
            assert!(
                app_config_response
                    .file_names
                    .iter()
                    .any(|file| file.ends_with("packages/app/src/bad.ts")),
                "app tsconfig did not include the invalid app file for {mode:?}: {app_config_response:?}"
            );

            let snapshot = client
                .update_snapshot(UpdateSnapshotParams {
                    open_project: Some(app_config_wire.clone()),
                    file_changes: None,
                    overlay_changes: None,
                })
                .await
                .unwrap();
            let app_project = snapshot.projects[0].id.clone();

            if client
                .describe_capabilities()
                .await
                .unwrap()
                .diagnostics
                .file
            {
                let good_diagnostics = client
                    .get_diagnostics_for_file(
                        snapshot.handle.clone(),
                        app_project.clone(),
                        good_file_wire.as_str(),
                    )
                    .await
                    .unwrap();
                assert_no_diagnostics(&good_diagnostics, mode);

                let bad_diagnostics = client
                    .get_diagnostics_for_file(
                        snapshot.handle.clone(),
                        app_project.clone(),
                        bad_file_wire.as_str(),
                    )
                    .await
                    .unwrap();
                assert_monorepo_diagnostics(&bad_diagnostics, mode);
            } else {
                let error = client
                    .get_diagnostics_for_file(
                        snapshot.handle.clone(),
                        app_project.clone(),
                        bad_file_wire.as_str(),
                    )
                    .await
                    .unwrap_err();
                assert!(
                    matches!(error, CorsaError::Unsupported(_)),
                    "expected unsupported diagnostics for current real corsa in {mode:?}, got {error:?}"
                );
            }

            assert_rendered_type(
                &client,
                snapshot.handle.clone(),
                app_project.clone(),
                good_file_wire.as_str(),
                position_in(&good_text, "label ="),
                "string",
                mode,
            )
            .await;
            assert_rendered_type(
                &client,
                snapshot.handle.clone(),
                app_project.clone(),
                good_file_wire.as_str(),
                position_in(&good_text, "retryLiteral ="),
                "3",
                mode,
            )
            .await;
            assert_rendered_type(
                &client,
                snapshot.handle.clone(),
                app_project.clone(),
                good_file_wire.as_str(),
                position_in(&good_text, "billingEnabled ="),
                "true",
                mode,
            )
            .await;
            assert_rendered_type(
                &client,
                snapshot.handle.clone(),
                app_project.clone(),
                good_file_wire.as_str(),
                position_in(&good_text, "result:"),
                "Result<number>",
                mode,
            )
            .await;

            snapshot.release().await.unwrap();
            client.close().await.unwrap();
        }
    });
}

fn assert_monorepo_typecheck_errors(binary: &std::path::Path, config: &std::path::Path) {
    let output = Command::new(binary)
        .arg("--pretty")
        .arg("false")
        .arg("-p")
        .arg(config)
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "expected the invalid monorepo app file to fail typecheck"
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "Type 'number' is not assignable to type 'UserId'.",
        "Type '3' is not assignable to type '4'.",
        "Type 'string' is not assignable to type 'number'.",
        "Type '{ label: undefined; }' is not assignable to type '{ label?: string; }' with 'exactOptionalPropertyTypes: true'.",
        "Type 'string | undefined' is not assignable to type 'string'.",
    ] {
        assert!(
            stdout.contains(expected),
            "missing expected typecheck output {expected:?} in:\n{stdout}"
        );
    }
}

fn assert_monorepo_declaration_emit(binary: &std::path::Path, root: &std::path::Path) {
    for config in [
        root.join("packages/core/tsconfig.json"),
        root.join("packages/config/tsconfig.json"),
        root.join("packages/app/tsconfig.emit.json"),
    ] {
        let output = Command::new(binary)
            .arg("--pretty")
            .arg("false")
            .arg("-p")
            .arg(&config)
            .output()
            .unwrap();

        assert!(
            output.status.success(),
            "expected monorepo declaration emit for {} to pass:\nstdout:\n{}\nstderr:\n{}",
            config.display(),
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(output.stderr).unwrap()
        );
    }

    let declaration = fs::read_to_string(root.join("packages/app/dist-types/good.d.ts")).unwrap();
    for expected in [
        "export declare const label: string;",
        "export declare const retryLiteral: 3;",
        "export declare const billingEnabled: true;",
        "export declare const result: Result<number>;",
    ] {
        assert!(
            declaration.contains(expected),
            "missing expected declaration {expected:?} in:\n{declaration}"
        );
    }
}

fn assert_no_diagnostics(diagnostics: &FileDiagnosticsResponse, mode: ApiMode) {
    assert!(
        diagnostics.syntactic.is_empty()
            && diagnostics.semantic.is_empty()
            && diagnostics.suggestion.is_empty(),
        "expected no diagnostics for valid monorepo file in {mode:?}: {diagnostics:#?}"
    );
}

fn assert_monorepo_diagnostics(diagnostics: &FileDiagnosticsResponse, mode: ApiMode) {
    assert!(
        diagnostics.syntactic.is_empty(),
        "expected semantic-only failures for invalid monorepo file in {mode:?}: {diagnostics:#?}"
    );
    assert!(
        diagnostics.semantic.len() >= 5,
        "expected strict monorepo fixture to surface multiple semantic diagnostics in {mode:?}: {diagnostics:#?}"
    );

    let codes = diagnostic_codes(&diagnostics.semantic);
    for expected in ["TS2322", "TS2375"] {
        assert!(
            codes.contains(expected),
            "missing diagnostic code {expected} in {mode:?}: {diagnostics:#?}"
        );
    }
    for expected in [
        "Type 'number' is not assignable to type 'UserId'.",
        "Type '3' is not assignable to type '4'.",
        "Type 'string' is not assignable to type 'number'.",
        "exactOptionalPropertyTypes: true",
        "Type 'string | undefined' is not assignable to type 'string'.",
    ] {
        assert!(
            diagnostics
                .semantic
                .iter()
                .any(|diagnostic| diagnostic.message.contains(expected)),
            "missing diagnostic message fragment {expected:?} in {mode:?}: {diagnostics:#?}"
        );
    }
}

async fn assert_rendered_type(
    client: &ApiClient,
    snapshot: SnapshotHandle,
    project: ProjectHandle,
    file: &str,
    position: u32,
    expected: &str,
    mode: ApiMode,
) {
    let observed_type = client
        .get_type_at_position(snapshot.clone(), project.clone(), file, position)
        .await
        .unwrap()
        .unwrap_or_else(|| panic!("expected a type at position {position} in {file} for {mode:?}"));
    let rendered = client
        .type_to_string(snapshot, project, observed_type.id, None, None)
        .await
        .unwrap();
    assert_eq!(rendered, expected, "unexpected rendered type for {mode:?}");
}

fn diagnostic_codes(diagnostics: &[Diagnostic]) -> BTreeSet<String> {
    diagnostics
        .iter()
        .filter_map(|diagnostic| match diagnostic.code.as_ref()? {
            NumberOrString::Number(code) => Some(format!("TS{code}")),
            NumberOrString::String(code) => Some(code.clone()),
        })
        .collect()
}

fn position_in(text: &str, marker: &str) -> u32 {
    u32::try_from(text.find(marker).unwrap() + 1).unwrap()
}

fn fixture(files: &[(&str, &str)]) -> tempfile::TempDir {
    let project = tempdir().unwrap();
    for (relative, contents) in files {
        let path = project.path().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, contents).unwrap();
    }
    project
}
