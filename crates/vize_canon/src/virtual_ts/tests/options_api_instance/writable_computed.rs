use crate::virtual_ts::{VirtualTsOptions, generate_virtual_ts_with_offsets_options_api};

/// The `const` / `var` declarations of the Options API template bindings the
/// generator emits for `script` and `template`, in emission order.
fn options_api_declarations(script: &str, template: &str) -> Vec<std::string::String> {
    let allocator = vize_carton::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full())
        .with_options_api();
    analyzer.analyze_script_plain(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let output = generate_virtual_ts_with_offsets_options_api(
        &summary,
        Some(script),
        Some(&root),
        0,
        0,
        &VirtualTsOptions::default(),
    );
    output
        .code
        .lines()
        .filter(|line| line.contains("__VizeOptionsBinding"))
        .filter(|line| {
            let trimmed = line.trim_start();
            trimmed.starts_with("const ") || trimmed.starts_with("var ")
        })
        .map(std::string::String::from)
        .collect()
}

/// A `{ get, set }` computed is a writable instance property, so a template
/// assignment such as `@input="ratio = $event"` must not report `TS2588`.
/// A getter-only computed stays `const`, like methods and props.
#[test]
fn writable_computed_template_bindings_emit_var() {
    let script = r#"export default {
    data() {
        return { store: { ratio: '1' } }
    },
    computed: {
        ratio: {
            get(): string {
                return this.store.ratio
            },
            set(value: string) {
                this.store.ratio = value
            },
        },
        arrowSetter: {
            get: () => 'a',
            set: (_value: string) => {},
        },
        get accessor() {
            return 'a'
        },
        set accessor(_value: string) {},
        label() {
            return this.store.ratio
        },
        getterOnly: {
            get() {
                return this.store.ratio
            },
        },
    },
    methods: {
        reset() {
            this.store.ratio = '1'
        },
    },
}
"#;
    let declarations = options_api_declarations(
        script,
        r#"<input :value="ratio" @input="ratio = $event; arrowSetter = $event; accessor = $event" />{{ label }} {{ getterOnly }} {{ reset }}"#,
    );
    assert_eq!(
        declarations,
        [
            "  var accessor: __VizeOptionsBinding<typeof __default__, \"accessor\"> = undefined as any;",
            "  var arrowSetter: __VizeOptionsBinding<typeof __default__, \"arrowSetter\"> = undefined as any;",
            "  const getterOnly: __VizeOptionsBinding<typeof __default__, \"getterOnly\"> = undefined as any;",
            "  const label: __VizeOptionsBinding<typeof __default__, \"label\"> = undefined as any;",
            "  var ratio: __VizeOptionsBinding<typeof __default__, \"ratio\"> = undefined as any;",
            "  const reset: __VizeOptionsBinding<typeof __default__, \"reset\"> = undefined as any;",
            "  var store: __VizeOptionsBinding<typeof __default__, \"store\"> = undefined as any;",
        ]
    );
}

/// The collector follows the same same-file `mixins` / `extends` objects the
/// template-binding collector reads, so a writable computed inherited from a
/// mixin is writable in the template too.
#[test]
fn writable_computed_from_same_file_mixin_emits_var() {
    let script = r#"import { defineComponent } from 'vue'

const sizing = {
    computed: {
        ratio: {
            get(): string {
                return '1'
            },
            set(_value: string) {},
        },
    },
}

const base = {
    computed: {
        title: {
            get(): string {
                return 't'
            },
            set(_value: string) {},
        },
        fixed() {
            return 'f'
        },
    },
}

export default defineComponent({
    mixins: [sizing],
    extends: base,
})
"#;
    let declarations = options_api_declarations(
        script,
        r#"<div @click="ratio = '2'; title = 't2'">{{ fixed }}</div>"#,
    );
    assert_eq!(
        declarations,
        [
            "  const fixed: __VizeOptionsBinding<typeof __default__, \"fixed\"> = undefined as any;",
            "  var ratio: __VizeOptionsBinding<typeof __default__, \"ratio\"> = undefined as any;",
            "  var title: __VizeOptionsBinding<typeof __default__, \"title\"> = undefined as any;",
        ]
    );
}

