---
title: vue-tsc を超える型チェック
description: Vize が vue-tsc や Volar より精密に答えるエラーとエディターの応答です。どれもテストで固定しているので、このページは Vize の実際の出力と一致します。
---

# vue-tsc を超える型チェック

> vue-tsc との完全な互換性は目指していません。目指しているのは、より正しく、より精密な型チェックです。

vue-tsc、`@vue/content-mapper`、TypeScript は参考にする実装であって、仕様ではありません。Vize と
vue-tsc の結果が食い違うときは、書かれたコードが正しいほうを決めます。

- Vize が正しければ挙動を残し、このページに載せます。
- Vize が間違っていれば、その差はバグとして直します。

Vize は misskey や n8n などの実プロジェクトで vue-tsc との差分を取りますが、それは差を見つけるため
だけです。見つかった差は、コードそのものを根拠に分類します。位置、コード、メッセージも vue-tsc に
合わせるのではなく、タグ全体ではなく該当する属性を指すなど、正確さを基準に選びます。

以下の例はどれも、同じファイルをネイティブの TypeScript コンパイラーで検査するテストで固定しています。
出力は Vize が報告する内容で、`vize check` の形式です。TypeScript 7.1 や tsrs でも、同じコードと位置が
そのコンパイラーの形式で出ます。vue-tsc の出力は vue-tsc 3.3.11、TypeScript 6.0.3、Vue 3.5.44 と
デフォルトの `vueCompilerOptions` で記録しました。

## 診断

### 兄弟の prop の型が見つからなくても、prop は自分の型を保つ

パッケージが入っていないなどの理由で 1 つの prop の型が解決できないと、vue-tsc の
`toRef(props, …)` はすべての prop を `any` にします。Vize は prop ごとの型を保つので、宣言した型が
許さない読み取りはエラーのままです。

```vue
<!-- src/NodePanel.vue -->
<script setup lang="ts">
import { toRef } from "vue";
import type { RunData } from "missing-run-data";

interface PanelNode {
  name: string;
}

const props = withDefaults(defineProps<{ node?: PanelNode | null; run?: RunData }>(), {
  node: null,
});
const node = toRef(props, "node");
</script>

<template>
  <span>{{ node?.label }}</span>
</template>
```

Vize は見つからないモジュールと、その読み取りを報告します。

```diagnostic
src/NodePanel.vue
  error:3:30 [TS2307] Cannot find module 'missing-run-data' or its corresponding type declarations.
  error:16:18 [TS2339] Property 'label' does not exist on type 'PanelNode'.
```

vue-tsc が報告するのは、見つからないモジュールだけです。

```diagnostic
src/NodePanel.vue(3,30): error TS2307: Cannot find module 'missing-run-data' or its corresponding type declarations.
```

`PanelNode` が宣言しているのは `label` ではなく `name` で、これは `RunData` とは関係ありません。
`node?.name` を読むように直すと、残るのは見つからないモジュールのエラーだけです。n8n の
`RunData.vue` では、16 か所のプロパティの読み取りがこうして隠れていました。固定しているテスト:
[`unresolved-prop-type`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/type-checking-beyond-vue-tsc/unresolved-prop-type)。

### ネイティブ要素のリスナーは DOM の `Event` を受け取る

ネイティブ要素に付けた `v-on` のリスナーは、イベント名が何であっても DOM のイベントリスナーです。
Vize はその引数を `Event` として型付けします。vue-tsc は知らない名前の引数を型付けしません。

```vue
<!-- src/Accordion.vue -->
<script setup lang="ts">
function beforeEnter(el: HTMLElement) {
  el.style.height = "0";
}
</script>

<template>
  <div class="accordion" @before-enter="beforeEnter">
    <slot />
  </div>
</template>
```

```diagnostic
src/Accordion.vue
  error:8:41 [TS2345] Argument of type '(el: HTMLElement) => void' is not assignable to parameter of type '(_e: Event) => unknown'.
    Types of parameters 'el' and '_e' are incompatible.
    Type 'Event' is missing the following properties from type 'HTMLElement': accessKey, accessKeyLabel, autocapitalize, autocorrect, and 318 more.
```

vue-tsc は `strictTemplates` の有無にかかわらず何も報告しません。

`before-enter` は `<Transition>` のフックです。`<div>` に付けても、ハンドラーが要素を受け取ることは
なく、その名前の DOM イベントを誰かが発火したときにしか呼ばれません。このコードが意図していたのは
トランジショングループです。

