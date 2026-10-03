//! Real provider facts and actual project configuration refuse before spawn.

use super::{TestResult, configured, require, support};
use vize_canon::{
    CorsaBridgeConfig,
    native_program::{NativeProgramChecker, NativeProgramError},
};
use vize_l0::Allocator;
use vize_l1::embed::Lang;
use vize_l4::targets::ts::project_program;

#[test]
fn workspace_and_javascript_options_cannot_silently_select_a_scratch_or_typescript_path()
-> TestResult {
    require(
        matches!(
            NativeProgramChecker::with_config(CorsaBridgeConfig::default()),
            Err(NativeProgramError::WorkspaceRequired)
        ),
        "explicit actual workspace required",
    )?;
    let project = tempfile::TempDir::new()?;
    let config = CorsaBridgeConfig {
        working_dir: Some(project.path().to_path_buf()),
        ..Default::default()
    };
    require(
        matches!(
            NativeProgramChecker::with_config(config),
            Err(NativeProgramError::WorkspaceConfigurationRequired)
        ),
        "existing actual configuration required",
    )?;
    let config = configured(
        project.path(),
        project.path().join("missing-required-backend"),
    )?;
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"allowJs":true,"checkJs":false}}"#,
    )?;
    let mut checker = NativeProgramChecker::with_config(config)?;
    let arena = Allocator::default();
    let file = support::file(&arena, "const value=1;", Lang::Js).ok_or("original File")?;
    let projection = project_program(&file).map_err(|_| "genuine projection")?;
    require(
        matches!(
            corsa::runtime::block_on(checker.check(&projection)),
            Err(NativeProgramError::JavaScriptCheckingDisabled)
        ),
        "JavaScript checkJs refusal before backend",
    )
}

#[test]
fn original_import_exports_and_all_invocations_require_authored_filename_authority() -> TestResult {
    let project = tempfile::TempDir::new()?;
    let config = configured(
        project.path(),
        project.path().join("missing-required-backend"),
    )?;
    let mut checker = NativeProgramChecker::with_config(config)?;
    let arena = Allocator::default();
    for source in ["import 'dep';", "export * from 'dep';"] {
        let file = support::file(&arena, source, Lang::Js).ok_or("original imported File")?;
        let projection = project_program(&file).map_err(|_| "complete imported projection")?;
        require(
            matches!(
                corsa::runtime::block_on(checker.check(&projection)),
                Err(NativeProgramError::ImportedProgram)
            ),
            "original source row refusal before backend",
        )?;
    }
    for source in [
        "let target=1; target();",
        "let target=1; new target();",
        "let target=1; target`text`;",
        "import('dep');",
    ] {
        let file = support::file(&arena, source, Lang::Js).ok_or("original invocation File")?;
        let projection = project_program(&file).map_err(|_| "complete invocation projection")?;
        require(
            projection.unit().has_invocations(),
            "genuine existing-walk invocation observation",
        )?;
        require(
            matches!(
                corsa::runtime::block_on(checker.check(&projection)),
                Err(NativeProgramError::InvokedProgram)
            ),
            "original invocation refusal before backend",
        )?;
    }
    Ok(())
}

#[test]
fn every_original_comment_requires_authored_filename_authority_before_backend() -> TestResult {
    let project = tempfile::TempDir::new()?;
    let config = configured(
        project.path(),
        project.path().join("missing-required-backend"),
    )?;
    let mut checker = NativeProgramChecker::with_config(config)?;
    let arena = Allocator::default();
    for source in [
        "/// <reference path='./types.d.ts' />\nexport {};",
        "/** @type {import('./types').Value} */ let value;",
        "// ordinary comment\nconst value=1;",
        "const value=1; /* tail */",
    ] {
        for lang in [Lang::Js, Lang::Ts] {
            let file = support::file(&arena, source, lang).ok_or("original commented File")?;
            let projection = project_program(&file).map_err(|_| "complete commented projection")?;
            require(
                projection.unit().has_comments(),
                "same original Program comment observation",
            )?;
            require(
                matches!(
                    corsa::runtime::block_on(checker.check(&projection)),
                    Err(NativeProgramError::CommentedProgram)
                ),
                "all comments refused before nonexistent backend can start",
            )?;
        }
    }
    Ok(())
}

#[test]
fn config_content_extends_and_config_selection_changes_permanently_refuse_a_stale_session()
-> TestResult {
    let arena = Allocator::default();
    let file = support::file(&arena, "const value=1;", Lang::Ts).ok_or("original File")?;
    let projection = project_program(&file).map_err(|_| "genuine projection")?;
    let project = tempfile::TempDir::new()?;
    let config = configured(
        project.path(),
        project.path().join("missing-required-backend"),
    )?;
    std::fs::write(
        project.path().join("base.json"),
        r#"{"compilerOptions":{"strict":true}}"#,
    )?;
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{"extends":"./base.json"}"#,
    )?;
    let mut checker = NativeProgramChecker::with_config(config.clone())?;
    std::fs::write(
        project.path().join("base.json"),
        r#"{"compilerOptions":{"strict":false}}"#,
    )?;
    require(
        matches!(
            corsa::runtime::block_on(checker.check(&projection)),
            Err(NativeProgramError::ConfigurationChanged)
        ),
        "actual inherited config digest refusal",
    )?;
    std::fs::write(
        project.path().join("base.json"),
        r#"{"compilerOptions":{"strict":true}}"#,
    )?;
    require(
        matches!(
            corsa::runtime::block_on(checker.check(&projection)),
            Err(NativeProgramError::ConfigurationChanged)
        ),
        "stale-session refusal remains sticky",
    )?;
    std::fs::rename(
        project.path().join("tsconfig.json"),
        project.path().join("jsconfig.json"),
    )?;
    let mut checker = NativeProgramChecker::with_config(config)?;
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"strict":true}}"#,
    )?;
    require(
        matches!(
            corsa::runtime::block_on(checker.check(&projection)),
            Err(NativeProgramError::ConfigurationChanged)
        ),
        "actual preferred config selection refusal",
    )
}
