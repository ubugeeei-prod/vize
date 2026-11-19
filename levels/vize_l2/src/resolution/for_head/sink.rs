use super::{
    BindingLookup, ForResolutionError, ForResolutionErrorKind, Occurrence, ResolutionErrorKind,
};
use crate::resolution::sink::{ReferenceEvent, ReferenceSink};
use vize_l0::Span;
use vize_l1::embed::syntax::ForHeadPart;

pub(super) struct Collection<'a, 'b, B> {
    enclosing: &'b B,
    occurrence: Option<Occurrence<'a>>,
}
impl<'a, 'b, B> Collection<'a, 'b, B> {
    pub(super) fn new(enclosing: &'b B) -> Self {
        Self {
            enclosing,
            occurrence: None,
        }
    }
    pub(super) fn finish(self) -> Result<Occurrence<'a>, ForResolutionError> {
        self.occurrence.ok_or(ForResolutionError {
            part: ForHeadPart::Collection,
            span: Span::new(0, 0),
            kind: ForResolutionErrorKind::Reference(ResolutionErrorKind::UnsupportedSyntax),
        })
    }
}
impl<'a, B: BindingLookup> ReferenceSink<'a> for Collection<'a, '_, B> {
    type Checkpoint = Option<Occurrence<'a>>;
    fn checkpoint(&self) -> Self::Checkpoint {
        self.occurrence
    }
    fn reference(&mut self, event: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
        if self.occurrence.is_some() {
            return Err(ResolutionErrorKind::UnsupportedSyntax);
        }
        let binding = self
            .enclosing
            .lookup(event.name())
            .ok_or(ResolutionErrorKind::MissingBinding)?;
        self.occurrence = Some(Occurrence {
            span: event.span(),
            name: event.name(),
            binding,
            usage: event.usage(),
            shorthand: event.shorthand(),
            constructor: event.constructor(),
        });
        Ok(())
    }
    fn rollback(&mut self, checkpoint: Self::Checkpoint) {
        self.occurrence = checkpoint;
    }
}
