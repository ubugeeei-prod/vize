//! The props contract a `<script setup>` declares (#6844): Vue's reading
//! rules case by case, and every corpus setup compared with the prop names
//! and required flags Croquis records. Croquis is only this test's oracle.
#![expect(clippy::expect_used, reason = "tests assert by panicking")]

mod support {
    pub mod vue_setup;
}

use support::vue_setup::{corpus, observe, slice, text};
use vize_l0::{String, cstr};
use vize_l2::framework::vue::{ContractSource, SetupFacts};

/// `(name, required, value, default)` of one prop.
type Row = (String, bool, Option<String>, Option<String>);

#[derive(Debug, PartialEq)]
struct Read {
    source: Option<String>,
    rows: Vec<Row>,
    unexpanded: Vec<String>,
}

fn facts(script: &str) -> (String, u32, SetupFacts) {
    let source = cstr!("<script setup lang=\"ts\">{script}</script>");
    let (content, base, facts) = observe(&source).expect("the setup is admitted");
    (content, base, facts.expect("every statement is observed"))
}

fn read(script: &str) -> Read {
    let (content, base, facts) = facts(script);
    let text_of = |span| slice(&content, base, span).unwrap_or_default();
    let props = facts.contract().props.as_ref().expect("a defineProps call");
    Read {
        source: props.source.map(|source| match source {
            ContractSource::Type(span) => cstr!("type {}", text_of(span)),
            ContractSource::Runtime(span) => cstr!("runtime {}", text_of(span)),
        }),
        rows: props
            .props
            .iter()
            .map(|prop| {
                (
                    text_of(prop.name),
                    prop.required,
                    prop.value.map(text_of),
                    prop.default.map(text_of),
                )
            })
            .collect(),
        unexpanded: props.unexpanded.iter().map(|span| text_of(*span)).collect(),
    }
}

fn row(name: &str, required: bool, value: Option<&str>, default: Option<&str>) -> Row {
    (text(name), required, value.map(text), default.map(text))
}

fn texts(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| text(value)).collect()
}

#[test]
fn runtime_declarations_are_read_like_vue() {
    let array = read("defineProps(['a', \"b\", c, `d`])");
    assert_eq!(array.source, Some(text("runtime ['a', \"b\", c, `d`]")));
    assert_eq!(
        array.rows,
        vec![row("a", false, None, None), row("b", false, None, None)]
    );
    assert_eq!(array.unexpanded, texts(&["c", "`d`"]));

    let object = read(
        "defineProps({ a: String, /** doc */ 'b-c': { type: Number, required: true, default: 1 },\n\
         ['d']: [String, Number], [e]: Object, ...base, f: { required: flag } })",
    );
    assert_eq!(
        object.rows,
        vec![
            row("a", false, Some("String"), None),
            row(
                "b-c",
                true,
                Some("{ type: Number, required: true, default: 1 }"),
                Some("1")
            ),
            row("d", false, Some("[String, Number]"), None),
            row("f", false, Some("{ required: flag }"), None),
        ]
    );
    assert_eq!(object.unexpanded, texts(&["[e]: Object", "...base"]));

    // Vue reads the argument as written: a wrapper or a reference lists nothing.
    assert_eq!(
        read("defineProps({ g: String } as any)").unexpanded,
        texts(&["{ g: String } as any"])
    );
    assert_eq!(read("defineProps(options)").unexpanded, texts(&["options"]));
    assert_eq!(
        read("defineProps()"),
        Read {
            source: None,
            rows: Vec::new(),
            unexpanded: Vec::new()
        }
    );
}

#[test]
fn type_members_are_props_with_their_annotations() {
    let read = read(
        "defineProps<{ a: string; readonly b?: number; 'c-d': boolean; e(): void; [f]: string;\n\
         (x: 1): void; [key: string]: unknown }>()",
    );
    assert_eq!(
        read.source,
        Some(text(
            "type { a: string; readonly b?: number; 'c-d': boolean; e(): void; [f]: string;\n\
             (x: 1): void; [key: string]: unknown }"
        ))
    );
    assert_eq!(
        read.rows,
        vec![
            row("a", true, Some("string"), None),
            row("b", false, Some("number"), None),
            row("c-d", true, Some("boolean"), None),
            row("e", true, None, None),
        ]
    );
    assert_eq!(read.unexpanded, texts(&["[f]: string;"]));
}

