//! The emits, model, slots, expose and options contract a `<script setup>`
//! declares (#6844): Vue's reading rules case by case, and every corpus
//! setup compared with the event names and models Croquis records. Croquis
//! is only this test's oracle.
#![expect(clippy::expect_used, reason = "tests assert by panicking")]

mod support {
    pub mod vue_setup;
}

use support::vue_setup::{corpus, observe, slice, text};
use vize_l0::{Span, String, cstr};
use vize_l2::framework::vue::{ModelName, SetupFacts};

fn facts(script: &str) -> (String, u32, SetupFacts) {
    let source = cstr!("<script setup lang=\"ts\">{script}</script>");
    let (content, base, facts) = observe(&source).expect("the setup is admitted");
    (content, base, facts.expect("every statement is observed"))
}

/// `(name, declaration, payload)` of each event, and the unexpanded spans.
type Events = (Vec<(String, String, Option<String>)>, Vec<String>);

fn events(script: &str) -> Events {
    let (content, base, facts) = facts(script);
    let text_of = |span: Span| slice(&content, base, span).unwrap_or_default();
    let emits = facts.contract().emits.as_ref().expect("a defineEmits call");
    (
        emits
            .events
            .iter()
            .map(|emit| {
                (
                    text_of(emit.name),
                    text_of(emit.declaration),
                    emit.payload.map(text_of),
                )
            })
            .collect(),
        emits.unexpanded.iter().map(|span| text_of(*span)).collect(),
    )
}

fn event(name: &str, declaration: &str, payload: Option<&str>) -> (String, String, Option<String>) {
    (text(name), text(declaration), payload.map(text))
}

fn texts(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| text(value)).collect()
}

#[test]
fn runtime_emits_are_read_like_vue() {
    assert_eq!(
        events("defineEmits(['a', \"b\", c])"),
        (
            vec![event("a", "'a'", None), event("b", "\"b\"", None)],
            texts(&["c"])
        )
    );
    let (content, base, facts) =
        facts("const emit = defineEmits({ a: null, 'b-c': (x: number) => true, ...rest })");
    let emits = facts.contract().emits.as_ref().expect("emits");
    assert_eq!(
        emits
            .declarator
            .and_then(|span| slice(&content, base, span)),
        Some(text("emit"))
    );
    assert_eq!(
        events("defineEmits({ a: null, 'b-c': (x: number) => true, ...rest })"),
        (
            vec![
                event("a", "a: null", Some("null")),
                event(
                    "b-c",
                    "'b-c': (x: number) => true",
                    Some("(x: number) => true")
                ),
            ],
            texts(&["...rest"])
        )
    );
}

#[test]
fn type_events_come_from_members_and_call_signatures() {
    assert_eq!(
        events("defineEmits<{ change: [id: number]; 'update:modelValue': [value: string] }>()").0,
        vec![
            event("change", "change: [id: number];", Some("[id: number]")),
            event(
                "update:modelValue",
                "'update:modelValue': [value: string]",
                Some("[value: string]")
            ),
        ]
    );
    let (listed, unexpanded) = events(
        "defineEmits<{ (e: 'a' | 'b', id: number): void; (e: 'c'): void; (e: Name): void }>()",
    );
    let names: Vec<_> = listed
        .iter()
        .map(|(name, _, payload)| (name.clone(), payload.clone()))
        .collect();
    assert_eq!(
        names,
        vec![
            (text("a"), Some(text("id: number"))),
            (text("b"), Some(text("id: number"))),
            (text("c"), None),
        ]
    );
    assert_eq!(unexpanded, texts(&["e: Name"]));
    assert_eq!(
        events("defineEmits<(e: 'x', ...args: [number]) => void>()").0,
        vec![event(
            "x",
            "(e: 'x', ...args: [number]) => void",
            Some("...args: [number]")
        )]
    );
    assert_eq!(
        events("interface Events { (e: 'a'): void }\ndefineEmits<Events>()").0,
        vec![event("a", "(e: 'a'): void", None)]
    );
    assert_eq!(
        events("import type { E } from './e'\ndefineEmits<E>()"),
        (Vec::new(), texts(&["E"]))
    );
}

/// `(name, declarator, type argument, options, required, default)`.
type Model = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
    Option<String>,
);

fn models(script: &str) -> Vec<Model> {
    let (content, base, facts) = facts(script);
    let text_of = |span: Span| slice(&content, base, span).unwrap_or_default();
    facts
        .contract()
        .models
        .iter()
        .map(|model| {
            (
                match model.name {
                    ModelName::Default => None,
                    ModelName::Named(span) => Some(text_of(span)),
                    ModelName::Unread(span) => Some(cstr!("unread {}", text_of(span))),
                },
                model.declarator.map(text_of),
                model.type_argument.map(text_of),
                model.options.map(text_of),
                model.required,
                model.default.map(text_of),
            )
        })
        .collect()
}

#[test]
fn models_are_read_like_vue() {
    let some = |value: &str| Some(text(value));
    assert_eq!(
        models(
            "const model = defineModel<string>()\n\
             const [count, modifiers] = defineModel('count', { required: true, default: 0 })\n\
             defineModel({ default: () => [] })\n\
             defineModel('a' as const, options)\n\
             defineModel('\\u0062')"
        ),
        vec![
            (None, some("model"), some("string"), None, false, None),
            (
                some("count"),
                some("[count, modifiers]"),
                None,
                some("{ required: true, default: 0 }"),
                true,
                some("0")
            ),
            (
                None,
                None,
                None,
                some("{ default: () => [] }"),
                false,
                some("() => []")
            ),
            (some("a"), None, None, some("options"), false, None),
            (some("unread '\\u0062'"), None, None, None, false, None),
        ]
    );
}

