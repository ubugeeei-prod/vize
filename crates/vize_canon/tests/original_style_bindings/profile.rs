//! Compare complete cold and retained requests in a dedicated allocation process.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "profile validated original style requests"
)]
mod input;
use input::{options, originals};
use vize_atelier_sfc::{
    parse_sfc,
    vite_plugin::{
        transform_css_vars_for_pipeline_with_context, transform_original_style_bindings,
    },
};
use vize_canon::virtual_ts::StyleBindingReferences;
use vize_carton::FxHashSet;
use vize_l0::Allocator;
use vize_l1::container::vue::{AdmittedDescriptor, Vue};
use vize_l1_to_l2::{lower::OriginalStyleBindings, lower_style_block_in};
use vize_l2::op::Op;
#[global_allocator]
static ALLOCATOR: davinci_harness::alloc::CountingAllocator<std::alloc::System> =
    davinci_harness::alloc::CountingAllocator::system();

use davinci_harness::alloc;
use std::{hint::black_box, time::Instant};
use vize_l0::String as CompactString;

type Response = (
    Vec<CompactString>,
    Vec<CompactString>,
    FxHashSet<CompactString>,
    usize,
);

fn response<'a>(
    allocator: &'a Allocator,
    owner: AdmittedDescriptor<'_, 'a>,
    prepare_descriptor: bool,
) -> Option<Response> {
    let mut css = Vec::new();
    let mut calls = 0;
    let count = |op: Op<'_>| match black_box(op) {
        Op::Element(el) => el.bindings.len(),
        _ => panic!("style"),
    };
    let descriptor = if prepare_descriptor {
        Some(parse_sfc(black_box(owner.source()), Default::default()).ok()?)
    } else {
        None
    };
    let mut vars = Vec::new();
    let mut names = FxHashSet::default();
    for style in owner.styles() {
        let block = style.block();
        calls += count(lower_style_block_in(allocator, block));
        css.push(transform_css_vars_for_pipeline_with_context(
            block.source(),
            "data-v-53e8",
            None,
        ));
        if descriptor.is_none() {
            for expression in vize_croquis::sfc::__internal::extract_css_vars(block.source()) {
                if !vars.contains(&expression) {
                    vars.push(expression);
                }
            }
        }
    }
    if let Some(descriptor) = descriptor {
        vars.extend(
            descriptor
                .css_vars
                .iter()
                .map(|v| CompactString::from(v.as_ref())),
        );
        black_box(descriptor);
    }
    for expression in &vars {
        names.extend(vize_croquis::drawer::extract_identifiers_oxc(expression));
    }
    Some((css, vars, names, calls))
}

fn shared_response<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    retained: Option<AdmittedDescriptor<'_, 'a>>,
) -> Option<Response> {
    let mut css = Vec::new();
    let mut calls = 0;
    let emit = |style: OriginalStyleBindings<'_>| {
        css.push(transform_original_style_bindings(
            &style,
            "data-v-53e8",
            None,
            |_| {},
        ));
        let Op::Element(el) = black_box(style.into_op(allocator)) else {
            panic!("style")
        };
        calls += el.bindings.len();
    };
    let refs = if let Some(owner) = retained {
        let Some(refs) = StyleBindingReferences::observe(allocator, owner, emit) else {
            return Some((css, Vec::new(), FxHashSet::default(), calls));
        };
        refs
    } else {
        let (descriptor, refs) =
            StyleBindingReferences::prepare(allocator, source, Default::default(), emit).ok()?;
        black_box(descriptor);
        refs
    };
    let vars = refs
        .expressions()
        .iter()
        .copied()
        .map(CompactString::from)
        .collect();
    Some((css, vars, refs.into_names(), calls))
}

fn paired(retained: AdmittedDescriptor<'_, '_>, label: &str, n: u128) -> bool {
    let source = retained.source();
    let mode = |cold| {
        let run = |joined| {
            let arena = Allocator::new();
            if cold && joined {
                shared_response(&arena, black_box(source), None)
            } else if cold {
                let desc = Vue.observe_descriptor(&arena, black_box(source), options());
                response(&arena, desc.admitted().unwrap(), true)
            } else if joined {
                shared_response(&arena, source, Some(black_box(retained)))
            } else {
                response(&arena, black_box(retained), false)
            }
        };
        assert_eq!(run(false), run(true), "{label}");
        let old = alloc::measure(|| run(false)).unwrap();
        let new = alloc::measure(|| run(true)).unwrap();
        let sample = |n, reverse| {
            let time = |joined| {
                let t = Instant::now();
                for _ in 0..n {
                    drop(black_box(run(joined)));
                }
                t.elapsed().as_nanos() / n
            };
            let mut pairs = Vec::new();
            for round in 0..35 {
                let first = (round % 2 == 0) != reverse;
                let a = time(first);
                let b = time(!first);
                if round >= 4 {
                    pairs.push(if first { (b, a) } else { (a, b) });
                }
            }
            pairs
        };
        // A no-style retained query can finish within a single clock tick.
        let n = if !cold && retained.styles().len() == 0 {
            n.max(32)
        } else {
            n
        };
        let mut p = sample(n, false);
        let slower = |r: &[(u128, u128)]| r.iter().filter(|(a, b)| b > a).count() >= 23;
        let mut pass =
            new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start;
        if slower(&p) {
            let confirm = sample(n * 8, true);
            eprintln!("css_bind_confirm input={label} cold={cold} pairs={confirm:?}");
            pass &= !slower(&confirm);
        }
        if label.parse::<usize>().is_ok() || !pass {
            eprintln!("css_bind_pairs input={label} cold={cold} pairs={p:?}");
        }
        p.sort_unstable_by_key(|p| p.0);
        let before = p[15].0;
        p.sort_unstable_by_key(|p| p.1);
        eprintln!(
            "css_bind_request input={label} cold={cold} ns={}/{} calls={}/{} peak={}/{} pass={pass}",
            before,
            p[15].1,
            old.calls,
            new.calls,
            old.peak_bytes_over_start,
            new.peak_bytes_over_start
        );
        pass
    };
    let retained = mode(false);
    let cold = mode(true);
    retained && cold
}

fn paired_style_requests() {
    for count in [0, 1, 4, 32, 128] {
        let mut text = String::from("<template><i/></template><style lang='scss'>");
        for ix in 0..count {
            text.push_str(&format!(
                ".x{ix}{{color:v-bind(c{ix});height:v-bind('a + b')}}\n"
            ));
        }
        text.push_str("</style>");
        let arena = Allocator::new();
        let desc = Vue.observe_descriptor(&arena, &text, options());
        assert!(paired(desc.admitted().unwrap(), &count.to_string(), 4));
    }
}

fn original_style_requests() {
    let mut failures = Vec::new();
    let mut counted = 0;
    originals(|_, _, owner, path| {
        let Some(owner) = owner else { return };
        counted += 1;
        if !paired(owner, path, 1) {
            failures.push(path.to_owned());
        }
    });
    eprintln!("css_bind_requests files={counted}");
    assert!(counted > 0);
    assert!(failures.is_empty(), "{failures:?}");
}

fn main() {
    alloc::mark_installed();
    paired_style_requests();
    original_style_requests();
}