#[test]
fn same_file_interfaces_and_aliases_are_expanded() {
    // Own members win over inherited ones, and come first.
    assert_eq!(
        read(
            "defineProps<Props>()\n\
             interface Base { x?: string; a: number }\n\
             interface Props extends Base { a?: string }"
        )
        .rows,
        vec![
            row("a", false, Some("string"), None),
            row("x", false, Some("string"), None),
        ]
    );
    // Across an intersection the first member stays, optional when any is.
    let utilities = read(
        "type A = { a: string; b: number }\n\
         defineProps<Partial<A> & Pick<A, 'b'> & Omit<A, 'b'> & ({ c: boolean })>()",
    );
    assert_eq!(
        utilities.rows,
        vec![
            row("a", false, Some("string"), None),
            row("b", false, Some("number"), None),
            row("c", true, Some("boolean"), None),
        ]
    );
    assert!(utilities.unexpanded.is_empty(), "{utilities:?}");
    // A declared name wins over the utility type of that name.
    assert_eq!(
        read("type Partial<T> = T\ndefineProps<Partial<{ a: string }>>()").unexpanded,
        texts(&["Partial<{ a: string }>"])
    );
}

#[test]
fn types_this_file_does_not_declare_stay_unexpanded() {
    let read = read(
        "import type { P } from './p'\n\
         interface G<T> { a: T }\n\
         type Q = string\n\
         defineProps<P & G<string> & Q & Partial<P> & Pick<{ c: 1 }, K> & { b: string }>()",
    );
    assert_eq!(read.rows, vec![row("b", true, Some("string"), None)]);
    assert_eq!(
        read.unexpanded,
        texts(&["P", "G<string>", "string", "P", "Pick<{ c: 1 }, K>"])
    );
}

#[test]
fn reference_cycles_stop_at_the_repeated_name() {
    let read = read(
        "interface A extends B { a: string }\n\
         interface B extends A { b: string }\n\
         defineProps<A>()",
    );
    assert_eq!(
        read.rows,
        vec![
            row("a", true, Some("string"), None),
            row("b", true, Some("string"), None),
        ]
    );
    assert_eq!(read.unexpanded, texts(&["A"]));
}

#[test]
fn defaults_come_from_options_with_defaults_and_destructures() {
    let (content, base, facts) = facts(
        "withDefaults(defineProps<{ a?: number; b?: string[]; c?: string }>(), { a: 1, b: () => [], ...rest })",
    );
    let props = facts.contract().props.as_ref().expect("props");
    assert_eq!(
        slice(&content, base, props.declaration),
        Some(text(
            "defineProps<{ a?: number; b?: string[]; c?: string }>()"
        ))
    );
    assert_eq!(
        props.defaults.and_then(|span| slice(&content, base, span)),
        Some(text("{ a: 1, b: () => [], ...rest }"))
    );
    let defaults: Vec<_> = props
        .props
        .iter()
        .map(|prop| prop.default.and_then(|span| slice(&content, base, span)))
        .collect();
    assert_eq!(
        defaults,
        vec![Some(text("1")), Some(text("() => []")), None]
    );

    assert_eq!(
        read("const { a = 1, b: c = 'x', d } = defineProps<{ a?: number; b?: string; d: 1 }>()")
            .rows,
        vec![
            row("a", false, Some("number"), Some("1")),
            row("b", false, Some("string"), Some("'x'")),
            row("d", true, Some("1"), None),
        ]
    );
    // A runtime default is the declared one.
    assert_eq!(
        read("const { a = 2 } = defineProps({ a: { type: Number, default: 1 } })")
            .rows
            .first()
            .and_then(|row| row.3.clone()),
        Some(text("1"))
    );
}

