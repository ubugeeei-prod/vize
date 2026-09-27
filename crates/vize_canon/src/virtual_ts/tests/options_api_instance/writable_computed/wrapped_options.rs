use super::options_api_declarations;

/// `props` and `mixins` written through parentheses or TypeScript wrappers
/// (`(['a'] as const)`) resolve like plain arrays; a kebab-case prop shadows
/// its camelCase computed; and a descriptor whose `set` is statically absent
/// (`set: undefined`) or not callable stays read-only.
#[test]
fn wrapped_options_kebab_props_and_absent_setters() {
    let script = r#"const sizing = {
    computed: {
        ratio: {
            get(): string {
                return '1'
            },
            set(_value: string) {},
        },
    },
}

export default {
    mixins: ([sizing] as const),
    props: (['foo-bar', 'plain'] as const),
    computed: {
        fooBar: {
            get(): string {
                return 'f'
            },
            set(_value: string) {},
        },
        plain: {
            get(): string {
                return 'p'
            },
            set(_value: string) {},
        },
        absent: {
            get(): string {
                return 'a'
            },
            set: undefined,
        },
        literal: {
            get(): string {
                return 'l'
            },
            set: null,
        },
        arrow: {
            get: () => 'r',
            set: (_value: string) => {},
        },
    },
}
"#;
    let declarations = options_api_declarations(
        script,
        r#"<div>{{ ratio }} {{ fooBar }} {{ plain }} {{ absent }} {{ literal }} {{ arrow }}</div>"#,
    );
    assert_eq!(
        declarations,
        [
            "  const absent: __VizeOptionsBinding<typeof __default__, \"absent\"> = undefined as any;",
            "  var arrow: __VizeOptionsBinding<typeof __default__, \"arrow\"> = undefined as any;",
            "  const fooBar: __VizeOptionsBinding<typeof __default__, \"fooBar\"> = undefined as any;",
            "  const literal: __VizeOptionsBinding<typeof __default__, \"literal\"> = undefined as any;",
            "  const plain: __VizeOptionsBinding<typeof __default__, \"plain\"> = undefined as any;",
            "  var ratio: __VizeOptionsBinding<typeof __default__, \"ratio\"> = undefined as any;",
        ]
    );
}
