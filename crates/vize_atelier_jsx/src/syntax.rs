//! Source-factory-owned JSX syntax observations retained before realization.

use crate::diagnostics::JsxDiagnostic;
use crate::l2::{self, JsxL2Root, L2Refusal, ProjectionFacts};
use crate::{JsxLang, LowerOutput, LoweredRoot, Lowerer, LoweringOptions};
use vize_croquis::Croquis;
use vize_l0::{Allocator, Span};
use vize_l1_to_l2::framework::vue::jsx::{JsxComponent, JsxRoot, JsxSyntaxFacts};
use vize_l2::op::{ComponentOp, ElementOp};
use vize_relief::RootNode;

pub(crate) enum RootSyntax<'a> {
    Native(JsxSyntaxFacts<'a>),
    Projection(ProjectionFacts),
    Unavailable,
}

pub(crate) struct RetainedSyntaxRoot<'a> {
    lowered: LoweredRoot<'a>,
    source: &'a str,
    syntax: RootSyntax<'a>,
}

/// An immutable result of the original parse-to-root factory. Syntax facts
/// cannot be attached to arbitrary or subsequently edited render operations.
///
/// ```compile_fail
/// use vize_atelier_jsx::SyntaxLowerOutput;
/// let annotated = SyntaxLowerOutput { roots: Vec::new() };
/// ```
pub struct SyntaxLowerOutput<'a> {
    roots: std::vec::Vec<RetainedSyntaxRoot<'a>>,
    analysis: Croquis,
    diagnostics: std::vec::Vec<JsxDiagnostic>,
}

impl<'a> SyntaxLowerOutput<'a> {
    /// Original roots and their factory-paired syntax, in production order.
    pub fn roots(&self) -> impl ExactSizeIterator<Item = SyntaxRoot<'_, 'a>> {
        self.roots.iter().map(|root| SyntaxRoot { root })
    }

    /// The same whole-module semantic analysis produced by ordinary lowering.
    pub fn analysis(&self) -> &Croquis {
        &self.analysis
    }

    /// The complete original parse and lowering diagnostics.
    pub fn diagnostics(&self) -> &[JsxDiagnostic] {
        &self.diagnostics
    }

    /// Discard syntax custody and recover the ordinary mutable output.
    pub fn into_output(self) -> LowerOutput<'a> {
        LowerOutput {
            roots: self.roots.into_iter().map(|root| root.lowered).collect(),
            analysis: self.analysis,
            diagnostics: self.diagnostics,
        }
    }
}

/// Borrowed original root and exact-object syntax facts, minted only by the
/// source factory. Equal spans, copied ops and foreign roots cannot match.
#[derive(Clone, Copy)]
pub struct SyntaxRoot<'o, 'a> {
    root: &'o RetainedSyntaxRoot<'a>,
}

impl<'o, 'a> SyntaxRoot<'o, 'a> {
    /// Original lowered root, exposed immutably while syntax custody exists.
    pub fn lowered(self) -> &'o LoweredRoot<'a> {
        &self.root.lowered
    }

    /// Complete original module source.
    pub fn source(self) -> &'a str {
        self.root.source
    }

    /// Whether this root has a complete supported syntax-fact packet.
    pub fn has_syntax(self) -> bool {
        !matches!(self.root.syntax, RootSyntax::Unavailable)
    }

    /// Raw authored slot-function range for this exact produced element.
    pub fn slot_diagnostic_range(self, element: &ElementOp<'_>) -> Option<Span> {
        match &self.root.syntax {
            RootSyntax::Native(facts) => facts.slot_diagnostic_range(element),
            RootSyntax::Projection(facts) => facts.slot_diagnostic_range(element),
            RootSyntax::Unavailable => None,
        }
    }

    /// Whether this exact component began as an explicit intrinsic element.
    pub fn component_was_intrinsic(self, component: &ComponentOp<'_>) -> bool {
        match &self.root.syntax {
            RootSyntax::Native(facts) => facts.component_was_intrinsic(component),
            RootSyntax::Projection(facts) => facts.component_was_intrinsic(component),
            RootSyntax::Unavailable => false,
        }
    }
}

/// Parse once and retain original syntax facts at the actual L2 mint sites.
/// Ordinary [`crate::lower_source`] retains its zero-sized no-facts producer.
pub fn lower_source_with_lint_syntax<'a>(
    bump: &'a Allocator,
    allocator: &oxc_allocator::Allocator,
    source: &'a str,
    lang: JsxLang,
) -> SyntaxLowerOutput<'a> {
    lower_with_options(bump, allocator, source, lang, LoweringOptions::default())
}

