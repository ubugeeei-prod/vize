//! Physical and opened editor coordinates are independent provider views.

use super::{oracle, reference::DiskReference, require, require_equal, support};
use lsp_types::{Diagnostic, DiagnosticSeverity};
use std::path::Path;
use vize_canon::native_program::{NativeProgramChecker, NativeProgramError};
use vize_l0::{Allocator, Span, cstr, line_index::LineBreaks};
use vize_l1::embed::Lang;
use vize_l4::targets::ts::project_program;

pub(super) async fn check_case(
    case: &serde_json::Value,
    reference_dir: &Path,
    reference: &DiskReference,
    checker: &mut NativeProgramChecker,
    editor_oracle: &serde_json::Value,
) -> Result<bool, Box<dyn std::error::Error>> {
    let id = case
        .get("id")
        .and_then(serde_json::Value::as_str)
        .ok_or("case id")?;
    let source = case
        .get("source")
        .and_then(serde_json::Value::as_str)
        .ok_or("original source")?;
    let family = reference_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("reference family")?;
    let expected = oracle::case(editor_oracle, family, id)?;
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
    let path = reference_dir
        .join(cstr!("reference-{id}.{}", projection.source_kind().extension()).as_str());
    require(
        std::fs::read(&path)? == source.as_bytes(),
        "reference file retains exact original bytes",
    )?;
    let original = reference.diagnostics(&path).await?;
    let checked = checker.check(&projection).await?;
    eprintln!(
        "actual physical report {family}/{id}: {}",
        serde_json::to_string(&original)?
    );
    eprintln!(
        "actual native opened report {family}/{id}: {}",
        serde_json::to_string(checked.backend_report())?
    );
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
    require(
        original
            .related_documents
            .as_ref()
            .is_none_or(|documents| documents.is_empty()),
        "no unbaselined physical foreign document report",
    )?;
    // Compare each complete editor vector in its own actual source view.
    // The provider's physical BOM read is not shifted to the opened snapshot.
    require_equal(
        &serde_json::to_value(&original.full_document_diagnostic_report.items)?,
        expected
            .pointer("/physical/diagnostics")
            .ok_or("physical complete editor oracle")?,
        id,
    )?;
    require_equal(
        &serde_json::to_value(
            checked
                .diagnostics()
                .iter()
                .map(|item| item.backend())
                .collect::<Vec<_>>(),
        )?,
        expected
            .pointer("/openedProjection/diagnostics")
            .ok_or("actual projected opened complete editor oracle")?,
        id,
    )?;
    let physical_source = expected
        .pointer("/physical/source")
        .and_then(serde_json::Value::as_str)
        .ok_or("physical provider source view")?;
    for raw in &original.full_document_diagnostic_report.items {
        authored_span(physical_source, raw)?;
    }
    let opened: Vec<Diagnostic> = serde_json::from_value(
        expected
            .pointer("/openedProjection/diagnostics")
            .cloned()
            .ok_or("opened complete vector")?,
    )?;
    require(
        checked.has_errors()
            == opened
                .iter()
                .any(|item| item.severity == Some(DiagnosticSeverity::ERROR)),
        "meaningful positive/errors include every hint without treating it as an error",
    )?;
    require(
        checked.diagnostics().len() == opened.len(),
        "no missing or extra opened diagnostics",
    )?;
    for (mapped, expected) in checked.diagnostics().iter().zip(&opened) {
        require(
            mapped
                .span()
                .map_err(|_| "actual backend mapping refusal")?
                == authored_span(source, expected)?,
            "exact opened original byte range",
        )?;
        require_equal(
            &serde_json::to_value(mapped.original_range())?,
            &serde_json::to_value(expected.range)?,
            id,
        )?;
    }
    eprintln!(
        "native Program reference {family}/{id}: complete physical and opened editor vectors matched independently"
    );
    Ok(true)
}

fn authored_span(
    source: &str,
    diagnostic: &Diagnostic,
) -> Result<Span, Box<dyn std::error::Error>> {
    let offset = |position: lsp_types::Position| {
        LineBreaks::Lsp
            .position_to_offset(source, position.line, position.character)
            .ok_or("editor UTF16 source boundary")
    };
    let start = offset(diagnostic.range.start)?;
    let end = offset(diagnostic.range.end)?;
    require(
        start <= end && source.get(start..end).is_some(),
        "original editor range spelling/boundaries",
    )?;
    Ok(Span::new(u32::try_from(start)?, u32::try_from(end)?))
}
