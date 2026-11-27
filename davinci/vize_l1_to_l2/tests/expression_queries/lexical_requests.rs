//! Cold combined script symbols and component events from the two existing analyses or one File.
use super::component_projection::{Site, draw_descriptor, original, retained_product};
use davinci_harness::alloc::measure;
use oxc_span::SourceType;
use std::{hint::black_box, time::Instant};
use vize_l0::{
    Allocator, SourceRoot, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::lower_sfc_with_lexical_initializers;
use vize_l2::{
    ir::{DeclarationKind, InitializerKind},
    query::ComponentListenerIssue,
};

use super::lexical_symbols as oracle;

#[derive(Debug, PartialEq, Eq)]
struct Event {
    name: String,
    handler: Option<String>,
    declaration: Option<Span>,
    modifiers: Vec<String>,
    span: Span,
}
type Products = (oracle::Symbols, Vec<Site>, Vec<Event>);

fn independent(source: &str, resolved_sites: bool) -> Products {
    let descriptor = vize_atelier_sfc::parse_sfc(source, Default::default()).unwrap();
    let setup = descriptor.script_setup.as_ref().unwrap();
    let block = SourceRoot::new(source)
        .unwrap()
        .block(&setup.content, setup.loc.start as u32)
        .unwrap();
    let ty = if setup.lang.as_deref() == Some("ts") {
        SourceType::ts()
    } else {
        SourceType::mjs()
    };
    let (analysis, base, _) = draw_descriptor(&descriptor);
    let sites = if resolved_sites {
        original(source, &analysis, base)
    } else {
        Vec::new()
    };
    let mut events = Vec::new();
    for site in vize_croquis::facts::component_usage_slice(&analysis) {
        for e in &site.events {
            events.push(Event {
                name: e.name.to_string(),
                declaration: None,
                handler: e.handler.as_ref().map(ToString::to_string),
                modifiers: e.modifiers.iter().map(ToString::to_string).collect(),
                span: Span::new(base + e.start, base + e.end),
            });
        }
    }
    drop(analysis);
    let ty = ty.with_module(true);
    let symbols = if resolved_sites {
        oracle::with_oxc(block, ty, |scopes| {
            for row in &mut events {
                row.declaration = row.handler.as_deref().map(|name| {
                    let span = scopes.symbol_span(scopes.get_root_binding(name.into()).unwrap());
                    Span::new(block.start() + span.start, block.start() + span.end)
                });
            }
        })
    } else {
        oracle::oxc(block, ty)
    };
    (symbols, sites, events)
}

fn shared(source: &str, resolved_sites: bool, select_arrow_file: bool) -> Option<Products> {
    let arena = Allocator::new();
    let owner = lower_sfc_with_lexical_initializers(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        },
    );
    let imports = owner.setup_component_imports().ok()?;
    let file = owner
        .file()
        .map(|f| f.l2ir())
        .or_else(|| owner.rejected_file()?.file())?;
    if select_arrow_file
        && !file.bindings().filter_map(|b| b.declaration()).any(|d| {
            d.initializer == InitializerKind::Function
                && matches!(
                    d.kind,
                    DeclarationKind::Const | DeclarationKind::Let | DeclarationKind::Var
                )
        })
    {
        return None;
    }
    let mut events = Vec::new();
    imports
        .try_for_each_in_postorder(|site| {
            site.visit_static_listeners(|row| {
                events.push(Event {
                    name: row.name.into(),
                    declaration: if resolved_sites {
                        row.handler_binding.map(|b| b.declaration().unwrap().span)
                    } else {
                        None
                    },
                    handler: row.handler.map(|span| span.slice(source).into()),
                    modifiers: row.modifiers.iter().map(|m| (*m).into()).collect(),
                    span: row.span,
                })
            })?;
            Ok::<_, ComponentListenerIssue>(())
        })
        .ok()?;
    Some((
        oracle::file(file),
        if resolved_sites {
            retained_product(&imports)
        } else {
            Vec::new()
        },
        events,
    ))
}

fn sample(source: &str, name: &str, resolved_sites: bool, failures: &mut Vec<String>) -> bool {
    let Some(actual) = shared(source, resolved_sites, true) else {
        return false;
    };
    assert_eq!(actual, independent(source, resolved_sites), "{name}");
    let run = |joined| {
        if joined {
            shared(black_box(source), resolved_sites, false).unwrap()
        } else {
            independent(black_box(source), resolved_sites)
        }
    };
    let old = measure(|| drop(black_box(run(false)))).unwrap();
    let new = measure(|| drop(black_box(run(true)))).unwrap();
    let time = |joined| {
        let clock = Instant::now();
        for _ in 0..8 {
            drop(black_box(run(joined)));
        }
        clock.elapsed().as_nanos() / 8
    };
    let mut before = Vec::new();
    let mut after = Vec::new();
    for round in 0..35 {
        let first = round % 2 == 0;
        let a = time(first);
        let b = time(!first);
        if round >= 4 {
            let (old, new) = if first { (b, a) } else { (a, b) };
            before.push(old);
            after.push(new);
        }
    }
    before.sort_unstable();
    after.sort_unstable();
    eprintln!(
        "lexical_combined case={name} resolved_sites={resolved_sites} ns={}/{} allocations={}/{} peak={}/{}",
        before[15],
        after[15],
        old.calls,
        new.calls,
        old.peak_bytes_over_start,
        new.peak_bytes_over_start
    );
    if after[15] > before[15]
        || new.calls > old.calls
        || new.peak_bytes_over_start > old.peak_bytes_over_start
    {
        failures.push(format!("{name} complete={resolved_sites}"));
    }
    true
}

pub(super) fn run() {
    let mut failures = Vec::new();
    for complete in [false, true] {
        assert!(sample(
            "<script setup lang='ts'>import Card from './Card.vue'; const 外=3; const save=(n:number)=>n+外;</script><template><Card @click='save'/></template>",
            "captured method",
            complete,
            &mut failures
        ));
    }
    if let Some(path) = std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST") {
        let list = std::fs::read_to_string(path).unwrap();
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut admitted = 0;
        for name in list.split_terminator('\0') {
            let bytes = std::fs::read(root.join(name)).unwrap();
            if let Ok(source) = std::str::from_utf8(&bytes)
                && source.contains("=>")
            {
                if sample(source, name, false, &mut failures) {
                    admitted += 1;
                    assert!(sample(source, name, true, &mut failures));
                }
            }
        }
        if std::env::var_os("VIZE_REQUIRE_REPOSITORY_ARROWS").is_some() {
            assert!(admitted >= 23);
        }
        eprintln!("lexical_combined real_files={admitted}");
    }
    assert!(failures.is_empty(), "{failures:?}");
}
