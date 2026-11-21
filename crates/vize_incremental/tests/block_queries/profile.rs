//! Ordered independent A/B and B/A measurements; print raw nanoseconds.
mod workload;

use std::{hint::black_box, time::Instant};
use vize_incremental::ResidentDatabase;
use workload::{Work, offsets, source};

fn sample(work: Work, text: &str, warm: bool, candidate: bool) -> (usize, u128) {
    let db = ResidentDatabase::default();
    let file = db.open("Card.vue", text);
    let bytes = offsets(&db, file);
    let read = || {
        if candidate {
            work.candidate(&db, file, black_box(&bytes))
        } else {
            work.original(&db, file, black_box(&bytes))
        }
    };
    if warm {
        black_box(read());
    }
    let repeats = if warm { 48 } else { 1 };
    let mut value = 0;
    let start = Instant::now();
    for _ in 0..repeats {
        value = black_box(read());
    }
    (value, start.elapsed().as_nanos() / repeats)
}

#[test]
#[ignore = "records paired original and borrowed resident requests"]
fn paired_resident_requests() {
    for rows in [1, 16, 64] {
        let text = source(rows);
        for work in [Work::Pages, Work::Positions] {
            for warm in [false, true] {
                let mut pairs = Vec::new();
                for i in 0..33 {
                    let candidate_first = i % 2 == 1;
                    let a = sample(work, &text, warm, candidate_first);
                    let b = sample(work, &text, warm, !candidate_first);
                    assert_eq!(a.0, b.0);
                    if i >= 2 {
                        pairs.push(if candidate_first {
                            (b.1, a.1)
                        } else {
                            (a.1, b.1)
                        });
                    }
                }
                eprintln!("work={work:?} rows={rows} warm={warm} pairs={pairs:?}");
            }
        }
    }
}
