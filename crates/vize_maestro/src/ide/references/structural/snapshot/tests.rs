use super::*;
use crate::{ide::IdeContext, server::ServerState};

mod uncached;

const SOURCE: &str = "<script setup lang=\"ts\">const alpha = 1; const beta = 2; void outside;</script>\n<template>{{ alpha }}</template>";

fn local_owner(ctx: &IdeContext<'_>, budget: Arc<Budget>) -> Owner {
    Owner::new(
        Arc::new(Cell::new(budget)),
        1,
        ctx.uri.clone(),
        Arc::clone(&ctx.content),
    )
}

fn publish(owner: &Owner, ctx: &IdeContext<'_>) -> Arc<Entry> {
    let projection = super::super::fresh_projection(ctx).expect("projection");
    let facts = super::super::analyze_projection(ctx, &projection).expect("facts");
    owner.publish(projection, facts)
}

fn packet(ctx: &IdeContext<'_>, name: Option<&str>, entry: &Entry) -> Option<serde_json::Value> {
    Some(serde_json::Value::Array(super::super::select(ctx, name, entry)?.into_iter().map(|entry| serde_json::json!({
        "generated": [entry.generated.start, entry.generated.end], "authored": entry.authored,
        "declaration": entry.declaration,
        "edit": super::super::edits::render(entry.edit, "__authored__", "__new__"),
        "symbol": vize_l0::cstr!("{:?}", entry.symbol).as_str(),
    })).collect()))
}

fn warm<T: PartialEq + std::fmt::Debug>(cell: &Cell, expected: &T, query: impl FnOnce() -> T) -> T {
    let before = cell.hit_count();
    let retained = cell.retained();
    let result = query();
    assert_eq!(&result, expected);
    assert_eq!(cell.hit_count() - before, usize::from(retained));
    result
}

#[test]
fn retained_facts_outlive_parser_arena_and_preserve_fresh_query_guards() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Facts>();
    assert_send_sync::<Entry>();
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Owned.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, 0, SOURCE.into());
    let owner = local_owner(&ctx, Arc::new(Budget::default()));
    let entry = publish(&owner, &ctx);
    let expected_names = uncached::symbol_names(&entry.projection.code);
    assert!(entry.lease.is_some());
    let projection = super::super::fresh_projection(&ctx).unwrap();
    let retained = owner.get(&projection).unwrap();
    assert!(Arc::ptr_eq(&entry, &retained));
    for offset in [
        SOURCE.find("alpha =").unwrap(),
        SOURCE.rfind("alpha").unwrap(),
        SOURCE.find("beta =").unwrap(),
    ] {
        let query = IdeContext::with_content(&state, &uri, offset, Arc::clone(&ctx.content));
        for name in [
            None,
            Some("renamed"),
            Some("beta"),
            Some("outside"),
            Some("雪"),
        ] {
            assert_eq!(
                packet(&query, name, &retained),
                uncached::resolved_packet(&query, name)
            );
        }
    }
    drop(retained);
    drop(owner);
    drop(ctx);
    drop(state);
    // Both original ASTs/arenas and the document/cache owner are gone.
    let names: Vec<CompactString> = entry.facts.scoping.symbol_names().map(Into::into).collect();
    assert_eq!(names, expected_names);
    assert!(
        entry
            .facts
            .references
            .iter()
            .any(|reference| reference.name == "outside")
    );
    let entry = std::thread::spawn(move || {
        assert!(
            entry
                .facts
                .scoping
                .symbol_names()
                .any(|name| name == "alpha")
        );
        entry
    })
    .join()
    .unwrap();
    assert!(entry.facts.scoping.references_len() > 0);
}

#[test]
fn every_projection_coordinate_and_policy_change_refuses_the_old_entry() {
    #[derive(Clone, Copy, Debug)]
    enum Mutation {
        Filename,
        Mode,
        Patterned,
        Code,
        Mapping,
        SubSpan,
        Link,
        Order,
    }
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Witness.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, 0, SOURCE.into());
    let owner = local_owner(&ctx, Arc::new(Budget::default()));
    for mutation in [
        Mutation::Filename,
        Mutation::Mode,
        Mutation::Patterned,
        Mutation::Code,
        Mutation::Mapping,
        Mutation::SubSpan,
        Mutation::Link,
        Mutation::Order,
    ] {
        let entry = publish(&owner, &ctx);
        assert!(entry.lease.is_some());
        let mut changed = super::super::fresh_projection(&ctx).unwrap();
        match mutation {
            Mutation::Filename => changed.filename.push_str(".other"),
            Mutation::Mode => {
                changed.mode = if changed.mode == Mode::Standard {
                    Mode::LegacyVue2
                } else {
                    Mode::Standard
                }
            }
            Mutation::Patterned => changed.patterned = !changed.patterned,
            Mutation::Code => changed.code.push(' '),
            Mutation::Mapping => changed.mappings[0].src_range.start += 1,
            Mutation::SubSpan => {
                changed.mappings[0]
                    .sub_spans
                    .push(vize_canon::virtual_ts::mapping::VizeSubSpan {
                        gen_range: 0..1,
                        src_range: 1..2,
                    })
            }
            Mutation::Link => changed.links.push(VizeSemanticLink {
                source_range: 0..1,
                target_range: 1..2,
                kind: vize_canon::virtual_ts::VizeSemanticLinkKind::VueSetupTemplateRefUnwrap,
            }),
            Mutation::Order => {
                assert!(changed.mappings.len() > 1);
                changed.mappings.reverse();
            }
        }
        assert!(owner.get(&changed).is_none(), "mutation {mutation:?}");
        // Eviction did not discard a reader's previously owned facts.
        assert!(
            entry
                .facts
                .scoping
                .symbol_names()
                .any(|name| name == "alpha")
        );
    }
}

