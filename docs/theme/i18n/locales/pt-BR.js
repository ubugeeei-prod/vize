/**
 * Brazilian Portuguese sidebar labels and chrome strings.
 *
 * Only overrides live here; any path without an entry falls back to the
 * English label in `en.js`, which is where the blog post titles are kept.
 */
((locales) => {
  locales["pt-BR"] = {
    labels: {
      "/": "Visão geral",
      "/getting-started": "Primeiros passos",
      "/stability": "Estabilidade",
      "/credits": "Créditos",
      "/guide/configuration": "Configuração",
      "/guide/workflows": "Fluxos de trabalho",
      "/guide/jsx": "JSX e TSX",
      "/guide/jsx-babel-compat": "Compatibilidade com Babel JSX",
      "/guide/troubleshooting": "Solução de problemas",
      "/guide/cli": "CLI",
      "/guide/vite-plugin": "Plugin do Vite",
      "/guide/unplugin": "Integrações com bundlers",
      "/guide/wasm": "Bindings WASM",
      "/guide/static-analysis": "Análise estática",
      "/guide/cross-file-complexity": "Complexidade entre arquivos",
      "/guide/analysis-diagnostics": "Diagnósticos",
      "/guide/oxlint": "Plugin do Oxlint",
      "/guide/comment-annotations": "Anotações em comentários",
      "/rules": "Visão geral das regras",
      "/rules/all": "Todas as regras",
      "/rules/vue": "Vue",
      "/rules/type-and-script": "Tipos e scripts",
      "/rules/html": "HTML",
      "/rules/accessibility": "Acessibilidade",
      "/rules/ssr": "SSR",
      "/rules/vapor": "Vapor",
      "/rules/musea-and-css": "Musea e CSS",
      "/rules/cross-file": "Regras entre arquivos",
      "/guide/musea": "Musea",
      "/integrations/nuxt": "Nuxt",
      "/integrations/vscode": "VS Code",
      "/integrations/mcp": "Servidor MCP",
      "/architecture/overview": "Visão geral da arquitetura",
      "/architecture/crates": "Crates",
      "/architecture/source-guide": "Guia do código-fonte",
      "/architecture/language-engineering-practices": "Engenharia de linguagens",
      "/architecture/performance": "Desempenho",
      "/philosophy": "Filosofia",
      "/blog": "Visão geral",
      "/blog/notes": "Notas",
      "/blog/releases": "Versões",
    },
    ui: {
      groups: {
        start: "Início",
        projectSetup: "Configuração do projeto",
        staticAnalysis: "Análise estática",
        rules: "Regras",
        tooling: "Ferramentas",
        architecture: "Arquitetura",
        blog: "Blog",
      },
      language: "Idioma",
      more: "Mais",
      search: "Pesquisar",
      searchPlaceholder: "Pesquisar na documentação...",
      footer: 'Publicado sob a <a href="https://opensource.org/licenses/MIT">Licença MIT</a>.',
    },
  };
})((globalThis.__vizeDocsLocales ??= {}));
