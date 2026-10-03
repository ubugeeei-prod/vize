//! Only authentic selected setup membership or original alias rows are visible.

use crate::file::build::Facts;
use crate::file::{
    Declaration, DeclarationKind, Namespace, ScopeId, ScriptUnitId, TemplateDeclaration,
};
use vize_l1::{embed::Lang, markup::NativeTemplateComponent};

#[derive(Clone, Copy)]
pub(crate) struct NativeVisibility {
    setup: Option<(ScriptUnitId, ScopeId)>,
}

impl NativeVisibility {
    pub(super) fn checked(
        selected: &NativeTemplateComponent<'_>,
        facts: &Facts<'_>,
        setup: Option<ScriptUnitId>,
    ) -> Self {
        let setup = setup.and_then(|id| {
            let script = selected.setup()?;
            let unit = facts.units.iter().find(|unit| unit.id == id)?;
            let scope = facts.scopes.get(unit.scope.index() as usize)?;
            (id.index() as usize == script.container_index()
                && unit.span == script.block().span()
                && unit.profile.module
                && !unit.profile.jsx
                && unit.profile.typescript == (script.lang() == Lang::Ts)
                && matches!(script.lang(), Lang::Js | Lang::Ts)
                && unit.walk_completed()
                && unit.origin.setup_eligible
                && !unit.origin.has_call
                && !unit.origin.has_export
                && !unit.origin.reserved_binding
                && scope.id == unit.scope
                && scope.parent == Some(ScopeId(0))
                && scope.span == unit.span)
                .then_some((id, unit.scope))
        });
        Self { setup }
    }
}

impl crate::file::region::TemplatePolicy for NativeVisibility {
    fn visible(self, declaration: &Declaration) -> bool {
        self.setup.is_some_and(|(unit, scope)| {
            declaration.script_unit() == Some(unit)
                && declaration.scope == scope
                && declaration.is_direct_program()
                && declaration.namespace == Namespace::Value
                && matches!(
                    declaration.kind,
                    DeclarationKind::Const | DeclarationKind::Let | DeclarationKind::Var
                )
                && !crate::file::vue::reserved(declaration.name.as_str())
        })
    }
    fn visible_template(self, declaration: &TemplateDeclaration<'_, '_>) -> bool {
        declaration.original().is_some()
    }
}
