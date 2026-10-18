---
title: Lint の実行設定
---

<!-- Reviewed translation; source: guide/lint-execution.md -->

<span id="native-lint-execution-settings"></span>

# Lint の実行設定

ファイル間の関係も調べたい場合や、警告を CI の失敗条件にしたい場合に使うガイドです。
CLI と Vite+ のタスクで同じ方針を使うには、実行時のオプションを設定ファイルにまとめてください。
最初の実行方法とコマンドのオプションは、[CLI の Lint](./cli.md#lint)を参照してください。
ファイル間の関係を示すレポートは、[コンポーネントツリーと複雑度](./cross-file-complexity.md)で説明しています。

`vize.config.*` と Vite+ 連携では、Vize の Lint に同じ設定を使えます。
Vite+ では、次のように `lint.vize` に指定します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      preset: "opinionated",
      crossFile: true,
      strictReactivity: true,
      crossFileTree: true,
      crossFileComplexity: true,
      maxWarnings: 0,
    },
  },
});
```

`vize.config.*` では、これらのキーを `linter` に指定します。
`crossFileTree` と `crossFileComplexity` は、ファイル間の解析も有効にします。
ファイル間のチェックは、Vize の CLI 実行で対象に選ばれたファイルに対して行います。

`strictReactivity` は、CLI とエディターの両方で `type/no-reactivity-loss` を有効にします。
ルールを個別に指定した場合は、そちらが優先されます。ファイルの範囲を指定した `off` も優先されます。

`vize lint` を直接実行する場合は、コマンドのオプションで必要なチェックを有効にする方法も引き続き使えます。
`--max-warnings` は設定ファイルの警告数の上限より優先され、`--no-config` は Lint の設定をすべて無視します。
警告数の上限を指定しない場合は、警告だけで実行を失敗とはしません。
`0` を指定すると、警告が一つでも残っている場合に失敗します。負の値は指定できません。

Vite+ の Lint タスクには、対象のパスと `--fix` を渡せます。
Vize 固有のオプションは、Oxlint に渡すのではなく `lint.vize` に指定してください。
たとえば `vp run lint -- src` を実行すると、それぞれの設定を使って両方の Lint が実行されます。

## 静的な class 属性の既定のチェック

`html/no-duplicate-class` は、`class="button button"` のように、静的な
`class` 属性で同じクラス名が繰り返されている場合に警告します。
動的なバインディングは対象にせず、自動修正も行いません。

| プリセット | 変更前の既定値 | 変更後の既定値 |
| --- | --- | --- |
| `happy-path`、`ecosystem` | 無効 | 警告 |
| `opinionated`、`nuxt` | 警告 | 警告 |
| `essential`、`incremental` | 無効 | 無効 |

`rules` で個別に指定すると、警告を無効にしたり、重大度を変更したりできます。
`incremental` は引き続き空の状態から始まります。採用する場合は、このルールを明示的に有効にしてください。
