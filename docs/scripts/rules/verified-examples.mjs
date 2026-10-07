const vue = (source) => ({ language: "vue", source });
const template = (source) => vue(`<template>\n${source}\n</template>`);
const script = (source) => vue(`<script setup lang="ts">\n${source}\n</script>`);
const pair = (bad, good, name) => ({ bad, good, evidence: `crates/vize_patina/src/rules/${name}` });

export const verifiedOverrides = {
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
