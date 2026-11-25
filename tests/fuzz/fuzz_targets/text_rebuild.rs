#![no_main]

// Differential target for `vize_l2::analysis::text_facts::rebuild_source`,
// which copies the parts into the string's spare capacity under `unsafe`.
// The input is split on `\n` into parts; a part's first byte picks its kind
// (`$` dynamic, `!` raw-HTML dynamic, otherwise static) and the rest is its
// text. The rebuild must equal the plain one-`push_str`-per-piece spelling
// byte for byte, for every inline and heap length.
//
// The corpus is seeded from the `<template>` blocks of repository .vue
// fixtures by `tools/ci/commands/fuzz/seed-corpus.ts`.
use libfuzzer_sys::fuzz_target;
use vize_l0::{Span, String};
use vize_l2::analysis::text_facts::{TextPart, rebuild_source};

fn oracle(parts: &[TextPart]) -> String {
    let mut out = String::default();
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

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    let parts: Vec<TextPart> = source
        .split('\n')
        .map(|line| {
            let (dynamic, raw_html, text) = match line.as_bytes().first() {
                Some(b'$') => (true, false, line.get(1..).unwrap_or("")),
                Some(b'!') => (true, true, line.get(1..).unwrap_or("")),
                _ => (false, false, line),
            };
            TextPart { text: String::from(text), span: Span::new(0, 0), dynamic, raw_html }
        })
        .collect();
    let rebuilt = rebuild_source(&parts);
    assert_eq!(rebuilt, oracle(&parts));
});
