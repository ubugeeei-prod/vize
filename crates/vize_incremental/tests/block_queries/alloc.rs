//! One process owns the complete allocation window for each request.
mod workload;

use davinci_harness::alloc::{self, CountingAllocator};
use vize_incremental::ResidentDatabase;
use workload::{Work, offsets, source};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

#[expect(
    clippy::expect_used,
    reason = "the process installs the allocation counter before sampling"
)]
fn main() {
    alloc::mark_installed();
    for rows in [1, 16, 64] {
        let text = source(rows);
        for work in [Work::Pages, Work::Positions] {
            for warm in [false, true] {
                let window = |candidate| {
                    let db = ResidentDatabase::default();
                    let file = db.open("Card.vue", &text);
                    let bytes = offsets(&db, file);
                    let read = || {
                        if candidate {
                            work.candidate(&db, file, &bytes)
                        } else {
                            work.original(&db, file, &bytes)
                        }
                    };
                    if warm {
                        std::hint::black_box(read());
                    }
                    let (answer, metrics) = alloc::measure_returning(read);
                    (answer, metrics.expect("installed counting allocator"))
                };
                let (old_answer, old) = window(false);
                let (answer, new) = window(true);
                assert_eq!(old_answer, answer);
                if warm {
                    assert_eq!(new.calls, 0, "{work:?} rows={rows}");
                    assert_eq!(new.peak_bytes_over_start, 0, "{work:?} rows={rows}");
                }
                println!(
                    "work={work:?} rows={rows} warm={warm} calls={}/{} peak={}/{}",
                    old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
                );
            }
        }
    }
}
