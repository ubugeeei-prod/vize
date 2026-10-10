---
title: トラブルシューティング
---

<!-- Reviewed translation; source: guide/troubleshooting.md; scope: introduction, headings and reading order -->

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

一般的な移行ケースは、非 void HTML 要素の自己終了構文です。

```vue
<template>
  <div />
  <span />
</template>
```

`<div />` および `<span />` は有効な自己終了 HTML 要素ではありません。標準モードでは次のように書き換えられます。
空の要素 (`<div></div>` および `<span></span>` に相当) があり、警告が生成されます。ストリクトモード
それらをエラーとして報告します。 Quirks モードでは、警告なしで自動的に閉じるリーフとして保持されます。

明示的な終了タグを記述することを好みます。

```vue
<template>
  <div></div>
  <span></span>
</template>
```

移行時にモードを明示的に選択します。

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

無効な構文で失敗するには `"strict"` を使用します。プロジェクトが構文を受け入れる Vue に依存している場合は `"quirks"` を使用します。
タグは自己終了リーフとして使用されます。有効な void 要素 (`<input />`、`<img />`、`<br />`、および
`<meta />` には癖は必要ありません。

## ネイティブタイプのパッケージ解決

`vize check` は、バンドルされたものを使用する前に、チェックされたプロジェクトから Vue および Vite タイプのパッケージを解決します。
フォールバックのため、プロジェクト独自の `vue`、`@vue/runtime-dom`、`@vue`、および `vite` バージョンが
生成された仮想プロジェクト。通常とは異なるパッケージ マネージャー レイアウトの場合は、`VIZE_VUE_PACKAGE` を設定します。
`VIZE_VUE_NAMESPACE_PACKAGE`、`VIZE_VUE_RUNTIME_DOM_PACKAGE`、または `VIZE_VITE_PACKAGE` を明示的に指定する
パッケージのルート。 `VIZE_RUNTIME_NODE_MODULES` は、1 つ以上の `node_modules` ルートを
フォールバック検索パス。
