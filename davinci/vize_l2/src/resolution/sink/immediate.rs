// Shared semantic policy for ordinary and arena occurrence storage.
macro_rules! immediate_reference {
    () => {
        fn reference(&mut self, event: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
            let binding = self
                .bindings
                .lookup(event.name())
                .ok_or(ResolutionErrorKind::MissingBinding)?;
            self.occurrences.push(Occurrence {
                span: event.span(),
                name: event.name(),
                binding,
                usage: event.usage(),
                shorthand: event.shorthand(),
                constructor: event.constructor(),
            });
            Ok(())
        }
    };
}
