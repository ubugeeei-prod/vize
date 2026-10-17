---
title: プロジェクトの初期設定
---

<!-- Reviewed translation; source: guide/init.md -->

<span id="project-setup-with-vize-init"></span>

# `vize init` で既存プロジェクトに導入する

`vize init` は、既存のプロジェクトに Vize を導入するためのコマンドです。
検出結果を確認し、必要な機能を選ぶと、設定ファイルの編集と依存パッケージのインストールを進めます。
通常は対話形式で実行し、フラグで選択肢を指定することもできます。
プロジェクトの `package.json` があるディレクトリから実行してください。

```bash
vpx vize init
```

選べる機能は次の5つです。ビルド連携には、Vite と Nuxt の2通りがあります。

| 機能 | 設定する内容 |
| --- | --- |
| Oxlint プラグイン | lint コマンドが読み込む設定ファイルに `oxlint-plugin-vize` を追加 |
| Vite プラグイン | `@vizejs/vite-plugin` を `vite.config.*` に追加 |
| Nuxt モジュール | `@vizejs/nuxt` を `nuxt.config.*` に追加 |
| フォーマット | プロジェクトの設定と既定値を使い、`vize:fmt` スクリプトを追加 |
| 型チェック | 既存の TypeScript 設定を使うか、なければ作成し、`vize:check` スクリプトを追加 |
| エディター | `.vscode/extensions.json` に推奨する拡張機能を追加 |

Vite プラグインと Nuxt モジュールは、どちらも「ビルド連携」の選択肢です。
`init` がプロジェクトを検出し、使用する連携方法を選びます。

init は既存の設定を保ちながら編集し、専用の Vize 設定ファイルは新規作成しません。
共通設定と、使用しているリリースでの対応状況は[設定ガイド](./configuration.md)で確認してください。

<span id="what-it-detects"></span>

## 最初に確認する検出結果

機能を選ぶ前に、プロジェクトの検出結果が表示されます。
対象のディレクトリやパッケージマネージャーが正しいか、ファイルを変更する前に確認してください。

```
[vize init] detected in /path/to/project:
  framework:       Vite+ (vite.config.ts)
  package manager: pnpm
  language:        TypeScript (tsconfig.json)
  lint command:    vp lint
  Vue settings:    vue: in vite.config.*
  oxlint config:   none
```

| 検出する項目 | 判定方法 |
| --- | --- |
| フレームワーク | `nuxt.config.{ts,mts,js,mjs}` または `nuxt` の依存関係があれば Nuxt。それ以外で `vite.config.*` があれば Vite。どちらもなければ連携対象なし |
| 両方ある場合 | Nuxt を優先。Nuxt が Vite の設定を管理するため、Nuxt モジュールを使います。`--vite` で Vite プラグインの選択に変更できます |
| どちらもない場合 | ビルド連携は利用不可として表示されます。他の4機能は引き続き選べます。`init` がビルド環境を新しく作ることはありません |
| パッケージマネージャー | ロックファイルを `pnpm-lock.yaml`、`bun.lockb`／`bun.lock`、`yarn.lock`、`package-lock.json` の順に確認し、なければ `packageManager` フィールドを使います。Vite+ では既定で `vp add` を使います |
| 言語 | `tsconfig.json` または `typescript` の依存関係があれば TypeScript と判定。型チェックを選ぶと、まだない場合は `tsconfig.json` を作成します |
| lint コマンド | Vite+ なら `vp lint`、それ以外は `oxlint`。**この判定によって、Oxlint の設定を書き込むファイルが決まります** |
| 設定済みの機能 | 既存の Vize 設定、Oxlint 設定、`lint` ブロック、プラグインの登録、依存パッケージの有無を確認します |

<span id="which-lint-file-it-writes"></span>

## lint コマンドが読むファイルに設定する

設定ファイルを作っても、実行するコマンドがそのファイルを読み込まなければ lint は有効になりません。

| コマンド | 読み込む設定 |
| --- | --- |
| `vp lint`、`vp check` | `lint` ブロック（`vite.config.ts`） |
| `oxlint`、`oxlint-vize` | `.oxlintrc.json`／`oxlint.config.ts` |

Vite+ は `.oxlintrc.json` を読み込みません。`.oxlintrc.json` に `jsPlugins` と `vize/*` のルールを
書いても、`vp lint` では使われず、Oxlint に `vize/*` のルールが渡りません。
そのため、Vize の診断が**0件**でも終了コードは `0` になり、設定できたように見えてしまいます。
詳しくは [Oxlint プラグインの設定](./oxlint.md)を参照してください。

`init` は、プロジェクトで実行する lint コマンドに合わせて設定先を選びます。

