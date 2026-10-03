//! Independent TypeScript references and complete original Corsa observations.

use super::{
    TestResult, case::check_case, configured, oracle, reference::DiskReference, require, support,
};
use std::path::Path;
use vize_canon::native_program::NativeProgramChecker;
use vize_l0::{Allocator, cstr};
use vize_l1::embed::Lang;
use vize_l4::targets::ts::{MappingError, project_program_no_links};

#[test]
fn real_backend_preserves_complete_original_diagnostics_and_independent_reference() -> TestResult {
    corsa::runtime::block_on(async {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(std::path::Path::parent)
            .ok_or("workspace root")?;
        let backend = vize_carton::corsa_resolver::resolve_corsa_executable(
            vize_carton::corsa_resolver::CorsaResolveRequest {
                project_root: Some(root),
                ..Default::default()
            },
        )?;
        // Missing required executables fail this test; no optional backend skip.
        let project = tempfile::TempDir::new()?;
        configured(project.path(), backend.clone())?;
        // Native startup must honor an actual config-only workspace itself.
        let native_project = tempfile::TempDir::new()?;
        let native_config = configured(native_project.path(), backend.clone())?;
        let pack: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../davinci/vize_l4/tests/fixtures/program-checker.json"
        ))?;
        let cases = pack
            .get("cases")
            .and_then(serde_json::Value::as_array)
            .ok_or("independent reference cases")?;
        require(
            cases.len() == 10,
            "complete original independent reference family",
        )?;
        let bounded: serde_json::Value = serde_json::from_str(include_str!("checker.json"))?;
        let bounded_cases = bounded
            .get("cases")
            .and_then(serde_json::Value::as_array)
            .ok_or("independent comment-free references")?;
        require(
            bounded_cases.len() == 8,
            "complete bounded comment-free family",
        )?;
        let original_dir = project.path().join("original");
        let bounded_dir = project.path().join("bounded");
        prepare_references(&original_dir, cases)?;
        prepare_references(&bounded_dir, bounded_cases)?;
        let editor_oracle = oracle::capture(
            root,
            &[
                ("original", &original_dir, cases),
                ("bounded", &bounded_dir, bounded_cases),
            ],
        )?;
        // The actual configured project contains exact original sources before
        // backend startup; inferred virtual projects are not reference inputs.
        let mut reference = DiskReference::spawn(project.path(), &backend).await?;
        let mut checker = NativeProgramChecker::with_config(native_config)?;
        let unknown = project.path().join("not-in-open-snapshot.ts");
        std::fs::write(&unknown, "const unlisted=1;")?;
        require(
            reference.diagnostics(&unknown).await.is_err(),
            "an existing unknown closed file cannot synthesize empty diagnostics",
        )?;
        let probe = cases
            .iter()
            .find(|case| case.get("id") == Some(&serde_json::json!("ts-property")))
            .ok_or("fixed original nonempty diagnostic probe")?;
        require(
            probe
                .get("codes")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|codes| !codes.is_empty()),
            "fixed original probe requires a real nonempty compiler vector",
        )?;
        require(
            check_case(
                probe,
                &original_dir,
                &reference,
                &mut checker,
                &editor_oracle,
            )
            .await?,
            "fixed nonempty whole diagnostic payload actually checked first",
        )?;
        let mut accepted = 0;
        let mut refused = 0;
        for case in cases {
            if check_case(
                case,
                &original_dir,
                &reference,
                &mut checker,
                &editor_oracle,
            )
            .await?
            {
                accepted += 1;
            } else {
                refused += 1;
            }
        }
        require(
            accepted > 0 && refused > 0,
            "original admitted and commented reference cases observed",
        )?;
        for case in bounded_cases {
            require(
                check_case(case, &bounded_dir, &reference, &mut checker, &editor_oracle).await?,
                "every bounded reference actually checked",
            )?;
        }
        let arena = Allocator::default();
        let file = support::file(&arena, "const value=1; value.missing;", Lang::Js)
            .ok_or("original unrecorded File")?;
        let plain =
            project_program_no_links(&file).map_err(|_| "original unrecorded projection")?;
        let unmapped = checker.check(&plain).await?;
        require(
            !unmapped.is_complete() && unmapped.has_errors(),
            "actual unmapped error retained",
        )?;
        require(
            !unmapped.diagnostics().is_empty()
                && unmapped.diagnostics().iter().all(|item| {
                    item.span() == Err(MappingError::Unrecorded) && item.original_range().is_none()
                }),
            "all original unrecorded coordinates refused",
        )?;
        // The same real session sees bad then good JavaScript, with no stale
        // diagnostics retained by either the native adapter or its transport.
        require(
            check_case(
                cases
                    .iter()
                    .find(|case| case.get("id") == Some(&serde_json::json!("js-property")))
                    .ok_or("negative JavaScript reference")?,
                &original_dir,
                &reference,
                &mut checker,
                &editor_oracle,
            )
            .await?,
            "negative JavaScript reference actually checked",
        )?;
        require(
            check_case(
                bounded_cases
                    .iter()
                    .find(|case| case.get("id") == Some(&serde_json::json!("js-valid")))
                    .ok_or("positive JavaScript reference")?,
                &bounded_dir,
                &reference,
                &mut checker,
                &editor_oracle,
            )
            .await?,
            "positive JavaScript reference actually checked",
        )?;
        checker.shutdown().await?;
        reference.close().await?;
        require(
            reference
                .diagnostics(&bounded_dir.join("reference-js-valid.mjs"))
                .await
                .is_err(),
            "a closed actual backend cannot synthesize empty diagnostics",
        )?;
        Ok(())
    })
}

fn prepare_references(directory: &Path, cases: &[serde_json::Value]) -> TestResult {
    std::fs::create_dir_all(directory)?;
    for case in cases {
        let id = case
            .get("id")
            .and_then(serde_json::Value::as_str)
            .ok_or("reference case id")?;
        let extension = match case.get("kind").and_then(serde_json::Value::as_str) {
            Some("js") => "mjs",
            Some("ts") => "ts",
            _ => return Err("reference language".into()),
        };
        let source = case
            .get("source")
            .and_then(serde_json::Value::as_str)
            .ok_or("reference source")?;
        let name = cstr!("reference-{id}.{extension}");
        std::fs::write(directory.join(name.as_str()), source)?;
    }
    Ok(())
}
