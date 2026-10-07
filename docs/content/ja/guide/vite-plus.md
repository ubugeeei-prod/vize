---
title: Vite+ 統合
description: vite.config.ts に Vue コンパイル・型チェック・lint・フォーマットをまとめる。
---

# Vite+ 統合

Vite+ と Vize を1つの `vite.config.ts` で設定します。
プロジェクトの Vite+（0.2.3 以降）を使い、Vize が Vite+ を入れ替えることはありません。

```bash
vp install -D @vizejs/vite-plugin
```

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({});
```

既存の Vue plugin の import と `vue()` を削除します。
[移行ガイド](./migration.md)にそのまま使える差分を記載しています。
統合のタスクだけを使う場合、別の `vize.config.ts` は不要です。

## 設定

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  compiler: { sourceMap: true },
  typecheck: { strict: true },
  lint: {
    vize: {
      preset: "essential",
      rules: { "vue/no-v-html": "error" },
      locale: "ja",
      helpLevel: "short",
    },
    rules: { "no-debugger": "error" },
  },
  fmt: { vize: { printWidth: 100 } },
  server: { port: 3000 },
});
```

Vue の compiler 設定は `compiler`、Vize のルールは `lint.vize`、Vue のフォーマットは
`fmt.vize`、型チェックは `typecheck` に設定します。その他の `lint`・`fmt` は Vite+ の
Oxlint・Oxfmt 設定です。[設定ガイド](./configuration.md)に対応表をまとめています。

<span id="tasks"></span>

## 実行するタスク

| コマンド | 実行するもの |
| --- | --- |
| `vp dev`, `vp build` | Vize で Vue をコンパイル |
| `vp run check` | 型チェック、lint、フォーマット検査 |
| `vp run typecheck` | Vue の型チェック |
| `vp run lint` | Vize と Oxlint の lint |
| `vp run lint:fix` | 両方の lint 修正 |
| `vp run fmt` | Vue は Vize、その他は Oxfmt で書き込み |
| `vp run fmt:check` | 書き込まずフォーマットを検査 |
| `vp run editor:setup` | 推奨拡張機能と Vue エディター設定 |

`vp run check -- --fix` は lint 修正とフォーマットを適用します。
パスは `vp run lint -- src` のように `--` の後ろへ渡せます。

**組み込みの `vp check`・`vp lint`・`vp fmt` は生成タスクを実行しません。**
Vize を含む検査には上の `vp run` を使います。
既存の package script は保持されます。`check` がすでにあれば生成名は `vize:check`、
`lint` があれば `vize:lint` です。明示的な `run.tasks` が優先されます。

生成タスクの名前変更や無効化は、第2引数で設定します。

```ts
export default defineConfig({}, { tasks: { check: "verify", preview: false } });
```

`tasks: false` ですべての生成タスクを無効化できます。

<span id="lint-and-formatter-ownership"></span>

## lint とフォーマットの担当範囲

Vize のネイティブ linter は Oxlint と一緒に動作します。`oxlint-plugin-vize` の別登録は不要です。
Vue ルールの重複はデフォルトで調整され、明示的な `lint.rules` や overrides が優先されます。

Vize は Vue ファイル、Oxfmt はその他のファイルをフォーマットします。
`fmt.ignorePatterns` は両方に適用されます。
`lint.vize: false` または `fmt.vize: false` で、その担当を Vite+ に戻せます。
`compiler: false` なら既存の Vue compiler plugin を保持します。

## 次の設定

- [移行](./migration.md) — 既存の import・plugin・script から置き換える。
- [設定](./configuration.md) — 必要な設定場所を選ぶ。
- [型宣言・共有設定・editor の詳細（英語）](../../guide/vite-plus.md) — `pack.vize`、`extends` と高度なオプション。
- [エディター設定（英語）](../../guide/vite-plus-editor.md) — エディターと統合設定の役割。
