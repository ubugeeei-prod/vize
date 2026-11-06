---
title: "型と script のルール"
description: "`<script>` と `<script setup>` を検査します。コンパイラーマクロ、リアクティビティ、props と emits、TypeScript の型情報を使うルールが含まれます。"
---

# 型と script のルール

`<script>` と `<script setup>` を検査します。コンパイラーマクロ、リアクティビティ、props と emits、TypeScript の型情報を使うルールが含まれます。

各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。

## 基本 (39)

`essential` プリセットに含まれる、正しさを守るルールです。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="script-no-arrow-functions-in-watch"></span>[`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | Options API の watch ハンドラーにアロー関数を使う箇所を検出します。 |  |
| <span id="script-no-async-in-computed"></span>[`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | computed の getter で非同期関数を使う箇所を検出します。 |  |
| <span id="script-no-computed-properties-in-data"></span>[`script/no-computed-properties-in-data`](./reference/script-no-computed-properties-in-data.md) | 初期化前の data() で算出プロパティを読む箇所を検出します。 |  |
| <span id="script-no-deprecated-data-object-declaration"></span>[`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | Vue 3 で関数にすべき data オプションのオブジェクト指定を検出します。 |  |
| <span id="script-no-deprecated-delete-set"></span>[`script/no-deprecated-delete-set`](./reference/script-no-deprecated-delete-set.md) | Vue 3 で削除された $set / $delete を検出します。 |  |
| <span id="script-no-deprecated-destroyed-lifecycle"></span>[`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | Vue 2 の destroyed / beforeDestroy を検出し、Vue 3 のライフサイクルフックに置き換えます。 | ✓ |
| <span id="script-no-deprecated-dollar-listeners-api"></span>[`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | Vue 3 で $attrs に統合された $listeners を検出します。 |  |
| <span id="script-no-deprecated-dollar-scopedslots-api"></span>[`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | Vue 3 で $slots に統合された $scopedSlots を検出します。 |  |
| <span id="script-no-deprecated-events-api"></span>[`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | Vue 3 で削除された $on / $off / $once を検出します。 |  |
| <span id="script-no-deprecated-model-definition"></span>[`script/no-deprecated-model-definition`](./reference/script-no-deprecated-model-definition.md) | Vue 3 で削除された model オプションを検出します。 |  |
| <span id="script-no-deprecated-props-default-this"></span>[`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | prop の default / validator 内で使えなくなった this を検出します。 |  |
| <span id="script-no-deprecated-vue-config-keycodes"></span>[`script/no-deprecated-vue-config-keycodes`](./reference/script-no-deprecated-vue-config-keycodes.md) | 削除された Vue.config.keyCodes を検出します。 |  |
| <span id="script-no-dupe-keys"></span>[`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | Options API の props / data / computed などのキー重複を検出します。 |  |
| <span id="script-no-duplicate-attr-inheritance"></span>[`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。 |  |
| <span id="script-no-export-in-script-setup"></span>[`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | script setup 内の export 文を検出します。 |  |
| <span id="script-no-expose-after-await"></span>[`script/no-expose-after-await`](./reference/script-no-expose-after-await.md) | await の後の expose() と defineExpose() を禁止します。 |  |
| <span id="script-no-import-compiler-macros"></span>[`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | 自動的に使える Vue コンパイラーマクロの import を検出します。 |  |
| <span id="script-no-internal-imports"></span>[`script/no-internal-imports`](./reference/script-no-internal-imports.md) | Vue 内部モジュールからの import を検出します。 |  |
| <span id="script-no-lifecycle-after-await"></span>[`script/no-lifecycle-after-await`](./reference/script-no-lifecycle-after-await.md) | setup() 内で await の後にライフサイクルフックを登録することを禁止します。 |  |
| <span id="script-no-potential-component-option-typo"></span>[`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | Options API のオプション名の入力ミスを検出します。 |  |
| <span id="script-no-ref-as-operand"></span>[`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | ref を演算の値として使う際に .value を参照します。 |  |
| <span id="script-no-required-prop-with-default"></span>[`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | required: true と default を同時に持つ prop を検出します。 |  |
| <span id="script-no-reserved-identifiers"></span>[`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | Vue コンパイラーが予約した識別子の宣言を検出します。 |  |
| <span id="script-no-reserved-keys"></span>[`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | Options API のキーに Vue の予約名を使う箇所を検出します。 |  |
| <span id="script-no-reserved-props"></span>[`script/no-reserved-props`](./reference/script-no-reserved-props.md) | prop 宣言に Vue の予約名を使う箇所を検出します。 |  |
| <span id="script-no-side-effects-in-computed-properties"></span>[`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | Options API の computed getter 内の副作用を検出します。 |  |
| <span id="script-no-use-computed-property-like-method"></span>[`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | Options API の computed プロパティをメソッドとして呼ぶ箇所を検出します。 |  |
| <span id="script-no-watch-after-await"></span>[`script/no-watch-after-await`](./reference/script-no-watch-after-await.md) | setup() 内で await の後に停止ハンドルを保持せずにウォッチャーを登録することを禁止します。 |  |
| <span id="script-prefer-import-from-vue"></span>[`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | 内部パッケージではなく vue から import します。 | ✓ |
| <span id="script-require-prop-type-constructor"></span>[`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | prop の type に文字列ではなくコンストラクターを指定します。 |  |
| <span id="script-require-render-return"></span>[`script/require-render-return`](./reference/script-require-render-return.md) | render 関数が値を返すことを必須にします。 |  |
| <span id="script-require-slots-as-functions"></span>[`script/require-slots-as-functions`](./reference/script-require-slots-as-functions.md) | Vue 3 の $slots プロパティを関数として呼び出します。 |  |
| <span id="script-require-valid-default-prop"></span>[`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | prop の default を宣言した型に合う値にします。 |  |
| <span id="script-return-in-computed-property"></span>[`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | computed の getter に値を返す return を用意します。 |  |
| <span id="script-return-in-emits-validator"></span>[`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | Options API の emits validator に戻り値を用意します。 |  |
| <span id="script-valid-define-emits"></span>[`script/valid-define-emits`](./reference/script-valid-define-emits.md) | defineEmits の重複や型と実行時引数の併用を検出します。 |  |
| <span id="script-valid-define-options"></span>[`script/valid-define-options`](./reference/script-valid-define-options.md) | defineOptions の引数と使用回数を検査します。 |  |
| <span id="script-valid-define-props"></span>[`script/valid-define-props`](./reference/script-valid-define-props.md) | defineProps の重複や型と実行時引数の併用を検出します。 |  |
| <span id="script-valid-next-tick"></span>[`script/valid-next-tick`](./reference/script-valid-next-tick.md) | nextTick() の完了を await、then、またはコールバックで扱います。 |  |

## 既定 (5)

既定のプリセット `happy-path` で追加されるルールです。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="script-no-multiple-slot-args"></span>[`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | scoped slot 関数に複数の引数を渡す箇所を検出します。 |  |
| <span id="script-no-unstable-nested-components"></span>[`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | setup / render 内で毎回コンポーネントを定義する箇所を検出します。 |  |
| <span id="script-require-explicit-emits"></span>[`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | emit するイベントを defineEmits または emits に宣言します。 |  |
| <span id="type-require-typed-emits"></span>[`type/require-typed-emits`](./reference/type-require-typed-emits.md) | defineEmits に型定義を指定します。 |  |
| <span id="type-require-typed-props"></span>[`type/require-typed-props`](./reference/type-require-typed-props.md) | defineProps に型定義を指定します。 |  |

## より厳しいプリセット (22)

`opinionated`・`nuxt`・`ecosystem` のいずれかにだけ含まれます。どれに含まれるかはプリセット列を参照してください。

| ルール | 検出する内容 | プリセット | 自動修正 |
| --- | --- | --- | --- |
| <span id="script-component-options-name-casing"></span>[`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | コンポーネントの name オプションを PascalCase に揃えます。 | `opinionated`, `nuxt` |  |
| <span id="script-custom-event-name-casing"></span>[`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | emit するカスタムイベント名を指定した形式に揃えます。 | `opinionated`, `nuxt` |  |
| <span id="script-define-emits-declaration"></span>[`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | defineEmits を型による宣言形式に揃えます。 | `opinionated`, `nuxt` |  |
| <span id="script-define-macros-order"></span>[`script/define-macros-order`](./reference/script-define-macros-order.md) | script setup のコンパイラーマクロを一定の順に宣言します。 | `opinionated`, `nuxt` |  |
| <span id="script-define-props-declaration"></span>[`script/define-props-declaration`](./reference/script-define-props-declaration.md) | defineProps を型による宣言形式に揃えます。 | `opinionated`, `nuxt` |  |
| <span id="script-no-boolean-default"></span>[`script/no-boolean-default`](./reference/script-no-boolean-default.md) | Boolean prop の冗長な default を検出します。 | `opinionated`, `nuxt` |  |
| <span id="script-no-deep-destructure-in-props"></span>[`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | defineProps の深い分割代入を検出します。 | `opinionated`, `nuxt` |  |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | Vapor で null を返す getCurrentInstance() を検出します。 | `opinionated` |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | Vapor で Options API を使用する箇所を検出します。 | `opinionated` |  |
| <span id="script-no-unused-emit-declarations"></span>[`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | 宣言したまま emit していないイベントを検出します。 | `opinionated`, `nuxt` |  |
| <span id="script-no-with-defaults"></span>[`script/no-with-defaults`](./reference/script-no-with-defaults.md) | Vue 3.5 以降の props 分割代入の既定値を勧めます。 | `opinionated` |  |
| <span id="script-one-component-per-file"></span>[`script/one-component-per-file`](./reference/script-one-component-per-file.md) | 1 ファイルにコンポーネント定義が複数あることを検出します。 | `opinionated`, `nuxt` |  |
| <span id="script-order-in-components"></span>[`script/order-in-components`](./reference/script-order-in-components.md) | コンポーネントオプションのプロパティ順を既定の並びに揃えます。 | `opinionated`, `nuxt` |  |
| <span id="script-prefer-define-options"></span>[`script/prefer-define-options`](./reference/script-prefer-define-options.md) | name / inheritAttrs だけの通常 script を defineOptions() にまとめます。 | `opinionated`, `nuxt` |  |
| <span id="script-prefer-use-template-ref"></span>[`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | テンプレート参照に Vue 3.5 の useTemplateRef() を使います。 | `opinionated`, `nuxt` |  |
| <span id="script-require-default-prop"></span>[`script/require-default-prop`](./reference/script-require-default-prop.md) | 省略可能な、Boolean 以外の prop に default を用意します。 | `opinionated`, `nuxt` |  |
| <span id="script-require-explicit-slots"></span>[`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | useSlots() で使う slot を defineSlots の型で宣言します。 | `opinionated`, `nuxt` |  |
| <span id="script-require-prop-types"></span>[`script/require-prop-types`](./reference/script-require-prop-types.md) | 各 prop の型を宣言します。 | `opinionated` |  |
| <span id="script-require-typed-object-prop"></span>[`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | Object / Array の prop に具体的な型を指定します。 | `opinionated`, `nuxt` |  |
| <span id="type-no-floating-promises"></span>[`type/no-floating-promises`](./reference/type-no-floating-promises.md) | 処理しないまま放置された Promise を検出します。 | `nuxt`, `opinionated` |  |
| <span id="type-no-reactivity-loss"></span>[`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | リアクティブな値を代入や呼び出しで取り出し、元の更新に追従しないスナップショットになる箇所を検出します。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-template-binding"></span>[`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | テンプレートで安全でない型の値を使用する箇所を検出します。 | `nuxt`, `opinionated` |  |

## 個別に有効化 (16)

どのプリセットにも含まれません。`withVue` に渡す `linter.rules` でルール名を指定して有効にします。

| ルール | 検出する内容 | 自動修正 |
| --- | --- | --- |
| <span id="script-define-props-destructuring"></span>[`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | defineProps の分割代入スタイルを指定した方針に揃えます。 |  |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | Vapor 向けコンポーネントの nextTick() 使用を検出します。 |  |
| <span id="script-no-reactive-destructure"></span>[`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | reactive オブジェクトを分割代入し、リアクティビティを失う箇所を検出します。 |  |
| <span id="script-no-restricted-globals"></span>[`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | 設定で禁止した実行環境のグローバル参照を検出します。 |  |
| <span id="script-no-restricted-members"></span>[`script/no-restricted-members`](./reference/script-no-restricted-members.md) | 設定で禁止した object.property へのアクセスを検出します。 |  |
| <span id="script-no-top-level-ref-in-script"></span>[`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | 通常の script のトップレベルで、リクエスト間に共有される状態を作る箇所を検出します。 |  |
| <span id="script-prefer-computed"></span>[`script/prefer-computed`](./reference/script-prefer-computed.md) | 他の状態から導ける値を watcher で同期する代わりに computed で表現します。 |  |
| <span id="script-prefer-define-emits-binding"></span>[`script/prefer-define-emits-binding`](./reference/script-prefer-define-emits-binding.md) | template の $emit ではなく defineEmits から受け取った型付き関数を呼び出します。 | ✓ |
| <span id="script-prefer-ref-over-reactive"></span>[`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | 状態管理に reactive() より ref() を使う方針を適用します。 |  |
| <span id="script-prefer-use-attrs"></span>[`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | setup の context.attrs を useAttrs() に置き換えます。 |  |
| <span id="script-prefer-use-id"></span>[`script/prefer-use-id`](./reference/script-prefer-use-id.md) | 一意な ID の生成に Vue 3.5 の useId() を使います。 |  |
| <span id="script-prefer-use-slots"></span>[`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | setup の context.slots を useSlots() に置き換えます。 |  |
| <span id="script-require-function-return-type"></span>[`script/require-function-return-type`](./reference/script-require-function-return-type.md) | 関数に戻り値の型注釈を指定します。 |  |
| <span id="script-require-symbol-provide"></span>[`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | provide / inject のキーに衝突しにくい Symbol を使います。 |  |
| <span id="script-require-typed-ref"></span>[`script/require-typed-ref`](./reference/script-require-typed-ref.md) | 引数なし、null、undefined で呼び出す ref() に型引数を指定します。 |  |
| <span id="type-strict-boolean-expressions"></span>[`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | script とテンプレートの条件式で安全な真偽判定を使います。 |  |

[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)
