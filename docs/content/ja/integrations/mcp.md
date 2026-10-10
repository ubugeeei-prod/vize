---
title: MCPサーバー
---

<!-- Reviewed translation; source: integrations/mcp.md; scope: complete document and client setup -->

# MCPサーバー

Musea の MCP サーバーは、props・emit されるイベント・バリアント・デザイントークンを
MCP 対応のアシスタントに渡します。コンポーネントを探したり、実際の API に沿った
使用例を作ったりするときに使います。Vue SFC から props と emits を直接解析することもできます。

まず [パッケージをインストール](#インストール) し、[MCP クライアントを接続](#設定) してください。
「Button コンポーネントのバリアントを一覧にして」のような具体的な質問から試せます。
コンポーネント一覧を使う場合は、[Musea ガイド](../guide/musea.md) を読み、art ファイルを追加してください。
サーバーは実験段階です。採用前に [サポート区分](../stability.md) を確認してください。

## インストール

[Vite+ インストールガイド](https://viteplus.dev/guide/install) から `vp` を一度インストールし、
コンポーネントがあるプロジェクトのディレクトリで次のコマンドを実行します。

```bash
vp install -D @vizejs/musea-mcp-server
```

インストールされる実行コマンドは `musea-mcp` です。プロジェクトのルートを引数に渡します。

```bash
vp exec musea-mcp /absolute/path/to/project
```

起動後は stdio 経由で MCP クライアントからの接続を待ちます。起動メッセージは stderr に出力します。
この手動起動を終了してから、クライアントを接続してください。ルートを省略した場合は
`MUSEA_PROJECT_ROOT`、次に作業ディレクトリを使います。以下の接続例ではルートを明示しています。

## 設定

<a id="クロードコード付き"></a>

### Claude Code で使う

プロジェクトのディレクトリで、インストール済みのサーバーを登録します。

```bash
claude mcp add --transport stdio --scope project vize-musea -- \
  vp -C "$PWD" exec musea-mcp "$PWD"
claude mcp get vize-musea
```

project スコープの設定は、プロジェクトのルートにある `.mcp.json` に保存されます。
Claude Code を開き、`/mcp` で接続状況を確認してください。プロジェクトのサーバーの承認を求められたら、
表示された設定を確認します。スコープと接続状況の詳細は
[Claude Code の公式 MCP ガイド](https://code.claude.com/docs/en/mcp#project-scope) を参照してください。

`.mcp.json` を直接編集する場合は、次のエントリーを追加します。2 か所のプロジェクトパスを
実際の絶対パスに置き換え、`mcpServers` 内の既存のエントリーは残してください。

```json
{
  "mcpServers": {
    "vize-musea": {
      "type": "stdio",
      "command": "vp",
      "args": ["-C", "/absolute/path/to/project", "exec", "musea-mcp", "/absolute/path/to/project"]
    }
  }
}
```

`-C` は `vp exec` がインストール済みの依存を探すディレクトリを指定し、最後の引数は
Musea がファイルを読み取るルートを指定します。チーム内でディレクトリが異なる場合は、
[公式ガイドの環境変数展開](https://code.claude.com/docs/en/mcp#environment-variable-expansion-in-mcp-json)
を使ってパスを調整してください。

<a id="クロードデスクトップを使用"></a>

### Claude Desktop で使う

Developer 設定で **Edit Config** を選び、`claude_desktop_config.json` にエントリーを追加します。
`command` を `vp` の実行ファイルの絶対パスに、2 か所のプロジェクトパスを実際の絶対パスに
置き換えてください。他のサーバーのエントリーは残します。

```json
{
  "mcpServers": {
    "vize-musea": {
      "command": "/absolute/path/to/vp",
      "args": ["-C", "/absolute/path/to/project", "exec", "musea-mcp", "/absolute/path/to/project"]
    }
  }
}
```

Claude Desktop を完全に終了してから再起動し、Developer 設定でサーバーの接続状況とツールを確認します。
設定ファイルの場所とログの読み方は [公式のローカルサーバー接続ガイド](https://modelcontextprotocol.io/docs/develop/connect-local-servers)
を参照してください。実行ファイルを絶対パスで指定すれば、デスクトップアプリの `PATH` にも依存せず起動できます。

### 他の AI アシスタントとの併用

クライアントの設定形式に合わせて、ローカルの stdio サーバーを登録します。
実行ファイルと引数は Desktop の例と同じものを使い、2 か所のプロジェクトパスを絶対パスで指定してください。

## 使用例

以下のコンポーネント名は、ご自身のプロジェクトの名前に置き換えてください。

### コンポーネントの検出

> 「ボタンのコンポーネントを探して、VFButton のバリアントを一覧にしてください。」

`search_components` で候補を探し、`get_component` でメタデータ・props・バリアントを
確認してから使うコンポーネントを選びます。

### コード生成

> 「VFInput と VFTextarea を使い、バリデーションエラーの状態も含むフォームを作ってください。」

最初にコンポーネントの詳細を読み、返された prop 名とバリアントのテンプレートを使うように依頼します。
生成されたコードを確認し、プロジェクトのチェックを実行してから使ってください。

### API リファレンス

> 「VFNameBadgePreview の props と、user-role に指定できる値を教えてください。」

`analyze_component` は、解決した Vue ソースから静的に抽出した props と emits を返します。
メタデータが足りない場合は、ソースを確認して API を確かめてください。

<a id="文書作成の支援"></a>

### ドキュメント作成

> 「SponsorGrid の props とバリアントに基づくドキュメントを書いてください。」

`generate_docs` はコンポーネントの Markdown 文書を、`generate_catalog` は
コンポーネント一覧の文書を生成します。ソースと意図した使い方に合うかを確認してください。

## 機能

### コンポーネントの検出

- `list_components` は art ファイルのカテゴリ・タグ・ステータス・バリアント名を一覧にします。
- `search_components` はタイトル・説明・カテゴリ・タグ・コンポーネント名から候補を探します。
- `get_component` はメタデータ・バリアント・ソース解析・props のコントロール・リソースへのリンクを返します。
- `recommend_components` は、説明された UI の用途に合う候補を順位付きで返します。

### コンポーネント API

- `analyze_component` は props・デフォルト値・必須かどうか・emit されるイベントを返します。
- `get_palette` は props のコントロール・選択肢・取得できたデフォルト値を推論します。
- 解析結果に含まれない API は、コンポーネントのソースリソースから確認できます。

<a id="ストーリー情報"></a>

### バリアント情報

- `get_variant` はバリアントのテンプレート・メタデータ・デフォルトかどうかを返します。
- `generate_variants` は Vue コンポーネントから art ファイルのコード案を返します。
- `generate_csf` は art ファイルから Storybook CSF のコード案を返します。

### デザイントークン

- `get_tokens` は設定したデザイントークンを JSON または Markdown で読み取ります。
- `search_tokens` は名前・カテゴリのパス・値・説明からトークンを検索します。
- プロジェクト内のファイルやディレクトリを選ぶには、`--tokens-path tokens.json` または
  `MUSEA_TOKENS_PATH` を指定します。省略時は `tokens/`、`design-tokens/`、`style-dictionary/` を探します。

<a id="mcp-とは何ですか"></a>

## MCP とは

Model Context Protocol は、AI アシスタントをツールやデータに接続するためのプロトコルです。
Musea はコンポーネントのメタデータとソースリソースを公開するので、アシスタントは
プロジェクトの実際のコンポーネントを確認してから回答できます。

## 仕組み

```text
AI アシスタント
  ↕ MCP (JSON-RPC over stdio)
@vizejs/musea-mcp-server
  ↕ art ファイルとコンポーネントのソースを読み取る
プロジェクト (*.art.vue ファイルとコンポーネント)
```

サーバーは指定されたルートから `*.art.vue` ファイルを探し、ネイティブバインディングで解析します。
`node_modules/` と `dist/` は検索対象から除外します。ツールやリソースへのリクエストを受けると、
メタデータやリンクされたコンポーネントのソースを返します。空でない検索結果は 5 秒間キャッシュするため、
追加した art ファイルが次の検索で表示される場合があります。
