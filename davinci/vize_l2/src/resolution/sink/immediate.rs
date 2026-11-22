// Shared by the same binding policy for ordinary and arena occurrence storage.
{
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