#[test]
fn slots_expose_and_options_keep_their_calls() {
    let (content, base, facts) = facts(
        "const slots = defineSlots<{ default(): any }>()\n\
         defineExpose({ a })\n\
         defineOptions({ name: 'X' })\n\
         defineExpose()",
    );
    let text_of = |span: Span| slice(&content, base, span).unwrap_or_default();
    let contract = facts.contract();
    let slots = contract.slots.expect("slots");
    assert_eq!(
        slots.type_argument.map(text_of),
        Some(text("{ default(): any }"))
    );
    assert_eq!(slots.declarator.map(text_of), Some(text("slots")));
    assert_eq!(
        contract
            .expose
            .and_then(|expose| expose.argument)
            .map(text_of),
        Some(text("{ a }"))
    );
    assert_eq!(
        contract
            .options
            .and_then(|options| options.argument)
            .map(text_of),
        Some(text("{ name: 'X' }"))
    );
    let duplicates: Vec<_> = contract
        .duplicates
        .iter()
        .map(|span| text_of(*span))
        .collect();
    assert_eq!(duplicates, texts(&["defineExpose()"]));
}

/// Croquis disagrees with Vue, the authority, on these files: it records no
/// event of a same-file type alias or interface (`InterfaceChild`, both
/// `DefineEmitsMethodInterface`), of a function type (`#4387`) or of a
/// union of event literals (`union_type`).
const CROQUIS_DIFFERENCES: &[&str] = &[
    "crates/vize/tests/fixtures/component_api_hover/src/InterfaceChild.vue",
    "tests/_fixtures/_projects/compiler-macros/src/DefineEmitsMethodInterface.vue",
    "tests/_fixtures/_projects/typecheck-errors/src/DefineEmitsMethodInterfaceError.vue",
    "tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#4387/component.vue",
    "tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/events/union_type.vue",
];

/// Vue normalizes a model name like every prop name; Croquis records it so.
fn camelize(name: &str) -> String {
    let mut camel = String::default();
    let mut upper = false;
    for character in name.chars() {
        if character == '-' {
            upper = true;
        } else if upper {
            camel.extend(character.to_uppercase());
            upper = false;
        } else {
            camel.push(character);
        }
    }
    camel
}

/// A complete emits contract names exactly Croquis' events, an incomplete
/// one a subset; models agree on names and required flags.
#[test]
fn every_corpus_emits_and_model_contract_matches_croquis() {
    let (root, files) = corpus();
    let (mut emits, mut events, mut models) = (0usize, 0usize, 0usize);
    let mut mismatches = Vec::new();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let Some((content, base, Ok(facts))) = observe(&source) else {
            continue;
        };
        let text_of = |span: Span| slice(&content, base, span).unwrap_or_default();
        let parsed = vize_croquis::script_parser::parse_script_setup(&content);
        let contract = facts.contract();
        let mut agrees = true;
        if let Some(declared) = &contract.emits {
            emits += 1;
            let mut native: Vec<String> = declared
                .events
                .iter()
                .map(|emit| text_of(emit.name))
                .collect();
            let mut oracle: Vec<String> = parsed
                .macros
                .emits()
                .iter()
                .map(|emit| String::from(emit.name.as_str()))
                .collect();
            native.sort();
            native.dedup();
            oracle.sort();
            oracle.dedup();
            events += native.len();
            agrees &= if declared.is_complete() {
                native == oracle
            } else {
                native.iter().all(|name| oracle.contains(name))
            };
        }
        let mut native: Vec<(String, bool)> = contract
            .models
            .iter()
            .map(|model| {
                let name = match model.name {
                    ModelName::Default => text("modelValue"),
                    ModelName::Named(span) | ModelName::Unread(span) => camelize(&text_of(span)),
                };
                (name, model.required)
            })
            .collect();
        let mut oracle: Vec<(String, bool)> = parsed
            .macros
            .models()
            .iter()
            .map(|model| (String::from(model.name.as_str()), model.required))
            .collect();
        native.sort();
        oracle.sort();
        models += native.len();
        agrees &= native == oracle;
        if !agrees {
            let relative = file.strip_prefix(&root).unwrap_or(file).display();
            let declared = contract.emits.as_ref().map(|declared| {
                (
                    declared
                        .events
                        .iter()
                        .map(|emit| text_of(emit.name))
                        .collect::<Vec<_>>(),
                    declared
                        .unexpanded
                        .iter()
                        .map(|span| text_of(*span))
                        .collect::<Vec<_>>(),
                )
            });
            let croquis: Vec<_> = parsed
                .macros
                .emits()
                .iter()
                .map(|emit| emit.name.clone())
                .collect();
            mismatches.push(cstr!(
                "{relative}\n  emits: {declared:?}\n  croquis emits: {croquis:?}\n  models: {native:?}\n  croquis models: {oracle:?}"
            ));
        }
    }
    eprintln!("compared {emits} emits contracts ({events} events) and {models} models");
    assert!(emits > 200, "corpus shrank to {emits} emits contracts");
    let differences = mismatches
        .iter()
        .filter_map(|mismatch| mismatch.split_once('\n').map(|(path, _)| path))
        .collect::<Vec<_>>();
    assert_eq!(differences, CROQUIS_DIFFERENCES, "{mismatches:#?}");
}
