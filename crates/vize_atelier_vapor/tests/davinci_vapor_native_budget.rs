//! P3-6 performance gate, in its own process (the counters are global).
//!
//! Wall-clock ratios are too noisy on shared CI hosts, so the gate pins what
//! is deterministic: allocation calls per compile of every `vapor_native_pair`
//! fixture on the native S3 lane, exactly at the committed ceiling. The
//! retained lane's calls print beside them; interleaved timings live in the
//! P3-6 evidence record. Improvements ratchet a ceiling down; nothing raises
//! one.

#![expect(clippy::expect_used, reason = "tests assert by panicking")]
#![expect(
    clippy::disallowed_macros,
    reason = "test fixtures and insta snapshots use std strings and format"
)]

use davinci_harness::alloc::{CountingAllocator, mark_installed, measure};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator::mimalloc();

/// `(fixture, source, native allocation ceiling)`: the bench fixtures. The
/// retained lane measured 78, 81, 138, 141, 155, 122 and 180 calls when the
/// ceilings were set; the native surplus on text, events and templates is the
/// shared S1-to-S2 lowering's heap tables.
const FIXTURES: [(&str, &str, u64); 7] = [
    (
        "text_runs",
        "<main>Hello {{ name }}!<span>{{ first }} / {{ last }}</span><button @click=\"save\">Save {{ count }}</button></main>",
        75,
    ),
    (
        "events",
        "<main @keydown=\"save\"><button @click.stop=\"save\" @keydown.enter.stop=\"save\">{{ label }}</button><input @focus=\"save\" @change.once=\"save\"></main>",
        74,
    ),
    (
        "expressions",
        "<main class=\"shell\" :class=\"{ dense, [theme]: true }\"><button @click=\"count++\" :title=\"'n=' + count\">{{ count * 2 }} / {{ label.toUpperCase() }}</button><div v-show=\"open && ready\" v-text=\"items.map(i => i.name).join(', ')\"></div></main>",
        109,
    ),
    (
        "components",
        "<main><Counter :label=\"title\" :step=\"2\" @bump=\"total += $event\"><b>{{ total }}</b></Counter><slot name=\"aside\" :n=\"total\"><i>none</i></slot></main>",
        100,
    ),
    (
        "templates",
        "<main><template v-if=\"open\"><header>{{ title }}</header><section>{{ lead }}</section></template><ul><template v-for=\"row in rows\" :key=\"row.id\"><li>{{ row.label }}</li><li v-if=\"row.note\">{{ row.note }}</li></template></ul></main>",
        134,
    ),
    (
        "spreads",
        "<main class=\"shell\"><section id=\"card\" class=\"card\" v-bind=\"attrs\" :title=\"title\"><b v-bind=\"badge\">{{ count }}</b></section><Panel v-bind=\"panel\" v-on=\"{ close: save }\" :size=\"size\" /><button v-on=\"handlers\">go</button></main>",
        106,
    ),
    (
        "control_flow",
        "<main><section v-if=\"open\"><b>{{ title }}</b><span v-for=\"row in rows\" :key=\"row.id\" :title=\"row.title\">{{ row.label }}</span></section><i v-else>closed</i><ul><li v-for=\"(cell, i) in cells\" @click=\"save\">{{ i }}: {{ cell }}</li></ul></main>",
        158,
    ),
];

fn calls(source: &str, davinci_retained_lane: bool) -> u64 {
    let options = VaporCompilerOptions {
        prefix_identifiers: true,
        davinci_retained_lane,
        ..Default::default()
    };
    // Warm lazily initialized statics outside the measured compile.
    let allocator = Allocator::new();
    drop(compile_vapor(&allocator, source, options.clone()));
    drop(allocator);
    measure(|| {
        let allocator = Allocator::new();
        drop(compile_vapor(&allocator, source, options));
    })
    .expect("the counting allocator is installed")
    .calls
}

#[test]
fn native_lane_stays_within_its_allocation_ceilings() {
    mark_installed();
    let mut failures = Vec::new();
    for (name, source, ceiling) in FIXTURES {
        let native = calls(source, false);
        let retained = calls(source, true);
        println!("measured {name}: native {native} retained {retained}");
        if native > ceiling {
            failures.push(format!("{name}: {native} > ceiling {ceiling}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("; "));
}
