//! Retain native JSX roots inside otherwise plain expression children.
//!
//! Normal compilation keeps its existing raw-expression contract. Typecheck
//! lowering must also retain the JSX nodes nested in those expressions: their
//! source spans alone cannot be emitted as syntax in a plain `.ts` document.

use super::ScopedStyleExpr;
use oxc_ast::ast::{Expression, JSXElement, JSXFragment};
use oxc_ast_visit::Visit;
use vize_l0::String;
use vize_relief::RootNode;

use super::Lowerer;

pub(crate) struct TypecheckRoot<'a> {
    pub(crate) root: RootNode<'a>,
    pub(crate) scoped_style: Option<(String, std::vec::Vec<ScopedStyleExpr>)>,
}

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    pub(crate) fn retain_nested_typecheck_roots(&mut self, expression: &Expression<'_>) {
        if self.preserve_slot_parameter_types {
            NestedRootLowerer { lowerer: self }.visit_expression(expression);
        }
    }

    pub(crate) fn take_typecheck_roots(&mut self) -> std::vec::Vec<TypecheckRoot<'a>> {
        std::mem::take(&mut self.pending_typecheck_roots)
    }
}

struct NestedRootLowerer<'l, 'a, 'm, 's: 'a> {
    lowerer: &'l mut Lowerer<'a, 'm, 's>,
}

impl<'ast> Visit<'ast> for NestedRootLowerer<'_, '_, '_, '_> {
    fn visit_jsx_element(&mut self, element: &JSXElement<'ast>) {
        self.lowerer.retain_typecheck_element(element);
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'ast>) {
        self.lowerer.retain_typecheck_fragment(fragment);
    }
}

impl<'a, 'm, 's: 'a> Lowerer<'a, 'm, 's> {
    pub(crate) fn retain_typecheck_element(&mut self, element: &JSXElement<'_>) {
        if !self.preserve_slot_parameter_types {
            return;
        }
        // Structural children are lowered once by the existing native lowerer.
        // Its plain-expression branches retain any deeper expression roots.
        let outer_styles = std::mem::take(&mut self.pending_styles);
        let root = self.lower_element_root(element);
        let scoped_style = self.take_scoped_styles();
        self.pending_styles = outer_styles;
        self.pending_typecheck_roots
            .push(TypecheckRoot { root, scoped_style });
    }

    pub(crate) fn retain_typecheck_fragment(&mut self, fragment: &JSXFragment<'_>) {
        if !self.preserve_slot_parameter_types {
            return;
        }
        let outer_styles = std::mem::take(&mut self.pending_styles);
        let root = self.lower_fragment_root(fragment);
        let scoped_style = self.take_scoped_styles();
        self.pending_styles = outer_styles;
        self.pending_typecheck_roots
            .push(TypecheckRoot { root, scoped_style });
    }
}
