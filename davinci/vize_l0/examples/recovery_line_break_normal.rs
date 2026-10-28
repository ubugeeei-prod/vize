//! Normal source-parser preflight; generated inputs and reports are outside the window.
#![expect(
    clippy::expect_used,
    reason = "the source benchmark fails closed on invalid controls"
)]

fn main() {
    let args: vize_l0::SmallVec<[_; 4]> = std::env::args()
        .skip(1)
        .map(vize_l0::String::from)
        .collect();
    let iterations = args
        .first()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(20);
    let minimum = args
        .get(1)
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(131_072);
    let scenario = args.get(2).map_or("irreparable", |value| value.as_str());
    let encoding = args.get(3).map_or("unicode", |value| value.as_str());
    assert!(iterations > 0 && minimum > 0);
    assert!(matches!(scenario, "irreparable" | "valid" | "repairable"));
    assert!(matches!(
        encoding,
        "ascii"
            | "unicode"
            | "e2-dense"
            | "early-em-dash"
            | "late-em-dash"
            | "u2028-dense"
            | "crlf-neighbors"
    ));

    let mut source = vize_l0::String::default();
    if scenario == "irreparable" {
        source.push_str("const broken = ;\r\n");
    }
    if encoding == "early-em-dash" {
        source.push_str("// —\r\n");
    }
    let dense = "\u{2009}\u{2014}\u{2027}".repeat(32);
    let mut rows = 0;
    while source.len() < minimum {
        use std::fmt::Write;
        match encoding {
            "ascii" | "early-em-dash" | "late-em-dash" => {
                write!(source, "const item_{rows}: string = \"payload\";\r\n")
            }
            "u2028-dense" => write!(
                source,
                "const item_{rows}: string = \"payload\";\u{2028}\u{2029}\r\n"
            ),
            "crlf-neighbors" => write!(
                source,
                "const item_{rows}: string = \"payload\";\r\n\r\n\n\r"
            ),
            "e2-dense" => write!(source, "const 雪_{rows}: string = \"{dense}\";\r\n"),
            _ => write!(source, "const 雪_{rows}: string = \"日本🙂\";\r\n"),
        }
        .expect("actual authored declarations build");
        rows += 1;
    }
    if encoding == "late-em-dash" {
        source.push_str("// —\r\n");
    }
    if scenario != "irreparable" {
        source.push_str(if scenario == "valid" {
            "const damaged = 0;\r\n"
        } else {
            "const damaged = missing.;\r\n"
        });
    }
    assert!(source.contains("\r\n"));
    assert_eq!(
        source.is_ascii(),
        matches!(encoding, "ascii" | "crlf-neighbors")
    );
    if encoding == "e2-dense" {
        assert!(source.contains(&dense));
    }
    let e2_leads = source
        .as_bytes()
        .iter()
        .filter(|&&byte| byte == 0xe2)
        .count();
    let source_hash = vize_l0::hash::hash_bytes(source.as_bytes());
    let expected_bindings = if scenario == "irreparable" {
        0
    } else {
        rows + 1
    };
    let started = std::time::Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..iterations {
        let parsed = vize_croquis::script_parser::parse_script_setup(std::hint::black_box(&source));
        checksum += std::hint::black_box(parsed.bindings.bindings.len()) as u64;
    }
    let elapsed_ns = started.elapsed().as_nanos();
    assert_eq!(checksum, (expected_bindings * iterations) as u64);
    println!(
        "{}",
        serde_json::json!({
            "scope": "normal repeated source parse+analysis; no backend or project cold claim",
            "iterations": iterations, "minimumSourceBytes": minimum,
            "sourceBytes": source.len(), "sourceHash": source_hash, "rows": rows,
            "scenario": scenario, "encoding": encoding, "e2LeadBytesInInput": e2_leads,
            "checksum": checksum, "elapsedNs": elapsed_ns, "instrumented": false,
        })
    );
}
