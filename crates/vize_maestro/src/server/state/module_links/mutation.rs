//! Hold the real module write gate until applied fields and epoch are coherent.
use super::{ModuleLinkRetirement, Phase, Session};
use std::path::PathBuf;

pub(super) struct Mutation<'a> {
    session: &'a mut Session,
    next: Option<u64>,
}
impl<'a> Mutation<'a> {
    pub(super) fn begin(session: &'a mut Session) -> Self {
        let next = if matches!(session.phase, Phase::Live) {
            match session.generation.checked_add(1) {
                Some(next) => {
                    session.phase = Phase::Updating;
                    Some(next)
                }
                None => {
                    session.retire(ModuleLinkRetirement::GenerationExhausted);
                    None
                }
            }
        } else {
            None
        };
        Self { session, next }
    }
    pub(super) fn commit(mut self, origin: Option<PathBuf>) {
        if let Some(origin) = origin {
            self.session.load_origin = Some(origin);
        }
        if let Some(next) = self.next.take() {
            self.session.generation = next;
            self.session.phase = Phase::Live;
        }
    }
}
impl Drop for Mutation<'_> {
    fn drop(&mut self) {
        if self.next.is_some() {
            self.session.retire(ModuleLinkRetirement::MutationUnwound);
        }
    }
}
