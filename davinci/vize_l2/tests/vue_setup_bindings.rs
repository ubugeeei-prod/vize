//! Native `<script setup>` binding kinds and `defineProps` destructures
//! against Croquis' setup bindings on the repository corpus (#6844). Croquis
//! is only this test's oracle.

mod support {
    pub mod vue_setup;
}

use support::vue_setup::{corpus, observe, slice, text};
use vize_l0::{String, cstr};
use vize_l2::framework::vue::{SetupBindingKind, SetupFacts};

fn croquis_kind(kind: vize_relief::BindingType) -> Option<SetupBindingKind> {
    use vize_relief::BindingType;
    Some(match kind {
        BindingType::SetupConst => SetupBindingKind::SetupConst,
        BindingType::SetupLet => SetupBindingKind::SetupLet,
        BindingType::SetupRef => SetupBindingKind::SetupRef,
        BindingType::SetupMaybeRef => SetupBindingKind::SetupMaybeRef,
        BindingType::SetupReactiveConst => SetupBindingKind::SetupReactiveConst,
        BindingType::LiteralConst => SetupBindingKind::LiteralConst,
        BindingType::Props => SetupBindingKind::Props,
        _ => return None,
    })
}

/// Croquis binds every `defineProps` destructure local, rest included, as a
/// prop; Vue binds a renamed local as an aliased prop and the rest object as
/// a reactive constant (`definePropsDestructure.ts`), and so does the
/// compiler's own props pass, whose kinds reach the template.
fn croquis_view(kind: SetupBindingKind) -> SetupBindingKind {
    match kind {
        SetupBindingKind::PropsAliased => SetupBindingKind::Props,
        other => other,
    }
}

/// Every natively classified binding has Croquis' kind, and in files with no
/// unclassified pattern Croquis has no other binding except props read
/// without a local, which belong to the props contract.
#[test]
fn every_corpus_setup_binding_matches_croquis() {
    let (root, files) = corpus();
    let (mut compared, mut bindings, mut patterns) = (0usize, 0usize, 0usize);
    let mut mismatches = Vec::new();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let Some((content, base, facts)) = observe(&source) else {
            continue;
        };
        let relative = file.strip_prefix(&root).unwrap_or(file).display();
        let facts = match facts {
            Ok(facts) => facts,
            Err(refusal) => {
                mismatches.push(cstr!("{relative}: {refusal:?}"));
                continue;
            }
        };
        compared += 1;
        let oracle = vize_croquis::script_parser::parse_script_setup(&content).bindings;
        let rest = facts.props_destructure().and_then(|props| props.rest);
        let mut native = Vec::new();
        for binding in facts.bindings() {
            let name = slice(&content, base, binding.name).unwrap_or_default();
            let expected = oracle.get(&name).and_then(croquis_kind);
            let kind = if Some(binding.name) == rest {
                SetupBindingKind::Props
            } else {
                croquis_view(binding.kind)
            };
            bindings += 1;
            if expected != Some(kind) {
                mismatches.push(cstr!(
                    "{relative}: {name} native {:?} croquis {:?}",
                    binding.kind,
                    oracle.get(&name)
                ));
            }
            native.push(name);
        }
        patterns += usize::from(!facts.unclassified().is_empty());
        if facts.unclassified().is_empty() {
            for (name, kind) in oracle.iter() {
                let setup = croquis_kind(kind).is_some_and(|kind| kind != SetupBindingKind::Props);
                if setup && !native.iter().any(|known| known == name) {
                    mismatches.push(cstr!("{relative}: {name} only croquis {kind:?}"));
                }
            }
        }
    }
    eprintln!(
        "compared {bindings} bindings in {compared} setup scripts ({patterns} with unclassified props)"
    );
    assert!(compared > 1000, "corpus shrank to {compared} setup scripts");
    assert_eq!(mismatches, Vec::<String>::new());
}

/// `(name, kind)` of each native binding, plus the unclassified patterns.
type Kinds = (Vec<(String, SetupBindingKind)>, Vec<String>);

fn kinds(script: &str) -> Option<Kinds> {
    let source = cstr!("<script setup lang=\"ts\">{script}</script>");
    let (content, base, facts) = observe(&source)?;
    let facts = facts.ok()?;
    let bindings = facts
        .bindings()
        .iter()
        .map(|binding| {
            let name = slice(&content, base, binding.name).unwrap_or_default();
            (name, binding.kind)
        })
        .collect();
    let patterns = facts
        .unclassified()
        .iter()
        .map(|span| slice(&content, base, *span).unwrap_or_default())
        .collect();
    Some((bindings, patterns))
}

#[test]
fn binding_kinds_follow_the_compiler_rules() {
    use SetupBindingKind::{
        LiteralConst, Props, SetupConst, SetupLet, SetupMaybeRef, SetupReactiveConst, SetupRef,
    };
    let script = "import { ref as r, type Ref } from 'vue'\nimport Child from './Child.vue'\n\
        import * as api from './api'\nimport type { T } from './t'\n\
        const props = withDefaults(defineProps<{ a?: number }>(), {})\nconst model = defineModel()\n\
        const emit = defineEmits()\nconst count = r(0)\nlet state = reactive({})\nconst c = ref\n\
        const d = c(1)\nconst n = -1\nconst s = `x`\nconst o = {}\nconst f = () => 1\n\
        const m = useThing()\nlet l = 1\nfunction g() {}\nclass K {}\nenum E { A }\n\
        declare enum D { B }\nconst { x, y: [z] } = useX()\nconst [v, mods] = defineModel()\n\
        let { w } = h\nconst { p } = defineProps<{ p: string }>()";
    let expected = [
        ("r", SetupMaybeRef),
        ("Child", SetupConst),
        ("api", SetupConst),
        ("props", SetupReactiveConst),
        ("model", SetupRef),
        ("emit", SetupConst),
        ("count", SetupRef),
        ("state", SetupReactiveConst),
        ("c", SetupMaybeRef),
        ("d", SetupRef),
        ("n", LiteralConst),
        ("s", LiteralConst),
        ("o", SetupConst),
        ("f", SetupConst),
        ("m", SetupMaybeRef),
        ("l", SetupLet),
        ("g", SetupConst),
        ("K", SetupConst),
        ("E", SetupConst),
        ("x", SetupMaybeRef),
        ("z", SetupMaybeRef),
        ("v", SetupMaybeRef),
        ("mods", SetupMaybeRef),
        ("w", SetupLet),
        ("p", Props),
    ]
    .map(|(name, kind)| (text(name), kind));
    assert_eq!(kinds(script), Some((expected.to_vec(), Vec::new())));
}