```vue
<!-- src/Accordion.vue (fixed) -->
<script setup lang="ts">
function beforeEnter(el: Element) {
  (el as HTMLElement).style.height = "0";
}
</script>

<template>
  <TransitionGroup tag="div" class="accordion" @before-enter="beforeEnter">
    <slot />
  </TransitionGroup>
</template>
```

n8n の `ItemsRenderer.vue` では、`<div>` に 4 つのトランジションフックが付いていました。固定している
テスト:
[`unknown-native-event`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/type-checking-beyond-vue-tsc/unknown-native-event)。

### コンポーネントが宣言していない slot

コンポーネントが宣言していない slot に内容を渡すと、Vize は報告します。この検査はデフォルトで有効で、
有効にするためのオプションはありません。

```vue
<!-- src/EmptyState.vue -->
<script setup lang="ts">
defineSlots<{ icon(): unknown; default(): unknown; actions(): unknown }>();
</script>

<template>
  <article>
    <slot name="icon" />
    <p><slot /></p>
    <slot name="actions" />
  </article>
</template>
```

```vue
<!-- src/Panel.vue -->
<script setup lang="ts">
import EmptyState from "./EmptyState.vue";
</script>

<template>
  <EmptyState>
    <template #description>Connect a node first.</template>
  </EmptyState>
</template>
```

```diagnostic
src/Panel.vue
  error:7:16 [TS2339] Property 'description' does not exist on type 'Partial<$Slots>'.
```

vue-tsc は `strictTemplates` の有無にかかわらず何も報告しません。

`EmptyState` は `description` slot を描画しないので、このテキストはページに表示されません。デフォルトの
slot (`#default`) に渡すと何も報告されません。n8n の `InputPanel.vue` で見つかりました。固定している
テスト:
[`unknown-slot-name`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/type-checking-beyond-vue-tsc/unknown-slot-name)。

## エディター

[言語サーバー](./language-server.md)も同じチェッカーを使います。ここに載せる応答は記録したエディターの
応答で、Vize のテストが変更のたびに比較しています。Volar の応答は、同じファイルに対して
Vue - Official (`@vue/language-server` 3.3.11 と `@vue/typescript-plugin`) で確認したものです。

### コンポーネントタグのホバーで受け取れるものが分かる

コンポーネントタグにホバーすると、そのコンポーネントが宣言した props、emits、slots、model が出ます。

```typescript
const ShortComponent: VueComponent & {
  props: { label: string; count?: number };
  emits: { save: [value: string] };
  slots: { default(props: { value: string }): unknown };
  model: { modelValue: boolean };
};
```

長いコンポーネントはメンバーを 1 行ずつ表示します。`defineAsyncComponent` や単純な別名を通しても
同じです。Volar が出すのは import の別名 (`import LongComponent`) だけで、非同期の別名では props
ではなく生成された型名を含む `DefineComponent<__VLS_Props, …>` を出します。固定しているテスト:
[`component-hover-readability`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/differential/lsp/component-hover-readability)、
[`component-alias-hover-10787`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/differential/lsp/component-alias-hover-10787)。

### 別名で登録したコンポーネントの定義へ移動できる

Options API で `components: { "lottie": Lottie }` と登録したコンポーネントでは、`<lottie />` から
定義へ移動すると `lottie.vue` が開きます。Volar は登録キーの `"lottie": Lottie` で止まり、そこでの
ホバーは `any` です。固定しているテスト:
[`registered-lowercase-component-alias`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/differential/lsp/registered-lowercase-component-alias)。

### slot 名の補完はそのコンポーネントの slot だけを出す

コンポーネント内の `<template>` で `#` や `v-slot:` の直後に補完すると、そのコンポーネントが宣言した
slot だけが、`#item="{ item }"` のように slot の props を分割代入するスニペット付きで出ます。同じ位置で
Volar は 500 件を出し、ほとんどは HTML の属性とディレクティブです。固定しているテスト:
[`template-slot-completion-9907`](https://github.com/ubugeeei-prod/vize/tree/main/tests/_fixtures/differential/lsp/template-slot-completion-9907)
の入力と `nested_template_slot_completion_only_lists_component_slots`。

## 関連ページ

- 方針と設定は[型チェック](./type-checking.md#型チェックの方針)
- [言語サーバー](./language-server.md)
- vue-tsc から切り替える前に結果を比べる手順は[移行ガイド](./migration.md)
