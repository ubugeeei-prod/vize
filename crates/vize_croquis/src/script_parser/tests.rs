use super::{ScriptParserOptions, parse_script, parse_script_setup, parse_script_with_options};
use crate::croquis::ComponentShape;
use vize_carton::{CompactString, append, cstr};
use vize_relief::BindingType;

#[test]
fn test_parse_define_props_type() {
    let result = parse_script_setup(
        r#"
            const props = defineProps<{
                msg: string
                count?: number
            }>()
        "#,
    );

    assert_eq!(result.macros.all_calls().len(), 1);
    assert_eq!(result.macros.props().len(), 2);

    let prop_names: Vec<_> = result
        .macros
        .props()
        .iter()
        .map(|p| p.name.as_str())
        .collect();
    assert!(prop_names.contains(&"msg"));
    assert!(prop_names.contains(&"count"));
}

#[test]
fn test_parse_define_props_runtime() {
    let result = parse_script_setup(
        r#"
            const props = defineProps(['foo', 'bar'])
        "#,
    );

    assert_eq!(result.macros.props().len(), 2);
}

#[test]
fn test_parse_define_props_runtime_object_spread_local_literal() {
    let result = parse_script_setup(
        r#"
            const common = {
                bar: String,
                count: { type: Number, required: true, default: 1 },
            } as const
            const props = defineProps({ ...common, foo: Boolean })
        "#,
    );

    let props = result.macros.props();
    assert_eq!(props.len(), 3);
    assert!(props.iter().any(|prop| prop.name == "bar"));
    assert!(props.iter().any(|prop| prop.name == "foo"));

    let count = props
        .iter()
        .find(|prop| prop.name == "count")
        .expect("spread prop should be extracted");
    assert!(count.required);
    assert_eq!(count.prop_type.as_deref(), Some("number"));
    assert_eq!(count.default_value.as_deref(), Some("1"));

    assert!(result.bindings.contains("bar"));
    assert!(result.bindings.contains("count"));
}

#[test]
fn test_parse_define_emits() {
    let result = parse_script_setup(
        r#"
            const emit = defineEmits(['update', 'delete'])
        "#,
    );

    assert_eq!(result.macros.all_calls().len(), 1);
    assert_eq!(result.macros.emits().len(), 2);
}

#[test]
fn test_parse_define_emits_runtime_args_with_spread() {
    let result = parse_script_setup(
        r#"
            const emit = defineEmits({
                ...emitObject,
            })
            defineEmits([...dialogEmits])
        "#,
    );

    let calls = result.macros.all_calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(
        calls[0].runtime_args.as_deref(),
        Some("{\n                ...emitObject,\n            }")
    );
    assert_eq!(calls[1].runtime_args.as_deref(), Some("[...dialogEmits]"));
}

#[test]
fn test_parse_define_art() {
    let result = parse_script_setup(
        r#"
import Button from "./Button.vue";

defineArt(Button, {
  title: "Button",
  description: "A button component",
  category: "Components",
  tags: ["button", "ui"],
  status: "draft",
  order: 2,
});
"#,
    );

    let art = result.macros.define_art().expect("defineArt metadata");
    assert_eq!(art.component_name.as_str(), "Button");
    assert_eq!(art.component_source.as_deref(), Some("./Button.vue"));
    assert_eq!(art.title.as_deref(), Some("Button"));
    assert_eq!(art.description.as_deref(), Some("A button component"));
    assert_eq!(art.category.as_deref(), Some("Components"));
    assert_eq!(
        art.tags.iter().map(|tag| tag.as_str()).collect::<Vec<_>>(),
        ["button", "ui"]
    );
    assert_eq!(art.status.as_deref(), Some("draft"));
    assert_eq!(art.order, Some(2));
    assert!(result.macros.define_art_call().is_some());
}

#[test]
fn test_parse_define_art_with_source_literal() {
    let result = parse_script_setup(
        r#"
defineArt("./forms/base-button.vue", {
  title: "Base Button",
});
"#,
    );

    let art = result.macros.define_art().expect("defineArt metadata");
    assert_eq!(art.component_name.as_str(), "BaseButton");
    assert_eq!(
        art.component_source.as_deref(),
        Some("./forms/base-button.vue")
    );
    assert!(art.component_source_span.is_some());
    assert!(art.component_source_value_span.is_some());
    assert_eq!(art.title.as_deref(), Some("Base Button"));
}

