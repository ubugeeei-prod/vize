---
title: トラブルシューティング
---

<!-- Reviewed translation; source: guide/troubleshooting.md; scope: complete document -->

# トラブルシューティング

既存の Vue プロジェクトを Vize に移行して、テンプレートの警告や型パッケージの解決エラーが出たときに使うページです。
まず、症状に合う節を選んでください。

- [HTML 要素の自己終了タグ](#テンプレート構文モード): 3 つの構文モードを比較し、タグを直すか、必要なモードを選びます。
- [Vue・Vite の型が見つからない](#ネイティブタイプのパッケージ解決): 解決先を確認し、特殊な配置ではパッケージのパスを指定します。

解決しない場合は [コンパイラ インスペクター](./compiler-inspector.md) でコンパイルの再現例を絞り込み、
[テストとフィードバック](./testing.md) の手順で小さな再現例を報告してください。

## テンプレート構文モード

`compiler.templateSyntax` のデフォルト値は `"standard"` です。修復できる構文上の問題には
警告を出し、有効なテンプレートに書き換えます。

移行時によくあるのは、終了タグが必要な HTML 要素を自己終了タグで書いているケースです。

```vue
<template>
  <div />
  <span />
</template>
```

`<div />` と `<span />` は、HTML の自己終了要素としては有効ではありません。standard モードはこれらを `<div></div>` と `<span></span>` に相当する空の要素へ書き換え、警告を出します。strict モードはエラーとして報告します。quirks モードは警告を出さず、子を持たない自己終了要素として扱います。

できるだけ終了タグを明示してください。

```vue
<template>
  <div></div>
  <span></span>
</template>
```

移行時には、必要なモードを明示的に選べます。

```ts
import vize from "@vizejs/vite-plugin";

export default {
  plugins: [
    vize({
      templateSyntax: "standard",
    }),
  ],
};
```

無効な構文で処理を失敗させたい場合は `"strict"` を使います。Vue がこうしたタグを自己終了要素として受け入れる動作にプロジェクトが依存している場合は、`"quirks"` を選んでください。
`<input />`、`<img />`、`<br />`、`<meta />` のような有効な void 要素には、quirks モードは必要ありません。

## ネイティブタイプのパッケージ解決

`vize check` は、型チェック対象のプロジェクトから Vue と Vite の型パッケージを解決し、見つからない場合にバンドル済みの型へフォールバックします。生成する仮想プロジェクトには、プロジェクト自身の `vue`、`@vue/runtime-dom`、`@vue`、`vite` のバージョンが反映されます。

通常とは異なるパッケージマネージャーの配置では、`VIZE_VUE_PACKAGE`、`VIZE_VUE_NAMESPACE_PACKAGE`、`VIZE_VUE_RUNTIME_DOM_PACKAGE`、`VIZE_VITE_PACKAGE` にパッケージのルートを明示できます。
`VIZE_RUNTIME_NODE_MODULES` には、フォールバック先として検索する 1 つ以上の `node_modules` ルートを指定できます。
