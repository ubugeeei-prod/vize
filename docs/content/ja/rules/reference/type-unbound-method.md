---
title: "type/unbound-method"
description: "レシーバーから切り離したメソッド参照を検出します。"
---

# `type/unbound-method`

レシーバーから切り離したメソッド参照を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

counter.increment はレシーバーなしでメソッドを取り出すため、後で呼ぶと this が失われます。

```vue annotate="remove:9"
<script setup lang="ts">
class Counter {
  value = 0;
  increment(): void {
    this.value += 1;
  }
}
const counter = new Counter();
const unbound = counter.increment;
</script>
```

## 良い例

counter.increment() は counter 上でメソッドを呼ぶため、this は束縛されたままです。

```vue annotate="add:9"
<script setup lang="ts">
class Counter {
  value = 0;
  increment(): void {
    this.value += 1;
  }
}
const counter = new Counter();
counter.increment();
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/unbound-method": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## typescript-eslint との違い

グローバルオブジェクトから読み出したライブラリのメソッドは報告しません。レシーバーの型が `Window & typeof globalThis` または `typeof globalThis` の場合が対象です。`window.setTimeout`、`const { matchMedia } = window`、`self.requestAnimationFrame`、`document.defaultView?.setTimeout` もこれに含まれます。プラットフォームの操作はレシーバーなしで呼ばれるとグローバルオブジェクトを使うため、書いたとおりに動きます。自分のコードで `Window` に追加したメソッドは引き続き報告します。別のフレームの `Window` (`parent.setTimeout`) や、グローバルからたどったオブジェクトのメソッド (`window.localStorage.getItem`) も同様です。`checkGlobalObjectMethods: true` を指定すると、typescript-eslint と同じくこれらも報告します。

宣言をマージした組み込み (`declare global { interface Math {} }`) も除外の対象のままです。`RegExp.escape` は `RegExp` コンストラクターへのどの経路でも除外されます。除外は linter を動かす Node.js のバージョンではなく、TypeScript のライブラリ宣言に従います。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/unbound_method.rs#L5)
