import type { TranslatedLocale } from "./rule-translations.ts";

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
  Related: ["相关页面", "Páginas relacionadas", "Voir aussi"],
  "This code is not currently emitted by the CLI, so there is no CLI rule configuration to enable here.":
    [
      "CLI 当前不会输出这个诊断代码，因此这里没有可启用它的 CLI 规则配置。",
      "A CLI não emite este código atualmente, portanto não há uma configuração de regra da CLI para habilitá-lo aqui.",
      "Le CLI n’émet pas ce code actuellement ; aucune configuration de règle du CLI ne permet donc de l’activer ici.",
    ],
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

export function commonVueText(locale: TranslatedLocale) {
  const index = { "zh-CN": 0, "pt-BR": 1, fr: 2 }[locale];
  return new Map(
    Object.entries(common).map(([source, translations]) => [source, translations[index]]),
  );
}
