---
title: 貢献する
---

<!-- Generated translation; source: contributing.md -->

# 貢献する

Vize のシャープ化にご協力いただきありがとうございます。プロジェクトは**現実世界テスト**段階にあり、現在進行中です
v1 アルファに向けては、明確な検証を伴う小規模で焦点を絞った変更が最もレビューしやすいです。もしあなたが
PR を開くのではなく、調査結果を報告するためにここにいます。まず、
[テストとフィードバック](./guide/testing.md) ガイド。

## 設定

Node.js バージョンは `.node-version` から、Rust バージョンは `rust-toolchain.toml` から使用します。の
ワークスペースは、`Cargo.toml` で `1.98.0` のサポートされる最小 Rust バージョン (MSRV) を宣言します
(`[workspace.package].rust-version`);コントリビューションはそのバージョンでコンパイルする必要があります。

デフォルトの Nix シェルには、再現可能なローカル ツールチェーンが含まれています。 Blacksmith テストボックスのサポートは
オプションであり、固定された Blacksmith CLI、`rsync`、および GitHub CLI を含む別のシェルに存在します。

```sh
nix develop             # local development
nix develop .#testbox   # hosted Testbox workflows
```

ワークスペースのルートから依存関係をインストールします。

```sh
vp install --frozen-lockfile --prefer-offline
```