#[test]
fn test_parse_define_slots() {
    let result = parse_script_setup(
        r#"
defineSlots<{
  default(props: { user: User }): any
  icon: (props: { size: number }) => any
}>()
"#,
    );

    let slots = result.macros.slots();
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0].name.as_str(), "default");
    assert_eq!(slots[0].props_type.as_deref(), Some("{ user: User }"));
    assert_eq!(slots[1].name.as_str(), "icon");
    assert_eq!(slots[1].props_type.as_deref(), Some("{ size: number }"));
}

#[test]
fn test_parse_define_emits_runtime_object() {
    let result = parse_script_setup(
        r#"
            type SavePayload = { id: number }
            const emit = defineEmits({
                save: (payload: SavePayload) => payload.id > 0,
                close() { return true },
                cancel: null,
            })
        "#,
    );

    assert_eq!(result.macros.all_calls().len(), 1);
    assert_eq!(result.macros.emits().len(), 3);

    let save = result
        .macros
        .emits()
        .iter()
        .find(|emit| emit.name == "save")
        .expect("save emit should be extracted");
    assert_eq!(save.payload_type.as_deref(), Some("[payload: SavePayload]"));

    let close = result
        .macros
        .emits()
        .iter()
        .find(|emit| emit.name == "close")
        .expect("close emit should be extracted");
    assert_eq!(close.payload_type.as_deref(), Some("[]"));

    let cancel = result
        .macros
        .emits()
        .iter()
        .find(|emit| emit.name == "cancel")
        .expect("cancel emit should be extracted");
    assert_eq!(cancel.payload_type, None);
}

#[test]
fn test_parse_define_emits_runtime_object_spread_local_literal() {
    let result = parse_script_setup(
        r#"
            type SavePayload = { id: number }
            const commonEmits = {
                save: (payload: SavePayload) => payload.id > 0,
                close() { return true },
            } as const
            const emit = defineEmits({ ...commonEmits, cancel: null })
        "#,
    );

    assert_eq!(result.macros.emits().len(), 3);

    let save = result
        .macros
        .emits()
        .iter()
        .find(|emit| emit.name == "save")
        .expect("spread emit should be extracted");
    assert_eq!(save.payload_type.as_deref(), Some("[payload: SavePayload]"));

    let close = result
        .macros
        .emits()
        .iter()
        .find(|emit| emit.name == "close")
        .expect("method spread emit should be extracted");
    assert_eq!(close.payload_type.as_deref(), Some("[]"));

    assert!(
        result
            .macros
            .emits()
            .iter()
            .any(|emit| emit.name == "cancel")
    );
}

#[test]
fn test_parse_plain_script_exported_bindings() {
    let result = parse_script(
        r#"
export const foo = 'bar'
export function hello() {}
export class MyClass {}
"#,
    );

    assert!(result.bindings.contains("foo"));
    assert!(result.bindings.contains("hello"));
    assert!(result.bindings.contains("MyClass"));
    assert!(result.invalid_exports.is_empty());
}

