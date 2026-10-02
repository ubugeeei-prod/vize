//! Pinned Vue role visibility, independent of runtime spelling and purity.
use super::{VueDeclaration, VueFileIssue, VueFileIssueKind, VueScriptReceipt, VueScriptRole};
use alloc::vec::Vec;
use oxc_ast::ast::Statement;
use vize_l2::file::{Declaration, Namespace, ScopeId, ScriptUnit, ScriptUnitId, TemplatePolicy};
use vize_l2::lang::js::{CallEvent, DeclaredEvent, FileObserver, StatementEvent};
use vize_l2::resolution::ResolutionErrorKind;

#[derive(Clone, Copy)]
pub(crate) struct Visibility {
    ordinary: Option<(ScriptUnitId, ScopeId)>,
    setup: Option<(ScriptUnitId, ScopeId)>,
}
impl Visibility {
    pub(super) fn new(ordinary: Option<VueScriptReceipt>, setup: Option<VueScriptReceipt>) -> Self {
        Self {
            ordinary: setup
                .and(ordinary)
                .map(|receipt| (receipt.unit, receipt.scope)),
            setup: setup.map(|receipt| (receipt.unit, receipt.scope)),
        }
    }
    pub(super) fn accepts(self, declaration: &Declaration) -> bool {
        declaration.namespace == Namespace::Value
            && [self.setup, self.ordinary]
                .into_iter()
                .flatten()
                .any(|(unit, scope)| declaration.unit == unit && declaration.scope == scope)
    }
}
impl TemplatePolicy for Visibility {
    fn visible(self, declaration: &Declaration) -> bool {
        self.accepts(declaration)
    }
}

pub(super) struct PolicyObserver<'f> {
    pub role: VueScriptRole,
    pub source_type: oxc_span::SourceType,
    pub receipt: Option<VueScriptReceipt>,
    pub declarations: &'f mut Vec<VueDeclaration>,
    pub issues: &'f mut Vec<VueFileIssue>,
}
impl<'a> FileObserver<'a> for PolicyObserver<'_> {
    type Checkpoint = ();
    fn unit(&mut self, unit: &ScriptUnit) {
        self.receipt = Some(VueScriptReceipt {
            unit: unit.id,
            scope: unit.scope,
            span: unit.span,
            profile: unit.profile,
            source_type: self.source_type,
        });
    }
    fn statement(&mut self, event: StatementEvent<'_, 'a>) {
        let kind = match event.statement {
            Statement::ExportNamedDeclaration(_)
            | Statement::ExportAllDeclaration(_)
            | Statement::ExportDefaultDeclaration(_)
                if self.role == VueScriptRole::Setup =>
            {
                Some(VueFileIssueKind::SetupExport)
            }
            Statement::ExportDefaultDeclaration(_) => Some(VueFileIssueKind::UnsupportedOptions),
            _ => None,
        };
        if let Some(kind) = kind {
            self.issues.push(VueFileIssue {
                unit: Some(event.unit),
                scope: Some(event.scope),
                span: event.span,
                kind,
            });
        }
    }
    fn declared(&mut self, event: DeclaredEvent<'_, 'a>) {
        if !self.receipt.is_some_and(|receipt| {
            event.declaration.unit == receipt.unit && event.declaration.scope == receipt.scope
        }) {
            return;
        }
        self.declarations.push(VueDeclaration {
            binding: event.declaration.id,
            role: self.role,
            namespace: event.declaration.namespace,
            initializer_span: event.initializer.map(|(_, span)| span),
        });
    }
    fn checkpoint(&self) {}
    fn call(&mut self, event: CallEvent<'_, 'a>) -> Result<(), ResolutionErrorKind> {
        // Refusal evidence is retained; no accepted call rows are registered.
        self.issues.push(VueFileIssue {
            unit: Some(event.unit),
            scope: Some(event.scope),
            span: event.span,
            kind: VueFileIssueKind::UnsupportedCall {
                optional: event.call.optional,
            },
        });
        Err(ResolutionErrorKind::UnsupportedSyntax)
    }
    fn rollback(&mut self, _: ()) {}
}
