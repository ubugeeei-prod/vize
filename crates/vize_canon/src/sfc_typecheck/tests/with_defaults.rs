use super::{SfcTypeCheckOptions, type_check_sfc};

#[test]
fn test_type_check_with_defaults_template_props_are_default_resolved() {
    let source = r#"<template>
  <svg>
    <line :stroke-width="props.thickness / 2"></line>
    <text>{{ label.toUpperCase() }} {{ props.label.toUpperCase() }}</text>
  </svg>
</template>

<script setup lang="ts">
const props = withDefaults(defineProps<{
  thickness?: number;
  label?: string;
  raw?: string;
}>(), {
  thickness: 0.1,
  label: 'ok',
});

const { thickness, label } = props;
</script>"#;
    let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
    let result = type_check_sfc(source, &options);
    let virtual_ts = result.virtual_ts.expect("virtual ts should be generated");

    assert!(
        virtual_ts.contains(
            r#"type __WithDefaultsResult<T, D, __BKeys extends keyof T = never> = T extends unknown ? Readonly<__VizeMappedOmit<T, keyof D>> & { readonly [K in keyof D as K extends keyof T ? K : never]-?: K extends keyof T ? D[K] extends undefined ? __VizeIfAny<D[K], __VizeNotUndefined<T[K]>, T[K]> : __VizeNotUndefined<T[K]> : never } & { readonly [K in __BKeys]-?: K extends keyof D ? D[K] extends undefined ? boolean | undefined : boolean : boolean } : never;"#
        ),
        "{virtual_ts}"
    );
    assert!(
        virtual_ts.contains(
            r#"const props: __WithDefaultsResult<Props, Pick<Props, "label" | "thickness">>"#
        ),
        "{virtual_ts}"
    );
    assert!(!virtual_ts.contains(r#"const thickness = props["thickness"]"#));
    assert!(!virtual_ts.contains(r#"const label = props["label"]"#));
    assert!(
        virtual_ts.contains(r#"const raw = props["raw"];"#),
        "{virtual_ts}"
    );
}

#[test]
fn test_type_check_with_defaults_narrows_direct_template_prop_identifiers() {
    let source = r#"<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    count?: number;
    label: string;
  }>(),
  { count: 0 },
);

const emit = defineEmits<{
  increment: [value: number];
}>();

void props;
</script>

<template>
  <button type="button" @click="emit('increment', count + 1)">
    {{ label }}: {{ count }}
  </button>
</template>"#;
    let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
    let result = type_check_sfc(source, &options);
    let virtual_ts = result.virtual_ts.expect("virtual ts should be generated");

    assert!(
        virtual_ts.contains(
            r#"const count = props["count"] as Exclude<__WithDefaultsResult<Props, Pick<Props, "count">>["count"], undefined>;"#
        ),
        "{virtual_ts}"
    );
    assert!(
        !virtual_ts.contains(r#"const count = props["count"];"#),
        "{virtual_ts}"
    );
    assert!(
        virtual_ts.contains(r#"void (emit('increment', count + 1));"#),
        "{virtual_ts}"
    );
}

#[test]
fn never_prop_is_not_treated_as_boolean() {
    let source = r#"<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    disabled?: boolean;
    type?: never;
    variant?: 'button-primary' | 'button-secondary' | 'link';
    size?: 'sm' | 'md';
    to?: string;
  }>(),
  { variant: 'link', size: 'md' },
);

props.to;
props.variant;
props.size;
</script>

<template>
  {{ props.to }} {{ props.variant }} {{ props.size }}
</template>"#;
    let options = SfcTypeCheckOptions::new("LinkBase.vue").with_virtual_ts();
    let result = type_check_sfc(source, &options);

    assert!(
        !result
            .diagnostics
            .iter()
            .any(|d| d.message.contains("reduced to 'never'")),
        "{:#?}",
        result.diagnostics
    );
    assert!(!result.has_errors(), "{:#?}", result.diagnostics);
}
