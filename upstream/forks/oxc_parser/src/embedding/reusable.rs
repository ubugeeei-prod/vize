//! Reuse the exact synthesized wrapper text while invoking the ordinary parser
//! for each authored occurrence. Each observation retains its own content and AST.
use super::{EmbeddingGoal, EmbeddingInput, EmbeddingObservation, SourceType};

pub struct ReusableExpressionInput<'a> {
    input: EmbeddingInput<'a>,
    source: &'a str,
}

impl<'a> EmbeddingInput<'a> {
    /// Promote an expression's prepared wrapper to its own arena once.
    /// Other embedding goals keep their original input.
    pub fn reusable_expression(self) -> Result<ReusableExpressionInput<'a>, Self> {
        if self.goal != EmbeddingGoal::Expr {
            return Err(self);
        }
        let source = self.allocator.alloc_str(self.wrapper.as_str());
        Ok(ReusableExpressionInput { input: self, source })
    }
}

impl<'a> ReusableExpressionInput<'a> {
    #[must_use]
    pub fn observe(&self) -> EmbeddingObservation<'a> {
        self.input.observe_source(self.input.content, self.source)
    }

    /// A different occurrence can share these immutable parser bytes only for
    /// identical complete content and source type in this expression grammar.
    #[must_use]
    pub fn observe_matching(
        &self,
        content: &'a str,
        source_type: SourceType,
    ) -> Option<EmbeddingObservation<'a>> {
        (self.input.source_type == source_type && self.input.content == content)
            .then(|| self.input.observe_source(content, self.source))
    }
}