#[test]
fn test_parse_reactivity() {
    let result = parse_script_setup(
        r#"
            const count = ref(0)
            const doubled = computed(() => count.value * 2)
            const state = reactive({ name: 'hello' })
        "#,
    );

    assert!(result.reactivity.is_reactive("count"));
    assert!(result.reactivity.is_reactive("doubled"));
    assert!(result.reactivity.is_reactive("state"));
    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_imports() {
    let result = parse_script_setup(
        r#"
            import { ref, computed } from 'vue'
            import MyComponent from './MyComponent.vue'
        "#,
    );

    insta::assert_debug_snapshot!(result);
}

#[test]
fn test_parse_options_api_component_registrations() {
    let output = options_api_parse_snapshot(
        r#"
            import Style from './style.vue'
            import Basic from './basic.vue'
            import { defineComponent } from 'vue'

            export default defineComponent({
                components: {
                    FourStyle: Style,
                    Basic,
                    'string-name': Basic,
                    Ignored: defineComponent({}),
                },
            })
        "#,
    );

    insta::assert_snapshot!(output);
}

#[test]
fn test_parse_options_api_component_registrations_through_bindings() {
    let output = options_api_parse_snapshot(
        r#"
            import LocalButton from './LocalButton.vue'
            import SharedBadge from './SharedBadge.vue'
            import LateCard from './LateCard.vue'
            import { defineComponent } from 'vue'

            const component = defineComponent(options)
            const sharedComponents = {
                SharedBadge,
            }
            const components = {
                ...sharedComponents,
                PrimaryButton: LocalButton,
                LocalButton,
                'late-card': LateCard as any,
            }
            const options = {
                components,
            }

            export default component
        "#,
    );

    insta::assert_snapshot!(output);
}

fn options_api_parse_snapshot(source: &str) -> String {
    let result = parse_script(source);
    let mut output = String::new();

    output.push_str("=== Component Registrations ===\n");
    for registration in &result.component_registrations {
        append!(
            output,
            "{} -> {}\n",
            registration.name,
            registration.local_name
        );
    }

    output.push_str("=== Invalid Exports ===\n");
    for invalid_export in &result.invalid_exports {
        append!(
            output,
            "{}: {:?}\n",
            invalid_export.name,
            invalid_export.kind
        );
    }

    output
}

#[test]
fn test_parse_invalid_exports() {
    let result = parse_script_setup(
        r#"
            export const foo = 'bar'
            export let count = 0
            export function hello() {}
            export class MyClass {}
            export default {}
        "#,
    );

    assert_eq!(result.invalid_exports.len(), 5);
}

#[test]
fn test_parse_type_exports() {
    let result = parse_script_setup(
        r#"
            export type Props = { msg: string }
            export interface Emits {
                (e: 'update', value: string): void
            }
        "#,
    );

    assert_eq!(result.type_exports.len(), 2);
}

#[test]
fn test_macro_span_tracking() {
    let source = "const props = defineProps<{ msg: string }>()";
    let result = parse_script_setup(source);

    let call = result.macros.all_calls().first().unwrap();
    assert!(call.start > 0);
    assert!(call.end > call.start);
    assert!(call.end as usize <= source.len());
}

#[test]
fn test_nested_callback_scopes() {
    let result = parse_script_setup(
        r#"
            const items = computed(() => {
                return list.map(item => item.value)
            })
        "#,
    );

    assert!(
        result.scopes.len() >= 3,
        "Expected at least 3 scopes, got {}",
        result.scopes.len()
    );
}

#[test]
fn test_parse_legacy_vue2_options_api_template_bindings() {
    let source = r#"
export default {
  props: {
    message: String,
    'user-id': Number
  },
  data() {
    return {
      count: 0
    }
  },
  asyncData() {
    return {
      pageTitle: 'Hello'
    }
  },
  computed: {
    doubled() {
      return this.count * 2
    }
  },
  methods: {
    save() {}
  },
  setup() {
    return {
      setupValue: 1
    }
  }
}
"#;
    let result = parse_script_with_options(
        source,
        ScriptParserOptions {
            options_api: false,
            legacy_vue2: true,
        },
    );

    for name in [
        "message",
        "userId",
        "count",
        "pageTitle",
        "doubled",
        "save",
        "setupValue",
        "$route",
        "$nuxt",
    ] {
        assert!(result.bindings.contains(name), "missing binding {name}");
    }
}

#[test]
fn test_parse_class_component_decorated_members() {
    // Class components are auto-detected by shape (default export is a
    // class), independent of the `options_api` flag.
    let source = r#"
import Vue from 'vue'
import Component from 'vue-class-component'
import UserBadge from './UserBadge.vue'

@Component({
  components: {
    UserBadge,
  },
})
export default class HelloWorld extends Vue {
  count = 0
  private msg!: string
  protected items: string[] = []
  #internal = 'hidden'
  static version = '1.0.0'
  declare ambient: string

  get doubled() {
    return this.count * 2
  }

  set doubled(value: number) {
    this.count = value / 2
  }

  save() {}

  private reset() {}

  constructor() {
    super()
  }
}
"#;
    let result = parse_script(source);

    assert_eq!(result.component_shape, ComponentShape::ClassApi);

    // Fields -> Data (TS `private`/`protected` are erased at runtime, so the
    // template still resolves them; the canonical Vue CLI class-component
    // scaffold renders `private` members).
    assert_eq!(result.bindings.get("count"), Some(BindingType::Data));
    assert_eq!(result.bindings.get("msg"), Some(BindingType::Data));
    assert_eq!(result.bindings.get("items"), Some(BindingType::Data));

    // Methods and get/set accessors -> Options (methods/computed-like).
    assert_eq!(result.bindings.get("doubled"), Some(BindingType::Options));
    assert_eq!(result.bindings.get("save"), Some(BindingType::Options));
    assert_eq!(result.bindings.get("reset"), Some(BindingType::Options));

    // Hard-private (#), static, declare, and constructor never resolve in
    // templates.
    assert!(!result.bindings.contains("internal"));
    assert!(!result.bindings.contains("#internal"));
    assert!(!result.bindings.contains("version"));
    assert!(!result.bindings.contains("ambient"));
    assert!(!result.bindings.contains("constructor"));

    // The `@Component({ components: { ... } })` argument reuses the Options
    // API registration collector.
    assert_eq!(result.component_registrations.len(), 1);
    assert_eq!(result.component_registrations[0].name, "UserBadge");
    assert_eq!(result.component_registrations[0].local_name, "UserBadge");

    // Members carry definition spans for Go-to-Definition.
    assert!(result.binding_spans.contains_key("count"));
    assert!(result.binding_spans.contains_key("doubled"));
}

#[test]
fn test_parse_class_component_undecorated_extends_vue() {
    let source = r#"
import Vue from 'vue'

export default class Counter extends Vue {
  count = 0

  get label() {
    return `count: ${this.count}`
  }

  increment() {
    this.count += 1
  }
}
"#;
    let result = parse_script(source);

    assert_eq!(result.component_shape, ComponentShape::ClassApi);
    assert_eq!(result.bindings.get("count"), Some(BindingType::Data));
    assert_eq!(result.bindings.get("label"), Some(BindingType::Options));
    assert_eq!(result.bindings.get("increment"), Some(BindingType::Options));
}

#[test]
fn test_parse_class_component_expression_export() {
    // Class *expressions* behind parens / TS wrappers are classified too.
    let source = r#"
import Vue from 'vue'

export default (class extends Vue {
  count = 0

  increment() {}
})
"#;
    let result = parse_script(source);

    assert_eq!(result.component_shape, ComponentShape::ClassApi);
    assert_eq!(result.bindings.get("count"), Some(BindingType::Data));
    assert_eq!(result.bindings.get("increment"), Some(BindingType::Options));
}

#[test]
fn test_parse_class_component_decorator_options_template_bindings() {
    // Options declared inside the decorator argument behave exactly like an
    // options component (vue-class-component merges them).
    let source = r#"
import { Options, Vue } from 'vue-class-component'

@Options({
  data() {
    return { fromDecorator: 1 }
  },
  computed: {
    decoratedComputed() {
      return 2
    },
  },
  methods: {
    decoratedMethod() {},
  },
})
export default class App extends Vue {
  local = 0
}
"#;
    let result = parse_script(source);

    assert_eq!(result.component_shape, ComponentShape::ClassApi);
    assert_eq!(result.bindings.get("local"), Some(BindingType::Data));
    assert_eq!(
        result.bindings.get("fromDecorator"),
        Some(BindingType::Data)
    );
    assert_eq!(
        result.bindings.get("decoratedComputed"),
        Some(BindingType::Options)
    );
    assert_eq!(
        result.bindings.get("decoratedMethod"),
        Some(BindingType::Options)
    );
}

#[test]
fn test_parse_non_class_components_keep_unspecified_shape() {
    // Options-object and script-setup analysis are untouched by the class
    // path: shape stays `Unspecified` and no class-style bindings appear.
    let options_result = parse_script_with_options(
        "export default { data() { return { count: 0 } } }",
        ScriptParserOptions {
            options_api: true,
            legacy_vue2: false,
        },
    );
    assert_eq!(options_result.component_shape, ComponentShape::Unspecified);
    assert_eq!(
        options_result.bindings.get("count"),
        Some(BindingType::Data)
    );

    let setup_result = parse_script_setup("const count = ref(0)");
    assert_eq!(setup_result.component_shape, ComponentShape::Unspecified);
}

#[test]
fn test_deeply_nested_callbacks() {
    let result = parse_script_setup(
        r#"
            onMounted(() => {
                watch(
                    () => state.value,
                    (newVal, oldVal) => {
                        console.log(newVal)
                    }
                )
            })
        "#,
    );

    assert!(
        result.scopes.len() >= 4,
        "Expected at least 4 scopes for deeply nested callbacks, got {}",
        result.scopes.len()
    );
}

#[test]
fn test_closure_params_extracted() {
    use crate::scope::{ScopeData, ScopeKind};

    let result = parse_script_setup(
        r#"
            const doubled = list.map((item, index) => item * index)
        "#,
    );

    let closure_scope = result.scopes.iter().find(|s| s.kind == ScopeKind::Closure);

    assert!(closure_scope.is_some(), "Should have a closure scope");

    if let ScopeData::Closure(data) = closure_scope.unwrap().data() {
        assert!(
            data.param_names.contains(&CompactString::new("item")),
            "Closure scope should have 'item' param"
        );
        assert!(
            data.param_names.contains(&CompactString::new("index")),
            "Closure scope should have 'index' param"
        );
        assert!(data.is_arrow, "Should be an arrow function");
    } else {
        panic!("Expected closure scope data");
    }
}

#[test]
fn test_binding_spans_captured() {
    let source = r#"
import { ref } from 'vue'
const count = ref(0)
function increment() {}
class MyClass {}
"#;
    let result = parse_script_setup(source);

    // ref is an import specifier
    assert!(
        result.binding_spans.contains_key("ref"),
        "Should capture import specifier span"
    );

    // count is a variable declaration
    assert!(
        result.binding_spans.contains_key("count"),
        "Should capture variable declaration span"
    );
    let (start, end) = result.binding_spans["count"];
    assert_eq!(&source[start as usize..end as usize], "count");

    // increment is a function declaration
    assert!(
        result.binding_spans.contains_key("increment"),
        "Should capture function declaration span"
    );
    let (start, end) = result.binding_spans["increment"];
    assert_eq!(&source[start as usize..end as usize], "increment");

    // MyClass is a class declaration
    assert!(
        result.binding_spans.contains_key("MyClass"),
        "Should capture class declaration span"
    );
    let (start, end) = result.binding_spans["MyClass"];
    assert_eq!(&source[start as usize..end as usize], "MyClass");
}

#[test]
fn test_binding_spans_imports() {
    let source = r#"
import { ref, computed } from 'vue'
import MyComp from './MyComp.vue'
import * as utils from './utils'
"#;
    let result = parse_script_setup(source);

    for name in &["ref", "computed", "MyComp", "utils"] {
        assert!(
            result.binding_spans.contains_key(*name),
            "Should capture span for import '{}'",
            name
        );
        let (start, end) = result.binding_spans[*name];
        assert_eq!(&source[start as usize..end as usize], *name);
    }
}

#[test]
fn test_binding_spans_stay_byte_aligned_with_unicode_comments() {
    let source = r#"
const before = 1
// あいうえおかきくけこさしすせそたちつてとなにぬねの
const heightLimit = "65vh"
// はひふへほまみむめもやいゆえよらりるれろわをん
"#;
    let result = parse_script_setup(source);

    let (start, end) = result.binding_spans["heightLimit"];
    assert_eq!(&source[start as usize..end as usize], "heightLimit");
}

// === Snapshot Tests ===

#[test]
fn test_parse_result_snapshot() {
    use insta::assert_snapshot;

    let result = parse_script_setup(
        r#"
import { ref, computed, watch } from 'vue'
import MyComponent from './MyComponent.vue'

const props = defineProps<{
    msg: string
    count?: number
}>()

const emit = defineEmits(['update', 'delete'])

const counter = ref(0)
const doubled = computed(() => counter.value * 2)

watch(counter, (newVal) => {
    console.log(newVal)
})

function increment() {
    counter.value++
}

const MyAlias = MyComponent
"#,
    );

    // Create a summary of the parse result for snapshot
    let bindings: Vec<_> = result.bindings.iter().collect();
    let mut bindings_sorted: Vec<_> = bindings
        .iter()
        .map(|(name, ty)| cstr!("{name}: {ty:?}"))
        .collect();
    bindings_sorted.sort();

    let mut output = String::new();
    output.push_str("=== Bindings ===\n");
    for b in &bindings_sorted {
        output.push_str(b);
        output.push('\n');
    }

    output.push_str("\n=== Macros ===\n");
    append!(output, "Props count: {}\n", result.macros.props().len());
    for p in result.macros.props() {
        append!(output, "  - {} (required: {})\n", p.name, p.required);
    }
    append!(output, "Emits count: {}\n", result.macros.emits().len());
    for e in result.macros.emits() {
        append!(output, "  - {}\n", e.name);
    }

    output.push_str("\n=== Reactivity ===\n");
    append!(
        output,
        "counter: reactive={}\n",
        result.reactivity.is_reactive("counter")
    );
    append!(
        output,
        "doubled: reactive={}\n",
        result.reactivity.is_reactive("doubled")
    );

    assert_snapshot!(output);
}

#[test]
fn test_reactivity_loss_snapshot() {
    use insta::assert_snapshot;

    let result = parse_script_setup(
        r#"
const state = reactive({ count: 0, name: 'test' })
const { count, name } = state

const countRef = ref(0)
const value = countRef.value

const copy = { ...state }
"#,
    );

    let mut output = String::new();
    output.push_str("=== Reactivity Losses ===\n");
    append!(
        output,
        "Total losses: {}\n\n",
        result.reactivity.losses().len()
    );

    for (i, loss) in result.reactivity.losses().iter().enumerate() {
        append!(output, "Loss #{}: {:?}\n", i + 1, loss.kind);
        append!(output, "  span: {}..{}\n", loss.start, loss.end);
    }

    assert_snapshot!(output);
}

#[test]
fn test_props_snapshot_crossing_call_and_getter_context() {
    use crate::reactivity::ReactivityLossKind;

    let result = parse_script_setup(
        r#"
const { count } = defineProps<{ count: number }>()

const ctx = useMyComposable(count)

const ctx2 = useMyComposable(() => count)
const a = ctx2.count()
"#,
    );

    assert!(result.reactivity.losses().iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name,
        } if source_name == "count"
            && argument_name == "count"
            && callee_name == "useMyComposable"
    )));
    assert!(result.reactivity.losses().iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::GetterCallExtract {
            context_name,
            getter_name,
            target_name,
            callee_name,
            source_name,
        } if context_name == "ctx2"
            && getter_name == "count"
            && target_name == "a"
            && callee_name == "useMyComposable"
            && source_name == "count"
    )));
}

