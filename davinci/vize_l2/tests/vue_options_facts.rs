//! Native Options API facts of a normal `<script>`: the members the compiler
//! forwards as template bindings equal Croquis' Options API bindings on every
//! normal `<script>` in the repository corpus (#6844). Croquis is only this
//! test's oracle.

#[expect(dead_code, reason = "this test reads the corpus, not setup facts")]
mod support {
    pub mod vue_options;
    pub mod vue_setup;
}

use std::collections::BTreeMap;

use support::vue_options::observe;
use support::vue_setup::{corpus, slice, text};
use vize_l0::{String, cstr};
use vize_l2::framework::vue::{ComponentShape, OptionMemberKind, OptionsFacts};
use vize_relief::BindingType;

/// Files where Croquis disagrees with Vue, which is the authority. A
/// top-level declaration spelled like a member replaces the member's
/// binding in Croquis (it guards imports only), but a normal `<script>`
/// template reads the instance: Vue binds `filters`, `todos` and
/// `calendarTypeOptions` as `data` in the first two (`compileScript`,
/// compiler-sfc 3.5.41); Vue reads no `defineComponent` options, which
/// `NaiveUiOptionsApi.vue` and `PrimeVueOptionsApi.vue` use. The pinned
/// fixtures under `_git` show the same shape. They are hydrated only on some
/// lanes, so a path counts only when it is present.
const CROQUIS_DEFECTS: &[&str] = &[
    "crates/vize_patina/tests/fixtures/deprecated-diagnostic-spans/originals/src/views/dashboard/admin/components/TodoList/index.vue",
    "crates/vize_patina/tests/fixtures/deprecated-diagnostic-spans/originals/src/views/table/complex-table.vue",
    "tests/_fixtures/_git/bootstrap-vue/docs/components/importdoc.vue",
    "tests/_fixtures/_git/buefy/packages/buefy/src/components/colorpicker/ColorpickerHSLRepresentationTriangle.vue",
    "tests/_fixtures/_git/buefy/packages/docs/src/pages/components/image/examples/ExSimple.vue",
    "tests/_fixtures/_git/vue-element-admin/src/views/dashboard/admin/components/TodoList/index.vue",
    "tests/_fixtures/_git/vue-element-admin/src/views/dashboard/admin/index.vue",
    "tests/_fixtures/_git/vue-element-admin/src/views/profile/components/Activity.vue",
    "tests/_fixtures/_git/vue-element-admin/src/views/table/complex-table.vue",
    "tests/_fixtures/_projects/ecosystem-products/src/NaiveUiOptionsApi.vue",
    "tests/_fixtures/_projects/ecosystem-products/src/PrimeVueOptionsApi.vue",
];

/// Croquis' template-binding names: identifiers, and kebab-case names
/// whose camel case is one.
fn template_name(name: &str) -> Option<String> {
    let valid = |name: &str| {
        let mut chars = name.chars();
        chars
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic() || first == '_' || first == '$')
            && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
    };
    if valid(name) {
        return Some(String::from(name));
    }
    let mut camel = String::from("");
    let mut upper = false;
    for ch in name.chars() {
        if ch == '-' {
            upper = true;
        } else if upper {
            camel.push(ch.to_ascii_uppercase());
            upper = false;
        } else {
            camel.push(ch);
        }
    }
    (name.contains('-') && valid(&camel)).then_some(camel)
}

/// The template bindings the compiler forwards, one per name, last wins.
fn native_bindings(
    content: &str,
    base: u32,
    facts: &OptionsFacts,
) -> BTreeMap<String, BindingType> {
    let mut bindings = BTreeMap::new();
    let Some(component) = facts.component() else {
        return bindings;
    };
    for member in &component.members {
        let binding = match member.kind {
            OptionMemberKind::Prop => BindingType::Props,
            OptionMemberKind::Computed | OptionMemberKind::Method | OptionMemberKind::Inject => {
                BindingType::Options
            }
            OptionMemberKind::Data | OptionMemberKind::AsyncData => BindingType::Data,
            OptionMemberKind::Setup => BindingType::SetupMaybeRef,
            OptionMemberKind::Filter => continue,
        };
        let name = slice(content, base, member.name).and_then(|name| template_name(&name));
        if let Some(name) = name {
            bindings.insert(name, binding);
        }
    }
    bindings.retain(|_, binding| forwarded(*binding));
    bindings
}

fn forwarded(binding: BindingType) -> bool {
    matches!(
        binding,
        BindingType::Data | BindingType::Options | BindingType::Props | BindingType::PropsAliased
    )
}

