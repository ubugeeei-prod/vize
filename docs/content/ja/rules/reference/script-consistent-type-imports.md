---
title: "script/consistent-type-imports"
description: "型としてしか使わないインポートを import type に揃えます。テンプレートでの使用は値の使用として数えます。"
---

# `script/consistent-type-imports`

型としてしか使わないインポートを import type に揃えます。テンプレートでの使用は値の使用として数えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | あり | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 値としての使用に数えるもの

コンパイラーが読み取るものと同じバインディング解析を使い、コンポーネント全体でのインポートの使用を数えます。テンプレートの式、Vue が解決するすべての表記のコンポーネントタグ、カスタムディレクティブ、テンプレート参照、`<style>` の `v-bind()`、もう一方のスクリプトブロックが対象です。

同じ名前の `v-for` やスロットのエイリアスはインポートを隠します。テンプレート内の `(item as Foo)` や `generic` 属性のような型の位置は、型としての使用です。

読み取れないブロック（外部の `src` ブロック、解析できない式、直接の `eval`、Musea の `<variant>` マークアップ）がある場合、そのコンポーネントについては何も報告しません。

`import type` でインポートした名前をテンプレートの式または `<style>` の `v-bind()` が評価する場合も報告します。そのバインディングは実行時に存在しないためです。テンプレートでの読み取りは、`vue/no-undefined-refs` が有効な場合はそのルールに任せます。コンポーネントタグとディレクティブは、登録済みまたは自動インポートされたコンポーネントが描画できるため報告しません。

## オプション

`prefer`、`fixStyle`、`disallowTypeAnnotations` は typescript-eslint のオプションです。`emitDecoratorMetadata`（既定値 `true`）には、プロジェクトがデコレーターのメタデータを出力するかどうかを指定します。lint は `tsconfig.json` を読みません。`true` の場合、デコレーター付きのシグネチャが直列化する型を値としての使用に数えます。

## 自動修正

自動修正は報告した宣言だけを書き換え、残る値の指定子は元の位置を保つため、モジュールの評価順は変わりません。分割が必要で、コメントまたはインポート属性を含む宣言は、修正なしで報告します。

## 悪い例

`Foo` を値としてインポートしていますが、使用箇所は `defineProps` の型引数だけです。`Bar` はテンプレートが `Bar.label` を読み取るため値としての使用であり、報告しません。

```vue annotate="remove:3"
<script setup lang="ts">
import { Bar } from "./Foo";
import { Foo } from "./Foo";

defineProps<{ item: Foo }>();
</script>

<template>
  <p>{{ Bar.label }} {{ item.label }}</p>
</template>
```

## 良い例

`Foo` を `import type` でインポートするため、ビルド時にインポートが消去されます。`Bar` はテンプレートが評価するので、値のインポートのままです。

```vue annotate="add:3"
<script setup lang="ts">
import { Bar } from "./Foo";
import type { Foo } from "./Foo";

defineProps<{ item: Foo }>();
</script>

<template>
  <p>{{ Bar.label }} {{ item.label }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/consistent-type-imports": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## typescript-eslint との差異

typescript-eslint がテンプレートでの使用を認識するには `vue-eslint-parser` が必要で、oxlint はこのルールで `.vue` ファイルを対象外にします。デコレーター付きのシグネチャは、`emitDecoratorMetadata` が `false` でない限り値としての使用に数えます。typescript-eslint は `tsconfig.json` を読み、該当する場合はデコレーターを含むファイル全体を対象外にします。自動修正はほかの宣言に統合しません。テンプレートの式または `<style>` の `v-bind()` が評価する型のみのインポートを報告します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/consistent_type_imports.rs#L55)
