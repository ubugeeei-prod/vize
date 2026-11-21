use super::*;
use davinci_harness::alloc::{self, CountingAllocator};
use std::{alloc::System, hint::black_box, time::Instant};
use vize_l1::edit::selection::SelectedEdits;
use vize_maestro::source_project::{SnapshotKey, SourceSnapshot};

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

pub(super) fn paired(
    label: &str,
    store: &DocumentStore,
    snapshot: &SourceSnapshot,
    selected: &SelectedEdits<'_, '_, SnapshotKey<'_>>,
) {
    alloc::mark_installed();
    let set = selected.as_edit_set();
    let run = |shared| {
        if shared {
            black_box(snapshot)
                .workspace_edit_for_fixes(black_box(store), black_box(set))
                .unwrap()
        } else {
            black_box(snapshot)
                .workspace_edit(black_box(store), black_box(set))
                .unwrap()
        }
    };
    assert_eq!(run(false), run(true));
    let old = alloc::measure(|| run(false)).unwrap();
    let new = alloc::measure(|| run(true)).unwrap();
    assert!(new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start);
    let time = |shared| {
        let start = Instant::now();
        for _ in 0..32 {
            drop(black_box(run(shared)));
        }
        start.elapsed().as_nanos() / 32
    };
    let mut pairs = Vec::new();
    for i in 0..35 {
        let reverse = i % 2 != 0;
        let x = time(reverse);
        let y = time(!reverse);
        if i >= 4 {
            pairs.push(if reverse { (y, x) } else { (x, y) });
        }
    }
    eprintln!(
        "fix_time {label} edits={} calls={}/{} peak={}/{} ns={pairs:?} first={:?} last={:?} bytes={}",
        set.edits().len(),
        old.calls,
        new.calls,
        old.peak_bytes_over_start,
        new.peak_bytes_over_start,
        set.edits().first().map(|edit| edit.span()),
        set.edits().last().map(|edit| edit.span()),
        snapshot.source().len(),
    );
}

#[test]
#[ignore = "paired complete LSP publications on one actual open document revision"]
fn publication_cost() {
    for (kind, line) in [
        ("ascii", "let row = 'tree';\r\n"),
        ("unicode", "const 雪 = '🪴';\r\n"),
    ] {
        for rows in [1, 32, 512] {
            let text = line.repeat(rows);
            let store = DocumentStore::new();
            let uri = open(&store, "file:///timing.vue", &text);
            let snapshot = SourceSnapshotCache::default()
                .capture(&store, &uri)
                .unwrap();
            let frame = snapshot.versioned_source().unwrap();
            for count in [1, 2, 16, 128] {
                if count > rows {
                    continue;
                }
                for at_end in [false, true] {
                    for width in [0, 1] {
                        let candidates = (0..count).map(|i| {
                            let row = if count == 1 {
                                if at_end { rows - 1 } else { 0 }
                            } else {
                                i * (rows - 1) / (count - 1)
                            };
                            let at = (row * line.len()
                                + if at_end {
                                    line.len() - 2 - width as usize
                                } else {
                                    0
                                }) as u32;
                            (Span::new(at, at + width), "🛠")
                        });
                        let selected = frame.select_edits(candidates).unwrap().unwrap();
                        let label = vize_l0::cstr!(
                            "{kind} rows={rows} count={count} end={at_end} width={width}"
                        );
                        paired(&label, &store, &snapshot, &selected);
                    }
                }
            }
        }
    }
}
