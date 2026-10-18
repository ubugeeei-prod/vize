---
title: 貢献する
---

<!-- Reviewed translation; source: contributing.md -->

# 貢献する

Vize の改善に協力するための、開発環境の準備、検証、プルリクエストの手順を説明します。
現在は実際のプロジェクトで正確さを確かめながら、v1 アルファに向けて開発を進めています。
変更は一つの目的に絞り、検証結果を添えてください。
問題の報告から始める場合は、[テストとフィードバックのガイド](./guide/testing.md)を参照してください。

<a id="設定"></a>

## 開発環境の準備

Node.js は `package.json#devEngines.runtime`、Rust は `rust-toolchain.toml` に指定されたバージョンを使います。
サポートする Rust の最小バージョン（MSRV）は、`Cargo.toml` の
`[workspace.package].rust-version` で `1.99.0` と定めています。変更後もこのバージョンでコンパイルできる必要があります。

通常の Nix シェルには、ローカル開発用のツールが含まれています。
Blacksmith Testbox を使う場合は、バージョンを固定した Blacksmith CLI、`rsync`、GitHub CLI を含む専用シェルを選びます。

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

まず変更箇所に対応するチェックを実行します。複数の機能で使う処理を変えた場合は、影響する機能も検証してください。

```sh
vp check <changed-files>
node --test tests/tooling/<test-file>.test.ts
cargo fmt --all -- --check
cargo test -p <crate>
```

共有ツール、リリースの自動化、ネイティブバインディング、コンパイラの挙動を変える場合は、
可能な範囲で CI と同じワークスペースタスクをローカルでも実行してから PR を作成してください。

ルートのビルド、テスト、lint は通常、ローカルで実行します。外部サービスの認証情報は必要ありません。

```sh
vp run --workspace-root build
vp run --workspace-root test
vp run --workspace-root lint
```

Nix の開発シェルでは、`vp build`、`vp test`、`vp lint` で同じワークスペースタスクを実行できます。

Linux の CI と同じ環境で検証する場合は、専用の Testbox シェルを起動します。
通常の `nix develop` シェルには Blacksmith を含めていないため、その配布ファイルや認証情報なしで利用できます。

```sh
nix develop .#testbox
```

続いて、次の関数を実行します。前回のボックス ID を消去してから起動し、認証、push、起動のいずれかに失敗したら
リモートのタスクを実行しません。起動に成功したボックスは、検証が失敗した場合も停止処理を行います。

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

Blacksmith Testbox のジョブを変更した場合は、
`node --test tests/tooling/github-workflows.test.ts` でワークフローの構成も検証してください。

<span id="言語プロセッサの規律変更"></span>

## 言語処理系の変更方針

Vize は rustc、TypeScript、TypeScript-Go、Flow の開発手法を参考にしています。
変更の種類を整理し、挙動を確認できる小さなテスト入力を追加して、生成結果をレビューします。
影響する機能に応じて、互換性、性能、リリース前の検証へと範囲を広げます。
変更の種類と必要な検証の対応は、[言語処理系の開発ガイド](./architecture/language-engineering-practices.md)にまとめています。

PR の説明では、該当する変更の種類を示してください。

- パーサーまたは AST
- コンパイラとコード生成
- 意味解析、lint、複数ファイルの解析
- 仮想 TypeScript と型チェック
- フォーマッタと LSP
- ランタイムの配布、リリース、ドキュメント

言語の挙動を変える場合は、その挙動を確かめるテスト入力やスナップショットの差分を添えてください。
スナップショットを更新するときは、新しい結果が正しい理由を説明します。
その出力全体の変更を扱う PR でなければ、無関係な期待値をまとめて更新しないでください。

外部からの再現例や手元のファイルでコンパイラの出力に差が見つかった場合は、プレイグラウンドの
[コンパイラインスペクタ](./guide/compiler-inspector.md)で、公式 Vue と Vize の出力、
仮想 TypeScript、VIR、複数ファイルのグラフを比較します。
インスペクタのパーマリンクを PR に添え、再現例を最小化したテスト入力や出力全体のスナップショットを追加してください。
手元のファイルをまとめる場合は `vize inspector <file-or-glob>`、
エージェントに引き継ぐ場合は `vize inspector --format agent` を使えます。

## プルリクエスト

- コミットメッセージと PR タイトルには Conventional Commits を使います。
  例: `fix(vite-plugin): surface SFC compile errors`。
- PR は一つの挙動の変更、または一つのドキュメント・運用方針の変更に絞ります。
- PR の本文に検証コマンドと結果を記載します。
- 出力全体の変更を扱う PR でなければ、スナップショットの期待値を大量に更新しないでください。
- 秘密情報、レジストリのトークン、未公開の脆弱性の詳細、手元の端末のローカルパスを、
  報告、コミット、PR に含めないでください。

<a id="修正リクエスト"></a>

## 不具合や機能の提案を報告する

以前は動いていた機能の不具合、クラッシュ、誤った診断、パッケージのインストールやリリースの問題には、
不具合報告のテンプレートを使ってください。
新しい連携、API の変更、作業手順の改善には、機能リクエストのテンプレートを使います。
小さな再現例やプレイグラウンドのパーマリンクがあると、問題を調べやすくなります。

セキュリティの問題は公開の不具合報告に書き込まず、
[`SECURITY.md`](https://github.com/ubugeeei-prod/vize/blob/main/SECURITY.md) の手順に従って報告してください。

外部の利用者から届いた再現可能な不具合報告は、ロードマップの作業より優先します。
小さな回帰テスト、修正のレビュー、GitHub Actions の成功、merge queue での実際のマージ、
公開された成果物の検証まで対応してください。
PR のマージだけでは対応は完了しません。公開パッケージやリリースの成果物で再現例を確認してから Issue を閉じます。

再現例や調査を提供した報告者は、修正コミットの `Co-authored-by` トレーラで記録し、squash 時も保持します。
GitHub の公開プロフィールを確認し、本人が公開したメールアドレスか、確認した
`ID+LOGIN@users.noreply.github.com` を使ってください。非公開のメールアドレスを推測してはいけません。
報告へのリンクと謝辞はリリースノートにも残します。
過去の修正への謝辞を追加する場合は、新しい変更として記録し、公開済みの修正やタグを書き換えないでください。

## 行動規範とガバナンス

プロジェクトへの参加にあたっては、
[Contributor Covenant v2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/) の行動規範を守ってください。
運営方針と意思決定の手順は [`GOVERNANCE.md`](https://github.com/ubugeeei-prod/vize/blob/main/GOVERNANCE.md)、
問い合わせ先は [`SUPPORT.md`](https://github.com/ubugeeei-prod/vize/blob/main/SUPPORT.md) にまとめています。
