const vue = (source) => ({ language: "vue", source });
const template = (source) => vue(`<template>\n${source}\n</template>`);
const script = (source) => vue(`<script lang="ts">\n${source}\n</script>`);
const pair = (bad, good, evidence, extra = {}) => ({ bad, good, evidence, ...extra });
const noSfcFinding = {
  availability: "no-sfc-finding",
  note: "This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.",
  noteJa:
    "このカタログ項目は現在の SFC lint では固有の検出を生成しません。悪い例・良い例は意図した規約の説明で、実行すると検出される例ではありません。ID を設定しても未対応の SFC 検査は追加されません。",
};
export const runtimeOverrides = {
  "vue/require-component-registration": pair(
    template("<MissingWidget />"),
    template("<MyButton />"),
    "crates/vize_patina/tests/global_component_registration.rs",
    {
      ruleOptions: { globals: ["MyButton", "MyIcon"] },
      note: "List explicit component names supplied by application plugins or Musea previewSetup. PascalCase and kebab-case spellings are accepted; regular expressions are not interpreted. Options do not enable the rule. Later layers replace the list; an empty list clears inherited names.",
      noteJa:
        "application plugin や Musea previewSetup が登録する component 名を明示します。PascalCase と kebab-case を許可し、正規表現は解釈しません。option だけではルールは有効になりません。後の設定は list 全体を置き換え、空 list は継承した名前を消します。",
    },
  ),
  "a11y/click-events-have-key-events": pair(
    template('<div @click="activate">Activate</div>'),
    template('<button @click="activate">Activate</button>'),
    "crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs",
    {
      note: "Checks non-interactive elements without an interactive role. Native buttons and elements with an interactive ARIA role are outside this rule's finding.",
      noteJa:
        "対話的な役割を持たない通常要素が対象です。button や対話的な ARIA role を持つ要素はこの検出の対象外です。",
    },
  ),
  "css/prefer-slotted": pair(
    vue("<style scoped>\nslot { color: red; }\n</style>"),
    vue("<style scoped>\n:slotted(.label) { color: red; }\n</style>"),
    "crates/vize_patina/src/rules/css/prefer_slotted.rs",
  ),
  "ecosystem/router-link-require-to": pair(
    template("<nav><RouterLink>Settings</RouterLink></nav>"),
    template('<nav><RouterLink to="/settings">Settings</RouterLink></nav>'),
    "crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs",
    {
      note: "A single SFC root link may inherit its target from parent attributes. This example uses a nested link, whose target must be explicit.",
      noteJa:
        "SFC の単一ルートにあるリンクは、親から属性を継承できるため対象外になる場合があります。この例は明示的な遷移先が必要な内部のリンクです。",
    },
  ),
  "html/deprecated-attr": pair(
    template('<p align="center">Notice</p>'),
    vue(
      '<template><p class="notice">Notice</p></template>\n<style scoped>.notice { text-align: center; }</style>',
    ),
    "crates/vize_patina/src/attribute_policy.rs",
  ),
  "script/prefer-use-attrs": pair(
    script("export default { setup(_props, { attrs }) { console.log(attrs.class); } };"),
    script(
      'import { useAttrs } from "vue";\nexport default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };',
    ),
    "crates/vize_patina/src/rules/script/prefer_use_attrs.rs",
  ),
  "script/return-in-emits-validator": pair(
    script(
      "export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };",
    ),
    script(
      "export default { emits: { submit: (payload: unknown) => { return payload != null; } } };",
    ),
    "crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator/tests.rs",
    {
      note: "Use a block-body arrow for the currently supported SFC filter. The underlying validator also handles method shorthand, but the current SFC prefilter does not reliably dispatch that shape.",
      noteJa:
        "現在の SFC フィルターが対象とする block-body arrow を使います。validator 本体には method shorthand の処理もありますが、現在の SFC prefilter はその形を確実には実行しません。",
    },
  ),
  "vue/no-template-shadow": pair(
    template(
      '<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>',
    ),
    template(
      '<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>',
    ),
    "crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs",
    {
      note: "The current check compares nested v-for bindings. It does not report a single v-for binding merely because it shares a script binding's name.",
      noteJa:
        "現在の検査は入れ子になった v-for の変数を比較します。script 内の名前と同じという理由だけで単独の v-for を検出するものではありません。",
    },
  ),
  "vue/no-useless-template-attributes": pair(
    template('<section><template v-if="ready" class="notice"><p>Ready</p></template></section>'),
    template('<section><template v-if="ready"><p class="notice">Ready</p></template></section>'),
    "crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs",
  ),
  "vue/prop-name-casing": pair(
    vue(
      '<script setup lang="ts">\ndefineProps<{ user_name: string }>();\n</script>\n<template><p>{{ user_name }}</p></template>',
    ),
    vue(
      '<script setup lang="ts">\ndefineProps<{ userName: string }>();\n</script>\n<template><p>{{ userName }}</p></template>',
    ),
    "crates/vize_patina/src/rules/vue/prop_name_casing/tests.rs",
    {
      note: "Checks declared prop names, not the casing of attributes passed to a child.",
      noteJa:
        "宣言した props の名前を検査します。子に渡す属性名の表記を検査するルールではありません。",
    },
  ),
  "vue/slot-name-casing": pair(
    template("<MyCard><template #mySlot>Content</template></MyCard>"),
    template("<MyCard><template #my-slot>Content</template></MyCard>"),
    "crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs",
  ),
  "vue/valid-attribute-name": pair(
    template('<div my"attr="value"></div>'),
    template('<div my-attr="value"></div>'),
    "crates/vize_patina/src/rules/vue/valid_attribute_name.rs",
    {
      badDiagnostic: "parser/template",
      note: "Malformed attribute spelling is diagnosed by parser/template before this defensive rule sees an attribute. Bad therefore reports parser/template; it does not promise a separate vue/valid-attribute-name finding.",
      noteJa:
        "不正な属性名は、この防御的なルールに届く前に parser/template で検出されます。悪い例で確認するのは parser/template の検出で、vue/valid-attribute-name が別に出るとは限りません。",
    },
  ),
  "vue/no-template-lang": pair(
    vue('<template lang="pug">\np Notice\n</template>'),
    template("<p>Notice</p>"),
    "crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs",
    noSfcFinding,
  ),
  "vue/no-preprocessor-lang": pair(
    vue(
      '<template><p>Notice</p></template>\n<style lang="scss">\n.notice { color: red; }\n</style>',
    ),
    vue("<template><p>Notice</p></template>\n<style>\n.notice { color: red; }\n</style>"),
    "crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs",
    noSfcFinding,
  ),
  "vue/no-script-non-standard-lang": pair(
    vue('<script lang="coffee">\ncount = 0\n</script>\n<template><p>Notice</p></template>'),
    vue('<script lang="ts">\nconst count = 0;\n</script>\n<template><p>Notice</p></template>'),
    "crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs",
    noSfcFinding,
  ),
  "vapor/require-vapor-attribute": pair(
    vue("<script setup>\nconst count = 0;\n</script>\n<template><p>{{ count }}</p></template>"),
    vue(
      "<script setup vapor>\nconst count = 0;\n</script>\n<template><p>{{ count }}</p></template>",
    ),
    "crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs",
    {
      ...noSfcFinding,
      note: "This rule is a placeholder with an empty callback. Adding vapor selects Vapor compilation; the current linter does not report this catalog ID for its absence.",
      noteJa:
        "このルールは callback が空の placeholder です。vapor 属性は Vapor でのコンパイルを選択するものですが、現在の linter は属性がないことをこの ID では検出しません。",
    },
  ),
};
