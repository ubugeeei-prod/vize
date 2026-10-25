//! Source-phase probe; build first, then run without competing compilation.
//! `cargo run --profile ci-opt -p vize_l2 --example jsx_markup_profile -- 64 1000`
//! VIZE_JSX_MARKUP_PROFILE=1 separately exports instrumented source buckets.

use oxc_parser::Parser;
use oxc_span::SourceType;
use std::{hint::black_box, time::Instant};
use vize_l0::{Allocator, SourceRoot, String, append, profiler::global_profiler};
use vize_l2::{analysis::template_complexity, lang::js::JsxFileProducer};

fn parse_us(value: u128) -> f64 {
    value as f64 / 1000.0
}

fn main() -> Result<(), &'static str> {
    let mut args = std::env::args().skip(1);
    let children: usize = args
        .next()
        .ok_or("children")?
        .parse()
        .map_err(|_| "children")?;
    let rounds: usize = args.next().ok_or("rounds")?.parse().map_err(|_| "rounds")?;
    if children > 256 || rounds == 0 {
        return Err("probe bounds: children <=256, rounds >0");
    }
    let profile = std::env::var("VIZE_JSX_MARKUP_PROFILE").as_deref() == Ok("1");
    if profile {
        global_profiler().enable();
    }
    let mut source = String::from("export const view = <main>");
    for index in 0..children {
        append!(&mut source, "<i data-index=\"{index}\"/>");
    }
    source.push_str("</main>;");
    let block = SourceRoot::new(&source)
        .map_err(|_| "source")?
        .whole_block();
    let mut phases = [0u128; 4];
    for _ in 0..rounds {
        let arena = Allocator::default();
        let at = Instant::now();
        let observation = Parser::new(&arena, &source, SourceType::jsx()).parse_observed();
        let parsed = at.elapsed().as_nanos();
        let at = Instant::now();
        let mut producer =
            JsxFileProducer::new(&arena, observation, block, 0).map_err(|_| "producer")?;
        producer.walk().map_err(|_| "walk")?;
        let walked = at.elapsed().as_nanos();
        let at = Instant::now();
        let native = producer.finish_markup().map_err(|_| "native markup")?;
        let finished = at.elapsed().as_nanos();
        let at = Instant::now();
        let artifact = native.file().artifact();
        let facts = template_complexity::run(artifact.root(), artifact.node_count());
        if artifact.node_count() != children as u32 + 1
            || facts != template_complexity::ComplexityFacts::empty()
        {
            return Err("full native result");
        }
        black_box((&native, &facts));
        let analyzed = at.elapsed().as_nanos();
        for (sum, next) in phases.iter_mut().zip([parsed, walked, finished, analyzed]) {
            *sum += next;
        }
    }
    let [parsed, walked, finished, analyzed] = phases.map(|ns| parse_us(ns) / rounds as f64);
    println!(
        "{{\"children\":{children},\"rounds\":{rounds},\"source_bytes\":{},\"nodes\":{},\"parse_us\":{parsed},\"walk_us\":{walked},\"canonical_us\":{finished},\"analysis_us\":{analyzed}}}",
        source.len(),
        children + 1
    );
    if profile {
        for row in global_profiler().summary().entries {
            println!(
                "{{\"span\":\"{}\",\"count\":{},\"total_us\":{}}}",
                row.name,
                row.count,
                row.total.as_secs_f64() * 1_000_000.0
            );
        }
    }
    Ok(())
}
