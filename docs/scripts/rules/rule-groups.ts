export const ruleLocales = ["en", "ja", "zh-CN", "pt-BR", "fr"] as const;
export type RuleLocale = (typeof ruleLocales)[number];

import "../../theme/i18n/sitemap.js";

interface RuleSection {
  page: string;
  namespaces?: readonly string[];
  directory?: string;
}
const sections = (globalThis as { __vizeDocsSitemap?: { ruleSections?: RuleSection[] } })
  .__vizeDocsSitemap?.ruleSections;
if (!sections) throw new Error("docs/theme/i18n/sitemap.js must declare ruleSections");

/** Category pages in sidebar order; the sidebar's namespace table owns membership. */
export const ruleGroups = sections
  .filter((section) => section.namespaces)
  .map((section) => section.page.replace(/^\/rules\//, ""));
export type RuleGroup = string;
/** A category page, "all" or "cross-file". */
export type RulePage = string;

/** The category whose namespace owns a rule ID; its page and sidebar section list it. */
export function owningRuleGroup(id: string, project: boolean): RuleGroup {
  if (project) return "cross-file";
  // ESLint core `max-lines` has no slash. It counts a whole SFC, so it is listed with the Vue rules.
  if (id === "max-lines") return "vue";
  const namespace = id.split("/")[0];
  const section = sections!.find((candidate) => candidate.namespaces?.includes(namespace));
  if (!section) throw new Error(`No rule category owns the ${namespace}/ namespace: ${id}`);
  return section.page.replace(/^\/rules\//, "");
}

/** Rules a category page also lists although another category owns them. */
const crossListed: Record<string, readonly string[]> = {
  accessibility: ["vue/use-unique-element-ids"],
  vapor: ["script/no-options-api", "script/no-get-current-instance", "script/no-next-tick"],
};
export const groupIncludes = (group: RuleGroup, id: string) =>
  owningRuleGroup(id, false) === group || (crossListed[group]?.includes(id) ?? false);

const titles: Record<RuleLocale, Record<RulePage, string>> = {
  en: {
    all: "All lint rules",
    vue: "Vue rules",
    "type-and-script": "Type and script rules",
    html: "HTML rules",
    accessibility: "Accessibility rules",
    ssr: "SSR rules",
    vapor: "Vapor rules",
    ecosystem: "Ecosystem and Nuxt rules",
    css: "CSS rules",
    musea: "Musea rules",
    "petite-vue": "petite-vue rules",
    "cross-file": "Cross-file rules",
  },
  ja: {
    all: "全 lint ルール",
    vue: "Vue ルール",
    "type-and-script": "型と script のルール",
    html: "HTML ルール",
    accessibility: "アクセシビリティのルール",
    ssr: "SSR ルール",
    vapor: "Vapor ルール",
    ecosystem: "エコシステムと Nuxt のルール",
    css: "CSS ルール",
    musea: "Musea ルール",
    "petite-vue": "petite-vue ルール",
    "cross-file": "ファイル間ルール",
  },
  "zh-CN": {
    all: "全部 lint 规则",
    vue: "Vue 规则",
    "type-and-script": "类型与脚本规则",
    html: "HTML 规则",
    accessibility: "无障碍规则",
    ssr: "SSR 规则",
    vapor: "Vapor 规则",
    ecosystem: "生态系统与 Nuxt 规则",
    css: "CSS 规则",
    musea: "Musea 规则",
    "petite-vue": "petite-vue 规则",
    "cross-file": "跨文件规则",
  },
  "pt-BR": {
    all: "Todas as regras de lint",
    vue: "Regras Vue",
    "type-and-script": "Regras de tipo e script",
    html: "Regras HTML",
    accessibility: "Regras de acessibilidade",
    ssr: "Regras de SSR",
    vapor: "Regras de Vapor",
    ecosystem: "Regras do ecossistema e Nuxt",
    css: "Regras CSS",
    musea: "Regras Musea",
    "petite-vue": "Regras de petite-vue",
    "cross-file": "Regras entre arquivos",
  },
  fr: {
    all: "Toutes les règles de lint",
    vue: "Règles Vue",
    "type-and-script": "Règles de type et de script",
    html: "Règles HTML",
    accessibility: "Règles d’accessibilité",
    ssr: "Règles SSR",
    vapor: "Règles Vapor",
    ecosystem: "Règles de l’écosystème et de Nuxt",
    css: "Règles CSS",
    musea: "Règles Musea",
    "petite-vue": "Règles petite-vue",
    "cross-file": "Règles inter-fichiers",
  },
};

export function categoryTitle(page: RulePage, locale: RuleLocale) {
  const title = titles[locale][page];
  if (!title) throw new Error(`${locale}: missing rule category title for ${page}`);
  return title;
}

/** One sentence that says what a category checks. */
export const categoryPurpose: Record<RuleLocale, Record<RulePage, string>> = {
  en: {
    all: "Every rule Vize's linter ships, grouped by category. Open a rule to see what it reports, a failing and a passing example, and how to turn it on.",
    vue: "Correctness, conventions and Vue 3 migration checks for SFC templates, directives, components and blocks.",
    "type-and-script":
      "Checks for `<script>` and `<script setup>`: compiler macros, reactivity, props and emits, and rules that use TypeScript type information.",
    html: "HTML validity and semantic markup in templates.",
    accessibility:
      "ARIA, keyboard interaction, labels, landmarks and media alternatives, so templates work with assistive technology.",
    ssr: "Code that breaks or behaves differently during server rendering and hydration.",
    vapor: "Template and script constraints for components compiled in Vapor mode.",
    ecosystem:
      "Checks for Nuxt, Vue Router, Pinia, Vue I18n, Vue Test Utils and Void Vue projects.",
    css: "Plain CSS in `<style>` blocks, including correctness and maintainability checks.",
    musea: "Musea `.art.vue` stories, variants and design-token usage.",
    "petite-vue": "Directives and scopes in HTML pages that use petite-vue.",
    "cross-file":
      "Checks that need more than one file: a parent and its children, a provider and its injectors, or a router and its links.",
  },
  ja: {
    all: "Vize のリンターに含まれるすべてのルールを、カテゴリごとにまとめています。各ルールのページで、検出内容・悪い例と良い例・有効にする設定を確認できます。",
    vue: "SFC のテンプレート・ディレクティブ・コンポーネント・各ブロックについて、正しさ・書き方の統一・Vue 3 への移行を検査します。",
    "type-and-script":
      "`<script>` と `<script setup>` を検査します。コンパイラーマクロ、リアクティビティ、props と emits、TypeScript の型情報を使うルールが含まれます。",
    html: "テンプレートの HTML の妥当性と、意味に沿ったマークアップを検査します。",
    accessibility:
      "ARIA、キーボード操作、ラベル、ランドマーク、メディアの代替手段を検査し、支援技術で使えるテンプレートにします。",
    ssr: "サーバーレンダリングやハイドレーションで壊れる、または動作が変わるコードを検出します。",
    vapor: "Vapor モードでコンパイルするコンポーネントのテンプレートと script の制約です。",
    ecosystem:
      "Nuxt、Vue Router、Pinia、Vue I18n、Vue Test Utils、Void Vue を使うプロジェクト向けの検査です。",
    css: "`<style>` ブロックのプレーン CSS について、正しさと保守性を検査します。",
    musea: "Musea の `.art.vue` ストーリー、variant、デザイントークンの使い方を検査します。",
    "petite-vue": "petite-vue を使う HTML ページのディレクティブとスコープを検査します。",
    "cross-file":
      "複数のファイルを見る必要がある検査です。親子コンポーネント、provide と inject、ルーターとリンクの関係を確認します。",
  },
  "zh-CN": {
    all: "Vize linter 提供的全部规则，按类别分组。打开规则页面即可查看它报告的问题、错误与正确示例以及启用方式。",
    vue: "针对 SFC 模板、指令、组件和代码块的正确性、写法约定以及 Vue 3 迁移检查。",
    "type-and-script":
      "检查 `<script>` 与 `<script setup>`：编译器宏、响应式、props 与 emits，以及使用 TypeScript 类型信息的规则。",
    html: "检查模板中 HTML 的有效性和语义化标记。",
    accessibility: "检查 ARIA、键盘交互、标签、地标和媒体替代内容，让模板可以配合辅助技术使用。",
    ssr: "找出在服务端渲染和水合时会出错或行为不同的代码。",
    vapor: "以 Vapor 模式编译的组件在模板和脚本上的限制。",
    ecosystem: "面向 Nuxt、Vue Router、Pinia、Vue I18n、Vue Test Utils 和 Void Vue 项目的检查。",
    css: "检查 `<style>` 块中普通 CSS 的正确性和可维护性。",
    musea: "检查 Musea 的 `.art.vue` 故事、变体和设计令牌用法。",
    "petite-vue": "检查使用 petite-vue 的 HTML 页面中的指令和作用域。",
    "cross-file": "需要查看多个文件的检查：父组件与子组件、provide 与 inject、路由与链接。",
  },
  "pt-BR": {
    all: "Todas as regras do linter do Vize, agrupadas por categoria. Abra uma regra para ver o que ela reporta, um exemplo incorreto e um correto, e como ativá-la.",
    vue: "Verificações de correção, convenções e migração para o Vue 3 em templates, diretivas, componentes e blocos de SFC.",
    "type-and-script":
      "Verificações de `<script>` e `<script setup>`: macros do compilador, reatividade, props e emits, e regras que usam informações de tipo do TypeScript.",
    html: "Validade do HTML e marcação semântica nos templates.",
    accessibility:
      "ARIA, interação por teclado, rótulos, landmarks e alternativas de mídia, para que os templates funcionem com tecnologias assistivas.",
    ssr: "Código que quebra ou se comporta de outro jeito durante a renderização no servidor e a hidratação.",
    vapor: "Restrições de template e script para componentes compilados no modo Vapor.",
    ecosystem:
      "Verificações para projetos com Nuxt, Vue Router, Pinia, Vue I18n, Vue Test Utils e Void Vue.",
    css: "CSS puro em blocos `<style>`, incluindo verificações de correção e manutenção.",
    musea: "Histórias `.art.vue`, variantes e uso de tokens de design do Musea.",
    "petite-vue": "Diretivas e escopos em páginas HTML que usam petite-vue.",
    "cross-file":
      "Verificações que precisam de mais de um arquivo: um pai e seus filhos, um provider e seus injetores, ou um roteador e seus links.",
  },
  fr: {
    all: "Toutes les règles du linter de Vize, regroupées par catégorie. Ouvrez une règle pour voir ce qu’elle signale, un mauvais et un bon exemple, et comment l’activer.",
    vue: "Vérifications de justesse, de conventions et de migration vers Vue 3 pour les templates, directives, composants et blocs des SFC.",
    "type-and-script":
      "Vérifications de `<script>` et `<script setup>` : macros du compilateur, réactivité, props et emits, et règles qui utilisent les informations de type de TypeScript.",
    html: "Validité du HTML et balisage sémantique dans les templates.",
    accessibility:
      "ARIA, interactions au clavier, libellés, repères et alternatives aux médias, pour que les templates fonctionnent avec les technologies d’assistance.",
    ssr: "Le code qui casse ou se comporte autrement pendant le rendu serveur et l’hydratation.",
    vapor: "Contraintes de template et de script pour les composants compilés en mode Vapor.",
    ecosystem:
      "Vérifications pour les projets Nuxt, Vue Router, Pinia, Vue I18n, Vue Test Utils et Void Vue.",
    css: "Le CSS standard des blocs `<style>`, avec des vérifications de justesse et de maintenabilité.",
    musea: "Les stories `.art.vue`, les variantes et l’usage des jetons de design de Musea.",
    "petite-vue": "Directives et portées dans les pages HTML qui utilisent petite-vue.",
    "cross-file":
      "Vérifications qui ont besoin de plusieurs fichiers : un parent et ses enfants, un provider et ses injecteurs, ou un routeur et ses liens.",
  },
};
