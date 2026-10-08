import type { CodeExample, RuleExample } from "./types.ts";
const vue = (source: string): CodeExample => ({ language: "vue", source });
const template = (source: string) => vue(`<template>\n${source}\n</template>`);
const script = (source: string) => vue(`<script setup lang="ts">\n${source}\n</script>`);
const pair = (bad: CodeExample, good: CodeExample, name: string): RuleExample => ({
  bad,
  good,
  evidence: `crates/vize_patina/src/rules/${name}`,
});

export const verifiedOverrides: Record<string, RuleExample> = {
  "script/no-deep-destructure-in-props": pair(
    script("const { user: { name } } = defineProps<{ user: { name: string } }>();"),
    script(
      'import { computed } from "vue";\nconst props = defineProps<{ user: { name: string } }>();\nconst userName = computed(() => props.user.name);',
    ),
    "script/no_deep_destructure_in_props.rs",
  ),
  "script/no-import-compiler-macros": pair(
    script(
      'import { defineProps, defineEmits } from "vue";\nconst props = defineProps<{ title: string }>();\nconst emit = defineEmits<{ save: [id: number] }>();',
    ),
    script(
      "const props = defineProps<{ title: string }>();\nconst emit = defineEmits<{ save: [id: number] }>();",
    ),
    "script/no_import_compiler_macros.rs",
  ),
  "script/no-reserved-identifiers": pair(
    script('const __props = { name: "Ada" };\nconst __emit = () => {};\nconst __sfc__ = {};'),
    script(
      "const props = defineProps<{ name: string }>();\nconst emit = defineEmits<{ save: [] }>();\nconst componentData = {};",
    ),
    "script/no_reserved_identifiers.rs",
  ),
  "script/no-with-defaults": pair(
    script(
      'const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });',
    ),
    script('const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();'),
    "script/no_with_defaults.rs",
  ),
  "script/require-typed-object-prop": pair(
    script("const props = defineProps({ user: Object, items: { type: Array } });"),
    script(
      'import type { PropType } from "vue";\ninterface User { name: string }\nconst props = defineProps({\n  user: Object as PropType<User>,\n  items: { type: Array as PropType<User[]> },\n});',
    ),
    "script/props_emits/require_typed_object_prop.rs",
  ),
  "script/define-emits-declaration": pair(
    script('const emit = defineEmits(["change"]);\nemit("change", 1);'),
    script('const emit = defineEmits<{ change: [id: number] }>();\nemit("change", 1);'),
    "script/define_emits_declaration.rs",
  ),
  "script/define-props-declaration": pair(
    script("const props = defineProps({ title: String });\nconsole.log(props.title);"),
    script("const props = defineProps<{ title: string }>();\nconsole.log(props.title);"),
    "script/define_props_declaration.rs",
  ),
  "script/no-async-in-computed": pair(
    script(
      'import { computed } from "vue";\nconst data = computed(async () => {\n  const response = await fetch("/api/data");\n  return response.json();\n});',
    ),
    script(
      'import { ref, watch } from "vue";\nconst query = ref("");\nconst data = ref<unknown>(null);\nwatch(query, async (value, _oldValue, onCleanup) => {\n  const controller = new AbortController();\n  let active = true;\n  onCleanup(() => { active = false; controller.abort(); });\n  const response = await fetch(`/api/data?q=${encodeURIComponent(value)}`, { signal: controller.signal });\n  const next: unknown = await response.json();\n  if (active) data.value = next;\n});',
    ),
    "script/no_async_in_computed.rs",
  ),
  "script/no-reactive-destructure": pair(
    script(
      'import { reactive } from "vue";\nconst state = reactive({ count: 0, name: "Ada" });\nconst { count, name } = state;',
    ),
    script(
      'import { reactive, toRefs } from "vue";\nconst state = reactive({ count: 0, name: "Ada" });\nconst { count, name } = toRefs(state);',
    ),
    "script/no_reactive_destructure.rs",
  ),
  "script/prefer-use-id": pair(
    vue(
      '<script setup lang="ts">\nconst id = `input-${Math.random()}`;\n</script>\n<template><label :for="id">Name</label><input :id="id" /></template>',
    ),
    vue(
      '<script setup lang="ts">\nimport { useId } from "vue";\nconst id = useId();\n</script>\n<template><label :for="id">Name</label><input :id="id" /></template>',
    ),
    "script/prefer_use_id.rs",
  ),
  "script/prefer-use-slots": pair(
    vue(
      '<script lang="ts">\nimport { defineComponent, h } from "vue";\nexport default defineComponent({\n  setup(_props, { slots }) { return () => h("div", slots.default?.()); },\n});\n</script>',
    ),
    vue(
      '<script lang="ts">\nimport { defineComponent, h, useSlots } from "vue";\nexport default defineComponent({\n  setup() {\n    const slots = useSlots();\n    return () => h("div", slots.default?.());\n  },\n});\n</script>',
    ),
    "script/prefer_use_slots.rs",
  ),
  "script/no-get-current-instance": {
    bad: vue(
      '<script setup lang="ts" vapor>\nimport { getCurrentInstance } from "vue";\nconst instance = getCurrentInstance();\n</script>',
    ),
    good: vue(
      '<script setup lang="ts" vapor>\nimport { inject } from "vue";\nconst appConfig = inject("app-config");\n</script>',
    ),
    evidence: "crates/vize_patina/src/rules/script/no_get_current_instance.rs",
  },
  "script/no-next-tick": {
    bad: vue(
      '<script setup lang="ts" vapor>\nimport { nextTick } from "vue";\nawait nextTick();\n</script>',
    ),
    good: vue(
      '<script setup lang="ts" vapor>\nimport { onMounted, useTemplateRef } from "vue";\nconst input = useTemplateRef<HTMLInputElement>("input");\nonMounted(() => { input.value?.focus(); });\n</script>\n<template><input ref="input"></template>',
    ),
    evidence: "crates/vize_patina/src/rules/script/no_next_tick.rs",
  },
  "script/valid-define-props": pair(
    script("defineProps<{ title: string }>({ title: String });"),
    script("defineProps<{ title: string }>();"),
    "script/valid_define_props.rs",
  ),
  "script/valid-define-emits": pair(
    script('defineEmits<{ save: [] }>(["save"]);'),
    script("defineEmits<{ save: [] }>();"),
    "script/valid_define_emits.rs",
  ),
  "script/valid-next-tick": pair(
    script('import { nextTick } from "vue";\nnextTick();'),
    script('import { nextTick } from "vue";\nawait nextTick();'),
    "script/valid_next_tick.rs",
  ),
  "vue/no-unsafe-url": pair(
    template('<a href="javascript:alert(1)">Continue</a>'),
    template('<a href="/next">Continue</a>'),
    "vue/no_unsafe_url.rs",
  ),
  "vue/no-deprecated-filter": pair(
    template("{{ message | capitalize }}"),
    template("{{ capitalize(message) }}"),
    "vue/no_deprecated_filter.rs",
  ),
  "vue/attribute-hyphenation": pair(
    template('<UserCard firstName="Ada" />'),
    template('<UserCard first-name="Ada" />'),
    "vue/attribute_hyphenation.rs",
  ),
  "vue/no-array-index-key": pair(
    template('<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>'),
    template('<li v-for="item in items" :key="item.id">{{ item.name }}</li>'),
    "opinionated/vue/no_array_index_key.rs",
  ),
  "vue/no-template-key": pair(
    template('<template :key="section"><div>Details</div></template>'),
    template(
      '<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>',
    ),
    "vue/no_template_key.rs",
  ),
  "vue/no-deprecated-scope-attribute": pair(
    template('<Card><template scope="props">{{ props.name }}</template></Card>'),
    template('<Card><template #default="props">{{ props.name }}</template></Card>'),
    "vue/no_deprecated_scope_attribute.rs",
  ),
  "vue/no-deprecated-slot-scope-attribute": pair(
    template('<Card><template slot-scope="props">{{ props.name }}</template></Card>'),
    template('<Card><template #default="props">{{ props.name }}</template></Card>'),
    "vue/no_deprecated_slot_scope_attribute.rs",
  ),
  "vue/no-v-for-template-key-on-child": pair(
    template('<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>'),
    template('<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>'),
    "vue/no_v_for_template_key_on_child.rs",
  ),
  "vue/no-lone-template": pair(
    template("<div><template><p>Details</p></template></div>"),
    template("<div><p>Details</p></div>"),
    "vue/no_lone_template.rs",
  ),
  "vue/no-useless-template-attributes": pair(
    template('<template v-if="ready" class="notice"><p>Ready</p></template>'),
    template('<template v-if="ready"><p class="notice">Ready</p></template>'),
    "vue/no_useless_template_attributes.rs",
  ),
};