#[test]
fn test_plain_reactive_values_inside_call_arguments() {
    use crate::reactivity::ReactivityLossKind;

    let result = parse_script_setup(
        r#"
const props = defineProps<{ count: number }>()
const { count: localCount } = props
const countRef = ref(0)

useMyComposable({ count: localCount })
useMyComposable(props.count)
useMyComposable(countRef.value)
watch(() => localCount, () => {})
"#,
    );

    let losses = result.reactivity.losses();
    assert!(losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name,
        } if source_name == "props.count"
            && argument_name == "localCount"
            && callee_name == "useMyComposable"
    )));
    assert!(losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name,
        } if source_name == "props.count"
            && argument_name == "props.count"
            && callee_name == "useMyComposable"
    )));
    assert!(losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name,
        } if source_name == "countRef.value"
            && argument_name == "countRef.value"
            && callee_name == "useMyComposable"
    )));
    assert!(!losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::FunctionArgumentExtract {
            argument_name,
            callee_name,
            ..
        } if argument_name == "localCount" && callee_name == "watch"
    )));
}

#[test]
fn test_plain_reactive_values_ignore_value_sink_calls() {
    use crate::reactivity::ReactivityLossKind;

    let result = parse_script_setup(
        r#"
const { count } = defineProps<{ count: number }>()
const emit = defineEmits<{ (e: 'update', value: number): void }>()

console.log(count)
console.warn({ count })
emit('update', count)
Math.max(count, 1)
Number(count)
JSON.stringify({ count })

watch(count, () => {})
useMyComposable(count)
"#,
    );

    let losses = result.reactivity.losses();
    for ignored_callee in ["log", "warn", "emit", "max", "Number", "stringify"] {
        assert!(!losses.iter().any(|loss| matches!(
            &loss.kind,
            ReactivityLossKind::FunctionArgumentExtract {
                callee_name,
                ..
            } if callee_name == ignored_callee
        )));
    }
    assert!(losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::FunctionArgumentExtract {
            argument_name,
            callee_name,
            ..
        } if argument_name == "count" && callee_name == "watch"
    )));
    assert!(losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::FunctionArgumentExtract {
            argument_name,
            callee_name,
            ..
        } if argument_name == "count" && callee_name == "useMyComposable"
    )));
}

