---
title: "型と script のルール"
---

# 型と script のルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`script/component-options-name-casing`](#script-component-options-name-casing) | [悪い](#script-component-options-name-casing-bad) · [良い](#script-component-options-name-casing-good) | コンポーネントの name オプションを PascalCase に揃えます。 |
| [`script/custom-event-name-casing`](#script-custom-event-name-casing) | [悪い](#script-custom-event-name-casing-bad) · [良い](#script-custom-event-name-casing-good) | emit するカスタムイベント名を指定した形式に揃えます。 |
| [`script/define-emits-declaration`](#script-define-emits-declaration) | [悪い](#script-define-emits-declaration-bad) · [良い](#script-define-emits-declaration-good) | defineEmits を型による宣言形式に揃えます。 |
| [`script/define-macros-order`](#script-define-macros-order) | [悪い](#script-define-macros-order-bad) · [良い](#script-define-macros-order-good) | script setup のコンパイラーマクロを一定の順に宣言します。 |
| [`script/define-props-declaration`](#script-define-props-declaration) | [悪い](#script-define-props-declaration-bad) · [良い](#script-define-props-declaration-good) | defineProps を型による宣言形式に揃えます。 |
| [`script/define-props-destructuring`](#script-define-props-destructuring) | [悪い](#script-define-props-destructuring-bad) · [良い](#script-define-props-destructuring-good) | defineProps の分割代入スタイルを指定した方針に揃えます。 |
| [`script/no-arrow-functions-in-watch`](#script-no-arrow-functions-in-watch) | [悪い](#script-no-arrow-functions-in-watch-bad) · [良い](#script-no-arrow-functions-in-watch-good) | Options API の watch に this を持たないアロー関数を使う箇所を検出します。 |
| [`script/no-async-in-computed`](#script-no-async-in-computed) | [悪い](#script-no-async-in-computed-bad) · [良い](#script-no-async-in-computed-good) | computed の getter で非同期関数を使う箇所を検出します。 |
| [`script/no-boolean-default`](#script-no-boolean-default) | [悪い](#script-no-boolean-default-bad) · [良い](#script-no-boolean-default-good) | Boolean prop の冗長な default を検出します。 |
| [`script/no-deep-destructure-in-props`](#script-no-deep-destructure-in-props) | [悪い](#script-no-deep-destructure-in-props-bad) · [良い](#script-no-deep-destructure-in-props-good) | defineProps の深い分割代入を検出します。 |
| [`script/no-deprecated-data-object-declaration`](#script-no-deprecated-data-object-declaration) | [悪い](#script-no-deprecated-data-object-declaration-bad) · [良い](#script-no-deprecated-data-object-declaration-good) | Vue 3 で関数にすべき data オプションのオブジェクト指定を検出します。 |
| [`script/no-deprecated-destroyed-lifecycle`](#script-no-deprecated-destroyed-lifecycle) | [悪い](#script-no-deprecated-destroyed-lifecycle-bad) · [良い](#script-no-deprecated-destroyed-lifecycle-good) | Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 の hook に置き換えます。 |
| [`script/no-deprecated-dollar-listeners-api`](#script-no-deprecated-dollar-listeners-api) | [悪い](#script-no-deprecated-dollar-listeners-api-bad) · [良い](#script-no-deprecated-dollar-listeners-api-good) | Vue 3 で $attrs に統合された $listeners を検出します。 |
| [`script/no-deprecated-dollar-scopedslots-api`](#script-no-deprecated-dollar-scopedslots-api) | [悪い](#script-no-deprecated-dollar-scopedslots-api-bad) · [良い](#script-no-deprecated-dollar-scopedslots-api-good) | Vue 3 で $slots に統合された $scopedSlots を検出します。 |
| [`script/no-deprecated-events-api`](#script-no-deprecated-events-api) | [悪い](#script-no-deprecated-events-api-bad) · [良い](#script-no-deprecated-events-api-good) | Vue 3 で削除された $on / $off / $once を検出します。 |
| [`script/no-deprecated-props-default-this`](#script-no-deprecated-props-default-this) | [悪い](#script-no-deprecated-props-default-this-bad) · [良い](#script-no-deprecated-props-default-this-good) | prop の default / validator 内で使えなくなった this を検出します。 |
| [`script/no-dupe-keys`](#script-no-dupe-keys) | [悪い](#script-no-dupe-keys-bad) · [良い](#script-no-dupe-keys-good) | Options API の props / data / computed などのキー重複を検出します。 |
| [`script/no-duplicate-attr-inheritance`](#script-no-duplicate-attr-inheritance) | [悪い](#script-no-duplicate-attr-inheritance-bad) · [良い](#script-no-duplicate-attr-inheritance-good) | fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。 |
| [`script/no-export-in-script-setup`](#script-no-export-in-script-setup) | [悪い](#script-no-export-in-script-setup-bad) · [良い](#script-no-export-in-script-setup-good) | script setup 内の export 文を検出します。 |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [悪い](#script-no-get-current-instance-bad) · [良い](#script-no-get-current-instance-good) | Vapor で null を返す getCurrentInstance() を検出します。 |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [悪い](#script-no-import-compiler-macros-bad) · [良い](#script-no-import-compiler-macros-good) | 自動的に使える Vue コンパイラーマクロの import を検出します。 |
| [`script/no-internal-imports`](#script-no-internal-imports) | [悪い](#script-no-internal-imports-bad) · [良い](#script-no-internal-imports-good) | Vue 内部モジュールからの import を検出します。 |
| [`script/no-multiple-slot-args`](#script-no-multiple-slot-args) | [悪い](#script-no-multiple-slot-args-bad) · [良い](#script-no-multiple-slot-args-good) | scoped slot 関数に複数の引数を渡す箇所を検出します。 |
| [`script/no-next-tick`](#script-no-next-tick) | [悪い](#script-no-next-tick-bad) · [良い](#script-no-next-tick-good) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 |
| [`script/no-options-api`](#script-no-options-api) | [悪い](#script-no-options-api-bad) · [良い](#script-no-options-api-good) | Vapor で Options API を使用する箇所を検出します。 |
| [`script/no-potential-component-option-typo`](#script-no-potential-component-option-typo) | [悪い](#script-no-potential-component-option-typo-bad) · [良い](#script-no-potential-component-option-typo-good) | Options API のオプション名の入力ミスを検出します。 |
| [`script/no-reactive-destructure`](#script-no-reactive-destructure) | [悪い](#script-no-reactive-destructure-bad) · [良い](#script-no-reactive-destructure-good) | reactive オブジェクトの反応性を失う分割代入を検出します。 |
| [`script/no-ref-as-operand`](#script-no-ref-as-operand) | [悪い](#script-no-ref-as-operand-bad) · [良い](#script-no-ref-as-operand-good) | ref を演算の値として使う際に .value を参照します。 |
| [`script/no-required-prop-with-default`](#script-no-required-prop-with-default) | [悪い](#script-no-required-prop-with-default-bad) · [良い](#script-no-required-prop-with-default-good) | required: true と default を同時に持つ prop を検出します。 |
| [`script/no-reserved-identifiers`](#script-no-reserved-identifiers) | [悪い](#script-no-reserved-identifiers-bad) · [良い](#script-no-reserved-identifiers-good) | Vue コンパイラーが予約した識別子の宣言を検出します。 |
| [`script/no-reserved-keys`](#script-no-reserved-keys) | [悪い](#script-no-reserved-keys-bad) · [良い](#script-no-reserved-keys-good) | Options API のキーに Vue の予約名を使う箇所を検出します。 |
| [`script/no-reserved-props`](#script-no-reserved-props) | [悪い](#script-no-reserved-props-bad) · [良い](#script-no-reserved-props-good) | prop 宣言に Vue の予約名を使う箇所を検出します。 |
| [`script/no-restricted-globals`](#script-no-restricted-globals) | [悪い](#script-no-restricted-globals-bad) · [良い](#script-no-restricted-globals-good) | 設定で禁止した実行環境のグローバル参照を検出します。 |
| [`script/no-restricted-members`](#script-no-restricted-members) | [悪い](#script-no-restricted-members-bad) · [良い](#script-no-restricted-members-good) | 設定で禁止した object.property へのアクセスを検出します。 |
| [`script/no-side-effects-in-computed-properties`](#script-no-side-effects-in-computed-properties) | [悪い](#script-no-side-effects-in-computed-properties-bad) · [良い](#script-no-side-effects-in-computed-properties-good) | Options API の computed getter 内の副作用を検出します。 |
| [`script/no-top-level-ref-in-script`](#script-no-top-level-ref-in-script) | [悪い](#script-no-top-level-ref-in-script-bad) · [良い](#script-no-top-level-ref-in-script-good) | 通常の script のトップレベルで、リクエスト間に共有される状態を作る箇所を検出します。 |
| [`script/no-unstable-nested-components`](#script-no-unstable-nested-components) | [悪い](#script-no-unstable-nested-components-bad) · [良い](#script-no-unstable-nested-components-good) | setup / render 内で毎回コンポーネントを定義する箇所を検出します。 |
| [`script/no-unused-emit-declarations`](#script-no-unused-emit-declarations) | [悪い](#script-no-unused-emit-declarations-bad) · [良い](#script-no-unused-emit-declarations-good) | 宣言したまま emit していないイベントを検出します。 |
| [`script/no-use-computed-property-like-method`](#script-no-use-computed-property-like-method) | [悪い](#script-no-use-computed-property-like-method-bad) · [良い](#script-no-use-computed-property-like-method-good) | Options API の computed プロパティをメソッドとして呼ぶ箇所を検出します。 |
| [`script/no-with-defaults`](#script-no-with-defaults) | [悪い](#script-no-with-defaults-bad) · [良い](#script-no-with-defaults-good) | Vue 3.5 以降の props 分割代入の既定値を勧めます。 |
| [`script/prefer-computed`](#script-prefer-computed) | [悪い](#script-prefer-computed-bad) · [良い](#script-prefer-computed-good) | 他の状態から導ける値を watcher で同期する代わりに computed で表現します。 |
| [`script/prefer-define-options`](#script-prefer-define-options) | [悪い](#script-prefer-define-options-bad) · [良い](#script-prefer-define-options-good) | name / inheritAttrs だけの通常 script を defineOptions() にまとめます。 |
| [`script/prefer-import-from-vue`](#script-prefer-import-from-vue) | [悪い](#script-prefer-import-from-vue-bad) · [良い](#script-prefer-import-from-vue-good) | 内部パッケージではなく vue から import します。 |
| [`script/prefer-ref-over-reactive`](#script-prefer-ref-over-reactive) | [悪い](#script-prefer-ref-over-reactive-bad) · [良い](#script-prefer-ref-over-reactive-good) | 状態管理に reactive() より ref() を使う方針を適用します。 |
| [`script/prefer-use-attrs`](#script-prefer-use-attrs) | [悪い](#script-prefer-use-attrs-bad) · [良い](#script-prefer-use-attrs-good) | setup の context.attrs を useAttrs() に置き換えます。 |
| [`script/prefer-use-id`](#script-prefer-use-id) | [悪い](#script-prefer-use-id-bad) · [良い](#script-prefer-use-id-good) | 一意な ID の生成に Vue 3.5 の useId() を使います。 |
| [`script/prefer-use-slots`](#script-prefer-use-slots) | [悪い](#script-prefer-use-slots-bad) · [良い](#script-prefer-use-slots-good) | setup の context.slots を useSlots() に置き換えます。 |
| [`script/prefer-use-template-ref`](#script-prefer-use-template-ref) | [悪い](#script-prefer-use-template-ref-bad) · [良い](#script-prefer-use-template-ref-good) | テンプレート参照に Vue 3.5 の useTemplateRef() を使います。 |
| [`script/require-default-prop`](#script-require-default-prop) | [悪い](#script-require-default-prop-bad) · [良い](#script-require-default-prop-good) | 任意指定で Boolean ではない prop に default を用意します。 |
| [`script/require-explicit-emits`](#script-require-explicit-emits) | [悪い](#script-require-explicit-emits-bad) · [良い](#script-require-explicit-emits-good) | emit するイベントを defineEmits または emits に宣言します。 |
| [`script/require-explicit-slots`](#script-require-explicit-slots) | [悪い](#script-require-explicit-slots-bad) · [良い](#script-require-explicit-slots-good) | useSlots() で使う slot を defineSlots の型で宣言します。 |
| [`script/require-function-return-type`](#script-require-function-return-type) | [悪い](#script-require-function-return-type-bad) · [良い](#script-require-function-return-type-good) | 関数に戻り値の型注釈を指定します。 |
| [`script/require-prop-type-constructor`](#script-require-prop-type-constructor) | [悪い](#script-require-prop-type-constructor-bad) · [良い](#script-require-prop-type-constructor-good) | prop の type に文字列ではなくコンストラクターを指定します。 |
| [`script/require-prop-types`](#script-require-prop-types) | [悪い](#script-require-prop-types-bad) · [良い](#script-require-prop-types-good) | 各 prop の型を宣言します。 |
| [`script/require-symbol-provide`](#script-require-symbol-provide) | [悪い](#script-require-symbol-provide-bad) · [良い](#script-require-symbol-provide-good) | provide / inject のキーに衝突しにくい Symbol を使います。 |
| [`script/require-typed-object-prop`](#script-require-typed-object-prop) | [悪い](#script-require-typed-object-prop-bad) · [良い](#script-require-typed-object-prop-good) | Object / Array の prop に具体的な型を指定します。 |
| [`script/require-typed-ref`](#script-require-typed-ref) | [悪い](#script-require-typed-ref-bad) · [良い](#script-require-typed-ref-good) | 空・null・undefined で初期化する ref() に型引数を指定します。 |
| [`script/require-valid-default-prop`](#script-require-valid-default-prop) | [悪い](#script-require-valid-default-prop-bad) · [良い](#script-require-valid-default-prop-good) | prop の default を宣言した型に合う値にします。 |
| [`script/return-in-computed-property`](#script-return-in-computed-property) | [悪い](#script-return-in-computed-property-bad) · [良い](#script-return-in-computed-property-good) | computed の getter に値を返す return を用意します。 |
| [`script/return-in-emits-validator`](#script-return-in-emits-validator) | [悪い](#script-return-in-emits-validator-bad) · [良い](#script-return-in-emits-validator-good) | Options API の emits validator に戻り値を用意します。 |
| [`script/valid-define-emits`](#script-valid-define-emits) | [悪い](#script-valid-define-emits-bad) · [良い](#script-valid-define-emits-good) | defineEmits の重複や型と実行時引数の併用を検出します。 |
| [`script/valid-define-options`](#script-valid-define-options) | [悪い](#script-valid-define-options-bad) · [良い](#script-valid-define-options-good) | defineOptions の引数と使用回数を検査します。 |
| [`script/valid-define-props`](#script-valid-define-props) | [悪い](#script-valid-define-props-bad) · [良い](#script-valid-define-props-good) | defineProps の重複や型と実行時引数の併用を検出します。 |
| [`script/valid-next-tick`](#script-valid-next-tick) | [悪い](#script-valid-next-tick-bad) · [良い](#script-valid-next-tick-good) | nextTick() の完了を await、then、または callback で扱います。 |
| [`type/no-floating-promises`](#type-no-floating-promises) | [悪い](#type-no-floating-promises-bad) · [良い](#type-no-floating-promises-good) | 処理しないまま放置された Promise を検出します。 |
| [`type/no-reactivity-loss`](#type-no-reactivity-loss) | [悪い](#type-no-reactivity-loss-bad) · [良い](#type-no-reactivity-loss-good) | 代入や呼び出しによって反応性を失うスナップショットを検出します。 |
| [`type/no-unsafe-template-binding`](#type-no-unsafe-template-binding) | [悪い](#type-no-unsafe-template-binding-bad) · [良い](#type-no-unsafe-template-binding-good) | テンプレートで安全でない型の値を使用する箇所を検出します。 |
| [`type/require-typed-emits`](#type-require-typed-emits) | [悪い](#type-require-typed-emits-bad) · [良い](#type-require-typed-emits-good) | defineEmits に型定義を指定します。 |
| [`type/require-typed-props`](#type-require-typed-props) | [悪い](#type-require-typed-props-bad) · [良い](#type-require-typed-props-good) | defineProps に型定義を指定します。 |
| [`type/strict-boolean-expressions`](#type-strict-boolean-expressions) | [悪い](#type-strict-boolean-expressions-bad) · [良い](#type-strict-boolean-expressions-good) | script とテンプレートの条件式で安全な真偽判定を使います。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `script/component-options-name-casing`

コンポーネントの name オプションを PascalCase に揃えます。

[悪い例](#script-component-options-name-casing-bad) · [良い例](#script-component-options-name-casing-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-component-options-name-casing-bad"></span>

**悪い**

コンポーネントの `name: 'my-component'` が kebab-case です。このルールは文字列リテラルのコンポーネント名に PascalCase を求めます。

```vue annotate="remove:3"
<script lang="ts">
export default {
name: 'my-component' // kebab-case
}
</script>
```

<span id="script-component-options-name-casing-good"></span>

**良い**

`MyComponent` は大文字で始まり、英数字だけで構成されるため、名前の検査条件を満たします。

```vue annotate="add:3"
<script lang="ts">
export default {
name: 'MyComponent'
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) · [全ルール](all.md)

### `script/custom-event-name-casing`

emit するカスタムイベント名を指定した形式に揃えます。

[悪い例](#script-custom-event-name-casing-bad) · [良い例](#script-custom-event-name-casing-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/custom-event-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-custom-event-name-casing-bad"></span>

**悪い**

emit する文字列 `my-event` にハイフンが含まれ、既定の camelCase イベント命名規則に違反しています。

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

<span id="script-custom-event-name-casing-good"></span>

**良い**

宣言と呼び出しの両方を `myEvent` にそろえ、イベント名の一致を保ったまま既定の命名規則を満たします。kebab-case に設定した場合の期待値は異なります。

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) · [全ルール](all.md)

### `script/define-emits-declaration`

defineEmits を型による宣言形式に揃えます。

[悪い例](#script-define-emits-declaration-bad) · [良い例](#script-define-emits-declaration-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-emits-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-emits-declaration-bad"></span>

**悪い**

`defineEmits(["change"])` は実行時の配列による宣言です。このスタイルルールは型ベースの宣言を推奨します。

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

<span id="script-define-emits-declaration-good"></span>

**良い**

`defineEmits<{ change: [id: number] }>()` で宣言を型引数へ移し、`emit("change", 1)` が渡す数値のペイロードも明示します。

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [全ルール](all.md)

### `script/define-macros-order`

script setup のコンパイラーマクロを一定の順に宣言します。

[悪い例](#script-define-macros-order-bad) · [良い例](#script-define-macros-order-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-macros-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-macros-order-bad"></span>

**悪い**

`defineProps` が `defineModel` より先にありますが、定められたマクロ順序では `defineModel` が先です。

```vue annotate="remove:2,3"
<script setup lang="ts">
// defineProps before defineModel (out of canonical order)
const props = defineProps<{ count: number }>()
const model = defineModel<string>()
</script>
```

<span id="script-define-macros-order-good"></span>

**良い**

宣言を `defineOptions`、`defineModel`、`defineProps`、`defineEmits`、`defineSlots` の順に並べ、無関係な実行時処理より前に置きます。

```vue annotate="add:2,4,5,6"
<script setup lang="ts">
defineOptions({ name: 'MyComponent' })
const model = defineModel<string>()
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
defineSlots<{ default(props: {}): any }>()
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L46) · [全ルール](all.md)

### `script/define-props-declaration`

defineProps を型による宣言形式に揃えます。

[悪い例](#script-define-props-declaration-bad) · [良い例](#script-define-props-declaration-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-declaration-bad"></span>

**悪い**

`defineProps({ title: String })` は実行時オブジェクトを渡しており、このルールが推奨する型ベースの props 宣言と異なります。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

<span id="script-define-props-declaration-good"></span>

**良い**

`defineProps<{ title: string }>()` の型引数で `title` を宣言し、実行時の宣言引数を使わずに `props.title` の参照を保ちます。

```vue annotate="add:2"
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) · [全ルール](all.md)

### `script/define-props-destructuring`

defineProps の分割代入スタイルを指定した方針に揃えます。

[悪い例](#script-define-props-destructuring-bad) · [良い例](#script-define-props-destructuring-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-destructuring": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-destructuring-bad"></span>

**悪い**

`defineProps` の結果を分割代入せず、単一の `props` 変数に代入しており、既定の分割代入の推奨に従っていません。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

<span id="script-define-props-destructuring-good"></span>

**良い**

オブジェクトパターンで `foo` と `bar` を直接取り出し、省略可能な `bar` に既定値を付けます。Vue 3.5 以降のリアクティブな props 分割代入を前提とし、設定を `never` にした場合は逆の形式を推奨します。

```vue annotate="add:2"
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) · [全ルール](all.md)

### `script/no-arrow-functions-in-watch`

Options API の watch に this を持たないアロー関数を使う箇所を検出します。

[悪い例](#script-no-arrow-functions-in-watch-bad) · [良い例](#script-no-arrow-functions-in-watch-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-arrow-functions-in-watch": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-arrow-functions-in-watch-bad"></span>

**悪い**

Options API の watcher `value` と `other.handler` がアロー関数です。アロー関数の `this` は外側から引き継がれ、コンポーネントインスタンスとして束縛されません。

```vue annotate="remove:4,5,9"
<script lang="ts">
export default {
watch: {
// `this` is not the component instance inside an arrow function.
value: () => {
this.doSomething()
},
other: {
handler: () => {}
}
}
}
</script>
```

<span id="script-no-arrow-functions-in-watch-good"></span>

**良い**

両ハンドラーを通常のメソッドに変え、Vue が `this` をコンポーネントに束縛できるようにします。オブジェクト形式の `deep: true` オプションも維持できます。

```vue annotate="add:4,8,9"
<script lang="ts">
export default {
watch: {
value(newValue, oldValue) {
this.doSomething()
},
other: {
handler(newValue) {},
deep: true
}
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) · [全ルール](all.md)

### `script/no-async-in-computed`

computed の getter で非同期関数を使う箇所を検出します。

[悪い例](#script-no-async-in-computed-bad) · [良い例](#script-no-async-in-computed-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-async-in-computed": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-async-in-computed-bad"></span>

**悪い**

`computed` の getter が `async` であり、fetch の結果を同期的な計算値ではなく Promise として返します。

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
import { computed } from "vue";
const data = computed(async () => {
  const response = await fetch("/api/data");
  return response.json();
});
</script>
```

<span id="script-no-async-in-computed-good"></span>

**良い**

非同期の fetch を `watch` に移し、結果を `data.value` に保存します。クリーンアップで古い要求を中止し、無効になったコールバックが古い結果を書き込むのを防ぎます。async な computed getter は残りません。

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
import { ref, watch } from "vue";
const query = ref("");
const data = ref<unknown>(null);
watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;
  onCleanup(() => { active = false; controller.abort(); });
  const response = await fetch(`/api/data?q=${encodeURIComponent(value)}`, { signal: controller.signal });
  const next: unknown = await response.json();
  if (active) data.value = next;
});
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) · [全ルール](all.md)

### `script/no-boolean-default`

Boolean prop の冗長な default を検出します。

[悪い例](#script-no-boolean-default-bad) · [良い例](#script-no-boolean-default-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-boolean-default": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-boolean-default-bad"></span>

**悪い**

`disabled` と `checked` は、単独の型が `Boolean` である prop に `default` を指定しています。明示的な `false` もこのルールの対象です。

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
props: {
// Boolean props already default to false; an explicit default is confusing.
disabled: { type: Boolean, default: true },
checked: { type: Boolean, default: false }
}
}
</script>
```

<span id="script-no-boolean-default-good"></span>

**良い**

Boolean のみの props では `default` を省き、Vue の暗黙の false を使います。`[Boolean, String]` の共用型と Number の prop は、この検査が単独の `Boolean` コンストラクターに限られることを示します。

```vue annotate="add:4,5,6,7,8,9,10"
<script lang="ts">
export default {
props: {
// No explicit default: defaults to false.
disabled: { type: Boolean },
disabled2: Boolean,
// Union type may legitimately need a default.
value: { type: [Boolean, String], default: '' },
// Non-Boolean prop.
count: { type: Number, default: 0 }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) · [全ルール](all.md)

### `script/no-deep-destructure-in-props`

defineProps の深い分割代入を検出します。

[悪い例](#script-no-deep-destructure-in-props-bad) · [良い例](#script-no-deep-destructure-in-props-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deep-destructure-in-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deep-destructure-in-props-bad"></span>

**悪い**

代入パターンが `user` の内部まで進んで `name` を取り出し、既定で許される浅い props 分割代入の深さを超えています。

```vue annotate="remove:2"
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

<span id="script-no-deep-destructure-in-props-good"></span>

**良い**

props オブジェクトを保ち、computed の getter で `props.user.name` を参照します。深い代入パターンを使わず、入れ子の参照を明示します。

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) · [全ルール](all.md)

### `script/no-deprecated-data-object-declaration`

Vue 3 で関数にすべき data オプションのオブジェクト指定を検出します。

[悪い例](#script-no-deprecated-data-object-declaration-bad) · [良い例](#script-no-deprecated-data-object-declaration-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-data-object-declaration": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-data-object-declaration-bad"></span>

**悪い**

Options API の `data` がオブジェクトリテラルであり、Vue 3 では受け付けない Vue 2 の形式です。

```vue annotate="remove:3,4,5"
<script lang="ts">
export default {
// `data` must be a function in Vue 3, not an object literal.
data: {
count: 0
}
}
</script>
```

<span id="script-no-deprecated-data-object-declaration-good"></span>

**良い**

`data()` が新しい `{ count: 0 }` を返すようにし、Vue 3 が求める関数形式のデータ宣言にします。

```vue annotate="add:3,4"
<script lang="ts">
export default {
data() {
return { count: 0 }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) · [全ルール](all.md)

### `script/no-deprecated-destroyed-lifecycle`

Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 の hook に置き換えます。

[悪い例](#script-no-deprecated-destroyed-lifecycle-bad) · [良い例](#script-no-deprecated-destroyed-lifecycle-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-destroyed-lifecycle": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

**悪い**

タイマーの後片付けに、Vue 3 で削除された Vue 2 のライフサイクルオプション `beforeDestroy` を使っています。

```vue annotate="remove:2"
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

**良い**

フック名を `beforeUnmount` に変え、後片付けの本体を Vue 3 のライフサイクル名で保ちます。

```vue annotate="add:2"
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) · [全ルール](all.md)

### `script/no-deprecated-dollar-listeners-api`

Vue 3 で $attrs に統合された $listeners を検出します。

[悪い例](#script-no-deprecated-dollar-listeners-api-bad) · [良い例](#script-no-deprecated-dollar-listeners-api-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

**悪い**

メンバー参照と引数の裸の参照がいずれも `$listeners` を使っています。Vue 3 ではリスナーが属性に統合され、この API は削除されました。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const handlers = this.$listeners
const forwarded = ctx.$listeners
emit('input', $listeners)
</script>
```

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

**良い**

参照を `this.$attrs` と setup コンテキストの `ctx.attrs` に移し、削除されたリスナー API を置き換えます。例の参照元は、それぞれのコンポーネントコンテキストで用意されている必要があります。

```vue annotate="add:2,3"
<script setup lang="ts">
const handlers = this.$attrs
const forwarded = ctx.attrs
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) · [全ルール](all.md)

### `script/no-deprecated-dollar-scopedslots-api`

Vue 3 で $slots に統合された $scopedSlots を検出します。

[悪い例](#script-no-deprecated-dollar-scopedslots-api-bad) · [良い例](#script-no-deprecated-dollar-scopedslots-api-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-scopedslots-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

**悪い**

`this.$scopedSlots`、`ctx.$scopedSlots`、裸の `$scopedSlots` 参照が、Vue 3 で削除された Vue 2 の scoped slot API を使っています。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

**良い**

`$scopedSlots` を `$slots` に置き換え、統合された slot API を使います。この例は削除された API 名の置換を示すもので、参照元の setup コンテキストを作る例ではありません。

```vue annotate="add:2,3,4"
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) · [全ルール](all.md)

### `script/no-deprecated-events-api`

Vue 3 で削除された $on / $off / $once を検出します。

[悪い例](#script-no-deprecated-events-api-bad) · [良い例](#script-no-deprecated-events-api-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-events-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-events-api-bad"></span>

**悪い**

`$on`、`$once`、`$off` の呼び出しが、Vue 3 で削除されたインスタンスのイベントバスメソッドを使っています。

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
this.$on('event', handler)
this.$once('event', handler)
this.$off('event', handler)
emitter.$off('event')
</script>
```

<span id="script-no-deprecated-events-api-good"></span>

**良い**

有効な `$emit` は維持し、イベントバスへの購読は外部 emitter の `on` に移します。親へのイベント送信と外部イベントバスを別々の API で表します。

```vue annotate="add:2,3,4,5,6,7,8"
<script setup lang="ts">
// $emit is still valid in Vue 3
this.$emit('event', payload)

// Use an external emitter instead
import mitt from 'mitt'
const emitter = mitt()
emitter.on('event', handler)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L42) · [全ルール](all.md)

### `script/no-deprecated-props-default-this`

prop の default / validator 内で使えなくなった this を検出します。

[悪い例](#script-no-deprecated-props-default-this-bad) · [良い例](#script-no-deprecated-props-default-this-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-props-default-this": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-props-default-this-bad"></span>

**悪い**

prop の既定値関数と validator が `this` を参照していますが、Vue 3 ではこれらの関数からコンポーネントインスタンスに依存できません。

```vue annotate="remove:6,7,8,13,14"
<script lang="ts">
export default {
props: {
size: {
type: Number,
// `this` is not the component instance in Vue 3.
default() {
return this.defaultSize
}
},
value: {
type: Number,
validator() {
return this.value > 0
}
}
}
}
</script>
```

<span id="script-no-deprecated-props-default-this-good"></span>

**良い**

既定値関数は引数の `props.baseSize` を使い、validator は引数の `value` を検査します。どちらも利用できないインスタンスの `this` に依存しなくなります。

```vue annotate="add:6,7,8,13,14"
<script lang="ts">
export default {
props: {
size: {
type: Number,
// Vue 3 passes the raw props as the first argument instead.
default(props) {
return props.baseSize
}
},
value: {
type: Number,
validator(value) {
return value > 0
}
}
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) · [全ルール](all.md)

### `script/no-dupe-keys`

Options API の props / data / computed などのキー重複を検出します。

[悪い例](#script-no-dupe-keys-bad) · [良い例](#script-no-dupe-keys-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-dupe-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-dupe-keys-bad"></span>

**悪い**

`foo` が props と data の両方に、`bar` が computed と methods の両方に宣言され、コンポーネントインスタンス上の同じキーを取り合っています。

```vue annotate="remove:5,8,9,10,11"
<script lang="ts">
export default {
props: ['foo'],
data() {
return { foo: 1 } // duplicate of prop `foo`
},
computed: {
bar() { return 2 }
},
methods: {
bar() {} // duplicate of computed `bar`
}
}
</script>
```

<span id="script-no-dupe-keys-good"></span>

**良い**

prop、data、computed に別々の名前 `foo`、`bar`、`baz` を使い、オプション間の重複を取り除きます。

```vue annotate="add:5,8"
<script lang="ts">
export default {
props: ['foo'],
data() {
return { bar: 1 }
},
computed: {
baz() { return 2 }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L52) · [全ルール](all.md)

### `script/no-duplicate-attr-inheritance`

fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。

[悪い例](#script-no-duplicate-attr-inheritance-bad) · [良い例](#script-no-duplicate-attr-inheritance-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-duplicate-attr-inheritance-bad"></span>

**悪い**

明示的な `inheritAttrs: true` が Vue の既定動作を繰り返しています。このルールは、ルートの `$attrs` 展開が例にない場合もこの冗長なリテラルを報告します。

```vue annotate="remove:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: true })
export default { inheritAttrs: true }
</script>
```

<span id="script-no-duplicate-attr-inheritance-good"></span>

**良い**

`inheritAttrs: false` は継承を無効にする指定であり、空のオプションは既定の継承を暗黙に使います。どちらも冗長な `true` を指定しません。

```vue annotate="add:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: false }) // intentional opt-out
export default {}                      // default inheritance, unstated
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L73) · [全ルール](all.md)

### `script/no-export-in-script-setup`

script setup 内の export 文を検出します。

[悪い例](#script-no-export-in-script-setup-bad) · [良い例](#script-no-export-in-script-setup-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-export-in-script-setup-bad"></span>

**悪い**

`export const count` が `<script setup>` からモジュールの値をエクスポートしようとしています。このブロックでは実行時の export は禁止されています。

```vue annotate="remove:2"
<script setup lang="ts">
export const count = 1;
</script>
```

<span id="script-no-export-in-script-setup-good"></span>

**良い**

`export` を取り除き、`count` をモジュールの export ではなく setup の変数にします。

```vue annotate="add:2"
<script setup lang="ts">
const count = 1;
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) · [全ルール](all.md)

### `script/no-get-current-instance`

Vapor で null を返す getCurrentInstance() を検出します。

[悪い例](#script-no-get-current-instance-bad) · [良い例](#script-no-get-current-instance-good)

既定の重大度: `error`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-get-current-instance-bad"></span>

**悪い**

Vapor を指定した setup が `getCurrentInstance` をインポートして呼び出し、Vapor 向けコンポーネントでこのルールが禁止するインスタンス API に依存しています。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**良い**

`inject("app-config")` で明示的に提供された設定を受け取り、`getCurrentInstance` のインポートも呼び出しも使いません。

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [全ルール](all.md)

### `script/no-import-compiler-macros`

自動的に使える Vue コンパイラーマクロの import を検出します。

[悪い例](#script-no-import-compiler-macros-bad) · [良い例](#script-no-import-compiler-macros-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-import-compiler-macros": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-import-compiler-macros-bad"></span>

**悪い**

`vue` から `defineProps` と `defineEmits` をインポートしていますが、これらは `<script setup>` で直接使えるコンパイラーマクロです。

```vue annotate="remove:2"
<script setup lang="ts">
import { defineProps, defineEmits } from "vue";
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

<span id="script-no-import-compiler-macros-good"></span>

**良い**

マクロのインポートを除き、型付きのマクロ呼び出しは保ちます。これらの宣言に実行時のインポートは不要です。

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) · [全ルール](all.md)

### `script/no-internal-imports`

Vue 内部モジュールからの import を検出します。

[悪い例](#script-no-internal-imports-bad) · [良い例](#script-no-internal-imports-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-internal-imports": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-internal-imports-bad"></span>

**悪い**

両方の import が Vue の公開エントリーポイントではなく内部の `dist` ファイルを参照し、コンポーネントをビルド成果物のパスに依存させています。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
import { bar } from 'vue/dist/vue.esm-bundler'
</script>
```

<span id="script-no-internal-imports-good"></span>

**良い**

必要なヘルパーを `vue` からインポートし、内部の配布ファイルの配置への依存を取り除きます。

```vue annotate="add:2"
<script setup lang="ts">
import { ref, computed } from 'vue'
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) · [全ルール](all.md)

### `script/no-multiple-slot-args`

scoped slot 関数に複数の引数を渡す箇所を検出します。

[悪い例](#script-no-multiple-slot-args-bad) · [良い例](#script-no-multiple-slot-args-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-multiple-slot-args": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-multiple-slot-args-bad"></span>

**悪い**

slot 呼び出しが複数の位置引数、または個数不明の引数展開を渡しています。Vue の slot は位置引数の列ではなく、一つの props オブジェクトを受け取ります。

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

<span id="script-no-multiple-slot-args-good"></span>

**良い**

`{ foo, bar }` でデータを一つの引数にまとめます。`slotProps` を渡す呼び出しと引数を省いた呼び出しも、対応する slot 呼び出し形式に収まります。

```vue annotate="add:2,3,4"
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) · [全ルール](all.md)

### `script/no-next-tick`

Vapor 向けコンポーネントの nextTick() 使用を検出します。

[悪い例](#script-no-next-tick-bad) · [良い例](#script-no-next-tick-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-next-tick-bad"></span>

**悪い**

Vapor 向けコンポーネントが `nextTick` をインポートして await し、この移行ルールが拒否する DOM 更新待ちの依存を作っています。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**良い**

`useTemplateRef` で input を取得し、`onMounted` でフォーカスします。例の `nextTick` への依存を、明示的なマウント時の処理に置き換えます。

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [全ルール](all.md)

### `script/no-options-api`

Vapor で Options API を使用する箇所を検出します。

[悪い例](#script-no-options-api-bad) · [良い例](#script-no-options-api-good)

既定の重大度: `error`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-options-api-bad"></span>

**悪い**

default export のオブジェクトに Options API の `data()` を宣言しており、このルールが禁止するコンポーネントオプションの形式です。

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

<span id="script-no-options-api-good"></span>

**良い**

状態を Vapor の `<script setup>` 内の Composition API `ref` に移し、Options API のオブジェクトと `data` オプションを取り除きます。

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [全ルール](all.md)

### `script/no-potential-component-option-typo`

Options API のオプション名の入力ミスを検出します。

[悪い例](#script-no-potential-component-option-typo-bad) · [良い例](#script-no-potential-component-option-typo-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-potential-component-option-typo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-potential-component-option-typo-bad"></span>

**悪い**

オプション名が既知の `methods` から一文字欠けた `method` になり、意図したメソッド宣言として扱われません。

```vue annotate="remove:2"
<script lang="ts">
export default { method: { save() {} } };
</script>
```

<span id="script-no-potential-component-option-typo-good"></span>

**良い**

キーを `methods` に修正し、`save()` を既知のコンポーネントオプション内に置きます。

```vue annotate="add:2"
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) · [全ルール](all.md)

### `script/no-reactive-destructure`

reactive オブジェクトの反応性を失う分割代入を検出します。

[悪い例](#script-no-reactive-destructure-bad) · [良い例](#script-no-reactive-destructure-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reactive-destructure": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reactive-destructure-bad"></span>

**悪い**

`const { count, name } = state` が `reactive` オブジェクトからプリミティブ値をコピーし、その後のプロパティ更新とのつながりを失います。

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = state;
</script>
```

<span id="script-no-reactive-destructure-good"></span>

**良い**

`toRefs(state)` を分割代入して `count` と `name` の ref を作り、それぞれを元のリアクティブなプロパティにつなげます。

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRefs } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = toRefs(state);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) · [全ルール](all.md)

### `script/no-ref-as-operand`

ref を演算の値として使う際に .value を参照します。

[悪い例](#script-no-ref-as-operand-bad) · [良い例](#script-no-ref-as-operand-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-ref-as-operand": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-ref-as-operand-bad"></span>

**悪い**

`count + 1` は ref が包む数値ではなく、ref オブジェクトそのものを算術演算の対象にしています。

```vue annotate="remove:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count + 1;
</script>
```

<span id="script-no-ref-as-operand-good"></span>

**良い**

`count.value + 1` で内側の数値を取り出してから加算します。script の演算では ref の値を明示的に参照します。

```vue annotate="add:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count.value + 1;
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) · [全ルール](all.md)

### `script/no-required-prop-with-default`

required: true と default を同時に持つ prop を検出します。

[悪い例](#script-no-required-prop-with-default-bad) · [良い例](#script-no-required-prop-with-default-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-required-prop-with-default": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-required-prop-with-default-bad"></span>

**悪い**

`title` に必須指定と `"Untitled"` の既定値を同時に付け、必須の入力という契約と未指定時の代替値を併記しています。

```vue annotate="remove:2"
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

<span id="script-no-required-prop-with-default-good"></span>

**良い**

`required: true` を除いて `title` を任意入力にし、`"Untitled"` を未指定時の既定値として残します。

```vue annotate="add:2"
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L29) · [全ルール](all.md)

### `script/no-reserved-identifiers`

Vue コンパイラーが予約した識別子の宣言を検出します。

[悪い例](#script-no-reserved-identifiers-bad) · [良い例](#script-no-reserved-identifiers-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-identifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-identifiers-bad"></span>

**悪い**

変数名 `__props`、`__emit`、`__sfc__` が Vue コンパイラーの生成コード用に予約された識別子と重なっています。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const __props = { name: "Ada" };
const __emit = () => {};
const __sfc__ = {};
</script>
```

<span id="script-no-reserved-identifiers-good"></span>

**良い**

通常の名前 `props`、`emit`、`componentData` を使い、props と emits の宣言を保ったまま生成用識別子との重複を避けます。

```vue annotate="add:2,3,4"
<script setup lang="ts">
const props = defineProps<{ name: string }>();
const emit = defineEmits<{ save: [] }>();
const componentData = {};
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) · [全ルール](all.md)

### `script/no-reserved-keys`

Options API のキーに Vue の予約名を使う箇所を検出します。

[悪い例](#script-no-reserved-keys-bad) · [良い例](#script-no-reserved-keys-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-keys-bad"></span>

**悪い**

data の `$el` が Vue 組み込みのインスタンスプロパティと重なり、予約された `$` 接頭辞も使っています。

```vue annotate="remove:2"
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

<span id="script-no-reserved-keys-good"></span>

**良い**

アプリケーションのデータ名を `elementLabel` に変え、組み込みのインスタンス API と予約接頭辞を避けます。

```vue annotate="add:2"
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L32) · [全ルール](all.md)

### `script/no-reserved-props`

prop 宣言に Vue の予約名を使う箇所を検出します。

[悪い例](#script-no-reserved-props-bad) · [良い例](#script-no-reserved-props-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-props-bad"></span>

**悪い**

オブジェクト形式の `ref` と `$foo`、配列形式の `key` は予約された prop 名です。`ref` と `key` はフレームワーク用であり、`$` で始まる名前も拒否されます。

```vue annotate="remove:4,5,6,8,9,10"
<script lang="ts">
export default {
props: {
ref: String,   // reserved
$foo: Number    // `$`-prefixed names are reserved
}
}

export default {
props: ['key']    // reserved (array form)
}
</script>
```

<span id="script-no-reserved-props-good"></span>

**良い**

通常の prop 名 `name` と `refValue` に変え、予約された名前と接頭辞を避けます。

```vue annotate="add:4,5"
<script lang="ts">
export default {
props: {
name: String,
refValue: Number
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) · [全ルール](all.md)

### `script/no-restricted-globals`

設定で禁止した実行環境のグローバル参照を検出します。

[悪い例](#script-no-restricted-globals-bad) · [良い例](#script-no-restricted-globals-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-globals": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-restricted-globals-bad"></span>

**悪い**

既定で制限されるグローバル `process`、`localStorage`、`sessionStorage` を直接参照し、設定やストレージ用の明示的なヘルパーを通していません。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

<span id="script-no-restricted-globals-good"></span>

**良い**

`useFeatureFlag`、`authStorage.read`、`viewStorage.write` に移し、制限対象のグローバルの直接参照を除きます。残る `window.scrollY` はこのルールの既定の制限対象ではなく、SSR の安全性は別途確認が必要です。

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) · [全ルール](all.md)

### `script/no-restricted-members`

設定で禁止した object.property へのアクセスを検出します。

[悪い例](#script-no-restricted-members-bad) · [良い例](#script-no-restricted-members-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

この例では window.localStorage を禁止しています。既定の禁止リストはなく、有効にするだけでは検出されません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage"
            }
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-restricted-members-bad"></span>

**悪い**

`ruleOptions` に `{ object: "window", property: "localStorage" }` を設定した場合、`window.localStorage` は禁止されたオブジェクトとメンバーの組み合わせです。このルールに既定の禁止メンバーはありません。

```vue annotate="remove:2"
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

<span id="script-no-restricted-members-good"></span>

**良い**

`authStorage.read("token")` でアプリケーションのストレージヘルパーに処理を任せ、設定で禁止した `window.localStorage` を参照しなくなります。

```vue annotate="add:2"
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) · [全ルール](all.md)

### `script/no-side-effects-in-computed-properties`

Options API の computed getter 内の副作用を検出します。

[悪い例](#script-no-side-effects-in-computed-properties-bad) · [良い例](#script-no-side-effects-in-computed-properties-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-side-effects-in-computed-properties": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-side-effects-in-computed-properties-bad"></span>

**悪い**

`doubled` が `this.count` に代入し、`reversed` が `reverse()` で `this.items` を変更しています。どちらも値を導出すべき getter が元の状態を変更しています。

```vue annotate="remove:8,9,12"
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
this.count = this.count * 2 // side effect: assigns to data
return this.count
},
reversed() {
return this.items.reverse() // side effect: mutates the array
}
}
}
</script>
```

<span id="script-no-side-effects-in-computed-properties-good"></span>

**良い**

`doubled` は代入せず乗算結果を返します。`reversed` は配列をコピーしてから反転し、getter が元のコンポーネント状態を変更しないようにします。

```vue annotate="add:8,11"
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
return this.count * 2
},
reversed() {
return [...this.items].reverse() // operate on a copy
}
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L70) · [全ルール](all.md)

### `script/no-top-level-ref-in-script`

通常の script のトップレベルで、リクエスト間に共有される状態を作る箇所を検出します。

[悪い例](#script-no-top-level-ref-in-script-bad) · [良い例](#script-no-top-level-ref-in-script-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-top-level-ref-in-script": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-top-level-ref-in-script-bad"></span>

**悪い**

通常の `<script>` がモジュール直下で `count` と `user` を初期化しています。SSR では、この状態がコンポーネントインスタンスやリクエスト間で共有され得ます。

```vue annotate="remove:1,2,4,8"
<script>
// This state is shared across all requests in SSR!
const count = ref(0)
const user = reactive({ name: '' })

export default {
setup() {
return { count, user }
}
}
</script>
```

<span id="script-no-top-level-ref-in-script-good"></span>

**良い**

script setup の ref はコンポーネントごとに初期化されます。通常の script には定数、状態を作る関数、`setup()` 内で作る ref を置き、モジュール直下でリアクティブな状態を作りません。

```vue annotate="add:1,2,4,6,7,8,9,10,11,12,13,14,17,18,19"
<script setup>
// Script setup creates fresh state per request
const count = ref(0)
</script>

<script>
// Constants are fine
const API_URL = 'https://api.example.com'

// Functions that create state are fine
function createState() {
return reactive({ count: 0 })
}

export default {
setup() {
// Create state inside setup
const count = ref(0)
return { count }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) · [全ルール](all.md)

### `script/no-unstable-nested-components`

setup / render 内で毎回コンポーネントを定義する箇所を検出します。

[悪い例](#script-no-unstable-nested-components-bad) · [良い例](#script-no-unstable-nested-components-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unstable-nested-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unstable-nested-components-bad"></span>

**悪い**

親の `setup()` 内で `defineComponent` を呼び、setup が実行されるたびに新しい `Child` のコンポーネント定義を作っています。

```vue annotate="remove:3"
<script lang="ts">
import { defineComponent } from "vue";
export default { setup() { const Child = defineComponent({ render() { return null; } }); return { Child }; } };
</script>
```

<span id="script-no-unstable-nested-components-good"></span>

**良い**

`Child` の定義をモジュール直下に移し、`setup()` は作り直さず既存の定義を返すようにします。

```vue annotate="add:3,4"
<script lang="ts">
import { defineComponent } from "vue";
const Child = defineComponent({ render() { return null; } });
export default { setup() { return { Child }; } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) · [全ルール](all.md)

### `script/no-unused-emit-declarations`

宣言したまま emit していないイベントを検出します。

[悪い例](#script-no-unused-emit-declarations-bad) · [良い例](#script-no-unused-emit-declarations-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unused-emit-declarations": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unused-emit-declarations-bad"></span>

**悪い**

`defineEmits` は `change` と `unused` を宣言していますが、受け取った `emit` 関数が送信する文字列イベントは `change` だけです。

```vue annotate="remove:2,4"
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

<span id="script-no-unused-emit-declarations-good"></span>

**良い**

`unused` を除き、イベント宣言を実際の送信にそろえます。この例では emit の参照を外部へ渡していないため、ローカルの使用状況から判断できます。

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) · [全ルール](all.md)

### `script/no-use-computed-property-like-method`

Options API の computed プロパティをメソッドとして呼ぶ箇所を検出します。

[悪い例](#script-no-use-computed-property-like-method-bad) · [良い例](#script-no-use-computed-property-like-method-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-use-computed-property-like-method": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-use-computed-property-like-method-bad"></span>

**悪い**

`this.total()` は computed getter の公開する値を関数として呼び出しています。この getter が返す `3` は呼び出せません。

```vue annotate="remove:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total()); } } };
</script>
```

<span id="script-no-use-computed-property-like-method-good"></span>

**良い**

呼び出しの括弧を除いて `this.total` とし、`log` から計算済みの数値を参照します。

```vue annotate="add:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total); } } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) · [全ルール](all.md)

### `script/no-with-defaults`

Vue 3.5 以降の props 分割代入の既定値を勧めます。

[悪い例](#script-no-with-defaults-bad) · [良い例](#script-no-with-defaults-good)

既定の重大度: `warning`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-with-defaults": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-with-defaults-bad"></span>

**悪い**

型付きの props 宣言を `withDefaults` で包んで `count` と `name` の既定値を指定しており、ここで推奨する Vue 3.5 以降の分割代入の既定値を使っていません。

```vue annotate="remove:2"
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

<span id="script-no-with-defaults-good"></span>

**良い**

分割代入の変数に `count = 0` と `name = "Ada"` を直接指定し、`withDefaults` のラッパーを取り除きます。

```vue annotate="add:2"
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) · [全ルール](all.md)

### `script/prefer-computed`

他の状態から導ける値を watcher で同期する代わりに computed で表現します。

[悪い例](#script-prefer-computed-bad) · [良い例](#script-prefer-computed-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

派生値だけを代入する watcher が対象です。ユーザーが編集するコピーや別の副作用を持つ処理は対象外です。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-computed": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-computed-bad"></span>

**悪い**

watcher が `count` から求めた値を別の ref `doubled` に書き込むだけであり、派生状態を手作業で同期しています。

```vue annotate="remove:2,4,5"
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

<span id="script-prefer-computed-good"></span>

**良い**

`computed(() => count.value * 2)` で導出を直接表し、書き込み可能な追加 ref と同期用 watcher を取り除きます。

```vue annotate="add:2,4"
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) · [全ルール](all.md)

### `script/prefer-define-options`

name / inheritAttrs だけの通常 script を defineOptions() にまとめます。

[悪い例](#script-prefer-define-options-bad) · [良い例](#script-prefer-define-options-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-define-options": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-define-options-bad"></span>

**悪い**

通常の script の処理が `name` と `inheritAttrs` だけを持つオブジェクトの export に限られ、`defineOptions` で表せるオプションだけを宣言しています。

```vue annotate="remove:2"
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

<span id="script-prefer-define-options-good"></span>

**良い**

例の `data()` が実際の Options API の処理を持つため、オプションだけの script を対象とする慎重な提案の範囲外になります。この Good は許可される例外を示します。直接移行する場合は `<script setup>` 内で `defineOptions({ name: 'MyComponent', inheritAttrs: false })` を使います。

```vue annotate="add:2,3,4,5,6"
<script lang="ts">
// Real options logic — keep the plain script.
export default {
name: 'MyComponent',
data() { return { count: 0 } },
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) · [全ルール](all.md)

### `script/prefer-import-from-vue`

内部パッケージではなく vue から import します。

[悪い例](#script-prefer-import-from-vue-bad) · [良い例](#script-prefer-import-from-vue-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: 対応する検出で利用可能  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-import-from-vue": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-import-from-vue-bad"></span>

**悪い**

`ref` と `h` を公開パッケージ `vue` ではなく、内部の `@vue/runtime-core` と `@vue/runtime-dom` からインポートしています。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

<span id="script-prefer-import-from-vue-good"></span>

**良い**

両ヘルパーを `vue` からまとめてインポートし、内部パッケージではなく公開エントリーポイントを使います。

```vue annotate="add:2"
<script setup lang="ts">
import { ref, h } from 'vue'
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L32) · [全ルール](all.md)

### `script/prefer-ref-over-reactive`

状態管理に reactive() より ref() を使う方針を適用します。

[悪い例](#script-prefer-ref-over-reactive-bad) · [良い例](#script-prefer-ref-over-reactive-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-ref-over-reactive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-ref-over-reactive-bad"></span>

**悪い**

状態を `reactive` で作り、この意見を持つルールが推奨する ref を使っていません。これはスタイルの推奨を示す例であり、reactive オブジェクト自体が不正という意味ではありません。

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
// reactive requires careful handling to avoid losing reactivity
const state = reactive({
count: 0,
name: 'foo'
})
</script>
```

<span id="script-prefer-ref-over-reactive-good"></span>

**良い**

スカラーとオブジェクトの状態をどちらも `ref` で作ります。関連するフィールドを個別の ref に分ける例も含め、推奨する状態の作成形式にそろえます。

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
// ref is more explicit and safer
const count = ref(0)
const name = ref('foo')

// For objects, ref still works
const user = ref({ name: 'foo', age: 20 })

// Or use multiple refs for related data
const userName = ref('foo')
const userAge = ref(20)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) · [全ルール](all.md)

### `script/prefer-use-attrs`

setup の context.attrs を useAttrs() に置き換えます。

[悪い例](#script-prefer-use-attrs-bad) · [良い例](#script-prefer-use-attrs-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-attrs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-attrs-bad"></span>

**悪い**

`setup` がコンテキスト引数の分割代入で `attrs` を受け取っています。このルールは Composition API のヘルパーへの置換を求めます。

```vue annotate="remove:2"
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

<span id="script-prefer-use-attrs-good"></span>

**良い**

setup 内で `useAttrs()` から `attrs` を取得し、第二引数に依存せず `attrs.class` の参照を保ちます。

```vue annotate="add:2,3"
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) · [全ルール](all.md)

### `script/prefer-use-id`

一意な ID の生成に Vue 3.5 の useId() を使います。

[悪い例](#script-prefer-use-id-bad) · [良い例](#script-prefer-use-id-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-id-bad"></span>

**悪い**

`id` に `Math.random()` の値が含まれ、input と label の識別子がサーバーとクライアントの描画で変わり得ます。ID の名前を持つこの変数が、ルールの認識する生成箇所です。

```vue annotate="remove:2"
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

<span id="script-prefer-use-id-good"></span>

**良い**

Vue 3.5 以降の `useId()` で識別子を作り、`:for` と `:id` は同じ変数を参照し続けます。ランダムな値の生成を取り除きます。

```vue annotate="add:2,3"
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) · [全ルール](all.md)

### `script/prefer-use-slots`

setup の context.slots を useSlots() に置き換えます。

[悪い例](#script-prefer-use-slots-bad) · [良い例](#script-prefer-use-slots-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-slots-bad"></span>

**悪い**

`setup` がコンテキスト引数から `slots` を分割代入で取り出しており、このルールが置換を推奨する参照形式です。

```vue annotate="remove:2,4"
<script lang="ts">
import { defineComponent, h } from "vue";
export default defineComponent({
  setup(_props, { slots }) { return () => h("div", slots.default?.()); },
});
</script>
```

<span id="script-prefer-use-slots-good"></span>

**良い**

setup 内で `useSlots()` から slot を取得し、コンテキスト引数を使わずに render 関数と default slot の任意の呼び出しを保ちます。

```vue annotate="add:2,4,5,6,7"
<script lang="ts">
import { defineComponent, h, useSlots } from "vue";
export default defineComponent({
  setup() {
    const slots = useSlots();
    return () => h("div", slots.default?.());
  },
});
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) · [全ルール](all.md)

### `script/prefer-use-template-ref`

テンプレート参照に Vue 3.5 の useTemplateRef() を使います。

[悪い例](#script-prefer-use-template-ref-bad) · [良い例](#script-prefer-use-template-ref-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-template-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-template-ref-bad"></span>

**悪い**

null で初期化した `input` の ref がテンプレートのリテラル `ref="input"` と対応し、通常の nullable なデータではなく要素参照であることが分かります。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from 'vue'
const input = ref<HTMLInputElement | null>(null)
</script>
<template>
<input ref="input" />
</template>
```

<span id="script-prefer-use-template-ref-good"></span>

**良い**

Vue 3.5 以降の `useTemplateRef<HTMLInputElement>('input')` でテンプレート参照を明示します。対応する要素参照のない `error = ref(null)` は通常のデータであり、このルールの対象外です。

```vue annotate="add:2,3,4,5,6,10"
<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'
// Paired with the template ref below.
const input = useTemplateRef<HTMLInputElement>('input')
// A nullable data ref the template never binds as a ref.
const error = ref(null)
</script>
<template>
<input ref="input" />
<p>{{ error }}</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) · [全ルール](all.md)

### `script/require-default-prop`

任意指定で Boolean ではない prop に default を用意します。

[悪い例](#script-require-default-prop-bad) · [良い例](#script-require-default-prop-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-default-prop-bad"></span>

**悪い**

`name` と `age` は必須でない Boolean 以外の実行時 props で、未指定時の既定値がありません。

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
props: {
// optional, non-Boolean, no default
name: String,
age: { type: Number },
}
}
</script>
```

<span id="script-require-default-prop-good"></span>

**良い**

`name` に `default: ''` を付けます。`enabled` は Boolean の暗黙の false を使い、必須の `id` には代替値が不要であるため、二つの除外条件も示しています。

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
props: {
name: { type: String, default: '' },
enabled: Boolean,                 // Boolean defaults to false
id: { type: Number, required: true },
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) · [全ルール](all.md)

### `script/require-explicit-emits`

emit するイベントを defineEmits または emits に宣言します。

[悪い例](#script-require-explicit-emits-bad) · [良い例](#script-require-explicit-emits-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-emits": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-emits-bad"></span>

**悪い**

受け取った emit 関数が `save` を送信しますが、`defineEmits([])` にはそのイベントが宣言されていません。

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

<span id="script-require-explicit-emits-good"></span>

**良い**

宣言に `"save"` を追加し、送信する文字列イベントをコンポーネントの明示的なイベント契約に含めます。

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) · [全ルール](all.md)

### `script/require-explicit-slots`

useSlots() で使う slot を defineSlots の型で宣言します。

[悪い例](#script-require-explicit-slots-bad) · [良い例](#script-require-explicit-slots-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-slots-bad"></span>

**悪い**

型付きの `defineProps<{ id: number }>()` により TypeScript の構文があることが分かりますが、setup が `defineSlots` の宣言なしに `useSlots()` を使い、参照する slot の明示的な契約がありません。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ id: number }>()
const slots = useSlots()
</script>
```

<span id="script-require-explicit-slots-good"></span>

**良い**

`defineSlots` で `msg: string` を props に持つ `default` slot を宣言し、`useSlots()` と明示的な型付き slot 契約を併記します。

```vue annotate="add:2"
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) · [全ルール](all.md)

### `script/require-function-return-type`

関数に戻り値の型注釈を指定します。

[悪い例](#script-require-function-return-type-bad) · [良い例](#script-require-function-return-type-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-function-return-type": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-function-return-type-bad"></span>

**悪い**

`add` と `greet` は引数の型を指定していますが、戻り値の型を省略しています。この明示的な型指定の規則では、戻り値の推論だけでは条件を満たしません。

```vue annotate="remove:2,6"
<script setup lang="ts">
const add = (a: number, b: number) => {
return a + b
}

function greet(name: string) {
return `Hello, ${name}`
}
</script>
```

<span id="script-require-function-return-type-good"></span>

**良い**

`add` に `: number`、`greet` に `: string` を付け、本体を変えずに戻り値の契約を明示します。

```vue annotate="add:2,6"
<script setup lang="ts">
const add = (a: number, b: number): number => {
return a + b
}

function greet(name: string): string {
return `Hello, ${name}`
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) · [全ルール](all.md)

### `script/require-prop-type-constructor`

prop の type に文字列ではなくコンストラクターを指定します。

[悪い例](#script-require-prop-type-constructor-bad) · [良い例](#script-require-prop-type-constructor-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-type-constructor": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-type-constructor-bad"></span>

**悪い**

props の実行時の型に文字列 `"String"` と `"Number"` を使い、コンストラクターの配列にも文字列を入れています。これらの文字列はコンストラクター関数ではありません。

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
props: {
// The type should be the `String` constructor, not the string "String".
name: "String",
age: { type: "Number" },
id: { type: ["String", "Number"] }
}
}
</script>
```

<span id="script-require-prop-type-constructor-good"></span>

**良い**

型を実際の `String` と `Number` の識別子にし、共用型の配列も `[String, Number]` に変えます。

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
props: {
name: String,
age: { type: Number },
id: { type: [String, Number] }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L59) · [全ルール](all.md)

### `script/require-prop-types`

各 prop の型を宣言します。

[悪い例](#script-require-prop-types-bad) · [良い例](#script-require-prop-types-good)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-types": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-types-bad"></span>

**悪い**

配列の要素は `status` という名前だけを宣言し、`null` の値と空の descriptor にも実行時の prop 型がありません。

```vue annotate="remove:3,4,5,6,8,9"
<script lang="ts">
export default {
props: ['status']            // array form: no types
}

export default {
props: {
status: null,              // no type
other: {}                  // empty descriptor: no type
}
}
</script>
```

<span id="script-require-prop-types-good"></span>

**良い**

`status: String` で省略形式のコンストラクターを指定し、`other` の descriptor に `type: Number` を付けます。両方の props が型の宣言を持つようになります。

```vue annotate="add:4,5"
<script lang="ts">
export default {
props: {
status: String,
other: { type: Number, default: 0 }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) · [全ルール](all.md)

### `script/require-symbol-provide`

provide / inject のキーに衝突しにくい Symbol を使います。

[悪い例](#script-require-symbol-provide-bad) · [良い例](#script-require-symbol-provide-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-symbol-provide": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-symbol-provide-bad"></span>

**悪い**

`provide` と `inject` が `'user'`、`'theme'` のような文字列キーを使い、同じ綴りを使うほかの provider と衝突し得ます。

```vue annotate="remove:1,2,3,4,6,7"
<script setup lang="ts">
// String keys can collide
provide('user', user)
const user = inject('user')

// Magic strings are error-prone
provide('theme', { dark: true })
</script>
```

<span id="script-require-symbol-provide-good"></span>

**良い**

共有する `UserKey` を `Symbol` で作り、`InjectionKey<User>` の型を付けます。両呼び出しに同じキーを渡し、文字列リテラルを使わないようにします。

```vue annotate="add:1,2,3,5,6,7,8,9"
<script lang="ts">
// Define injection key with Symbol
export const UserKey: InjectionKey<User> = Symbol('user')

// Provide with Symbol
provide(UserKey, user)

// Inject with Symbol
const user = inject(UserKey)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) · [全ルール](all.md)

### `script/require-typed-object-prop`

Object / Array の prop に具体的な型を指定します。

[悪い例](#script-require-typed-object-prop-bad) · [良い例](#script-require-typed-object-prop-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-object-prop": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-object-prop-bad"></span>

**悪い**

裸の `Object` と `Array` は広い実行時の分類しか表さず、`user` や `items` の要素の形を明示する静的な型がありません。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

<span id="script-require-typed-object-prop-good"></span>

**良い**

`PropType<User>` と `PropType<User[]>` を付け、実行時のコンストラクターを保ったままオブジェクトと要素の型を指定します。

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [全ルール](all.md)

### `script/require-typed-ref`

空・null・undefined で初期化する ref() に型引数を指定します。

[悪い例](#script-require-typed-ref-bad) · [良い例](#script-require-typed-ref-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-ref-bad"></span>

**悪い**

インポートした `ref` に型引数がなく、引数なし、`null`、`undefined` からは将来代入する値の型を推論できません。

```vue annotate="remove:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref()           // Ref<undefined>
const b = ref(null)       // Ref<null>
const c = ref(undefined)  // Ref<undefined>
</script>
```

<span id="script-require-typed-ref-good"></span>

**良い**

型引数で string と nullable な User の ref を指定します。`ref(0)` には具体的な数値の初期値があり、型推論を使えます。

```vue annotate="add:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref<string>()
const b = ref<User | null>(null)
const c = ref(0)          // inferred Ref<number>
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) · [全ルール](all.md)

### `script/require-valid-default-prop`

prop の default を宣言した型に合う値にします。

[悪い例](#script-require-valid-default-prop-bad) · [良い例](#script-require-valid-default-prop-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-valid-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-valid-default-prop-bad"></span>

**悪い**

Number と Boolean の props に型の合わないスカラー既定値を付け、Array と Object には factory ではなく共有されるリテラル値を使っています。

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
props: {
count: { type: Number, default: '0' },     // string default for Number
enabled: { type: Boolean, default: 1 },     // non-boolean default for Boolean
items: { type: Array, default: [] },        // literal must be a factory
config: { type: Object, default: {} }       // literal must be a factory
}
}
</script>
```

<span id="script-require-valid-default-prop-good"></span>

**良い**

スカラーの既定値を `0` と `false` にし、配列とオブジェクトの既定値を新しい値を返す関数にします。`[String, Number]` の文字列既定値は、宣言した型の一つに合うため許可されます。

```vue annotate="add:4,5,6,7,8"
<script lang="ts">
export default {
props: {
count: { type: Number, default: 0 },
enabled: { type: Boolean, default: false },
items: { type: Array, default: () => [] },
config: { type: Object, default: () => ({}) },
label: { type: [String, Number], default: '' }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) · [全ルール](all.md)

### `script/return-in-computed-property`

computed の getter に値を返す return を用意します。

[悪い例](#script-return-in-computed-property-bad) · [良い例](#script-return-in-computed-property-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-computed-property": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-computed-property-bad"></span>

**悪い**

ブロック形式の computed getter が `1 + 2` を計算するだけで返さず、computed の値が undefined になります。

```vue annotate="remove:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { 1 + 2; });
</script>
```

<span id="script-return-in-computed-property-good"></span>

**良い**

`return 1 + 2` で式を getter の戻り値にします。このルールは式文だけでなく、getter 自身の値を返す return を確認します。

```vue annotate="add:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { return 1 + 2; });
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) · [全ルール](all.md)

### `script/return-in-emits-validator`

Options API の emits validator に戻り値を用意します。

[悪い例](#script-return-in-emits-validator-bad) · [良い例](#script-return-in-emits-validator-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

現在の SFC フィルターが対象とする block-body arrow を使います。validator 本体には method shorthand の処理もありますが、現在の SFC prefilter はその形を確実には実行しません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-emits-validator": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-emits-validator-bad"></span>

**悪い**

`submit` の validator が payload をログ出力するだけで検査結果を返さず、ブロックの結果が undefined になります。

```vue annotate="remove:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

<span id="script-return-in-emits-validator-good"></span>

**良い**

`return payload != null` を追加し、値を返さず終了する代わりに payload の真偽値の検査結果を返します。

```vue annotate="add:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L59) · [全ルール](all.md)

### `script/valid-define-emits`

defineEmits の重複や型と実行時引数の併用を検出します。

[悪い例](#script-valid-define-emits-bad) · [良い例](#script-valid-define-emits-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-emits": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-emits-bad"></span>

**悪い**

同じ `defineEmits` に型引数と実行時配列 `["save"]` の両方を渡し、併用できない二つの宣言形式を混ぜています。

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

<span id="script-valid-define-emits-good"></span>

**良い**

実行時の引数を除き、`save` の宣言を型ベースの一つの形式に統一します。

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) · [全ルール](all.md)

### `script/valid-define-options`

defineOptions の引数と使用回数を検査します。

[悪い例](#script-valid-define-options-bad) · [良い例](#script-valid-define-options-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-options": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-options-bad"></span>

**悪い**

最初の呼び出しが専用マクロで宣言すべき `props` を `defineOptions` に入れています。後続の呼び出しはマクロを繰り返し、オブジェクトでない引数も渡しています。禁止された形式と呼び出し回数の制約を示す例です。

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

<span id="script-valid-define-options-good"></span>

**良い**

`defineOptions` を一度だけ呼び、通常の対応オプション `name` と `inheritAttrs` を持つオブジェクトを渡します。

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) · [全ルール](all.md)

### `script/valid-define-props`

defineProps の重複や型と実行時引数の併用を検出します。

[悪い例](#script-valid-define-props-bad) · [良い例](#script-valid-define-props-good)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-props-bad"></span>

**悪い**

同じ `defineProps` に型引数 `{ title: string }` と実行時引数 `{ title: String }` の両方を渡しています。コンパイラーはこの併用を許可しません。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

<span id="script-valid-define-props-good"></span>

**良い**

実行時オブジェクトを除き、二つの形式を併用せず `title` の型ベースの宣言だけを残します。

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) · [全ルール](all.md)

### `script/valid-next-tick`

nextTick() の完了を await、then、または callback で扱います。

[悪い例](#script-valid-next-tick-bad) · [良い例](#script-valid-next-tick-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-next-tick": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-next-tick-bad"></span>

**悪い**

インポートした `nextTick()` をコールバックなしの式文で呼び、返された Promise を無視しています。DOM 更新後まで待つ処理がありません。

```vue annotate="remove:3"
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

<span id="script-valid-next-tick-good"></span>

**良い**

`await nextTick()` で Promise を使い、後続の setup 処理へ進む前に次の DOM 更新を明示的に待ちます。

```vue annotate="add:3"
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) · [全ルール](all.md)

### `type/no-floating-promises`

処理しないまま放置された Promise を検出します。

[悪い例](#type-no-floating-promises-bad) · [良い例](#type-no-floating-promises-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-floating-promises": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-floating-promises-bad"></span>

**悪い**

async の `save` は Promise を返しますが、単独の `save()` 呼び出しが await も return もせず、意図的に捨てる指定もありません。

```vue annotate="remove:3"
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

<span id="type-no-floating-promises-good"></span>

**良い**

`void save()` で実行結果を意図的に捨てる指定をし、このルールの条件を満たします。結果を破棄する明示的な印であり、rejection を処理するものではありません。

```vue annotate="add:3"
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) · [全ルール](all.md)

### `type/no-reactivity-loss`

代入や呼び出しによって反応性を失うスナップショットを検出します。

[悪い例](#type-no-reactivity-loss-bad) · [良い例](#type-no-reactivity-loss-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-reactivity-loss": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-reactivity-loss-bad"></span>

**悪い**

`const count = state.count` がリアクティブなプロパティから数値のスナップショットを取り、その後の `state.count` の更新が変数へ反映されなくなります。

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0 });
const count = state.count;
</script>
```

<span id="type-no-reactivity-loss-good"></span>

**良い**

`toRef(state, "count")` で現在のプリミティブ値をコピーせず、`count` を元のリアクティブなプロパティにつなげます。

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRef } from "vue";
const state = reactive({ count: 0 });
const count = toRef(state, "count");
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) · [全ルール](all.md)

### `type/no-unsafe-template-binding`

テンプレートで安全でない型の値を使用する箇所を検出します。

[悪い例](#type-no-unsafe-template-binding-bad) · [良い例](#type-no-unsafe-template-binding-good)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-unsafe-template-binding": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-unsafe-template-binding-bad"></span>

**悪い**

補間する `value` の型を明示的に `any` とし、チェッカーがテンプレートの binding を安全な具体的な型として確認できません。

```vue annotate="remove:2"
<script setup lang="ts">
const value: any = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

<span id="type-no-unsafe-template-binding-good"></span>

**良い**

型注釈を `string` に変え、描画する値を変えずに、同じ補間へ検査できる具体的な型を与えます。

```vue annotate="add:2"
<script setup lang="ts">
const value: string = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) · [全ルール](all.md)

### `type/require-typed-emits`

defineEmits に型定義を指定します。

[悪い例](#type-require-typed-emits-bad) · [良い例](#type-require-typed-emits-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-emits": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-emits-bad"></span>

**悪い**

配列だけの `defineEmits(["save"])` はイベント名を宣言するだけで、型付きの payload 契約がありません。

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

<span id="type-require-typed-emits-good"></span>

**良い**

`defineEmits<{ save: [] }>()` で空の payload タプルを持つ型付きの `save` イベントを宣言し、payload の引数を受け取らないことを明示します。

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) · [全ルール](all.md)

### `type/require-typed-props`

defineProps に型定義を指定します。

[悪い例](#type-require-typed-props-bad) · [良い例](#type-require-typed-props-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-props": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-props-bad"></span>

**悪い**

配列だけの `defineProps(["title"])` は `title` の名前だけを宣言し、型を指定していません。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps(["title"]);
</script>
```

<span id="type-require-typed-props-good"></span>

**良い**

`defineProps<{ title: string }>()` で、名前だけの実行時宣言に代えて `title` の string 型を明示します。

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) · [全ルール](all.md)

### `type/strict-boolean-expressions`

script とテンプレートの条件式で安全な真偽判定を使います。

[悪い例](#type-strict-boolean-expressions-bad) · [良い例](#type-strict-boolean-expressions-good)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。  
オプション: [型付きオプションと既定値](options.md)を参照してください。

typeAware とルールを明示的に有効にします。既定では null を含み得る数値は許可されず、通常の数値は許可されます。

型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/strict-boolean-expressions": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-strict-boolean-expressions-bad"></span>

**悪い**

`if (count)` が nullable な数値の truthiness に依存し、明示的な真偽値の検査を使っていません。ゼロと未指定も同じ偽として扱います。

```vue annotate="remove:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

<span id="type-strict-boolean-expressions-good"></span>

**良い**

`count !== undefined && count > 0` で存在と正の値を別々に検査し、省略可能な値を絞り込んだうえで明示的な真偽値の条件を作ります。

```vue annotate="add:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) · [全ルール](all.md)
