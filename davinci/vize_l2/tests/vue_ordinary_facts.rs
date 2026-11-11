//! Native facts for the normal `<script>` half of #6844.

use vize_l0::{Allocator, SourceRoot, Span};
use vize_l1::embed::{
    EmbedSource, Lang,
    syntax::{ProgramOptions, parse_program_once},
};
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
