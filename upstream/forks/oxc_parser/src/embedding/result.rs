// Both original parser configurations retain their entire parse result here.
{
    let hole = if parsed.is_flow_language {
        Some(EmbeddingHole::UnsupportedFlow)
    } else if parsed.panicked || parsed.diagnostics.has_errors() {
        Some(EmbeddingHole::Syntax)
    } else {
        shapes::shape_hole(&parsed.program, self.goal, self.content_span).or_else(|| {
            (self.goal != EmbeddingGoal::Expr)
                .then(|| shapes::context_hole(&parsed.program))
                .flatten()
        })
    };
    EmbeddingObservation {
        allocator: self.allocator,
        content: self.content,
        source_type: self.source_type,
        goal: self.goal,
        content_span: self.content_span,
        parsed,
        options,
        hole,
    }
}
