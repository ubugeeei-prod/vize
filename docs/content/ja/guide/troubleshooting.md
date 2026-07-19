---
title: トラブルシューティング
---

<!-- Generated translation; source: guide/troubleshooting.md -->

# トラブルシューティング

## テンプレート構文モード

Vize のデフォルトは `compiler.templateSyntax` から `"standard"` です。標準モードでは回復可能なテンプレートを使用できます
構文の問題を検出し、警告を報告し、それらを有効な出力に書き換えます。

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