fn croquis_bindings(content: &str) -> BTreeMap<String, BindingType> {
    let parsed = vize_croquis::script_parser::parse_script_with_options_and_jsx(
        content,
        vize_croquis::script_parser::ScriptParserOptions {
            options_api: true,
            legacy_vue2: false,
        },
        false,
    );
    parsed
        .bindings
        .iter()
        .filter(|(_, binding)| forwarded(*binding))
        .map(|(name, binding)| (String::from(name), binding))
        .collect()
}

#[test]
fn every_corpus_options_component_matches_croquis() {
    let (root, files) = corpus();
    let (mut compared, mut components, mut classes, mut members) = (0usize, 0usize, 0usize, 0usize);
    let mut mismatches = Vec::new();
    for file in &files {
        let Ok(source) = std::fs::read_to_string(file) else {
            continue;
        };
        let Some((content, base, facts)) = observe(&source) else {
            continue;
        };
        compared += 1;
        let native = facts.map(|facts| {
            if let Some(component) = facts.component() {
                components += 1;
                classes += usize::from(component.shape == ComponentShape::Class);
                members += component.members.len();
                assert!(component.escaped.is_empty(), "{}", file.display());
            }
            native_bindings(&content, base, &facts)
        });
        let oracle = Ok(croquis_bindings(&content));
        if native != oracle {
            let relative = file.strip_prefix(&root).unwrap_or(file).display();
            mismatches.push(cstr!(
                "{relative}\n  native:  {native:?}\n  croquis: {oracle:?}"
            ));
        }
    }
    eprintln!(
        "compared {compared} normal scripts ({components} components, {classes} classes, {members} members)"
    );
    assert!(
        compared >= 100,
        "corpus shrank to {compared} normal scripts"
    );
    let defects = mismatches
        .iter()
        .filter_map(|mismatch| mismatch.split_once('\n').map(|(path, _)| path))
        .collect::<Vec<_>>();
    let expected = CROQUIS_DEFECTS
        .iter()
        .copied()
        .filter(|path| root.join(path).is_file())
        .collect::<Vec<_>>();
    assert_eq!(defects, expected, "{mismatches:#?}");
}

/// `(kind, name, owner text)` of each member, and the escaped keys' text.
type Shape = (Vec<(OptionMemberKind, String, String)>, Vec<String>);

fn shape(script: &str) -> Option<(ComponentShape, Shape)> {
    let source = cstr!("<script lang=\"ts\">{script}</script>");
    let (content, base, facts) = observe(&source)?;
    let component = facts.ok()?.component()?.clone();
    let text = |span| slice(&content, base, span).unwrap_or_default();
    let members = component
        .members
        .iter()
        .map(|member| {
            let owner = text(member.owner);
            let owner = owner.split_whitespace().collect::<Vec<_>>().join(" ");
            (member.kind, text(member.name), String::from(owner.as_str()))
        })
        .collect();
    let escaped = component.escaped.iter().map(|span| text(*span)).collect();
    Some((component.shape, (members, escaped)))
}

fn names(script: &str) -> Vec<(OptionMemberKind, String)> {
    shape(script)
        .map(|(_, (members, _))| {
            members
                .into_iter()
                .map(|(kind, name, _)| (kind, name))
                .collect()
        })
        .unwrap_or_default()
}

use OptionMemberKind::{AsyncData, Computed, Data, Filter, Inject, Method, Prop, Setup};

#[test]
fn groups_are_read_in_order_from_every_options_form() {
    let script = "export default {
  setup() { return { s } },
  asyncData: () => ({ n: 1 }),
  data() { return { d: 1 } },
  inject: ['i', 'my-key'],
  methods: { m() {}, 'q': 1, [x]: 2, ...rest },
  filters: { f() {} },
  computed: { c() { return 1 } },
  props: { p: String, 'kebab-prop': Number },
}";
    assert_eq!(
        names(script),
        [
            (Prop, text("p")),
            (Prop, text("kebab-prop")),
            (Computed, text("c")),
            (Filter, text("f")),
            (Method, text("m")),
            (Method, text("q")),
            (Inject, text("i")),
            (Inject, text("my-key")),
            (Data, text("d")),
            (AsyncData, text("n")),
            (Setup, text("s")),
        ]
    );
    for wrapped in [
        "export default defineComponent({ props: ['p'] })",
        "export default _defineComponent({ props: ['p'] })",
        "export default Vue.extend({ props: ['p'] })",
        "export default extend({ props: ['p'] })",
        "export default vue.defineComponent({ props: ['p'] }) as any",
        "const Base = { props: ['p'] }; export default Base",
        "export default C; const C = defineComponent(B); const B = { props: ['p'] }",
        "export default ({ props: ['p'] } satisfies object)",
        "export default defineComponent(({ props: (['p'] as const) }))",
    ] {
        assert_eq!(names(wrapped), [(Prop, text("p"))], "{wrapped}");
    }
    for unread in [
        "export default other.extend({ props: ['p'] })",
        "export default factory({ props: ['p'] })",
        "export { C as default }; const C = { props: ['p'] }",
        "export default <any>{ props: ['p'] }",
    ] {
        assert_eq!(names(unread), [], "{unread}");
    }
}