/// `(key, local, default)` of each destructured prop, sorted, plus the rest.
type Destructure = (Vec<(String, String, Option<String>)>, Option<String>);

fn native_destructure(content: &str, base: u32, facts: &SetupFacts) -> Option<Destructure> {
    let props = facts.props_destructure()?;
    let mut fields = props
        .fields
        .iter()
        .map(|field| {
            (
                slice(content, base, field.key).unwrap_or_default(),
                slice(content, base, field.local).unwrap_or_default(),
                field.default.and_then(|span| slice(content, base, span)),
            )
        })
        .collect::<Vec<_>>();
    fields.sort();
    let rest = props.rest.and_then(|span| slice(content, base, span));
    Some((fields, rest))
}

fn croquis_destructure(content: &str) -> Option<Destructure> {
    let parsed = vize_croquis::script_parser::parse_script_setup(content);
    let props = parsed.macros.props_destructure()?;
    let mut fields = props
        .bindings
        .iter()
        .map(|(key, binding)| {
            (
                String::from(key.as_str()),
                String::from(binding.local.as_str()),
                binding.default.as_deref().map(String::from),
            )
        })
        .collect::<Vec<_>>();
    fields.sort();
    let rest = props.rest_id.as_deref().map(String::from);
    Some((fields, rest))
}

/// Keys, locals, defaults and the rest equal Croquis' `props_destructure`.
#[test]
fn every_corpus_props_destructure_matches_croquis() {
    let (root, files) = corpus();
    let (mut compared, mut destructures) = (0usize, 0usize);
    let mut mismatches = Vec::new();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let Some((content, base, Ok(facts))) = observe(&source) else {
            continue;
        };
        compared += 1;
        let native = native_destructure(&content, base, &facts);
        destructures += usize::from(native.is_some());
        let oracle = croquis_destructure(&content);
        if native != oracle || !facts.unclassified().is_empty() {
            let relative = file.strip_prefix(&root).unwrap_or(file).display();
            mismatches.push(cstr!(
                "{relative}\n  native:  {native:?} {:?}\n  croquis: {oracle:?}",
                facts.unclassified()
            ));
        }
    }
    eprintln!("compared {compared} setup scripts ({destructures} props destructures)");
    assert!(
        destructures > 400,
        "corpus shrank to {destructures} destructures"
    );
    assert_eq!(mismatches, Vec::<String>::new());
}

fn destructure(script: &str) -> Option<(Destructure, Kinds)> {
    let source = cstr!("<script setup lang=\"ts\">{script}</script>");
    let (content, base, facts) = observe(&source)?;
    let facts = facts.ok()?;
    let fields = native_destructure(&content, base, &facts).unwrap_or_default();
    Some((fields, kinds(script)?))
}

#[test]
fn props_destructure_binds_locals_like_vue() {
    use SetupBindingKind::{Props, PropsAliased, SetupReactiveConst};
    let script = "const { a, b: c = 1, 'd-e': f, \"g\": g = () => 1, ...rest } =\n\
        withDefaults(defineProps<{ a: 1 }>(), {})";
    let fields = vec![
        (text("a"), text("a"), None),
        (text("b"), text("c"), Some(text("1"))),
        (text("d-e"), text("f"), None),
        (text("g"), text("g"), Some(text("() => 1"))),
    ];
    let bindings = [
        ("a", Props),
        ("c", PropsAliased),
        ("f", PropsAliased),
        ("g", Props),
        ("rest", SetupReactiveConst),
    ]
    .map(|(name, kind)| (text(name), kind));
    assert_eq!(
        destructure(script),
        Some((
            (fields, Some(text("rest"))),
            (bindings.to_vec(), Vec::new())
        ))
    );
}

/// Patterns Vue rejects, escaped keys and a second destructure stay
/// unclassified; a TypeScript-wrapped call is an ordinary destructure.
#[test]
fn unsupported_props_destructures_stay_unclassified() {
    for pattern in [
        "{ [k]: a }",
        "{ a: { b } }",
        "{ a: [b] }",
        "{ 'a\\x62': c }",
        "{ 1: d }",
    ] {
        let script = cstr!("const {pattern} = defineProps()");
        assert_eq!(
            destructure(&script),
            Some(((Vec::new(), None), (Vec::new(), vec![text(pattern)]))),
            "{pattern}"
        );
    }
    let twice = "const { a } = defineProps()\nconst { b } = defineProps()";
    assert_eq!(
        destructure(twice).map(|(_, kinds)| kinds),
        Some((
            vec![(text("a"), SetupBindingKind::Props)],
            vec![text("{ b }")]
        ))
    );
    assert_eq!(
        kinds("const { a } = defineProps() as any"),
        Some((
            vec![(text("a"), SetupBindingKind::SetupMaybeRef)],
            Vec::new()
        ))
    );
}