/// Vue merges `computed` with the component's own declaration winning over
/// `mixins`, and later mixins over earlier ones (and over `extends`). A local
/// getter-only computed therefore shadows an inherited writable one, and a
/// later mixin's getter-only computed shadows an earlier mixin's setter.
#[test]
fn local_and_later_declarations_shadow_inherited_writability() {
    let script = r#"const writableBase = {
    computed: {
        ratio: {
            get(): string {
                return '1'
            },
            set(_value: string) {},
        },
        title: {
            get(): string {
                return 't'
            },
            set(_value: string) {},
        },
    },
}

const readonlyOverride = {
    computed: {
        title() {
            return 't'
        },
    },
}

export default {
    mixins: [writableBase, readonlyOverride],
    computed: {
        ratio() {
            return '1'
        },
    },
}
"#;
    let declarations = options_api_declarations(script, r#"<div>{{ ratio }} {{ title }}</div>"#);
    assert_eq!(
        declarations,
        [
            "  const ratio: __VizeOptionsBinding<typeof __default__, \"ratio\"> = undefined as any;",
            "  const title: __VizeOptionsBinding<typeof __default__, \"title\"> = undefined as any;",
        ]
    );
}

/// A `set` accessor before its `get` accessor is the same writable pair.
#[test]
fn accessor_pair_is_writable_in_either_order() {
    let script = r#"export default {
    computed: {
        set first(_value: string) {},
        get first() {
            return 'a'
        },
    },
}
"#;
    let declarations = options_api_declarations(script, r#"<div @click="first = 'b'" />"#);
    assert_eq!(
        declarations,
        ["  var first: __VizeOptionsBinding<typeof __default__, \"first\"> = undefined as any;"]
    );
}

/// The same mixin listed again after another one that shadowed its setter
/// applies again at its later position, so the setter wins; and an exported
/// same-file mixin (`export const shared = { ... }`) is resolved like a
/// non-exported one.
#[test]
fn repeated_and_exported_mixins_apply_in_order() {
    let script = r#"export const shared = {
    computed: {
        ratio: {
            get(): string {
                return '1'
            },
            set(_value: string) {},
        },
        label: {
            get(): string {
                return 'l'
            },
            set(_value: string) {},
        },
    },
}

const readonlyOverride = {
    computed: {
        ratio() {
            return '1'
        },
        label() {
            return 'l'
        },
    },
}

export default {
    mixins: [shared, readonlyOverride, shared],
    computed: {
        label() {
            return 'l'
        },
    },
}
"#;
    let declarations = options_api_declarations(script, r#"<div>{{ ratio }} {{ label }}</div>"#);
    assert_eq!(
        declarations,
        [
            "  const label: __VizeOptionsBinding<typeof __default__, \"label\"> = undefined as any;",
            "  var ratio: __VizeOptionsBinding<typeof __default__, \"ratio\"> = undefined as any;",
        ]
    );
}

/// Vue resolves a template name from `data`, then `props`, and only then from
/// the context that exposes computed members. A writable computed that shares
/// a prop's name (its own, or one inherited from a mixin) never provides the
/// value, and the prop is read-only, so the binding stays `const`.
#[test]
fn writable_computed_sharing_a_prop_name_stays_const() {
    let script = r#"const withSize = {
    props: { size: String },
}

export default {
    mixins: [withSize],
    props: ['ratio'],
    computed: {
        ratio: {
            get(): string {
                return '1'
            },
            set(_value: string) {},
        },
        size: {
            get(): string {
                return 's'
            },
            set(_value: string) {},
        },
        free: {
            get(): string {
                return 'f'
            },
            set(_value: string) {},
        },
    },
}
"#;
    let declarations =
        options_api_declarations(script, r#"<div>{{ ratio }} {{ size }} {{ free }}</div>"#);
    assert_eq!(
        declarations,
        [
            "  var free: __VizeOptionsBinding<typeof __default__, \"free\"> = undefined as any;",
            "  const ratio: __VizeOptionsBinding<typeof __default__, \"ratio\"> = undefined as any;",
            "  const size: __VizeOptionsBinding<typeof __default__, \"size\"> = undefined as any;",
        ]
    );
}

mod setter_fixtures;
mod wrapped_options;