#[test]
fn test_plain_reactive_alias_chain_crosses_calls_and_getters() {
    use crate::reactivity::ReactivityLossKind;

    let result = parse_script_setup(
        r#"
const { count } = defineProps<{ count: number }>()

const alias = count
const second = alias
let assigned
assigned = second

useMyComposable(second)
useMyComposable(assigned)

const ctx = useMyComposable(() => second)
const a = ctx.second()
"#,
    );

    let losses = result.reactivity.losses();
    assert!(
        !losses
            .iter()
            .any(|loss| matches!(&loss.kind, ReactivityLossKind::PropsDestructure { .. }))
    );
    assert!(losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::PlainValueAlias {
            source_name,
            alias_name,
            target_name,
        } if source_name == "count" && alias_name == "count" && target_name == "alias"
    )));
    assert!(losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::PlainValueAlias {
            source_name,
            alias_name,
            target_name,
        } if source_name == "count" && alias_name == "alias" && target_name == "second"
    )));
    assert!(losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::PlainValueAlias {
            source_name,
            alias_name,
            target_name,
        } if source_name == "count" && alias_name == "second" && target_name == "assigned"
    )));
    assert!(losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name,
        } if source_name == "count"
            && argument_name == "second"
            && callee_name == "useMyComposable"
    )));
    assert!(losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name,
        } if source_name == "count"
            && argument_name == "assigned"
            && callee_name == "useMyComposable"
    )));
    assert!(losses.iter().any(|loss| matches!(
        &loss.kind,
        ReactivityLossKind::GetterCallExtract {
            context_name,
            getter_name,
            source_name,
            ..
        } if context_name == "ctx" && getter_name == "second" && source_name == "count"
    )));
}