| プロジェクトの構成 | `init` が書き込む設定 |
| --- | --- |
| Vite+ を使用 | `lint` ブロックを `vite.config.*` に追加。`createVizeLintConfig()` で作成 |
| Vite+ を使用しない | 読み込める設定がなければ `oxlint.config.ts` を作成。既存の設定があれば保持 |
| Vite+ に加えて `oxlint` も直接実行 | 両方の実行方法に対応。新しく作る設定は同じプリセットを使い、読み込める既存の Oxlint 設定は保持 |
| Vite+ の設定を安全に編集できない | lint の設定は手動で編集。[自動編集できない場合](#when-init-refuses-to-edit)を参照 |

`lint` ブロックは `createVizeLintConfig()` で作成します。
プラグインの読み込みに必要な `jsPlugins` を含む設定一式が返るため、その項目を追加し忘れることを防げます。
次の例は lint の設定部分を示しています。既存のプラグインの import などは、プロジェクトの構成に合わせてください。

```ts
// vite.config.ts
import { defineConfig } from "vite-plus";
import { createVizeLintConfig } from "oxlint-plugin-vize";

export default defineConfig({
  lint: createVizeLintConfig({
    preset: "happy-path",
    settings: {
      helpLevel: "short",
    },
  }),
  plugins: [vize()],
});
```

Vite+ を使わない場合は、同じ内容を `oxlint.config.ts` に設定します。

```ts
import { defineConfig } from "oxlint";
import { configs } from "oxlint-plugin-vize";

export default defineConfig({
  plugins: ["vue"],
  jsPlugins: ["oxlint-plugin-vize"],
  settings: {
    vize: {
      preset: "happy-path",
      helpLevel: "short",
    },
  },
  rules: configs.happyPath,
});
```

> [!NOTE]
> Oxlint が自動で読み込むのは `.oxlintrc.json`、`.oxlintrc.jsonc`、`oxlint.config.ts` です。
> `oxlint.config.mjs`、`.js`、`.cjs`、`.mts`、`.cts` は自動検出されません。
> これらのファイルがあっても、`init` は「存在するが読み込まれない」と表示し、有効な設定とは扱いません。

<span id="non-interactive-use"></span>

## CI など対話できない環境で実行する

標準入力が TTY でない場合、`init` は入力を待たずに終了します。
CI などでは `--yes` と機能を選ぶフラグを指定してください。

```bash
vpx vize init --yes --lint --vite --fmt --typecheck --editor
```

この例は `--vite` で Vite プラグインを明示的に選んでいます。Nuxt のプロジェクトでは、
代わりに Nuxt モジュールの選択肢を使ってください。機能のフラグは検出結果を上書きします。
指定していない機能は検出時の選択を引き継ぐため、不要な機能は次の表の無効化用フラグで除外します。

| オプション | 動作 |
| --- | --- |
| `-y`、`--yes` | 対話せずに選択を承認 |
| `--lint`／`--no-lint` | Oxlint プラグインを有効化／無効化 |
| `--vite` | Nuxt が検出されても Vite プラグインを選択 |
| `--nuxt` | Nuxt モジュールを選択 |
| `--bundler`／`--no-bundler` | 自動検出された Vite または Nuxt の連携を有効化／無効化 |
| `--fmt`／`--no-fmt` | `vize fmt` の設定を有効化／無効化 |
| `--typecheck`／`--no-typecheck` | `vize check` の設定を有効化／無効化。まだない場合は `tsconfig.json` を作成 |
| `--editor`／`--no-editor` | `.vscode/extensions.json` の推奨設定を有効化／無効化 |
| `--dry-run` | 変更予定を表示し、ファイルを書き込まずに終了 |
| `--no-install` | 依存パッケージのインストールを省略。設定ファイルは書き込みます |
| `--package-manager <PM>` | `pnpm`、`npm`、`yarn`、`bun`、`vp` のいずれかを使用 |

`--dry-run` は機能を選んだ後、変更予定を表示して終了します。ファイルの書き込みや
依存パッケージのインストールは行いませんが、自動承認を指定しなければ通常どおり対話が必要です。
対話できない環境で変更予定だけを確認するには、`vpx vize init --yes --dry-run` を実行してください。
次は、既存の TypeScript 設定がある Vite+ プロジェクトでの表示例です。

```
[vize init] plan:
  lint      configured Vite+ detected, so `vp lint` reads the `lint` block in vite.config.ts and never reads .oxlintrc.json; writes vite.config.ts
  bundler   configured adds vize() to vite.config.ts
  fmt       configured uses project settings and formatter defaults
  typecheck configured uses tsconfig.json and project settings
  editor    configured writes .vscode/extensions.json recommending ubugeeei.vize
[vize init] would create .vscode/extensions.json
[vize init] would update vite.config.ts
[vize init] would update package.json
[vize init] would add scripts: vize:lint, vize:fmt, vize:fmt:fix, vize:check
[vize init] would run: vp add -D @vizejs/vite-plugin oxlint oxlint-plugin-vize vize
```

<span id="running-it-twice"></span>

## 設定後にもう一度実行する

`init` で設定と依存パッケージの導入が正常に完了した後は、同じ機能とインストール設定で再実行しても、
プラグイン、スクリプト、依存パッケージが重複して追加されることはありません。変更が不要なファイルは、そのまま保たれます。

```
[vize init] plan:
  lint      unchanged  Vite+ detected, so `vp lint` reads the `lint` block in vite.config.ts and never reads .oxlintrc.json
  bundler   unchanged  vite.config.ts already uses @vizejs/vite-plugin
  fmt       unchanged  uses project settings and formatter defaults
  typecheck unchanged  uses tsconfig.json and project settings
  editor    unchanged  .vscode/extensions.json already recommends ubugeeei.vize
[vize init] nothing to do; the project is already configured
```

設定済みの機能は、選択画面でも選択済みの状態で表示されます。
後から機能を追加する場合も、すべてを選び直す必要はありません。

<span id="when-init-refuses-to-edit"></span>

## 自動編集できない場合と途中で失敗した場合

`init` は、安全に変更できない設定を推測で書き換えません。その設定の編集をユーザーに委ね、
対象の機能を**未設定**と表示して、追加するコード例を案内します。他の設定変更やスクリプトの追加、
依存パッケージのインストールは進む場合があります。編集できない機能があると終了コードは0以外になるため、
表示された計画と実際のファイルの差分を確認してください。

ファイルの書き込みと依存パッケージのインストールは別々に進みます。途中でエラーが起きると、
それまでの変更が残る場合があります。表示されたエラーを解消し、変更内容を確認してから再実行してください。

Vite+ に Vize の lint 機能を新しく導入する場合は、既存の `lint` ブロックに注意してください。
Vize の連携がまだ検出されていなければ、`init` は既存の設定を落とさないよう、
自動でまとめて書き換えることを避けます。Vize の連携がすでに検出されている場合は、その設定を保持します。
次は、新規導入時に自動編集できない場合の表示例です。

```
[vize init] lint: NOT configured — vp lint reads the `lint` block in the Vite config, but
vite.config.ts already has a `lint` block and merging into it would risk dropping settings, so
spread createVizeLintConfig() into it by hand

    import { createVizeLintConfig } from "oxlint-plugin-vize";

    export default defineConfig({
      lint: {
        ...createVizeLintConfig({
          preset: "happy-path",
          settings: {
            helpLevel: "short",
          },
        }),
        // keep your existing lint keys here
      },
    });
```

この場合、代わりに別の `oxlint.config.ts` を作ることはありません。それを作っても `vp lint` は
元の `lint` ブロックを読み続けるため、必要な診断が出ないままになってしまうからです。
表示されたコード例を、既存の設定を保ちながら手動で組み込んでください。

その他の設定ファイルの編集にも制限があります。Vite 設定が複数ある場合、`init` は lint の設定先を
決められません。単一の `defineConfig({ ... })` 呼び出しではない設定や、配列リテラルではない
`plugins` は、安全に編集できないことがあります。`.vscode/extensions.json` の JSON の形式が
対応していない場合も同様です。実行全体で何も変更されなかったとは限らないため、機能ごとの結果を確認してください。

<span id="editor-support"></span>

## チーム向けのエディター設定

`init` は、プロジェクトで推奨する拡張機能を設定ファイルに記録します。

```json
{
  "recommendations": ["ubugeeei.vize"]
}
```

このファイルをコミットすると、チーム全員が同じ推奨設定を使えます。`init` を実行した端末に
拡張機能が自動でインストールされるわけではありません。`init` の結果には、VS Code 以外の連携方法も表示されます。

| エディター | 連携方法 |
| --- | --- |
| VS Code | `ubugeeei.vize` |
| Zed | `editors/zed` |
| Neovim | `editors/nvim` |
| Vim | `editors/vim` |
| Helix | `editors/helix` |
| Emacs | `editors/emacs` |

各エディターへの導入手順は [VS Code とその他のエディター](../integrations/vscode.md)を参照してください。

<span id="relationship-to-vize-setup"></span>

## `vize setup` との使い分け

`vize setup` は、Vite または Vite+ のプロジェクトをまとめて設定する従来のコマンドです。
`init` では機能を選べるほか、Nuxt、パッケージマネージャーの検出、変更予定の確認、
対話なしでの実行にも対応しています。既存プロジェクトへ導入する機能を選びたい場合は、`vize init` を使ってください。

<span id="next-steps"></span>

## 導入後に確認すること

- [静的解析の使い方](./static-analysis.md)で、lint と型チェックの役割を確認します。
- [Oxlint プラグインの設定](./oxlint.md)で、プリセットや設定項目を確認します。
- [共通設定の使い方](./configuration.md)で、プロジェクトの設定を調整します。
- [CLI のコマンド一覧](./cli.md)で、他の操作を確認します。
