//! Read-only authored views; wrapper/context policy lives at the parser owner.

use oxc_ast::ast::{
    Directive, FormalParameter, FormalParameterRest, FormalParameters, FunctionBody, Statement,
};

#[derive(Debug, Clone, Copy)]
pub struct HandlerBodyView<'s, 'a> {
    directives: &'s [Directive<'a>],
    statements: &'s [Statement<'a>],
}

impl<'s, 'a> HandlerBodyView<'s, 'a> {
    pub(super) fn from_body(body: &'s FunctionBody<'a>) -> Self {
        Self {
            directives: body.directives.as_slice(),
            statements: body.statements.as_slice(),
        }
    }
    #[must_use]
    pub const fn directives(self) -> &'s [Directive<'a>] {
        self.directives
    }
    #[must_use]
    pub const fn statements(self) -> &'s [Statement<'a>] {
        self.statements
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SlotParamsView<'s, 'a> {
    parameters: &'s [FormalParameter<'a>],
    rest: Option<&'s FormalParameterRest<'a>>,
}

impl<'s, 'a> SlotParamsView<'s, 'a> {
    pub(super) fn from_parameters(parameters: &'s FormalParameters<'a>) -> Self {
        Self {
            parameters: parameters.items.as_slice(),
            rest: parameters.rest.as_deref(),
        }
    }
    #[must_use]
    pub const fn parameters(self) -> &'s [FormalParameter<'a>] {
        self.parameters
    }
    #[must_use]
    pub const fn rest(self) -> Option<&'s FormalParameterRest<'a>> {
        self.rest
    }
}
