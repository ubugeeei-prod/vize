//! Native facts for the normal `<script>` half of #6844.

#[expect(dead_code, reason = "this test reads the corpus, not setup facts")]
mod support {
    pub mod vue_setup;
}

use support::vue_setup::corpus;
use vize_l0::config::{VueDialect, VueVersion};
use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_l1::embed::{
    EmbedSource, Lang,
    syntax::{ProgramOptions, parse_program_once},
};
use vize_l1::markup::SurfaceParseOptions;
use vize_l2::framework::vue::{
    OrdinaryFacts, OrdinaryFactsIncomplete, OrdinaryObserver, ReactiveSource, SetupBindingKind,
};
use vize_l2::lang::js::{FileProducer, ProgramInput, ProgramScope};

fn observe(source: &str) -> Option<OrdinaryFacts> {
    let arena = Allocator::default();
    let span = Span::new(0, u32::try_from(source.len()).ok()?);
    let syntax = parse_program_once(
        &arena,
        EmbedSource::authored(source, span).ok()?,
        ProgramOptions::module(Lang::Ts),
    );
    let input = ProgramInput::checked(
        syntax.admitted_program()?,
        SourceRoot::new(source).ok()?.whole_block(),
        0,
    )
    .ok()?;
    let mut producer = FileProducer::new(&arena, source).ok()?;
    let mut observer = OrdinaryObserver::new();
    producer
        .program_observed(input, ProgramScope::Module, &mut observer)
        .ok()?;
    observer.finish().ok()
}

#[test]
fn normal_runtime_bindings_follow_vue_setup_access_kinds() {
    let source = r#"
import { ref as makeRef, reactive } from "vue";
import store from "./store";
import Child from "./Child.vue";
import * as namespace from "./namespace";
import type TypeOnly from "./types";
import { type NamedType } from "./types";

const literal = 1;
const maybe = store;
const refValue = makeRef(0);
const reactiveValue = reactive({});
let mutable = 0;
const { item, ...rest } = store;
const ordinaryCall = defineProps(["not-a-macro-here"]);
function helper() {}
class Model {}
enum StaticEnum { A }
export const exported = 2;
export function exportedFunction() {}
export class ExportedClass {}
export enum ExportedEnum { A }
"#;
    let Some(facts) = observe(source) else {
        panic!("ordinary facts");
    };
    let rows = facts
        .bindings()
        .iter()
        .map(|binding| {
            source
                .get(binding.name.start as usize..binding.name.end as usize)
                .map(|name| (name, binding.kind, binding.source))
        })
        .collect::<Option<Vec<_>>>();
    let Some(rows) = rows else {
        panic!("binding span");
    };

    assert_eq!(
        rows,
        vec![
            ("makeRef", SetupBindingKind::SetupConst, None),
            ("reactive", SetupBindingKind::SetupConst, None),
            ("store", SetupBindingKind::SetupMaybeRef, None),
            ("Child", SetupBindingKind::SetupConst, None),
            ("namespace", SetupBindingKind::SetupConst, None),
            ("literal", SetupBindingKind::LiteralConst, None),
            ("maybe", SetupBindingKind::SetupMaybeRef, None),
            (
                "refValue",
                SetupBindingKind::SetupRef,
                Some(ReactiveSource::Ref),
            ),
            (
                "reactiveValue",
                SetupBindingKind::SetupReactiveConst,
                Some(ReactiveSource::Reactive),
            ),
            ("mutable", SetupBindingKind::SetupLet, None),
            ("item", SetupBindingKind::SetupMaybeRef, None),
            ("rest", SetupBindingKind::SetupConst, None),
            ("ordinaryCall", SetupBindingKind::SetupMaybeRef, None),
            ("helper", SetupBindingKind::SetupConst, None),
            ("Model", SetupBindingKind::SetupConst, None),
            ("StaticEnum", SetupBindingKind::LiteralConst, None),
            ("exported", SetupBindingKind::LiteralConst, None),
            ("exportedFunction", SetupBindingKind::SetupConst, None),
            ("ExportedClass", SetupBindingKind::SetupConst, None),
            ("ExportedEnum", SetupBindingKind::LiteralConst, None),
        ]
    );
}

#[test]
fn unfinished_observer_claims_no_program() {
    assert_eq!(
        OrdinaryObserver::new().finish(),
        Err(OrdinaryFactsIncomplete::NoProgram)
    );
}

/// The facts of a corpus file's normal `<script>` as the walk observer and the
/// statement-only read produce them, or `None` when the levels refuse it first.
fn both_routes(
    source: &str,
) -> Option<(
    Result<OrdinaryFacts, OrdinaryFactsIncomplete>,
    Result<OrdinaryFacts, OrdinaryFactsIncomplete>,
)> {
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let view = descriptor.admitted().ok()?;
    let script = view.ordinary()?;
    let block = script.block();
    let parse = || {
        Some(parse_program_once(
            &arena,
            EmbedSource::authored(source, block.span()).ok()?,
            ProgramOptions::module(script.lang()),
        ))
    };
    let (first, second) = (parse()?, parse()?);
    let input = |syntax| ProgramInput::checked(syntax, block, script.container_index()).ok();
    let direct = OrdinaryFacts::of_module(&input(first.admitted_program()?)?);
    let mut producer = FileProducer::new(&arena, descriptor.source()).ok()?;
    let mut observer = OrdinaryObserver::new();
    producer
        .program_observed(
            input(second.admitted_program()?)?,
            ProgramScope::Module,
            &mut observer,
        )
        .ok()?;
    Some((observer.finish(), direct))
}

#[test]
fn statement_only_read_equals_the_walk_on_every_corpus_normal_script() {
    let (root, files) = corpus();
    let (mut compared, mut bindings) = (0usize, 0usize);
    let mut mismatches = Vec::new();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let Some((walked, direct)) = both_routes(&source) else {
            continue;
        };
        compared += 1;
        bindings += walked.as_ref().map_or(0, |facts| facts.bindings().len());
        if walked != direct {
            let relative = file.strip_prefix(&root).unwrap_or(file).display();
            mismatches.push(format!(
                "{relative}\n  walk:   {walked:?}\n  direct: {direct:?}"
            ));
        }
    }
    eprintln!("compared {compared} normal scripts ({bindings} bindings)");
    assert!(
        compared >= 100,
        "corpus shrank to {compared} normal scripts"
    );
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
