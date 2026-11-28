//! Supplemental instruction probe of the actual shipped L2 text rebuild.
//! Authored inputs and client windows are identical for every source build.

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
compile_error!("this source-only probe targets the existing Linux x86_64 Callgrind lane");

mod instruction;

use std::hint::black_box;
use vize_l0::{Span, String};
use vize_l2::analysis::text_facts::{TextPart, rebuild_source};

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

const FIXTURE: &str = "fixture:l2-text-rebuild-authored-v1";

fn part(text: &str, dynamic: bool, raw_html: bool) -> TextPart {
    TextPart {
        text: String::from(text),
        span: Span::new(0, 0),
        dynamic,
        raw_html,
    }
}

fn oracle(parts: &[TextPart]) -> std::string::String {
    let mut out = std::string::String::new();
    for part in parts {
        if part.raw_html {
            out.push_str("{{{ ");
            out.push_str(&part.text);
            out.push_str(" }}}");
        } else if part.dynamic {
            out.push_str("{{ ");
            out.push_str(&part.text);
            out.push_str(" }}");
        } else {
            out.push_str(&part.text);
        }
    }
    out
}

fn check_edges() -> usize {
    let mut cases = 0;
    for text in ["a", "é", "日", "🦀"] {
        for repetitions in 0..=64 {
            for flags in 0..4 {
                let text = text.repeat(repetitions);
                let parts = [
                    part("prefix: ", false, false),
                    part(&text, flags & 1 != 0, flags & 2 != 0),
                ];
                let expected = oracle(&parts);
                let actual = rebuild_source(&parts);
                assert_eq!(actual.as_str(), expected);
                assert_eq!(
                    actual.is_heap_allocated(),
                    actual.len() > std::mem::size_of::<String>()
                );
                cases += 1;
            }
        }
    }
    assert_eq!(rebuild_source(&[]), "");
    cases
}

fn main() {
    assert!(
        instruction::requested(),
        "run through the supplied Callgrind driver"
    );
    instruction::initialize();
    let cases = check_edges();
    let inputs = [
        vec![part("hello", false, false), part("x", true, false)],
        vec![part("こんにちは", false, false), part("key", true, false)],
        vec![
            part("The current count is ", false, false),
            part("count", true, false),
            part(" items now.", false, false),
        ],
        vec![
            part("The current count is ", false, false),
            part("count", true, false),
            part(" and selected item is ", false, false),
            part("selectedItem.label", true, false),
            part(". Refresh after next tick", false, false),
        ],
    ];
    let ids = [
        "text.rebuild.inline12",
        "text.rebuild.inline24",
        "text.rebuild.heap43",
        "text.rebuild.heap103",
    ];
    let expected_lengths = [12, 24, 43, 103];
    let mut outputs = Vec::new();
    let mut lengths = Vec::new();
    let mut heap = Vec::new();
    let mut capacities = Vec::new();
    for ((parts, id), expected_length) in inputs.iter().zip(ids).zip(expected_lengths) {
        let expected = oracle(parts);
        assert_eq!(expected.len(), expected_length);
        let rebuilt = instruction::measure(id, || rebuild_source(black_box(parts)));
        instruction::record(id, FIXTURE, "routine-return");
        assert_eq!(rebuilt.as_str(), expected);
        lengths.push(rebuilt.len());
        heap.push(rebuilt.is_heap_allocated());
        capacities.push(rebuilt.capacity());
        outputs.push(rebuilt.as_str().to_owned());
        black_box(rebuilt);
    }
    println!(
        "{}",
        serde_json::json!({
            "oracle": "l2-text-rebuild-v1",
            "cases": cases,
            "outputs": outputs,
            "lengths": lengths,
            "heap": heap,
            "capacities": capacities,
        })
    );
}