#[test]
fn test_scope_structure_snapshot() {
    use crate::scope::ScopeKind;
    use insta::assert_snapshot;

    let result = parse_script_setup(
        r#"
const items = ref([1, 2, 3])

const processed = items.value.map((item, index) => {
    return item * index
})

onMounted(() => {
    watch(() => items.value, (newVal) => {
        console.log(newVal)
    })
})

function processItem(item) {
    return item * 2
}
"#,
    );

    let mut output = String::new();
    output.push_str("=== Scope Structure ===\n");
    append!(output, "Total scopes: {}\n\n", result.scopes.len());

    // Count scopes by kind
    let mut closure_count = 0;
    let mut client_only_count = 0;
    let mut external_module_count = 0;
    let mut script_setup_count = 0;
    let mut module_count = 0;
    let mut js_global_count = 0;

    for scope in result.scopes.iter() {
        match scope.kind {
            ScopeKind::Closure => closure_count += 1,
            ScopeKind::ClientOnly => client_only_count += 1,
            ScopeKind::ExternalModule => external_module_count += 1,
            ScopeKind::ScriptSetup => script_setup_count += 1,
            ScopeKind::Module => module_count += 1,
            ScopeKind::JsGlobalUniversal | ScopeKind::JsGlobalBrowser | ScopeKind::JsGlobalNode => {
                js_global_count += 1
            }
            _ => {}
        }
    }

    append!(output, "Closure scopes: {closure_count}\n");
    append!(output, "ClientOnly scopes: {client_only_count}\n");
    append!(output, "ExternalModule scopes: {external_module_count}\n");
    append!(output, "ScriptSetup scopes: {script_setup_count}\n");
    append!(output, "Module scopes: {module_count}\n");
    append!(output, "JsGlobal scopes: {js_global_count}\n");

    assert_snapshot!(output);
}
