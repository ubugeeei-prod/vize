//! Paired position-query timing over one completed File. Pass a JS module path
//! to measure it, or use the fixed small, medium and large scope fixtures.
//! `cargo run --profile ci-opt -p vize_l2 --features position-index --example position_index_profile`

use oxc_parser::Parser;
use oxc_span::SourceType;
use std::{hint::black_box, time::Instant};
use vize_l0::{Allocator, SourceRoot, String, append};
use vize_l2::{
    file::L2Ir,
    lang::js::{L2IrProducer, ProgramInput, ProgramScope},
    query::Model,
};

const QUERIES: usize = 2048;
const SAMPLES: usize = 21;

fn file<'a>(arena: &'a Allocator, source: &'a str) -> Result<L2Ir<'a>, &'static str> {
    let block = SourceRoot::new(source)
        .map_err(|_| "source size")?
        .whole_block();
    let parsed = Parser::new(arena, source, SourceType::mjs()).parse_observed();
    let input = ProgramInput::checked(parsed.admitted().ok_or("parse refused")?, block, 0)
        .map_err(|_| "program input refused")?;
    let mut producer = L2IrProducer::new(arena, source).map_err(|_| "file refused")?;
    producer
        .program(input, ProgramScope::Module)
        .map_err(|_| "program refused")?;
    let file = producer.finish().map_err(|_| "file refused")?;
    file.is_complete()
        .then_some(file)
        .ok_or("incomplete semantic file")
}

fn elapsed(f: impl FnOnce()) -> u128 {
    let start = Instant::now();
    f();
    start.elapsed().as_nanos()
}

fn measure(source: &str) -> Result<(), &'static str> {
    let arena = Allocator::default();
    let file = file(&arena, source)?;
    let scan = Model::new(&file);
    let mut construction = [0; SAMPLES];
    for time in &mut construction {
        let start = Instant::now();
        let index = black_box(scan).index_positions();
        *time = start.elapsed().as_nanos();
        drop(black_box(index));
    }
    let index = scan.index_positions();
    let stored = index.model();
    let mut state = 7u64;
    let offsets = [(); QUERIES].map(|()| {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((state >> 32) % (source.len() as u64 + 1)) as u32
    });
    // Both arms exercise the same three questions at the same bytes.
    macro_rules! query {
        ($model:expr) => {{
            for at in &offsets {
                let _ = black_box($model.binding_at_offset(black_box(*at)));
                let _ = black_box($model.reference_at_offset(black_box(*at)));
                let _ = black_box($model.scope_at_offset(black_box(*at)));
            }
        }};
    }
    query!(scan);
    query!(stored);
    let mut scans = [0; SAMPLES];
    let mut indexes = [0; SAMPLES];
    for (round, (scan_time, index_time)) in scans.iter_mut().zip(&mut indexes).enumerate() {
        if round % 2 == 0 {
            *scan_time = elapsed(|| query!(scan));
            *index_time = elapsed(|| query!(stored));
        } else {
            *index_time = elapsed(|| query!(stored));
            *scan_time = elapsed(|| query!(scan));
        }
    }
    let times = |mut values: [u128; SAMPLES]| {
        values.sort_unstable();
        let requests = (3 * QUERIES) as u128;
        (values[SAMPLES / 2] / requests, values[19] / requests)
    };
    construction.sort_unstable();
    println!(
        "bytes={} scopes={} references={} build_median_ns={} scan_ns_p50_p95={:?} index_ns_p50_p95={:?}",
        source.len(),
        file.scopes().len(),
        file.references().len(),
        construction[SAMPLES / 2],
        times(scans),
        times(indexes)
    );
    Ok(())
}

fn main() -> Result<(), &'static str> {
    if let Some(path) = std::env::args_os().nth(1) {
        return measure(&std::fs::read_to_string(path).map_err(|_| "read module")?);
    }
    for functions in [1, 32, 512] {
        let mut source = String::from("const seed = 10;\n");
        for row in 0..functions {
            append!(
                &mut source,
                "function f{row}(n) {{ const x = seed + n; return x; }}\n"
            );
        }
        measure(&source)?;
    }
    Ok(())
}
