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
  "a11y/anchor-has-content": pair(
    template('<a href="/app"></a>'),
    template('<a href="/app"><img src="/badge.png" alt="Get the app" /></a>'),
    "crates/vize_patina/src/rules/a11y/anchor_has_content.rs",
    {
      ruleOptions: { accessibleChildren: ["RetryImage"] },
      note: "List component names whose rendered subtree always supplies accessible anchor content. PascalCase and kebab-case spellings are equivalent. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.",
      noteJa:
        "描画結果がリンクのアクセシブルな内容を必ず提供するコンポーネント名を指定します。PascalCase と kebab-case は同じ名前として扱います。オプションだけではルールを有効にしません。後の設定レイヤーは一覧全体を置き換え、[] で継承した名前を消します。",
    },
  ),
  "a11y/label-has-for": pair(
    template("<label>Name</label>"),
    template('<label>Name <input v-model="name" /></label>'),
    "crates/vize_patina/src/rules/a11y/label_has_for.rs",
    {
      ruleOptions: { controlComponents: ["CInputField"] },
      note: "List component names that render a native form control and therefore satisfy label nesting. PascalCase and kebab-case spellings are equivalent. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.",
      noteJa:
        "ネイティブのフォーム部品を描画し、label の入れ子条件を満たすコンポーネント名を指定します。PascalCase と kebab-case は同じ名前として扱います。オプションだけではルールを有効にしません。後の設定レイヤーは一覧全体を置き換え、[] で継承した名前を消します。",
    },
  ),
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
  "vue/no-bare-strings-in-template": pair(
    template("<p>Translate me</p>"),
    template('<kbd>{{ $t("keyboard.forward") }}</kbd>'),
    "crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs",
    {
      ruleOptions: { allowlist: ["/^[DRWXYmvx]$/u"] },
      note: "The allowlist replaces eslint-plugin-vue's punctuation defaults when present. Entries are literal substrings or /pattern/flags values and are removed before the remaining text is checked. Omit allowlist to retain the defaults; [] clears them. Options do not enable the rule.",
      noteJa:
        "allowlist を指定すると、eslint-plugin-vue の既定の記号一覧を置き換えます。各項目はリテラルな部分文字列または /pattern/flags 形式で、残りのテキストを検査する前に除外します。既定値を保つには allowlist を省略し、[] で既定値を消します。オプションだけではルールを有効にしません。",
    },
  ),
  "vue/warn-custom-directive": pair(
    template('<button type="button" v-focus>Focus</button>'),
    template('<button type="button" v-show="visible">Copy</button>'),
    "crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs",
    {
      ruleOptions: { ignore: ["hover-tooltip"] },
      note: "List exact globally registered directive names without the v- prefix. Local script-setup vName bindings are still discovered automatically. Options do not enable the rule; later config layers replace the complete list, and [] clears inherited names.",
      noteJa:
        "グローバル登録したディレクティブ名を v- 接頭辞なしの完全一致で指定します。script setup のローカルな vName 束縛は引き続き自動検出します。オプションだけではルールを有効にしません。後の設定レイヤーは一覧全体を置き換え、[] で継承した名前を消します。",
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
    template('<MyCard><slot name="my-slot" /></MyCard>'),
    template('<MyCard><slot name="mySlot" /></MyCard>'),
    "crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs",
    {
      note: "Checks the static name of a rendered slot outlet. The parent-side v-slot and #name provider syntax is outside this rule.",
      noteJa:
        "描画する slot outlet の静的な name を検査します。親側の v-slot や #name による提供構文は、このルールの対象外です。",
    },
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
