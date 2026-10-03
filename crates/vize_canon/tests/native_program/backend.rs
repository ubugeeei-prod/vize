//! Independent TypeScript references and complete original Corsa observations.

use super::{TestResult, configured, reference::DiskReference, require, require_equal, support};
use std::path::Path;
use vize_canon::LspDiagnostic;
use vize_canon::native_program::{NativeProgramChecker, NativeProgramError};
use vize_l0::{
    Allocator, Span, cstr,
    line_index::{LineBreaks, utf16_len, utf16_offset},
};
use vize_l1::embed::Lang;
use vize_l4::targets::ts::{MappingError, project_program, project_program_no_links};

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
        // The actual configured project contains exact original sources before
        // backend startup; inferred virtual projects are not reference inputs.
        let reference = DiskReference::spawn(project.path(), &backend).await?;
        let mut checker = NativeProgramChecker::with_config(native_config)?;
        let unknown = project.path().join("not-in-open-snapshot.ts");
        std::fs::write(&unknown, "const unlisted=1;")?;
        require(
            reference.diagnostics(&unknown).await.is_err(),
            "an existing unknown closed file cannot synthesize empty diagnostics",
        )?;
        let mut accepted = 0;
        let mut refused = 0;
        for case in cases {
            if check_case(case, &original_dir, &reference, &mut checker).await? {
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
                check_case(case, &bounded_dir, &reference, &mut checker).await?,
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

async fn check_case(
    case: &serde_json::Value,
    reference_dir: &Path,
    reference: &DiskReference,
    checker: &mut NativeProgramChecker,
) -> Result<bool, Box<dyn std::error::Error>> {
    let id = case
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or("case id")?;
    let source = case
        .get("source")
        .and_then(serde_json::Value::as_str)
        .ok_or("original source")?;
    let lang = match case.get("kind").and_then(serde_json::Value::as_str) {
        Some("js") => Lang::Js,
        Some("ts") => Lang::Ts,
        _ => return Err("unknown independent original language".into()),
    };
    let arena = Allocator::default();
    let file = support::file(&arena, source, lang).ok_or("genuine original File")?;
    require(file.is_complete(), "genuine original File completes")?;
    let projection = project_program(&file).map_err(|_| "genuine original L4 projection")?;
    if projection.unit().has_comments() {
        require(
            matches!(
                checker.check(&projection).await,
                Err(NativeProgramError::CommentedProgram)
            ),
            "original commented references remain typed refusals",
        )?;
        return Ok(false);
    }
    let name = cstr!("reference-{id}.{}", projection.source_kind().extension());
    let path = reference_dir.join(name.as_str());
    require(
        std::fs::read(&path)? == source.as_bytes(),
        "reference file retains exact original bytes",
    )?;
    let original = reference.diagnostics(&path).await?;
    let checked = checker.check(&projection).await?;
    require(
        core::ptr::eq(checked.projection(), &projection),
        "same sealed projection borrow",
    )?;
    require(
        core::ptr::eq(checked.projection().file(), &file),
        "same genuine original File borrow",
    )?;
    require(
        checked.is_complete(),
        "all real backend coordinates/configuration mapped",
    )?;
    require_equal(
        &serde_json::to_value(&original)?,
        &serde_json::to_value(
            checked
                .diagnostics()
                .iter()
                .map(|item| item.backend())
                .collect::<Vec<_>>(),
        )?,
        id,
    )?;
    let expected = case
        .get("diagnostics")
        .and_then(serde_json::Value::as_array)
        .ok_or("independent whole diagnostic vector")?;
    require(
        original.len() == expected.len(),
        "no unexpected or missing backend diagnostics",
    )?;
    let codes = serde_json::to_value(original.iter().map(|item| &item.code).collect::<Vec<_>>())?;
    require_equal(
        &codes,
        case.get("codes").ok_or("independent diagnostic codes")?,
        id,
    )?;
    require(
        checked.has_errors() == !expected.is_empty(),
        "meaningful positive and negative checking",
    )?;
    for ((raw, mapped), expected) in original.iter().zip(checked.diagnostics()).zip(expected) {
        require_equal(&semantic_observation(source, raw)?, expected, id)?;
        let start = expected
            .get("start")
            .and_then(serde_json::Value::as_u64)
            .ok_or("reference UTF16 start")?;
        let length = expected
            .get("length")
            .and_then(serde_json::Value::as_u64)
            .ok_or("reference UTF16 length")?;
        let start = u32::try_from(start)?;
        let end = start
            .checked_add(u32::try_from(length)?)
            .ok_or("reference end")?;
        let byte_start = utf16_offset(source, start).ok_or("reference UTF16 boundary")?;
        let byte_end = utf16_offset(source, end).ok_or("reference UTF16 end boundary")?;
        require(
            mapped
                .span()
                .map_err(|_| "actual backend mapping refusal")?
                == Span::new(u32::try_from(byte_start)?, u32::try_from(byte_end)?),
            "exact original byte range",
        )?;
        require_equal(
            &serde_json::to_value(mapped.original_range())?,
            &serde_json::to_value(&raw.range)?,
            id,
        )?;
    }
    eprintln!(
        "native Program reference {}/{id}: complete original payload matched",
        reference_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("reference family")?,
    );
    Ok(true)
}

fn semantic_observation(
    source: &str,
    raw: &LspDiagnostic,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let start = LineBreaks::Lsp
        .position_to_offset(source, raw.range.start.line, raw.range.start.character)
        .ok_or("original backend start boundary")?;
    let end = LineBreaks::Lsp
        .position_to_offset(source, raw.range.end.line, raw.range.end.character)
        .ok_or("original backend end boundary")?;
    let prefix = source.get(..start).ok_or("original backend prefix")?;
    let spelling = source.get(start..end).ok_or("original backend spelling")?;
    Ok(serde_json::json!({"code":raw.code,"category":raw.severity,
        "message":raw.message,"start":utf16_len(prefix),"length":utf16_len(spelling),"authored":spelling}))
}
