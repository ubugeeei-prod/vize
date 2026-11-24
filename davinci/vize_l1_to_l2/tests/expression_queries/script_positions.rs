//! Check all three answers at the same authored cursor against the scanning model.
use super::observe;
use std::{fs, path::Path, ptr};
use vize_l0::Allocator;
use vize_l2::query::{BindingRef, Model, ReferenceRef, ScopeRef};

#[derive(Default, Debug)]
struct Counts {
    files: usize,
    invalid: usize,
    no_l2: usize,
    complete: usize,
    incomplete: usize,
    offsets: usize,
    references: usize,
    bindings: usize,
    scopes: usize,
    refusals: usize,
}

#[test]
#[ignore = "reads a NUL-separated manifest of original Vue files"]
fn original_vue_files() {
    let names =
        fs::read_to_string(std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut counts = Counts::default();
    for name in names.split_terminator('\0') {
        counts.files += 1;
        let bytes = fs::read(root.join(name)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            counts.invalid += 1;
            continue;
        };
        let arena = Allocator::default();
        let observed = observe(&arena, source);
        let Some(file) = observed
            .file()
            .map(|vue| vue.file())
            .or_else(|| observed.rejected_file()?.file())
        else {
            counts.no_l2 += 1;
            continue;
        };
        if file.is_complete() {
            counts.complete += 1;
        } else {
            counts.incomplete += 1;
        }
        let original = Model::new(file);
        let index = original.index_positions();
        let indexed = index.model();
        let end = u32::try_from(source.len()).unwrap();
        let mut offsets = vec![0, end, u32::MAX];
        offsets.extend((0..end).step_by((source.len() / 32).max(1)));
        let spans = file
            .references()
            .iter()
            .map(|r| r.span)
            .chain(
                file.bindings()
                    .filter_map(|b| b.declaration().map(|r| r.span)),
            )
            .chain(file.scopes().iter().map(|s| s.span));
        for span in spans {
            offsets.extend([span.start, span.end]);
        }
        offsets.sort_unstable();
        offsets.dedup();
        for offset in offsets {
            let answer = indexed.script_position_at_offset(offset);
            let reference = |r: ReferenceRef<'_, '_>| {
                (
                    ptr::from_ref(r.file()).cast::<()>(),
                    ptr::from_ref(r.reference()),
                )
            };
            let binding = |r: BindingRef<'_, '_>| (ptr::from_ref(r.file()).cast::<()>(), r.id());
            let scope = |r: ScopeRef<'_, '_>| {
                (
                    ptr::from_ref(r.file()).cast::<()>(),
                    ptr::from_ref(r.scope()),
                )
            };
            assert_eq!(
                answer.reference.map(|r| r.map(reference)),
                original
                    .reference_at_offset(offset)
                    .map(|r| r.map(reference)),
                "{name} reference at {offset}"
            );
            assert_eq!(
                answer.binding.map(|r| r.map(binding)),
                original.binding_at_offset(offset).map(|r| r.map(binding)),
                "{name} binding at {offset}"
            );
            assert_eq!(
                answer.scope.map(|r| r.map(scope)),
                original.scope_at_offset(offset).map(|r| r.map(scope)),
                "{name} scope at {offset}"
            );
            counts.offsets += 1;
            counts.references += usize::from(matches!(answer.reference, Ok(Some(_))));
            counts.bindings += usize::from(matches!(answer.binding, Ok(Some(_))));
            counts.scopes += usize::from(matches!(answer.scope, Ok(Some(_))));
            counts.refusals += usize::from(answer.scope.is_err());
        }
    }
    assert_eq!(
        counts.files,
        counts.invalid + counts.no_l2 + counts.complete + counts.incomplete
    );
    assert!(
        counts.references > 0 && counts.bindings > 0 && counts.scopes > 0 && counts.refusals > 0,
        "{counts:?}"
    );
    eprintln!("composed script positions {counts:?}");
}
