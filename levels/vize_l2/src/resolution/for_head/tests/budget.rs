use super::*;
use crate::resolution::sink::{ReferenceEvent, ReferenceSink};

#[derive(Default)]
struct Failing {
    pending: usize,
    rolls: usize,
}
impl<'a> ReferenceSink<'a> for Failing {
    type Checkpoint = usize;
    fn checkpoint(&self) -> usize {
        self.pending
    }
    fn reference(&mut self, _: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
        self.pending += 1;
        Err(ResolutionErrorKind::MissingBinding)
    }
    fn rollback(&mut self, checkpoint: usize) {
        self.pending = checkpoint;
        self.rolls += 1;
    }
}

#[test]
fn actual_collection_sink_failure_rolls_back_all_new_facts() {
    let arena = Allocator::default();
    let owner = selected(&arena, "<template><b v-for='item in items'/></template>");
    let original = input(&owner);
    let source = original.references().unwrap();
    let mut sink = Failing {
        pending: 7,
        rolls: 0,
    };
    let rejected = walk::original_for_head(&source, &mut sink).unwrap_err();
    assert_eq!(rejected.part, ForHeadPart::Collection);
    assert_eq!(
        rejected.kind,
        ForResolutionErrorKind::Reference(ResolutionErrorKind::MissingBinding)
    );
    assert_eq!(sink.pending, 7);
    assert_eq!(sink.rolls, 1);
}

#[derive(Default)]
struct Pending {
    count: usize,
    rolls: usize,
}
impl<'a> ReferenceSink<'a> for Pending {
    type Checkpoint = usize;
    fn checkpoint(&self) -> usize {
        self.count
    }
    fn reference(&mut self, _: ReferenceEvent<'a>) -> Result<(), ResolutionErrorKind> {
        self.count += 1;
        Ok(())
    }
    fn rollback(&mut self, checkpoint: usize) {
        self.count = checkpoint;
        self.rolls += 1;
    }
}

#[test]
fn later_alias_failure_rolls_back_the_already_resolved_original_collection() {
    let arena = Allocator::default();
    let owner = selected(
        &arena,
        "<template><b v-for='(item, item) in items'/></template>",
    );
    let original = input(&owner);
    let mut sink = Pending { count: 5, rolls: 0 };
    let rejected = walk::original_for_head(&original.references().unwrap(), &mut sink).unwrap_err();
    assert_eq!(rejected.kind, ForResolutionErrorKind::DuplicateAlias);
    assert_eq!(sink.count, 5);
    assert_eq!(sink.rolls, 1);
}
