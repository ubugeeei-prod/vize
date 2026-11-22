#![expect(
    clippy::unwrap_used,
    reason = "qualification requires the original inputs and complete paired outputs"
)]
use super::*;
use crate::{
    SfcDescriptor, SfcScriptBlock, compile_script::runtime_bindings::collect_runtime_bindings,
};
use davinci_harness::alloc::{self, CountingAllocator};
use oxc_span::SourceType;
use std::alloc::System;
use vize_carton::{Allocator, SourceBlock, SourceRoot};
use vize_croquis::{EffectGraphScript, build_effect_graph_from_sfc_scripts};
use vize_l1::embed::authored_program::AuthoredProgram;

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();
mod corpus;
mod profile;

#[derive(Clone, Copy)]
struct Block<'a> {
    frame: SourceBlock<'a>,
    input: EffectGraphScript<'a>,
    grammar: SourceType,
}

fn blocks<'a>(source: &'a str, descriptor: &'a SfcDescriptor<'_>) -> [Option<Block<'a>>; 2] {
    let root = SourceRoot::new(source).unwrap();
    let role = |b: &'a SfcScriptBlock<'_>| {
        let text = source.get(b.loc.start..b.loc.end).unwrap();
        assert_eq!(text, b.content.as_ref());
        let frame = root.block(text, b.loc.start.try_into().unwrap()).unwrap();
        let lang = b.lang.as_deref();
        let path = match lang.map(str::trim) {
            Some(l) if l.eq_ignore_ascii_case("ts") => "file.ts",
            Some(l) if l.eq_ignore_ascii_case("tsx") => "file.tsx",
            Some(l) if l.eq_ignore_ascii_case("jsx") => "file.jsx",
            _ => "file.js",
        };
        Block {
            frame,
            input: EffectGraphScript::new(text, lang),
            grammar: SourceType::from_path(path).unwrap(),
        }
    };
    [
        descriptor.script.as_ref().map(role),
        descriptor.script_setup.as_ref().map(role),
    ]
}

fn parse<'a>(
    arena: &'a Allocator,
    blocks: [Option<Block<'a>>; 2],
) -> [Option<AuthoredProgram<'a>>; 2] {
    blocks.map(|block| block.map(|b| AuthoredProgram::parse(arena, b.frame, b.grammar)))
}