#[test]
fn two_process_leases_include_retired_readers_and_refusal_does_not_rebuild() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Budget.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, 0, SOURCE.into());
    let budget = Arc::new(Budget::default());
    let first = local_owner(&ctx, Arc::clone(&budget));
    let second = local_owner(&ctx, Arc::clone(&budget));
    let third = local_owner(&ctx, Arc::clone(&budget));
    let a = publish(&first, &ctx);
    let b = publish(&second, &ctx);
    assert!(a.lease.is_some() && b.lease.is_some());
    assert_eq!(budget.entries.load(Ordering::Acquire), 2);
    first.cell.entry.lock().take();
    second.cell.entry.lock().take();
    assert_eq!(
        budget.entries.load(Ordering::Acquire),
        2,
        "retired readers still retain Scoping"
    );
    let refused = publish(&third, &ctx);
    assert!(refused.lease.is_none());
    assert!(third.cell.entry.lock().is_none());
    assert_eq!(
        packet(&ctx, Some("renamed"), &refused),
        uncached::resolved_packet(&ctx, Some("renamed"))
    );
    drop(a);
    assert_eq!(budget.entries.load(Ordering::Acquire), 1);
    let admitted = publish(&third, &ctx);
    assert!(admitted.lease.is_some());
    drop(b);
    third.cell.entry.lock().take();
    drop(admitted);
    assert_eq!(budget.entries.load(Ordering::Acquire), 0);
    assert_eq!(budget.known_bytes.load(Ordering::Acquire), 0);
}

#[test]
fn admission_is_atomic_across_threads_and_panics_do_not_poison_the_cell() {
    let budget = Arc::new(Budget::default());
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let mut workers = Vec::new();
    for _ in 0..2 {
        let budget = Arc::clone(&budget);
        let barrier = Arc::clone(&barrier);
        workers.push(std::thread::spawn(move || {
            let lease = budget.lease(1).unwrap();
            barrier.wait();
            barrier.wait();
            drop(lease);
        }));
    }
    barrier.wait();
    assert!(budget.lease(1).is_none());
    assert_eq!(budget.entries.load(Ordering::Acquire), 2);
    barrier.wait();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(budget.entries.load(Ordering::Acquire), 0);
    assert_eq!(budget.known_bytes.load(Ordering::Acquire), 0);
    assert!(budget.lease(MAX_KNOWN_BYTES + 1).is_none());
    assert_eq!(
        budget.entries.load(Ordering::Acquire),
        0,
        "byte refusal rolls back count reservation"
    );
    let cell = Arc::new(Cell::default());
    let retained = Arc::clone(&cell);
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _guard = retained.entry.lock();
        panic!("test-only interrupted cache access");
    }));
    assert!(unwind.is_err());
    assert!(cell.entry.lock().is_none());
}

#[test]
fn enum_constant_storage_is_not_retained_by_the_cache() {
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Enum.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, 0, SOURCE.into());
    let owner = local_owner(&ctx, Arc::new(Budget::default()));
    let mut projection = super::super::fresh_projection(&ctx).unwrap();
    projection.code = "enum E { A = 'x', B = A + A, C = B + B }; const alpha = E.C;".into();
    let mut facts = super::super::analyze_projection(&ctx, &projection).unwrap();
    // Production preserves enum_eval=false. Independently evaluate here to
    // exercise admission of opaque constant storage without changing it.
    let allocator = oxc_allocator::Allocator::default();
    let parsed =
        oxc_parser::Parser::new(&allocator, &projection.code, oxc_span::SourceType::ts()).parse();
    facts.scoping = oxc_semantic::SemanticBuilder::new()
        .with_build_nodes(true)
        .with_enum_eval(true)
        .build(&parsed.program)
        .semantic
        .into_scoping();
    assert!(
        facts
            .scoping
            .symbol_ids()
            .any(|id| facts.scoping.get_enum_member_value(id).is_some())
    );
    let entry = owner.publish(projection, facts);
    assert!(entry.lease.is_none());
    assert!(owner.cell.entry.lock().is_none());
}