#[test]
fn rejected_declarations_are_reported() {
    let (content, base, facts) = facts(
        "defineProps<{ a: string }>({ b: String })\n\
         const props = defineProps(['c', '\\u0064'])",
    );
    let slices = |spans: &[vize_l0::Span]| -> Vec<String> {
        spans
            .iter()
            .map(|span| slice(&content, base, *span).unwrap_or_default())
            .collect()
    };
    let contract = facts.contract();
    let props = contract.props.as_ref().expect("props");
    assert_eq!(props.props.len(), 1);
    assert_eq!(slices(&props.unexpanded), texts(&["{ b: String }"]));
    assert_eq!(
        slices(&contract.duplicates),
        texts(&["defineProps(['c', '\\u0064'])"])
    );
    // The compile route still sees every call's runtime keys and refuses
    // the escaped one.
    assert_eq!(slices(facts.runtime_prop_keys()), texts(&["b", "c"]));
    assert_eq!(slices(facts.unclassified()), texts(&["'\\u0064'"]));
}

/// Croquis disagrees with Vue, the authority, on these files. Croquis lists
/// no member of an intersection that names a type it cannot read
/// (`defineProps<Imported & { readonly as?: T }>()`); Vue declares `as`.
/// ElementPlus' cascader menu intersects `{ nodes; index }` with
/// `Pick<CascaderCommonProps, …>` the same way. Pinned fixtures are hydrated
/// only on some CI lanes, so a path counts only when it is present.
const CROQUIS_DIFFERENCES: &[&str] = &[
    "npm/ui/src/families/navigation/pagination/pagination.vue",
    "tests/_fixtures/_git/element-plus/packages/components/cascader-panel/src/menu.vue",
];

/// Sorted `(name, required)` pairs.
fn pairs(mut pairs: Vec<(String, bool)>) -> Vec<(String, bool)> {
    pairs.sort();
    pairs
}

/// A complete contract lists exactly Croquis' props; an incomplete one lists
/// a subset of them.
#[test]
fn every_corpus_props_contract_matches_croquis() {
    let (root, files) = corpus();
    let (mut compared, mut complete, mut props) = (0usize, 0usize, 0usize);
    let mut mismatches = Vec::new();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let Some((content, base, Ok(facts))) = observe(&source) else {
            continue;
        };
        let Some(contract) = facts.contract().props.as_ref() else {
            continue;
        };
        compared += 1;
        let native = pairs(
            contract
                .props
                .iter()
                .map(|prop| {
                    let name = slice(&content, base, prop.name).unwrap_or_default();
                    (name, prop.required)
                })
                .collect(),
        );
        let parsed = vize_croquis::script_parser::parse_script_setup(&content);
        let oracle = pairs(
            parsed
                .macros
                .props()
                .iter()
                .map(|prop| (String::from(prop.name.as_str()), prop.required))
                .collect(),
        );
        props += native.len();
        let agrees = if contract.is_complete() {
            complete += 1;
            native == oracle
        } else {
            native.iter().all(|pair| oracle.contains(pair))
        };
        if !agrees {
            let relative = file.strip_prefix(&root).unwrap_or(file).display();
            let unexpanded: Vec<String> = contract
                .unexpanded
                .iter()
                .map(|span| slice(&content, base, *span).unwrap_or_default())
                .collect();
            mismatches.push(cstr!(
                "{relative}\n  native:  {native:?}\n  croquis: {oracle:?}\n  unexpanded: {unexpanded:?}"
            ));
        }
    }
    eprintln!("compared {compared} props contracts ({complete} complete, {props} props)");
    assert!(
        compared > 300,
        "corpus shrank to {compared} props contracts"
    );
    let differences = mismatches
        .iter()
        .filter_map(|mismatch| mismatch.split_once('\n').map(|(path, _)| path))
        .collect::<Vec<_>>();
    let expected = CROQUIS_DIFFERENCES
        .iter()
        .copied()
        .filter(|path| root.join(path).is_file())
        .collect::<Vec<_>>();
    assert_eq!(differences, expected, "{mismatches:#?}");
}
