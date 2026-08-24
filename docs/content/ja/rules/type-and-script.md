---
title: タイプとスクリプトのルール
---

<!-- Generated translation; source: rules/type-and-script.md -->

# タイプとスクリプトのルール

型ルールは、セマンティック情報が必要な場合に TypeScript チェッカーを使用します。Vize も同じだ
TypeScript が `tsconfig.json` から読み取るプロジェクト形状なので、共有アンビエント名はそこから取得する必要があります
`compilerOptions.types`、プロジェクト参照、または宣言ファイル。

スクリプト ルールは、Composition API および Vapor 指向のコードの Patina ルールです。彼らはパターンに焦点を当てています
効率的にコンパイルするのが難しい、または Vapor モードで推論するのが難しいものです。

型を認識したリンティングはオプトインです。 `linter.typeAware: true`、`vize lint --type-aware`、または
`type/*` ルールを明示的に有効にすることによって。 `type/no-reactivity-loss` は次のコマンドで直接有効にできます
`vize lint --strict-reactivity`。 Corsa を起動できない場合、Patina は `type/corsa-runtime` を報告し、
構成されたルールをサイレントに削除するのではなく、チェッカーによってバックアップされたルール パスをスキップします。

`--type-aware` は、`vize check` と同じ Corsa 実行可能解像度を使用します。構成する
プロジェクトで明示的な `tsgo` または Corsa バイナリが必要な場合は、`typeChecker.corsaPath`。デフォルトのまま
ゼロコスト: Patina は、フラグが設定されていない限り、チェッカーに裏付けられた lint について SFC を解析したり、Corsa を開始したりしません。
`linter.typeAware`、または明示的に有効化された `type/*` ルールがオプトインされます。

```ts
export default defineConfig({
  linter: { typeAware: true },
});
```

## `type/require-typed-props`

ランタイム配列宣言を使用する代わりに、`defineProps` を入力する必要があります。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<script setup lang="ts">
const props = defineProps(["label", "count"]);
</script>
```

良い：

```vue
<script setup lang="ts">
const props = defineProps<{
  label: string;
  count: number;
}>();
</script>
```

## `type/require-typed-emits`

発行されたイベント ペイロードを記述するには、`defineEmits` が必要です。

デフォルトの重大度: `warning`
プリセット: `happy-path`、`nuxt`、`opinionated`

悪い：

```vue
<script setup lang="ts">
const emit = defineEmits(["save"]);

emit("save", form.value);
</script>
```

良い：

```vue
<script setup lang="ts">
const emit = defineEmits<{
  save: [payload: FormValue];
}>();

emit("save", form.value);
</script>
```

## `type/no-unsafe-template-binding`

`any` などの安全でない値に解決されるテンプレート バインディングを報告します。ルールはチェッカーバックされており、
したがって、インポートされたタイプとプロジェクト構成に従います。

デフォルトの重大度: `warning`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<script setup lang="ts">
const payload: any = await loadPayload();
</script>

<template>
  <p>{{ payload.title }}</p>
</template>
```

良い：

```vue
<script setup lang="ts">
type Payload = { title: string };

const payload = await loadPayload<Payload>();
</script>

<template>
  <p>{{ payload.title }}</p>
</template>
```

## `type/no-floating-promises`

作成されたものの、待機、返され、または意図的に処理されていない Promise をレポートします。
このチェックは、`<script>` とテンプレート式の両方を対象としています。

デフォルトの重大度: `warning`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<script setup lang="ts">
function submit() {
  saveForm(form.value);
}
</script>

<template>
  <button @click="saveForm(form)">Save</button>
  <p>{{ loadPreview() }}</p>
</template>
```

良い：

```vue
<script setup lang="ts">
type Preview = { title: string };

async function submit() {
  await saveForm(form.value);
}

const preview = ref<Preview | null>(null);

async function loadPreviewIntoState() {
  preview.value = await loadPreview();
}
</script>

<template>
  <button @click="void submit()">Save</button>
  <button @click="void loadPreviewIntoState()">Preview</button>
  <PreviewPanel v-if="preview" :preview="preview" />
</template>
```

## `type/no-reactivity-loss`

フロー全体で使用されるリアクティブ値のプレーンなスナップショットをレポートします。このルールは次の場合にも実行されます。
`vize lint --strict-reactivity` が有効になります。

デフォルトの重大度: `warning`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const item = props.item;
</script>
```

良い：

```vue
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

## チェッカーの構成

タイプ認識ルールでは、TypeScript 名用の別個の Vize `globals` フィールドは必要ありません。好む
TypeScript ネイティブの構成:

悪い：

```ts
export default {
  globals: ["definePageMeta", "process"],
};
```

良い：

```json
{
  "compilerOptions": {
    "types": ["node", "nuxt/app"]
  }
}
```

## `script/no-options-api`

Vapor 指向のプリセットのレポート オプション API コンポーネント定義。

デフォルトの重大度: `error`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

良い：

```vue
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

## `script/no-next-tick`

Vapor 指向のコンポーネントで `nextTick()` を報告します。直接参照、ライフサイクルフック、または状態を優先します
次の DOM フラッシュに依存しないフロー。

デフォルトの重大度: `error`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<script setup lang="ts" vapor>
await nextTick();
input.value?.focus();
</script>
```

良い：

```vue
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>
```

## `script/no-get-current-instance`

Vapor 指向のコンポーネントで `getCurrentInstance()` を報告します。実行時の内部構造にまで影響を及ぼします。
Vapor を安全に最適化することはできません。

デフォルトの重大度: `error`
プリセット: `nuxt`、`opinionated`

悪い：

```vue
<script setup lang="ts" vapor>
const instance = getCurrentInstance();
const app = instance?.appContext.app;
</script>
```

良い：

```vue
<script setup lang="ts" vapor>
const appConfig = useAppConfig();
</script>
```
