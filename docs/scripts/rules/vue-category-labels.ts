export const vueLocales = ["en", "ja", "zh-CN", "pt-BR", "fr"] as const;
export type VueLocale = (typeof vueLocales)[number];

/** Existing translated category fragments remain valid after replacing the old guide. */
export const vueCategoryLegacyAnchors: Partial<Record<VueLocale, readonly string[]>> = {
  "zh-CN": ["句法与风格规则"],
  "pt-BR": ["regras-do-vue", "regras-de-sintaxe-e-estilo"],
  fr: ["syntaxe-et-règles-de-style"],
};

export const vueCategoryLabels: Record<
  VueLocale,
  {
    title: string;
    intro: string;
    rule: string;
    examples: string;
    purpose: string;
    bad: string;
    good: string;
    related: string;
  }
> = {
  en: {
    title: "Vue rules",
    intro:
      "Every Vue rule has its purpose, configuration, Bad and Good examples on this page. Highlighted lines show the change; copied code keeps the complete source.",
    rule: "Rule",
    examples: "Examples",
    purpose: "Purpose",
    bad: "Bad",
    good: "Good",
    related:
      "[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md) · [Attributes across components](./project/vue-cross-file-attrs-fallthrough.md)",
  },
  ja: {
    title: "Vue ルール",
    intro:
      "すべての Vue ルールの目的・設定・悪い例・良い例をこのページにまとめています。色付きの行で変更を示し、コピーしたコードには完全なソースを保持します。",
    rule: "ルール",
    examples: "例",
    purpose: "目的",
    bad: "悪い",
    good: "良い",
    related:
      "[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)",
  },
  "zh-CN": {
    title: "Vue 规则",
    intro:
      "本页列出每条 Vue 规则的用途、配置、错误示例和正确示例。高亮行表示修改；复制代码时会保留完整源代码。",
    rule: "规则",
    examples: "示例",
    purpose: "用途",
    bad: "错误示例",
    good: "正确示例",
    related:
      "[全部规则](./all.md) · [规则选项](./options.md) · [ESLint 迁移对应表](./migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](./project/vue-cross-file-attrs-fallthrough.md)",
  },
  "pt-BR": {
    title: "Regras Vue",
    intro:
      "Cada regra Vue reúne nesta página sua finalidade, configuração e exemplos incorreto e correto. As linhas destacadas mostram a alteração; o código copiado preserva o conteúdo completo.",
    rule: "Regra",
    examples: "Exemplos",
    purpose: "Finalidade",
    bad: "Incorreto",
    good: "Correto",
    related:
      "[Todas as regras](./all.md) · [Opções das regras](./options.md) · [Mapa de migração do ESLint](./migration.md) · [Verificações do projeto](./cross-file.md) · [Atributos entre componentes](./project/vue-cross-file-attrs-fallthrough.md)",
  },
  fr: {
    title: "Règles Vue",
    intro:
      "Chaque règle Vue présente son objectif, sa configuration et ses mauvais et bons exemples sur cette page. Les lignes surlignées indiquent les modifications ; le code copié conserve la source complète.",
    rule: "Règle",
    examples: "Exemples",
    purpose: "Objectif",
    bad: "Mauvais",
    good: "Bon",
    related:
      "[Toutes les règles](./all.md) · [Options des règles](./options.md) · [Correspondance de migration ESLint](./migration.md) · [Vérifications du projet](./cross-file.md) · [Attributs entre composants](./project/vue-cross-file-attrs-fallthrough.md)",
  },
};

/** Translate the shared prose only; identifiers and executable source remain exact. */
const common = {
  "Default severity": ["默认严重程度", "Severidade padrão", "Gravité par défaut"],
  Presets: ["预设", "Predefinições", "Préréglages"],
  "Automatic fix": ["自动修复", "Correção automática", "Correction automatique"],
  "Applies to": ["适用范围", "Aplicável a", "Champ d’application"],
  Options: ["选项", "Opções", "Options"],
  "Current support": ["当前支持情况", "Suporte atual", "Prise en charge actuelle"],
  "Bad diagnostic": [
    "错误示例的诊断",
    "Diagnóstico do exemplo incorreto",
    "Diagnostic du mauvais exemple",
  ],
  "None; review the suggested change": [
    "无；请检查建议的修改",
    "Nenhuma; revise a alteração sugerida",
    "Aucune ; examinez la modification proposée",
  ],
  "Available for supported findings": [
    "适用于已支持的诊断",
    "Disponível para os diagnósticos compatíveis",
    "Disponible pour les diagnostics pris en charge",
  ],
  "Not implemented for SFC lint": [
    "尚未在 SFC lint 中实现",
    "Não implementada no lint de SFC",
    "Non implémentée pour le lint des SFC",
  ],
  "Vue SFC templates and blocks, with script context where the rule requires it": [
    "Vue SFC 模板和代码块，包括规则所需的脚本上下文",
    "Templates e blocos de SFC Vue, com o contexto do script quando exigido pela regra",
    "Templates et blocs des SFC Vue, avec le contexte du script requis par la règle",
  ],
  "No rule-specific options. Severity and preset selection are configurable.": [
    "没有规则专属选项。可以配置严重程度和预设。",
    "Sem opções específicas da regra. A severidade e a seleção de predefinições são configuráveis.",
    "Aucune option propre à la règle. La gravité et le choix des préréglages sont configurables.",
  ],
  "See [typed options and defaults](../options.md).": [
    "参见[类型化选项和默认值](../options.md)。",
    "Consulte as [opções tipadas e os valores padrão](../options.md).",
    "Consultez les [options typées et leurs valeurs par défaut](../options.md).",
  ],
  "Configuration (Vite+)": ["配置（Vite+）", "Configuração (Vite+)", "Configuration (Vite+)"],
  "Configured ID (currently no SFC finding)": [
    "可配置的 ID（当前没有 SFC 诊断）",
    "ID configurado (sem diagnóstico de SFC atualmente)",
    "ID configuré (aucun diagnostic SFC actuellement)",
  ],
  "Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.":
    [
      "正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。",
      "O exemplo correto evita o diagnóstico desta regra com a configuração acima; outras regras ainda podem emitir diagnósticos.",
      "Le bon exemple évite le diagnostic de cette règle avec la configuration ci-dessus ; d’autres règles peuvent encore signaler des diagnostics.",
    ],
  "Good illustrates the intended convention; the current SFC path emits neither side's rule-specific finding.":
    [
      "正确示例展示预期规范；当前 SFC 流程不会为任一示例生成这条规则的诊断。",
      "O exemplo correto ilustra a convenção pretendida; o fluxo atual de SFC não emite o diagnóstico específico da regra para nenhum dos exemplos.",
      "Le bon exemple illustre la convention visée ; le traitement actuel des SFC n’émet le diagnostic propre à cette règle pour aucun des deux exemples.",
    ],
  Implementation: ["实现", "Implementação", "Implémentation"],
  "All rules": ["全部规则", "Todas as regras", "Toutes les règles"],
} satisfies Record<string, readonly [string, string, string]>;

export function commonVueText(locale: Exclude<VueLocale, "en" | "ja">) {
  const index = { "zh-CN": 0, "pt-BR": 1, fr: 2 }[locale];
  return new Map(
    Object.entries(common).map(([source, translations]) => [source, translations[index]]),
  );
}
