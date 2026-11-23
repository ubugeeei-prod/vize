//! Source-resolved SCSS variables, nested selectors and ordered declarations.
use alloc::vec::Vec;
use vize_l0::{SourceBlock, Span, String};
use vize_l1::css::lang::OriginalStyleStructure;

mod lower;
mod syntax;

/// A complete native result for this stylesheet. Refusal discards the partial
/// model, including declarations from earlier, otherwise valid rules.
#[derive(Debug)]
pub struct ScssProgram<'a> {
    block: SourceBlock<'a>,
    runs: Vec<RuleRun<'a>>,
    variables: Vec<Variable<'a>>,
    uses: Vec<VariableUse>,
}

#[derive(Debug)]
pub struct RuleRun<'a> {
    pub selector: String,
    pub selector_span: Span,
    pub group: usize,
    pub declarations: Vec<Property<'a>>,
}

#[derive(Debug)]
pub struct Property<'a> {
    pub name: &'a str,
    pub value: String,
    pub name_span: Span,
    pub value_span: Span,
}

#[derive(Debug)]
pub struct Variable<'a> {
    pub name: &'a str,
    pub span: Span,
    pub value: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct VariableUse {
    pub span: Span,
    pub declaration: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScssRefusal {
    Language,
    IncompleteStructure,
    Comment(Span),
    Selector(Span),
    Statement(Span),
    Value(Span),
    MissingVariable(Span),
    Expansion(Span),
}

impl<'a> ScssProgram<'a> {
    /// Lower the actual structure produced by the selected grammar. It can be
    /// borrowed concurrently by a lint rule and the authored-class analysis.
    pub fn lower(style: OriginalStyleStructure<'_, 'a>) -> Result<Self, ScssRefusal> {
        lower::program(style)
    }

    pub fn runs(&self) -> &[RuleRun<'a>] {
        &self.runs
    }

    pub fn block(&self) -> SourceBlock<'a> {
        self.block
    }

    pub fn variables(&self) -> &[Variable<'a>] {
        &self.variables
    }

    pub fn variable_uses(&self) -> &[VariableUse] {
        &self.uses
    }

    /// A value use refers to the most recent original definition at that point.
    pub fn definition_at(&self, offset: u32) -> Option<&Variable<'a>> {
        self.variables.get(self.use_at(offset)?.declaration)
    }

    fn use_at(&self, offset: u32) -> Option<&VariableUse> {
        let at = self.uses.partition_point(|use_| use_.span.end <= offset);
        self.uses.get(at).filter(|use_| use_.span.start <= offset)
    }

    /// A single variable inherits the original literal's range through aliases.
    /// A CSS list retains the range of the expression at the use site.
    pub fn value_origin(&self, expression: Span) -> Option<Span> {
        if !self.block.contains_block_span(expression) {
            return None;
        }
        if self
            .block
            .root_source()
            .as_bytes()
            .get(expression.start as usize)
            == Some(&b'$')
            && let Some(use_) = self
                .use_at(expression.start)
                .filter(|u| u.span == expression)
        {
            return self
                .block
                .span_of(self.variables.get(use_.declaration)?.value);
        }
        Some(expression)
    }
}
