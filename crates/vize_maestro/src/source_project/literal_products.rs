//! Explicitly requested editor diagnostics and binding types for one open TS script.
use super::{SnapshotRefusal, SourceSnapshot};
use crate::document::DocumentStore;
use crate::utils::{offset_to_position, position_to_offset};
use oxc_span::SourceType;
use tower_lsp::lsp_types::{
    Diagnostic, DiagnosticSeverity, Hover, NumberOrString, Position, Range,
};
use vize_canon::{
    SfcTypeDiagnostic,
    literal_assignments::{BindingType, visit_binding_types},
};
use vize_l0::{Allocator, Span};
use vize_l1::{
    container::vue::{DescriptorObservation, ScriptRole},
    embed::{Lang, authored_program::AuthoredProgram},
};
use vize_l2::query::script::assignments::{
    AssignmentError, LiteralAssignments, Row, ScriptRefusal,
};
use vize_patina::{
    LintDiagnostic,
    checked_prefer_const::{PreferConstRefusal, default_prefer_const},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LiteralProductError {
    Host(SnapshotRefusal),
    Descriptor,
    ForeignSource,
    MissingScript,
    Language,
    Script(ScriptRefusal),
    Evidence(AssignmentError),
    Position,
}

pub struct PreparedLiteralProducts<'s> {
    snapshot: &'s SourceSnapshot,
    script: Span,
    hover_role: Option<ScriptRole>,
    types: Vec<SfcTypeDiagnostic>,
    bindings: Vec<BindingType>,
    lint: Result<Vec<LintDiagnostic>, PreferConstRefusal>,
    outside: usize,
}

#[derive(Debug)]
pub struct LiteralEditorProducts {
    pub version: i32,
    pub type_errors: Vec<Diagnostic>,
    pub prefer_const: Result<Vec<Diagnostic>, PreferConstRefusal>,
    pub binding_hover: Option<Hover>,
    /// Statements or individual declarators outside the local type judgment.
    pub outside: usize,
}

impl SourceSnapshot {
    /// The original block is parsed once. The checked rows serve Canon, Patina
    /// and the editor. Refusal of a whole-script lint result is independent of
    /// usable local type judgments elsewhere in this same script.
    pub fn prepare_literal_products<'s>(
        &'s self,
        documents: &DocumentStore,
        owner: &DescriptorObservation<'_>,
        role: ScriptRole,
    ) -> Result<PreparedLiteralProducts<'s>, LiteralProductError> {
        self.check_current(documents)
            .map_err(LiteralProductError::Host)?;
        if self.language_id() != "vue" {
            return Err(LiteralProductError::Language);
        }
        if !core::ptr::eq(self.source(), owner.source()) {
            return Err(LiteralProductError::ForeignSource);
        }
        let view = owner
            .admitted()
            .map_err(|_| LiteralProductError::Descriptor)?;
        let selected = match role {
            ScriptRole::Ordinary => view.ordinary(),
            ScriptRole::Setup => view.setup(),
        }
        .ok_or(LiteralProductError::MissingScript)?;
        if selected.lang() != Lang::Ts {
            return Err(LiteralProductError::Language);
        }
        let arena = Allocator::default();
        let syntax = AuthoredProgram::parse(&arena, selected.block(), SourceType::ts());
        let query = LiteralAssignments::new(&syntax).map_err(LiteralProductError::Script)?;
        let rows = query.prove();
        let checked = query.check(&rows).map_err(LiteralProductError::Evidence)?;
        let mut types = Vec::new();
        let mut bindings = Vec::new();
        visit_binding_types(&checked, |binding, diagnostic| {
            bindings.push(binding);
            if let Some(d) = diagnostic {
                types.push(d);
            }
        });
        let mut warnings = Vec::new();
        let lint = default_prefer_const(owner, &checked, |warning| warnings.push(warning))
            .map(|()| warnings);
        Ok(PreparedLiteralProducts {
            snapshot: self,
            script: selected.block().span(),
            hover_role: (view.ordinary().is_none() || view.setup().is_none()).then_some(role),
            types,
            bindings,
            lint,
            outside: rows.iter().filter(|r| matches!(r, Row::Outside(_))).count(),
        })
    }
}
impl PreparedLiteralProducts<'_> {
    /// Resolve byte ranges through the current document's own UTF-16 index.
    pub fn publish(
        self,
        documents: &DocumentStore,
        hover: Option<Position>,
    ) -> Result<LiteralEditorProducts, LiteralProductError> {
        self.snapshot
            .with_current(documents, |doc| {
                let text = doc.position_text();
                let range = |span: Span| -> Result<Range, LiteralProductError> {
                    Ok(Range::new(
                        offset_to_position(text, span.start as usize)
                            .ok_or(LiteralProductError::Position)?,
                        offset_to_position(text, span.end as usize)
                            .ok_or(LiteralProductError::Position)?,
                    ))
                };
                let binding =
                    hover
                        .and_then(|at| position_to_offset(text, at))
                        .and_then(|offset| {
                            let i = self
                                .bindings
                                .partition_point(|item| item.binding.end as usize <= offset);
                            self.bindings
                                .get(i)
                                .filter(|item| item.binding.start as usize <= offset)
                        });
                let binding_hover = if let (Some(item), Some(role)) = (binding, self.hover_role) {
                    hover::typed_binding_hover(
                        self.snapshot.source(),
                        self.script,
                        role,
                        *item,
                        range(item.binding)?,
                    )
                } else {
                    None
                };
                let type_errors = self
                    .types
                    .into_iter()
                    .map(|d| {
                        Ok(Diagnostic {
                            range: range(Span::new(d.start, d.end))?,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::Number(2322)),
                            source: Some("ts".into()),
                            message: d.message.into(),
                            ..Default::default()
                        })
                    })
                    .collect::<Result<_, LiteralProductError>>()?;
                let prefer_const = match self.lint {
                    Err(reason) => Err(reason),
                    Ok(rows) => Ok(rows
                        .into_iter()
                        .map(|d| {
                            Ok(Diagnostic {
                                range: range(Span::new(d.start, d.end))?,
                                severity: Some(DiagnosticSeverity::WARNING),
                                code: Some(NumberOrString::String(d.rule_name.into())),
                                source: Some("vize".into()),
                                message: d.message.into(),
                                ..Default::default()
                            })
                        })
                        .collect::<Result<_, LiteralProductError>>()?),
                };
                Ok(LiteralEditorProducts {
                    version: doc.version,
                    type_errors,
                    prefer_const,
                    binding_hover,
                    outside: self.outside,
                })
            })
            .map_err(LiteralProductError::Host)?
    }
}

pub(super) mod hover;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod wire;

#[cfg(test)]
mod corpus;

#[cfg(all(test, feature = "micro-profiling"))]
mod profile;
