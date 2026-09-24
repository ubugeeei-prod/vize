---
title: ソース配布 (vize lib)
---

<!-- Generated translation; source: guide/lib-pull.md -->

# ソース配布 (`vize lib`)

`vize lib` は `@vizejs/ui` のコンポーネントと `@vizejs/composable` のコンポーザブルを、shadcn/ui と同じ
スタイルで **ソースとして** プロジェクトにコピーします。コピーしたファイルはあなたのものです。自由に編集でき、
Vize は各ファイルがどのパッケージバージョン由来かを記録するので、アップグレードも安全に行えます。

```bash
vpx vize lib pull rating
```

`rating` ファミリーと、それが import するもの (`controllable-state`、`id` など) がすべて
`src/components/vize/` に書き込まれ、`vize-lib.lock.json` に記録されます。

## ソースの取得元

公開されている `@vizejs/ui` と `@vizejs/composable` の tarball には、バージョン付きのレジストリが含まれています。

```text
node_modules/@vizejs/ui/
  registry/
    registry.json          # アイテム、ファイル、sha256 ダイジェスト、依存関係
    files/families/...     # 生の .vue / .ts / .css ソース (テストは除外)
```

`vize lib` は次の順序でレジストリを解決します。

1. `--registry <path>`: `registry.json`、そのディレクトリ、または展開済みパッケージのディレクトリ。ui と
   composable の両方を渡すにはフラグを繰り返します。この場合、自動検出は無効になります。
2. プロジェクトにインストール済みのパッケージ (`node_modules/@vizejs/ui/registry/registry.json`。Node の解決と
   同様に親ディレクトリも探索します)。
3. インストール済みのバージョンと一致しないバージョンが指定された場合、またはパッケージが未インストールの場合
   (`@latest`) は、`npm pack @vizejs/<pkg>@<version>` を一時ディレクトリに実行し `tar -xzf` で展開します。
   `--offline` を指定するとこの手順を禁止します。

レジストリサーバーや追加の HTTP クライアントは使いません。レジストリは npm がすでに配信している tarball
そのものなので、すべてのバージョンは不変で再現可能です。

## コマンド

| コマンド                                 | 内容                                                                       |
| ---------------------------------------- | -------------------------------------------------------------------------- |
| `vize lib list [--kind ui\|composable]`  | 取得可能なアイテムを一覧表示します。                                       |
| `vize lib search <words>`                | 名前、タイトル、説明、エイリアスで検索します。                             |
| `vize lib info <name>`                   | ファイル、レジストリ依存、npm peer、パッケージバージョンを表示します。     |
| `vize lib pull <item>... [--dir <dir>]`  | アイテムとレジストリ依存をコピーします。`--dry-run`、`--overwrite`。       |
| `vize lib status`                        | 取得済みファイルをロックファイルとインストール済みレジストリと比較します。 |
| `vize lib diff <name> [--to <version>]`  | ローカルのコピーからレジストリのバージョンへの unified diff を表示します。 |
| `vize lib update [<name>...] [--to <v>]` | ローカルの編集を壊さずに upstream の変更を適用します。`--dry-run`、`--force`。 |
| `vize lib remove <name>...`              | アイテムと、他に必要とされない依存を削除します。`--dry-run`、`--force`。   |

すべてのコマンドは機械可読な出力のための `--json` と、別のプロジェクトを対象にする `--root <dir>` を受け付けます。

### アイテムの指定

アイテムは正規名 (`rating`)、エイリアス (`star rating`、`useToggle`)、両方のパッケージに同名がある場合の
種類プレフィックス (`ui:locale`、`composable:locale`)、正確なパッケージバージョン (`rating@0.427.0`、
`composable:use-toggle@0.427.0`) で指定できます。

### 配置先ディレクトリ

取得したファイルは種類ごとに 1 つのディレクトリの下でレジストリのレイアウト
(`families/form/rating/rating.vue`、`foundations/id/deterministic-id.ts` など) を保つため、アイテム間の相対
import は書き換えなしでそのまま動作します。ディレクトリは次の順で決まります。