`vp` がまだ利用できない場合は、まず [Vite+](https://viteplus.dev/guide/install) をインストールしてください。

## 一般的なチェック

変更をカバーする最も狭いチェックを実行し、共有された動作に触れたら範囲を広げます。

```sh
vp check <changed-files>
node --test tests/tooling/<test-file>.test.ts
cargo fmt --all -- --check
cargo test -p <crate>
```

共有ツール、リリース自動化、ネイティブ バインディング、またはコンパイラを変更する PR を開く前に
動作を考慮し、実際的な場合は、CI から関連するワークスペース タスクをローカルで実行します。

ルートのビルド、テスト、および lint ワークフローはデフォルトでローカルであり、ホストされた認証情報は必要ありません。

```sh
vp run --workspace-root build
vp run --workspace-root test
vp run --workspace-root lint
```

Nix 開発シェル内では、`vp build`、`vp test`、および `vp lint` はこれらの短縮形です。
ワークスペースのタスク。

1 つのコマンドで Linux CI パリティを実行するには、専用の Testbox シェルを入力します。デフォルトの `nix develop` シェル
Blacksmith を意図的に省略し、ホストされるアーティファクトや認証情報を必要としません。

```sh
nix develop .#testbox
```

次に、以下の保護されたライフサイクルを実行します。ウォームアップ前に古いボックス ID をクリアし、次の場合はリモート タスクをスキップします。
認証、プッシュ、またはウォームアップが失敗し、正常にウォームアップされたボックスを常に停止しようとします。
タスクが失敗したとき:

```sh
run_testbox_checks() {
  unset BLACKSMITH_TESTBOX_ID testbox_output
  "$VIZE_BLACKSMITH_BIN" auth login || return
  git push --set-upstream origin "$(git branch --show-current)" || return

  if testbox_output="$(vp run --workspace-root testbox:warmup)"; then
    BLACKSMITH_TESTBOX_ID="$(printf '%s\n' "$testbox_output" | tail -n1)"
  else
    warmup_status=$?
    unset testbox_output
    return "$warmup_status"
  fi
  if [ -z "$BLACKSMITH_TESTBOX_ID" ]; then
    printf '%s\n' "Testbox warmup returned no box id." >&2
    unset BLACKSMITH_TESTBOX_ID testbox_output
    return 1
  fi
  export BLACKSMITH_TESTBOX_ID

  if vp run --workspace-root build:testbox &&
    vp run --workspace-root test:testbox &&
    vp run --workspace-root lint:testbox; then
    testbox_status=0
  else
    testbox_status=$?
  fi
  if vp run --workspace-root testbox:stop; then
    stop_status=0
  else
    stop_status=$?
  fi
  unset BLACKSMITH_TESTBOX_ID testbox_output

  if [ "$testbox_status" -ne 0 ]; then
    return "$testbox_status"
  fi
  return "$stop_status"
}
run_testbox_checks
```

GitHub Actions の変更については、プッシュする前に `actrun` を使用してワークフロー グラフをリントまたはプレビューします。

```sh
vp run actrun:lint
vp run actrun:dry-run
vp run actrun:job --job check-js
```

Blacksmith Testbox のジョブ変更の場合は、ワークフローの形状も検証します。
`node --test tests/tooling/github-workflows.test.ts`。

## 言語プロセッサの規律変更

Vize は、rustc、TypeScript、TypeScript-Go、Flow のコンパイラ プロジェクトの実践に従っています。
変更し、意味のある最小のフィクスチャを追加し、生成された出力をコントラクトとして確認してから、次のように拡張します。
パリティ、パフォーマンス、またはタッチされたサーフェスが必要なときにゲートを解放します。参照
[言語エンジニアリングの実践](./architecture/language-engineering-practices.md) (全文)
マトリックス。

該当する場合は、PR で次の変更クラスのいずれかを使用します。

- パーサーまたはAST
- コンパイラーとコード生成
- セマンティック分析、lint、およびクロスファイル分析
- 仮想 TypeScript と型チェック
- フォーマッタとLSP
- ランタイムパッケージ、リリース、またはドキュメント

言語に関係する変更の場合は、動作を証明するフィクスチャまたはスナップショットの差分を含めてください。のために
スナップショットを更新し、新しい出力が正しい理由を説明し、広範なベースラインの変動を回避します。
PR は特にその出力ファミリーに関するものです。

コンパイラの不一致が外部再現またはローカル プロジェクト ファイルから始まる場合は、プレイグラウンドを使用します。
[Compiler Inspector](./guide/compiler-inspector.md) 公式の Vue 出力、Vize 出力を検査します。
仮想 TS、VIR、およびクロスファイル グラフ。インスペクターのパーマリンクを PR 本文に追加し、
出力をレビュー済みの契約に変える最小化されたフィクスチャまたは完全なスナップショット。ローカルバッチでは、
`vize inspector <file-or-glob>` とともにパッケージ化され、エージェントのハンドオフで使用できます
`vize inspector --format agent`。

## プルリクエスト

- コミット メッセージと PR タイトルには従来のコミットを使用します。
  `fix(vite-plugin): surface SFC compile errors`。
- PR は 1 つの行動の変化または 1 つの文書/ガバナンスの変更に焦点を当て続けます。
- PR 本文に検証コマンドを含めます。
- PR が特にそれらの出力に関するものでない限り、大規模なスナップショット ベースラインを更新しないでください。
- シークレット、レジストリ トークン、プライベート脆弱性の詳細、またはマシンのローカル パスを含めないでください。
  レポート、コミット、または PR。

## 修正リクエスト

リグレッション、クラッシュ、誤った診断、パッケージのインストールには修正レポート テンプレートを使用します
問題やリリースの失敗など。新しい統合、API の変更、
またはワークフローの改善。最小限の複製 (理想的にはプレイグラウンド インスペクター リンク) により、
報告がはるかに早くなり、それに基づいて行動できるようになります。

セキュリティレポートはその後に続く必要があります
一般公開の代わりに [`SECURITY.md`](https://github.com/ubugeeei-prod/vize/blob/main/SECURITY.md)
テンプレートを修正します。

## 行動規範とガバナンス

参加すると、次の事項に従うことに同意したことになります。
[寄稿者規約 v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/)。の
ガバナンス モデルと意思決定プロセスは、以下に文書化されています。
[`GOVERNANCE.md`](https://github.com/ubugeeei-prod/vize/blob/main/GOVERNANCE.md)。探すのに役立つ
右チャンネルについては、[`SUPPORT.md`](https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md) を参照してください。
