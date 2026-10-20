import type { CodeExample, RuleExample, RuleExampleOptions } from "./types.ts";
const vue = (source: string): CodeExample => ({ language: "vue", source });
const template = (source: string) => vue(`<template>\n${source}\n</template>`);
const script = (source: string) => vue(`<script lang="ts">\n${source}\n</script>`);
const pair = (
  bad: CodeExample,
  good: CodeExample,
  evidence: string,
  extra: RuleExampleOptions = {},
): RuleExample => ({ bad, good, evidence, ...extra });
const noSfcFinding = {
  availability: "no-sfc-finding",
  note: "This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.",
  noteJa:
    "このカタログ項目は現在の SFC lint では固有の診断を出しません。悪い例・良い例は意図した規約の説明であり、実行時にこのルールの診断が出る例ではありません。ID を設定しても未対応の SFC 検査は追加されません。",
};
export const runtimeOverrides: Record<string, RuleExample> = {
  "vue/require-component-registration": pair(
    template("<MissingWidget />"),
    template("<MyButton />"),
    "crates/vize_patina/tests/global_component_registration.rs",
    {
      ruleOptions: { globals: ["MyButton", "MyIcon"] },
      note: "List explicit component names supplied by application plugins or Musea previewSetup. PascalCase and kebab-case spellings are accepted; regular expressions are not interpreted. Options do not enable the rule. Later layers replace the list; an empty list clears inherited names.",
      noteJa:
        "アプリケーションのプラグインや Musea の previewSetup が登録するコンポーネント名を明示します。PascalCase と kebab-case を許可し、正規表現は解釈しません。オプションだけではルールは有効になりません。後の設定は一覧全体を置き換え、空の一覧は継承した名前を消します。",
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
        "SFC の単一ルート要素がリンクである場合、親から属性を継承できるため対象外になることがあります。この例は別の要素の内側にあるリンクで、明示的な遷移先が必要です。",
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
        "この例では、現在の SFC フィルターが対象にする、関数本体を波括弧で囲んだアロー関数を使います。バリデーター自体はメソッドの省略記法にも対応していますが、現在の SFC 事前フィルターでは、その形式の検査が確実に呼び出されません。",
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
      note: "The check compares `v-for`, `v-slot` (including `#name`), and legacy scoped-slot bindings within the template. It reports the identifier that hides an earlier binding, including declarations on the same element in attribute order. Sibling elements have independent scopes. Property keys and default-value references are not declarations, and `v-pre` makes its subtree inert.\n\nScript and component property names are outside the current check: a single template binding does not produce a warning merely because a script binding has the same name. Diagnostic ranges cover the authored identifier, including escaped spellings and HTML entities.",
      noteJa:
        "検査はテンプレート内の `v-for`、`v-slot`（`#name` を含む）、従来のスコープ付きスロットの束縛を比較します。外側の束縛を隠す識別子を報告し、同じ要素内では属性が記述された順に比較します。兄弟要素のスコープは独立しています。プロパティのキーや既定値の参照は宣言ではなく、`v-pre` の部分木は検査しません。\n\nscript やコンポーネントのプロパティ名は現在の検査範囲に含みません。同じ名前の script 束縛があるだけで、単独のテンプレート束縛を報告することはありません。診断範囲は、エスケープや HTML エンティティを含む元の識別子を示します。",
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
  ),
  "vue/no-preprocessor-lang": pair(
    vue(
      '<template><p>Notice</p></template>\n<style lang="scss">\n.notice { color: red; }\n</style>',
    ),
    vue("<template><p>Notice</p></template>\n<style>\n.notice { color: red; }\n</style>"),
    "crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs",
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
        "このルールは、検査用のコールバックが空の仮実装です。vapor 属性は Vapor でのコンパイルを選択するものですが、現在の linter は属性がないことをこの ID では検出しません。",
    },
  ),
};