1. `--dir <dir>` (プロジェクト内である必要があります)
2. `vize.config.*` の `lib` セクション
3. レジストリの既定値: `src/components/vize` (ui)、`src/composables/vize` (composable)

```ts
// vize.config.ts
import { defineConfig } from "vize";

export default defineConfig({
  lib: {
    uiDir: "src/ui/vendor",
    composableDir: "src/composables/vendor",
    // dir: "src/vendor",        // 両方の種類に共通のフォールバック
    // lockfile: "vize-lib.lock.json",
  },
});
```

ある種類を一度ディレクトリに取得すると、その種類の以降の取得は同じディレクトリを使います。依存グラフを分割
してしまう競合する `--dir` は拒否されます。

取得したソースが import するのは相対パスと `vue` などの npm パッケージだけです。`pull` は `package.json` に
まだ宣言されていない npm 依存を報告しますが、パッケージのインストールは行いません。

## バージョン管理と安全な更新

`vize-lib.lock.json` (コミットしてください) には、アイテムごとに取得元のパッケージと正確なバージョン、
レジストリの `contentHash`、直接要求したか依存として入ったか、そして取得時の各ファイルの SHA-256 が記録されます。
これらのダイジェストが **あなたのファイル**、**取得時のファイル**、**新しいレジストリのファイル** の 3-way 比較の
マージベースになります。

| あなたのファイル vs 取得時 | レジストリ vs 取得時 | `update` / `pull` の動作                                   |
| -------------------------- | -------------------- | ---------------------------------------------------------- |
| 変更なし                   | 変更なし             | 何もしない (`unchanged`)                                   |
| 変更なし                   | 変更あり             | 置き換える (`update`)                                      |
| 編集あり                   | 変更なし             | 編集を保持する (`keep-local`)                              |
| 編集あり                   | 変更あり             | `--force` / `--overwrite` がなければ拒否 (`conflict`)      |
| 変更なし                   | upstream で削除      | 削除する (`delete`)                                        |
| 編集あり                   | upstream で削除      | `--force` がなければ拒否 (`conflict-delete`)               |
| 存在しない                 | 任意                 | 復元する (`create`)                                        |
| 存在するがロック外         | 任意                 | `--overwrite` がなければ拒否 (`conflict`)                  |

未解決の競合がある間は何も書き込まれないため、拒否された更新はファイルもロックファイルも変更しません。
`vize lib diff <name> --to <version>` で upstream の変更を確認し、手でマージしてから `update --force` を実行してください。

典型的なアップグレード:

```bash
pnpm add @vizejs/ui@latest       # または: vize lib update --to 0.428.0
vize lib status                  # 更新やローカル編集のあるアイテム
vize lib update --dry-run        # ファイル操作をプレビュー
vize lib update                  # 適用。競合があれば一覧表示
```

`remove` はアイテムと、そのためだけに取得された依存をすべて削除します。他の取得済みアイテムがまだ対象を
import している間は拒否し、`--force` がない限りローカルで編集されたファイルは残します。

## レジストリの形式

レジストリドキュメントは
[`vize-lib-registry.schema.json`](https://github.com/ubugeeei-prod/vize/blob/main/npm/cli/schemas/vize-lib-registry.schema.json)、
ロックファイルは
[`vize-lib-lock.schema.json`](https://github.com/ubugeeei-prod/vize/blob/main/npm/cli/schemas/vize-lib-lock.schema.json)
で定義されています。どちらも `vize` npm パッケージの `schemas/` に同梱されています。

- `registryDependencies` はビルド時に相対 import グラフから計算した完全な推移閉包です。共有の基盤は別アイテム
  なので、どちらも `id` を必要とする 2 つのコンポーネントを取得しても `id` は一度だけコピーされます。
- 各ファイルは `sha256` を持ち、`vize lib` はコピーするすべてのバイトをそれで検証します。
- `contentHash` はアイテムのファイルが変わったときだけ変化するため、`status` はバージョン間で実際にソースが
  異なるアイテムについてのみ「update available」を報告します。