#[test]
fn dependency_changes_are_observed_before_any_cache_lookup() {
    let directory = tempfile::tempdir().unwrap();
    let props = directory.path().join("props.ts");
    std::fs::write(&props, "export interface Props { alpha: string }").unwrap();
    let source = "<script setup lang=\"ts\">import type { Props } from './props'; defineProps<Props>();</script>\n<template>{{ alpha }} {{ beta }}</template>";
    let state = ServerState::new();
    let uri = Url::from_file_path(directory.path().join("Component.vue")).unwrap();
    let ctx = IdeContext::testing(&state, &uri, source.rfind("alpha").unwrap(), source.into());
    let owner = local_owner(&ctx, Arc::new(Budget::default()));
    let before = publish(&owner, &ctx);
    assert!(before.lease.is_some());
    std::fs::write(
        &props,
        "export interface Props { alpha: string; beta: number }",
    )
    .unwrap();
    let current = super::super::fresh_projection(&ctx).unwrap();
    assert_ne!(
        before.projection, current,
        "the unchanged source has a genuinely changed dependency-derived projection"
    );
    assert!(owner.get(&current).is_none());
    let facts = super::super::analyze_projection(&ctx, &current).unwrap();
    let after = owner.publish(current, facts);
    for offset in [
        source.rfind("alpha").unwrap(),
        source.rfind("beta").unwrap(),
    ] {
        let query = IdeContext::with_content(&state, &uri, offset, Arc::clone(&ctx.content));
        for name in [None, Some("renamed"), Some("alpha")] {
            assert_eq!(
                packet(&query, name, &after),
                uncached::resolved_packet(&query, name)
            );
        }
    }
}

#[test]
fn large_source_admission_refusal_returns_the_same_fresh_packet() {
    let source = vize_l0::cstr!(
        "<script setup>/*{}*/const alpha = 1;</script>\n<template>{{{{ alpha }}}}</template>",
        "x".repeat(256 * 1024)
    );
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Large.vue").unwrap();
    let offset = source.rfind("alpha").unwrap();
    let ctx = IdeContext::testing(&state, &uri, offset, source.into());
    let owner = local_owner(&ctx, Arc::new(Budget::default()));
    let entry = publish(&owner, &ctx);
    assert!(entry.lease.is_none());
    assert!(owner.cell.entry.lock().is_none());
    assert_eq!(
        packet(&ctx, Some("renamed"), &entry),
        uncached::resolved_packet(&ctx, Some("renamed"))
    );
    assert_eq!(owner.cell.budget.entries.load(Ordering::Acquire), 0);
}

#[test]
fn authored_reference_rename_and_prepare_anchors_are_complete() {
    use tower_lsp::lsp_types::{Location, Position, Range, TextEdit};
    let source = "<script setup>const alpha = 1; function read() { return alpha; }</script>\n<template>{{ alpha }}</template>";
    let state = ServerState::new();
    let uri = Url::parse("file:///workspace/Anchor.vue").unwrap();
    let ctx = IdeContext::testing(&state, &uri, source.find("alpha").unwrap(), source.into());
    let cell = state.documents.isolate_structural_snapshot(&uri).unwrap();
    let ranges: Vec<_> = source
        .match_indices("alpha")
        .map(|(offset, _)| {
            let (line, character) = crate::ide::offset_to_position(source, offset);
            let (end_line, end_character) =
                crate::ide::offset_to_position(source, offset + "alpha".len());
            Range {
                start: Position { line, character },
                end: Position {
                    line: end_line,
                    character: end_character,
                },
            }
        })
        .collect();
    assert_eq!(ranges.len(), 3);
    let expected: Vec<_> = ranges
        .iter()
        .map(|range| Location {
            uri: uri.clone(),
            range: *range,
        })
        .collect();
    assert_eq!(super::super::references(&ctx, true), Some(expected.clone()));
    assert_eq!(
        warm(&cell, &Some(expected.clone()), || super::super::references(
            &ctx, true
        )),
        Some(expected.clone())
    );
    assert_eq!(uncached::references(&ctx, true), Some(expected));
    let edits: Vec<_> = ranges
        .iter()
        .map(|range| TextEdit {
            range: *range,
            new_text: "renamed".into(),
        })
        .collect();
    assert_eq!(super::super::rename(&ctx, "renamed"), Some(edits.clone()));
    assert_eq!(
        warm(&cell, &Some(edits.clone()), || super::super::rename(
            &ctx, "renamed"
        )),
        Some(edits.clone())
    );
    assert_eq!(uncached::rename(&ctx, "renamed"), Some(edits));
    assert_eq!(super::super::prepare_rename(&ctx), Some(ranges[0]));
    assert_eq!(
        warm(&cell, &Some(ranges[0]), || super::super::prepare_rename(
            &ctx
        )),
        Some(ranges[0])
    );
    assert_eq!(uncached::prepare_rename(&ctx), Some(ranges[0]));
    assert_eq!(cell.hit_count(), 5);
}

