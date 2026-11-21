//! A descriptor consumer and a selected stage consumer of the same open SFC.
#![expect(
    clippy::unwrap_used,
    reason = "the generated SFCs have original descriptors and positions"
)]
use davinci_harness::alloc::{self, CountingAllocator};
use std::{alloc::System, hint::black_box, time::Instant};
use vize_incremental::{
    ResidentDatabase, ResidentDocuments, SharedDescriptor, block_query::DocumentBlocks,
};
use vize_l0::{String, key::ArtifactKey};

#[global_allocator]
static GLOBAL: CountingAllocator<System> = CountingAllocator::system();

#[derive(Debug, Clone, Copy)]
enum Work {
    Source,
    Surface,
    Page,
}

type Answer = (usize, u32, ArtifactKey, Option<ArtifactKey>);

fn inspect(
    descriptor: &SharedDescriptor,
    blocks: DocumentBlocks<'_>,
    byte: u32,
    work: Work,
) -> Answer {
    let block = blocks.at(black_box(byte)).unwrap().unwrap();
    let stage = match work {
        Work::Source => None,
        Work::Surface => block.surface().map(|s| s.key),
        Work::Page => block.page().map(|p| p.key),
    };
    (
        descriptor.styles.len(),
        block.start(),
        block.source().key,
        stage,
    )
}

fn sample(mut request: impl FnMut() -> Answer, warm: bool) -> (Answer, u64, u64, u128) {
    if warm {
        black_box(request());
    }
    let repeats = if warm { 64 } else { 1 };
    let ((answer, time), mem) = alloc::measure_returning(|| {
        let start = Instant::now();
        let mut answer = black_box(request());
        for _ in 1..repeats {
            answer = black_box(request());
        }
        (answer, start.elapsed().as_nanos() / repeats)
    });
    let mem = mem.unwrap();
    (answer, mem.calls, mem.peak_bytes_over_start, time)
}

fn original(text: &str, at: u32, work: Work, warm: bool) -> (Answer, u64, u64, u128) {
    let mut docs = ResidentDocuments::default();
    let db = ResidentDatabase::default();
    // Even the cold baseline receives a pre-opened stage input.
    let file = db.open("Named.vue", text);
    sample(
        || {
            let descriptor = docs
                .parsed("editor", "Named.vue", black_box(text))
                .into_descriptor()
                .unwrap();
            inspect(&descriptor, db.document_blocks(file).unwrap(), at, work)
        },
        warm,
    )
}

fn shared(text: &str, at: u32, work: Work, warm: bool) -> (Answer, u64, u64, u128) {
    let mut docs = ResidentDocuments::default();
    sample(
        || {
            docs.with_document_blocks("editor", "Named.vue", black_box(text), |named| {
                let named = named.unwrap();
                inspect(named.descriptor, named.blocks, at, work)
            })
        },
        warm,
    )
}

fn main() {
    alloc::mark_installed();
    for styles in [1, 8, 64] {
        let mut text = String::from(
            "<template><h1>雪{{ value }}</h1></template><script setup>const value=1</script>",
        );
        for _ in 0..styles {
            text.push_str("<style scoped>.button{color:v-bind(value)}</style>");
        }
        for work in [Work::Source, Work::Surface, Work::Page] {
            let at = match work {
                Work::Surface => text.find("<h1>"),
                _ => text.rfind(".button"),
            }
            .unwrap() as u32;
            for warm in [false, true] {
                let mut pairs = Vec::new();
                let mut calls = (0, 0);
                let mut peaks = (0, 0);
                for i in 0..35 {
                    let a = || original(&text, at, work, warm);
                    let b = || shared(&text, at, work, warm);
                    let (before, after) = if i % 2 == 0 {
                        (a(), b())
                    } else {
                        let y = b();
                        (a(), y)
                    };
                    assert_eq!(before.0, after.0);
                    assert!(
                        after.1 <= before.1,
                        "allocation calls {before:?} → {after:?}"
                    );
                    assert!(after.2 <= before.2, "peak bytes {before:?} → {after:?}");
                    if i >= 4 {
                        pairs.push((before.3, after.3));
                        calls = (before.1, after.1);
                        peaks = (before.2, after.2);
                    }
                }
                eprintln!(
                    "open_block styles={styles} stage={work:?} warm={warm} calls={calls:?} peak={peaks:?} ns={pairs:?}"
                );
            }
        }
    }
}