fn read(
    descriptor: &SfcDescriptor<'_>,
    parsed: &[Option<AuthoredProgram<'_>>; 2],
) -> Result<ScriptBindingProducts, ScriptBindingError> {
    script_binding_products(descriptor, parsed[0].as_ref(), parsed[1].as_ref())
}

fn original(
    blocks: [Option<Block<'_>>; 2],
    parsed: &[Option<AuthoredProgram<'_>>; 2],
) -> ScriptBindingProducts {
    let names = |i: usize| {
        parsed[i]
            .as_ref()
            .map_or_else(FxHashSet::default, |syntax| {
                collect_runtime_bindings(syntax.parsed().program.body.iter())
            })
    };
    ScriptBindingProducts {
        ordinary: names(0),
        setup: names(1),
        effects: build_effect_graph_from_sfc_scripts(
            blocks[0].map(|b| b.input),
            blocks[1].map(|b| b.input),
        ),
    }
}

fn compare(a: &ScriptBindingProducts, b: &ScriptBindingProducts, label: &str) {
    assert_eq!(a.ordinary, b.ordinary, "{label}");
    assert_eq!(a.setup, b.setup, "{label}");
    assert!(
        a.effects.edges().eq(b.effects.edges()),
        "{label}: {:?} {:?}",
        a.effects,
        b.effects
    );
    assert_eq!(a.effects.summary(), b.effects.summary(), "{label}");
    assert_eq!(a.effects.find_cycle(), b.effects.find_cycle(), "{label}");
}

#[test]
fn role_inheritance_shadowing_and_reactive_cycles_match_the_public_graph() {
    let source = r#"<script lang="ts">import { ref as r, computed as c } from 'vue'; export const count=r(0); const left=c(()=>right.value); const right=c(()=>left.value);</script>
<script setup lang="ts">import type { Phantom } from 'pkg'; const before=c(()=>count.value); const {a:[definePageMeta],...rest}=obj; const local=r(1); const after=c(()=>local.value);</script>"#;
    let descriptor = crate::parse_sfc(source, Default::default()).unwrap();
    let input = blocks(source, &descriptor);
    let arena = Allocator::new();
    let parsed = parse(&arena, input);
    let real = read(&descriptor, &parsed).unwrap();
    assert!(matches!(
        script_binding_products(&descriptor, parsed[1].as_ref(), parsed[0].as_ref()),
        Err(ScriptBindingError::Selection)
    ));
    assert!(matches!(
        script_binding_products(&descriptor, None, parsed[1].as_ref()),
        Err(ScriptBindingError::Selection)
    ));
    compare(&real, &original(input, &parsed), "two roles");
    assert!(real.ordinary.contains("r"));
    assert!(real.setup.contains("definePageMeta"));
    assert!(!real.setup.contains("Phantom"));
    assert!(real.effects.summary().cycle_count > 0);
    assert!(
        real.effects
            .edges()
            .any(|e| e.from.as_str() == "setup:before" && e.to.as_str() == "script:count")
    );
}

#[test]
fn different_bytes_profiles_and_overlapping_scripts_refuse() {
    let source = "export const x=1";
    let copy = std::string::String::from(source);
    let arena = Allocator::new();
    let program = AuthoredProgram::parse(
        &arena,
        SourceRoot::new(source).unwrap().whole_block(),
        SourceType::from_path("script.ts").unwrap(),
    );
    let input = EffectGraphScript::new(source, Some("ts"));
    assert!(matches!(
        EffectGraphScript::new(&copy, Some("ts")).with_original(&program),
        Err(ObservedEffectError::Source)
    ));
    assert!(matches!(
        EffectGraphScript::new(source, Some("tsx")).with_original(&program),
        Err(ObservedEffectError::Grammar)
    ));
    let receipt = input.with_original(&program).unwrap();
    assert!(matches!(
        visit_sfc_effect_graph(Some(&receipt), Some(&receipt), |_, _| {}),
        Err(ObservedEffectError::OverlappingBlocks)
    ));
    let other = AuthoredProgram::parse(
        &arena,
        SourceRoot::new(&copy).unwrap().whole_block(),
        program.source_type(),
    );
    let separate = EffectGraphScript::new(&copy, Some("ts"))
        .with_original(&other)
        .unwrap();
    assert!(matches!(
        visit_sfc_effect_graph(Some(&receipt), Some(&separate), |_, _| {}),
        Err(ObservedEffectError::Source)
    ));
}

#[test]
fn owned_descriptors_reuse_their_original_frame_and_external_scripts_refuse() {
    let source = "<script setup lang='ts'>const {x:[longName,...rest]}=input;</script>";
    let borrowed = crate::parse_sfc(source, Default::default()).unwrap();
    let owned = borrowed.clone().into_owned();
    let arena = Allocator::new();
    let original = parse(&arena, blocks(source, &borrowed));
    assert!(matches!(
        read(&owned, &original),
        Err(ScriptBindingError::Selection)
    ));
    let original = parse(&arena, blocks(owned.source.as_ref(), &owned));
    let products = read(&owned, &original).unwrap();
    assert!(products.setup.contains("longName") && products.setup.contains("rest"));
    let mut external = owned.clone();
    external.script_setup.as_mut().unwrap().src = Some("./external.ts".into());
    assert!(matches!(
        read(&external, &original),
        Err(ScriptBindingError::ExternalScript)
    ));
}

#[test]
#[ignore = "requires an isolated process-wide allocation window"]
fn visiting_destructured_names_and_original_ranges_does_not_allocate() {
    let name = "long_name_".repeat(18);
    let source = format!("export const {{field:[{{key:{name}=value}},,...rest],...other}}=object;");
    let arena = Allocator::new();
    let original = AuthoredProgram::parse(
        &arena,
        SourceRoot::new(&source).unwrap().whole_block(),
        SourceType::ts(),
    );
    let query = vize_l2::query::script::ScriptDeclarations::new(&original).unwrap();
    alloc::mark_installed();
    let mut count = 0;
    let observed = alloc::measure(|| {
        query.visit(|row| {
            std::hint::black_box((row.name, row.span));
            count += 1;
        })
    })
    .unwrap();
    assert_eq!(count, 3);
    assert_eq!(observed.calls, 0);
    assert_eq!(observed.peak_bytes_over_start, 0);
}
