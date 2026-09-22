//! Acceptance of an expression-dialect answer (the module docs of
//! [`crate::expression`] list the checks, in order).

use super::{
    AcceptedAnalysis, Analysis, AnalysisError, ExpressionBatch, ProjectionPage, read_facts,
    read_projection,
};
use crate::contract::Span;

/// Accept a guest's analysis of `batch`.
///
/// # Errors
///
/// The first check the analysis fails, as an [`AnalysisError`].
pub fn accept_analysis(
    batch: &ExpressionBatch,
    analysis: Analysis,
) -> Result<AcceptedAnalysis, AnalysisError> {
    let facts = read_facts(&analysis.facts)?;
    let projection = read_projection(&analysis.projection)?;

    let mut expected: Vec<u32> = batch.expressions.iter().map(|e| e.id).collect();
    expected.sort_unstable();
    let alpha = &facts.alpha;
    for section in [
        sorted(alpha.references.keys()),
        sorted(alpha.exact.keys()),
        sorted(alpha.constant.keys()),
    ] {
        if section != expected {
            return Err(AnalysisError::FactIds {
                expected,
                found: section,
            });
        }
    }
    for id in &expected {
        let exact = alpha.exact.get(id).copied().unwrap_or(false);
        let references = alpha.references.get(id).map_or("", |r| r.as_str());
        let unknown = references
            .split(',')
            .filter(|name| !name.is_empty())
            .find(|name| !batch.environment.iter().any(|b| b.name == *name));
        if let (true, Some(name)) = (exact, unknown) {
            return Err(AnalysisError::UnknownBinding {
                id: *id,
                name: name.into(),
            });
        }
    }

    check_rows(batch, &projection)?;
    let inside = |span: Span| {
        batch
            .expressions
            .iter()
            .any(|e| span.start <= span.end && e.span.start <= span.start && span.end <= e.span.end)
    };
    for (index, diagnostic) in analysis.diagnostics.iter().enumerate() {
        for span in core::iter::once(diagnostic.span).chain(diagnostic.parts.iter().map(|p| p.span))
        {
            if !inside(span) {
                return Err(AnalysisError::DiagnosticSpan { index, span });
            }
        }
    }
    Ok(AcceptedAnalysis {
        analysis,
        facts,
        projection,
    })
}

fn sorted<'a>(keys: impl Iterator<Item = &'a u32>) -> Vec<u32> {
    let mut keys: Vec<u32> = keys.copied().collect();
    keys.sort_unstable();
    keys
}

fn check_rows(batch: &ExpressionBatch, projection: &ProjectionPage) -> Result<(), AnalysisError> {
    let text = projection.text.as_str();
    let generated_ok = |r: super::Range| {
        (r.end as usize) <= text.len()
            && text.is_char_boundary(r.start as usize)
            && text.is_char_boundary(r.end as usize)
    };
    let authored_ok = |r: super::Range| {
        batch
            .expressions
            .iter()
            .any(|e| e.span.start <= r.start && r.end <= e.span.end)
    };
    let within = |inner: super::Range, outer: super::Range| {
        outer.start <= inner.start && inner.end <= outer.end
    };
    for (index, row) in projection.rows.iter().enumerate() {
        let reason = if !generated_ok(row.generated) {
            Some("leaves the generated text")
        } else if !authored_ok(row.authored) {
            Some("links outside every expression")
        } else if row.sub_spans.iter().any(|(generated, authored)| {
            !within(*generated, row.generated) || !within(*authored, row.authored)
        }) {
            Some("has a sub-span outside the row")
        } else if row.sub_spans.iter().any(|(generated, _)| !generated_ok(*generated)) {
            Some("has a sub-span off a character boundary")
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(AnalysisError::Row { index, reason });
        }
    }
    Ok(())
}