#[test]
fn full_reference_rename_and_prepare_packets_match_the_original_uncached_collector() {
    let cases = [
        "<script setup lang=\"ts\">const alpha = 1;</script>\n<template>{{ alpha }}</template>",
        "<script setup lang=\"ts\">let alpha = 1; function read(beta: number) { return alpha + beta; }</script>\n<template>{{ alpha }} {{ read(1) }}</template>",
        "<script setup>const { alpha } = { alpha: 1 };</script>\n<template>{{ alpha }}</template>",
        "<script setup>const { label: alpha } = { label: 1 };</script>\n<template>{{ alpha }}</template>",
        "<script setup>import { ref } from 'vue'; const alpha = ref(1);</script>\n<template>{{ alpha }}</template>",
        "<script setup>const alpha = 1; const beta = 2; function read() { let beta = 3; return alpha + beta; }</script>\n<template>{{ alpha }}</template>",
        "<script setup>const alpha = 1; const object = { alpha }; export { alpha };</script>\n<template>{{ alpha }}</template>",
        r#"<script setup>const \u0061lpha = 1;</script>
<template>雪😀 {{ alpha }}</template>"#,
        "<script setup lang=\"tsx\">const alpha = <span />;</script>\n<template>{{ alpha }}</template>",
        "<script setup>const alpha = 'red';</script>\n<template>{{ alpha }}</template>\n<style>.x { color: v-bind(alpha); width: v-bind(outside); }</style>",
        "<script setup>const alpha = ;</script>\n<template>{{ alpha }}</template>",
        "<script setup>const alpha = 1;</script>\n<template lang=\"pug\">p {{ alpha }}</template>",
    ];
    let mut comparisons = 0;
    for (case, input) in cases.iter().enumerate() {
        for newline in ["\n", "\r\n"] {
            let source = input.replace('\n', newline);
            let state = ServerState::new();
            let uri = Url::parse("file:///workspace/Packet.vue").unwrap();
            let context = IdeContext::testing(&state, &uri, 0, source.clone());
            let cell = state.documents.isolate_structural_snapshot(&uri).unwrap();
            for (slot, offset) in [
                0,
                source.find("alpha").unwrap(),
                source.rfind("alpha").unwrap(),
                source.len(),
            ]
            .into_iter()
            .enumerate()
            {
                let ctx =
                    IdeContext::with_content(&state, &uri, offset, Arc::clone(&context.content));
                for include in [false, true] {
                    let original = uncached::references(&ctx, include);
                    let candidate = super::super::references(&ctx, include);
                    assert_eq!(candidate, original);
                    let retained = cell.retained();
                    let warm = warm(&cell, &original, || super::super::references(&ctx, include));
                    println!(
                        "structural-snapshot-oracle {}",
                        serde_json::json!({"case":case,"newline":newline,"slot":slot,"method":"references","include":include,"original":original,"candidate":candidate,"warm":warm,"warmHit":retained})
                    );
                    comparisons += 1;
                }
                for name in ["renamed", "beta", "outside", "雪"] {
                    let original = uncached::rename(&ctx, name);
                    let candidate = super::super::rename(&ctx, name);
                    assert_eq!(candidate, original);
                    let retained = cell.retained();
                    let warm = warm(&cell, &original, || super::super::rename(&ctx, name));
                    println!(
                        "structural-snapshot-oracle {}",
                        serde_json::json!({"case":case,"newline":newline,"slot":slot,"method":"rename","name":name,"original":original,"candidate":candidate,"warm":warm,"warmHit":retained})
                    );
                    comparisons += 1;
                }
                let original = uncached::prepare_rename(&ctx);
                let candidate = super::super::prepare_rename(&ctx);
                assert_eq!(candidate, original);
                let retained = cell.retained();
                let warm = warm(&cell, &original, || super::super::prepare_rename(&ctx));
                println!(
                    "structural-snapshot-oracle {}",
                    serde_json::json!({"case":case,"newline":newline,"slot":slot,"method":"prepare","original":original,"candidate":candidate,"warm":warm,"warmHit":retained})
                );
                comparisons += 1;
            }
        }
    }
    assert_eq!(comparisons, 672);
}
