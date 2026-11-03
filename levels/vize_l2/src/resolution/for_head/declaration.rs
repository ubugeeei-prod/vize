//! Original declaration roots retained by the owning head walk, never expressions.

use oxc_ast::ast::{BindingPattern, FormalParameter};
use vize_l0::Span;

use super::{ForAlias, ForResolution};

#[derive(Debug)]
pub(in crate::resolution) struct OriginalForAlias<'a> {
    pub(in crate::resolution) parameter: &'a FormalParameter<'a>,
    pub(in crate::resolution) fact: ForAlias<'a>,
}

/// A short declaration capability tied to its whole original For resolution.
/// The owning walk retains the actual Params root beside its checked alias fact.
/// Reading the AST does not mint File identities, scope or canonical body authority.
///
/// Caller-written parameter/fact pairs cannot mint this capability:
/// ```compile_fail
/// use oxc_ast::ast::FormalParameter;
/// use vize_l2::resolution::{ForAlias, ForAliasDeclaration, ForResolution};
/// fn substitute<'h, 'a>(owner: &'h ForResolution<'a>,
///     parameter: &'a FormalParameter<'a>, fact: ForAlias<'a>) {
///     let _ = ForAliasDeclaration { owner, parameter, fact };
/// }
/// ```
/// The short capability cannot outlive the normally retained original owner:
/// ```compile_fail
/// use vize_l2::resolution::{ForAliasDeclaration, ForResolution};
/// fn escape<'h, 'a>(owner: ForResolution<'a>) -> ForAliasDeclaration<'h, 'a> {
///     owner.value_declaration()
/// }
/// ```
#[derive(Debug)]
pub struct ForAliasDeclaration<'h, 'a> {
    owner: &'h ForResolution<'a>,
    parameter: &'a FormalParameter<'a>,
    fact: ForAlias<'a>,
}

impl<'h, 'a> ForAliasDeclaration<'h, 'a> {
    #[must_use]
    pub const fn resolution(&self) -> &'h ForResolution<'a> {
        self.owner
    }
    #[must_use]
    pub const fn parameter(&self) -> &'a FormalParameter<'a> {
        self.parameter
    }
    #[must_use]
    pub const fn pattern(&self) -> &'a BindingPattern<'a> {
        &self.parameter.pattern
    }
    #[must_use]
    pub const fn fact(&self) -> ForAlias<'a> {
        self.fact
    }
    #[must_use]
    pub const fn decoded_span(&self) -> Span {
        self.fact.decoded_span()
    }
    #[must_use]
    pub const fn authored_span(&self) -> Span {
        self.fact.authored_span()
    }
}

impl<'a> ForResolution<'a> {
    /// Borrow the genuine original value declaration retained during resolution.
    #[must_use]
    pub const fn value_declaration(&self) -> ForAliasDeclaration<'_, 'a> {
        ForAliasDeclaration {
            owner: self,
            parameter: self.value_parameter,
            fact: self.value,
        }
    }
    /// A missing key remains absent; no synthesized parameter or range is made.
    #[must_use]
    pub fn key_declaration(&self) -> Option<ForAliasDeclaration<'_, 'a>> {
        self.key
            .zip(self.key_parameter)
            .map(|(fact, parameter)| ForAliasDeclaration {
                owner: self,
                parameter,
                fact,
            })
    }
}
