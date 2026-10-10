//! Retain native JSX roots inside otherwise plain expression children.
//!
//! Normal compilation keeps its existing raw-expression contract. Typecheck
//! lowering must also retain the JSX nodes nested in those expressions: their
//! source spans alone cannot be emitted as syntax in a plain `.ts` document.

use oxc_ast::ast::{Expression, JSXElement, JSXFragment};
use oxc_ast_visit::Visit;
use vize_relief::RootNode;

use super::Lowerer;

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    pub(crate) fn retain_nested_typecheck_roots(&mut self, expression: &Expression<'_>) {
        if self.preserve_slot_parameter_types {
            NestedRootLowerer { lowerer: self }.visit_expression(expression);
        }
    }

    pub(crate) fn take_typecheck_roots(&mut self) -> std::vec::Vec<RootNode<'a>> {
        std::mem::take(&mut self.pending_typecheck_roots)
    }
}

struct NestedRootLowerer<'l, 'a, 'm, 's: 'a> {
    lowerer: &'l mut Lowerer<'a, 'm, 's>,
}

impl<'ast> Visit<'ast> for NestedRootLowerer<'_, '_, '_, '_> {
    fn visit_jsx_element(&mut self, element: &JSXElement<'ast>) {
        // Structural children are lowered once by the existing native lowerer.
        // Its plain-expression branches retain any deeper expression roots.
        let root = self.lowerer.lower_element_root(element);
        self.lowerer.pending_typecheck_roots.push(root);
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'ast>) {
        let root = self.lowerer.lower_fragment_root(fragment);
        self.lowerer.pending_typecheck_roots.push(root);
    }
}
