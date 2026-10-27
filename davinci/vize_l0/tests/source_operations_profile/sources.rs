//! Real producer controls, separate from the allocator's exact-layout window.

use std::hash::Hasher;
use vize_l0::profiler::{global_profiler, micro};

pub(super) fn run() {
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let mut actual = vize_l0::micro_default_hasher!("test.default_hash");
    let mut expected = std::hash::DefaultHasher::new();
    actual.write(b"abc");
    expected.write(b"abc");
    actual.write_u64(17);
    expected.write_u64(17);
    assert_eq!(actual.finish(), expected.finish());
    profiler.disable();
    let sites = micro::snapshot();
    let mut counts = sites
        .iter()
        .map(|site| (site.name, site.calls, site.units))
        .collect::<Vec<_>>();
    counts.sort_by_key(|row| row.0);
    assert_eq!(
        counts,
        [
            ("test.default_hash.build", 1, 1),
            ("test.default_hash.finish", 1, 1),
            ("test.default_hash.write_bytes", 2, 11),
        ]
    );

    println!(
        "hash_control_counts={}",
        serde_json::to_string(&sites).expect("counts serialize")
    );

    profiler.clear();
    profiler.enable();
    let setup = "const value = 1;";
    let plain = "const plain = 2;";
    let incomplete = "const recovered = missing.;";
    let parsed = vize_croquis::script_parser::parse_script_setup(setup);
    assert_eq!(
        parsed.bindings.get("value"),
        Some(vize_croquis::BindingType::LiteralConst)
    );
    let _plain = vize_croquis::script_parser::parse_script(plain);
    // The current fatal path retries the original view before blanking its dot.
    // Count both actual recovery parses; a request count must not hide this work.
    let _recovered = vize_croquis::script_parser::parse_script_setup(incomplete);
    profiler.disable();
    let sites = micro::snapshot();
    let mut counts = sites
        .iter()
        .filter(|site| site.kind == "counter" && site.name.starts_with("croquis."))
        .map(|site| (site.name, site.calls, site.units))
        .collect::<Vec<_>>();
    counts.sort_by_key(|row| row.0);
    assert_eq!(
        counts,
        [
            (
                "croquis.oxc.original.bytes",
                3,
                (setup.len() + plain.len() + incomplete.len()) as u64
            ),
            (
                "croquis.oxc.recovery.bytes",
                2,
                (incomplete.len() * 2) as u64
            ),
            ("croquis.plain.parse_requests.bytes", 1, plain.len() as u64),
            (
                "croquis.setup.analysis_requests.bytes",
                2,
                (setup.len() + incomplete.len()) as u64
            ),
            (
                "croquis.setup.parse_requests.bytes",
                2,
                (setup.len() + incomplete.len()) as u64
            ),
        ]
    );
    let spans = sites
        .iter()
        .filter(|site| site.kind == "span" && site.name.starts_with("croquis."))
        .map(|site| (site.name, site.calls))
        .collect::<Vec<_>>();
    assert_eq!(
        spans,
        [("croquis.oxc.original", 3), ("croquis.oxc.recovery", 2)]
    );
    println!(
        "parser_control_counts={}",
        serde_json::to_string(&sites).expect("counts serialize")
    );
    println!("selected std hash writes and actual parser attempts passed");
}
