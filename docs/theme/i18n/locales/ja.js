/**
 * Japanese sidebar labels and chrome strings.
 *
 * Only overrides live here; any path without an entry falls back to the
 * English label in `en.js`, which is where the blog post titles are kept.
 */
((locales) => {
  locales.ja = {
    labels: {
      "/": "概要",
      "/getting-started": "はじめに",
      "/stability": "安定性",
      "/credits": "クレジット",
      "/guide/configuration": "設定",
      "/guide/workflows": "ユーザーワークフロー",
      "/guide/jsx": "JSX & TSX",
      "/guide/jsx-babel-compat": "Babel JSX 互換",
      "/guide/troubleshooting": "トラブルシューティング",
      "/guide/cli": "CLI",
      "/guide/vite-plugin": "Vite プラグイン",
      "/guide/unplugin": "バンドラー統合",
      "/guide/wasm": "WASM バインディング",
      "/guide/static-analysis": "静的解析",
      "/guide/cross-file-complexity": "ファイル横断の複雑性",
      "/guide/analysis-diagnostics": "診断",
      "/guide/oxlint": "Oxlint プラグイン",
      "/guide/comment-annotations": "コメント注釈",
      "/rules": "ルール概要",
      "/rules/all": "すべてのルール",
      "/rules/vue": "Vue",
      "/rules/type-and-script": "型とスクリプト",
      "/rules/html": "HTML",
      "/rules/accessibility": "アクセシビリティ",
      "/rules/ssr": "SSR",
      "/rules/vapor": "Vapor",
      "/rules/musea-and-css": "Musea と CSS",
      "/rules/cross-file": "ファイル横断ルール",
      "/guide/musea": "Musea",
      "/integrations/nuxt": "Nuxt",
      "/integrations/vscode": "VS Code",
      "/integrations/mcp": "MCP サーバー",
      "/architecture/overview": "アーキテクチャ概要",
      "/architecture/crates": "クレート",
      "/architecture/source-guide": "ソースガイド",
      "/architecture/language-engineering-practices": "言語エンジニアリング",
      "/architecture/performance": "パフォーマンス",
      "/philosophy": "思想",
      "/blog": "概要",
      "/blog/notes": "ノート",
      "/blog/releases": "リリース",
    },
    ui: {
      groups: {
        start: "スタート",
        projectSetup: "プロジェクト設定",
        staticAnalysis: "静的解析",
        rules: "ルール",
        tooling: "ツール",
        architecture: "アーキテクチャ",
        blog: "ブログ",
      },
      language: "言語",
      more: "その他",
      search: "検索",
      searchPlaceholder: "ドキュメントを検索...",
      footer:
        '<a href="https://opensource.org/licenses/MIT">MIT License</a> の下で公開されています。',
    },
  };
})((globalThis.__vizeDocsLocales ??= {}));