#[test]
fn group_values_resolve_same_file_objects() {
    let script = "const methods = { m() {} };
const state = { d: 1 };
const props = { p: String };
export default {
  props,
  methods,
  computed: unknown,
  computed: { c: () => 1 },
  data() { if (x) return other; return state; },
  setup: () => state,
}";
    assert_eq!(
        names(script),
        [
            (Prop, text("p")),
            (Computed, text("c")),
            (Method, text("m")),
            (Data, text("d")),
            (Setup, text("d")),
        ]
    );
    // `props` and `data` read their first property only.
    assert_eq!(
        names("export default { props: make(), props: ['p'], data: missing, data: { d: 1 } }"),
        []
    );
    // A later declarator of a name replaces the object.
    assert_eq!(
        names("var m = { a() {} }; var m = { b() {} }; export default { methods: m }"),
        [(Method, text("b"))]
    );
}

#[test]
fn mixins_and_extends_follow_same_file_targets_once() {
    let script = "const A = { mixins: [B], data: () => ({ a: 1 }) };
const B = { mixins: [A], methods: { b() {} } };
const C = defineComponent({ props: ['c'] });
export default {
  props: ['own'],
  mixins: [A, { computed: { inline() {} } }, imported, C],
  extends: (B as any),
}";
    let (_, (members, _)) = shape(script).expect("component");
    let owners = members
        .iter()
        .map(|(kind, name, owner)| {
            (
                *kind,
                name.as_str(),
                owner.split(':').next().unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        owners,
        [
            (Prop, "own", "{ props"),
            (Data, "a", "{ mixins"),
            (Method, "b", "{ mixins"),
            (Computed, "inline", "{ computed"),
        ]
    );
}

#[test]
fn class_components_read_decorators_and_instance_members() {
    let script = "@Component({ props: ['dp'], components: { X } })
export default class Panel extends Vue {
  @Prop() readonly p!: string
  @PropSync('q') sq!: string
  @Inject() readonly i!: string
  @Emit() changed() {}
  title = ''
  declare skipped: string
  static count = 0
  #hidden = 1
  [computedKey] = 2
  get total() { return 1 }
  constructor() { super() }
  method() {}
  accessor a = 1
}";
    let (kind, (members, escaped)) = shape(script).expect("class component");
    assert_eq!(kind, ComponentShape::Class);
    assert!(escaped.is_empty());
    let members = members
        .into_iter()
        .map(|(kind, name, _)| (kind, name))
        .collect::<Vec<_>>();
    assert_eq!(
        members,
        [
            (Prop, text("dp")),
            (Prop, text("p")),
            (Prop, text("sq")),
            (Inject, text("i")),
            (Method, text("changed")),
            (Data, text("title")),
            (Computed, text("total")),
            (Method, text("method")),
            (Data, text("a")),
        ]
    );
    assert_eq!(
        shape("export default (class extends Vue { x = 1 } as any)").map(|(kind, _)| kind),
        Some(ComponentShape::Class)
    );
}

#[test]
fn escaped_keys_are_listed_apart() {
    let (_, (members, escaped)) =
        shape("export default { props: ['a\\u0062', 'c'], data: () => ({ 'd\\x65': 1 }) }")
            .expect("component");
    assert_eq!(
        members,
        [(
            Prop,
            text("c"),
            text("{ props: ['a\\u0062', 'c'], data: () => ({ 'd\\x65': 1 }) }")
        )]
    );
    assert_eq!(escaped, [text("'a\\u0062'"), text("'d\\x65'")]);
}

#[test]
fn scripts_without_a_component_have_none() {
    for script in [
        "export const x = 1",
        "export default function () {}",
        "export default 1",
        "const a = { props: ['p'] }",
    ] {
        assert_eq!(shape(script), None, "{script}");
    }
}
