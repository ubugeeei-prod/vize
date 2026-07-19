#![cfg(feature = "application-analysis")]

use vize_croquis_cf::{
    CrossFileAnalyzer, CrossFileDiagnostic, CrossFileDiagnosticKind, CrossFileOptions,
    CrossFileResult, DiagnosticSeverity, FileId,
};
use vize_doctor::application_analysis::{
    ApplicationAnalysisError, findings_from_application_graph,
};

#[test]
fn fails_closed_when_a_primary_or_related_source_is_stale() {
    let analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    let primary_result = result_with_diagnostic(FileId::INVALID);
    let error = findings_from_application_graph(&analyzer, &primary_result).unwrap_err();
    assert!(matches!(
        error,
        ApplicationAnalysisError::MissingSource {
            file_id: u32::MAX,
            related: false,
            ..
        }
    ));

    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    let source = analyzer.add_file("src/App.ts", "export const app = 1");
    let mut related_result = result_with_diagnostic(source);
    related_result.diagnostics[0].related_files.push((
        FileId::INVALID,
        0,
        "Stale related source".into(),
    ));
    let error = findings_from_application_graph(&analyzer, &related_result).unwrap_err();
    assert!(matches!(
        error,
        ApplicationAnalysisError::MissingSource { related: true, .. }
    ));
}

#[test]
fn rejects_absolute_sources_without_a_workspace_boundary() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    let source = analyzer.add_file("/outside/App.ts", "export const app = 1");

    let error =
        findings_from_application_graph(&analyzer, &result_with_diagnostic(source)).unwrap_err();
    assert!(matches!(
        error,
        ApplicationAnalysisError::SourceOutsideWorkspace { related: false, .. }
    ));
}

#[test]
fn accepts_relative_sources_without_a_workspace_boundary() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    let source = analyzer.add_file("src/App.ts", "export const app = 1");

    let findings =
        findings_from_application_graph(&analyzer, &result_with_diagnostic(source)).unwrap();
    assert_eq!(findings[0].primary.path, "src/App.ts");
}

#[test]
fn collapses_parent_segments_in_relative_sources() {
    let mut analyzer = CrossFileAnalyzer::new(CrossFileOptions::minimal());
    let source = analyzer.add_file("src/../App.ts", "export const app = 1");

    let findings =
        findings_from_application_graph(&analyzer, &result_with_diagnostic(source)).unwrap();
    assert_eq!(findings[0].primary.path, "App.ts");
}

#[test]
fn rejects_unverified_case_mismatched_workspace_prefix() {
    let mut analyzer =
        CrossFileAnalyzer::with_project_root(CrossFileOptions::minimal(), "/Workspace");
    let source = analyzer.add_file("/workspace/src/App.ts", "export const app = 1");

    let error =
        findings_from_application_graph(&analyzer, &result_with_diagnostic(source)).unwrap_err();
    assert!(matches!(
        error,
        ApplicationAnalysisError::SourceOutsideWorkspace { .. }
    ));
}

#[cfg(unix)]
#[test]
fn resolves_workspace_symlinks_without_allowing_escape() {
    use std::{fs, os::unix::fs::symlink};

    let directory = tempfile::tempdir().unwrap();
    let workspace = directory.path().join("workspace");
    let alias = directory.path().join("workspace-alias");
    let outside = directory.path().join("outside");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(workspace.join("src/App.ts"), "export const app = 1").unwrap();
    fs::write(outside.join("Secret.ts"), "export const secret = 1").unwrap();
    symlink(&workspace, &alias).unwrap();
    symlink(&outside, workspace.join("escape")).unwrap();

    let mut analyzer = CrossFileAnalyzer::with_project_root(CrossFileOptions::minimal(), &alias);
    let app = analyzer.add_file(workspace.join("src/App.ts"), "export const app = 1");
    let findings =
        findings_from_application_graph(&analyzer, &result_with_diagnostic(app)).unwrap();
    assert_eq!(findings[0].primary.path, "src/App.ts");

    let secret = analyzer.add_file(
        workspace.join("escape/Secret.ts"),
        "export const secret = 1",
    );
    let error =
        findings_from_application_graph(&analyzer, &result_with_diagnostic(secret)).unwrap_err();
    assert!(matches!(
        error,
        ApplicationAnalysisError::SourceOutsideWorkspace { .. }
    ));
}

fn result_with_diagnostic(source: FileId) -> CrossFileResult {
    CrossFileResult {
        diagnostics: vec![CrossFileDiagnostic::new(
            CrossFileDiagnosticKind::UnresolvedImport {
                specifier: "./missing".into(),
                import_offset: 0,
            },
            DiagnosticSeverity::Error,
            source,
            0,
            "Import cannot be resolved",
        )],
        ..CrossFileResult::default()
    }
}