fn lower_with_options<'a>(
    bump: &'a Allocator,
    allocator: &oxc_allocator::Allocator,
    source: &'a str,
    lang: JsxLang,
    options: LoweringOptions<'_>,
) -> SyntaxLowerOutput<'a> {
    let parts = crate::source::lower_parts::<SyntaxRoots>(bump, allocator, source, lang, options);
    SyntaxLowerOutput {
        roots: parts.roots,
        analysis: parts.analysis,
        diagnostics: parts.diagnostics,
    }
}

pub(crate) trait RootProjection<'a> {
    type Facts;
    type Root;

    fn lower(
        lowerer: &Lowerer<'a, '_, '_>,
        component: JsxComponent<'a>,
        oxc_root: JsxRoot<'_, '_>,
        root: &RootNode<'a>,
    ) -> (Result<JsxL2Root<'a>, L2Refusal>, Self::Facts);

    fn project(
        allocator: &'a Allocator,
        source: &'a str,
        root: &RootNode<'a>,
    ) -> (Result<JsxL2Root<'a>, L2Refusal>, Self::Facts);

    fn seal(root: LoweredRoot<'a>, source: &'a str, facts: Self::Facts) -> Self::Root;
}

pub(crate) struct PlainRoots;

impl<'a> RootProjection<'a> for PlainRoots {
    type Facts = ();
    type Root = LoweredRoot<'a>;

    fn lower(
        lowerer: &Lowerer<'a, '_, '_>,
        component: JsxComponent<'a>,
        oxc_root: JsxRoot<'_, '_>,
        root: &RootNode<'a>,
    ) -> (Result<JsxL2Root<'a>, L2Refusal>, ()) {
        (l2::lower_root(lowerer, component, oxc_root, root), ())
    }

    fn project(
        allocator: &'a Allocator,
        source: &'a str,
        root: &RootNode<'a>,
    ) -> (Result<JsxL2Root<'a>, L2Refusal>, ()) {
        (l2::try_lower_root(allocator, source, root), ())
    }

    fn seal(root: LoweredRoot<'a>, _source: &'a str, _facts: ()) -> LoweredRoot<'a> {
        root
    }
}

struct SyntaxRoots;

impl<'a> RootProjection<'a> for SyntaxRoots {
    type Facts = RootSyntax<'a>;
    type Root = RetainedSyntaxRoot<'a>;

    fn lower(
        lowerer: &Lowerer<'a, '_, '_>,
        component: JsxComponent<'a>,
        oxc_root: JsxRoot<'_, '_>,
        root: &RootNode<'a>,
    ) -> (Result<JsxL2Root<'a>, L2Refusal>, RootSyntax<'a>) {
        use vize_l1_to_l2::framework::vue::jsx::{
            JsxDialect, JsxL2Ir, lower_component_with_syntax,
        };
        let allocator = lowerer.bump();
        let source = lowerer.mapper().source();
        if lowerer.admits_level_l2()
            && let Ok((component, facts)) = lower_component_with_syntax(
                allocator,
                &JsxL2Ir {
                    source,
                    scoping: lowerer.scoping(),
                    booleans: lowerer.boolean_bindings(),
                },
                JsxDialect::for_semantics(lowerer.semantics()),
                component,
                oxc_root,
            )
        {
            return (
                Ok(JsxL2Root::from_component(component)),
                RootSyntax::Native(facts),
            );
        }
        Self::project(allocator, source, root)
    }

    fn project(
        allocator: &'a Allocator,
        source: &'a str,
        root: &RootNode<'a>,
    ) -> (Result<JsxL2Root<'a>, L2Refusal>, RootSyntax<'a>) {
        match l2::try_lower_root_with_syntax(allocator, source, root) {
            Ok((root, facts)) => (Ok(root), RootSyntax::Projection(facts)),
            Err(error) => (Err(error), RootSyntax::Unavailable),
        }
    }

    fn seal(
        root: LoweredRoot<'a>,
        source: &'a str,
        mut facts: RootSyntax<'a>,
    ) -> RetainedSyntaxRoot<'a> {
        if root
            .l2
            .as_ref()
            .is_ok_and(|root| !core::ptr::eq(root.source, source))
        {
            facts = RootSyntax::Unavailable;
        }
        RetainedSyntaxRoot {
            lowered: root,
            source,
            syntax: facts,
        }
    }
}

#[cfg(test)]
mod tests;
