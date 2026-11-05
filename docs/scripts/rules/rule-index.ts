import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import {
  categoryPurpose,
  categoryTitle,
  groupIncludes,
  owningRuleGroup,
  ruleGroups,
  ruleLocales,
  type RuleGroup,
  type RuleLocale,
} from "./rule-groups.ts";
import { presetList, ruleSlug } from "./rule-page.ts";

export interface IndexedRule {
  id: string;
  project: boolean;
  fixable: boolean;
  presets: readonly string[];
  /** Project rules only. */
  status?: "cli" | "library" | "contract";
  purpose: Record<RuleLocale, string>;
}

type Tier = "essential" | "default" | "stricter" | "optIn";
const tiers: readonly Tier[] = ["essential", "default", "stricter", "optIn"];
const tierOf = (presets: readonly string[]): Tier =>
  presets.includes("essential")
    ? "essential"
    : presets.includes("happy-path") || presets.includes("general-recommended")
      ? "default"
      : presets.length
        ? "stricter"
        : "optIn";

const text: Record<
  RuleLocale,
  {
    rule: string;
    checks: string;
    preset: string;
    presets: string;
    fix: string;
    status: string;
    open: string;
    tiers: Record<Tier, readonly [heading: string, intro: string]>;
    // These pages render as `<name>/index.html`, where `./index.md` is a self-link.
    allLegend: string;
    statuses: Record<"cli" | "library" | "contract", string>;
    crossFileUse: string;
    related: string;
  }
> = {
  en: {
    rule: "Rule",
    checks: "What it checks",
    preset: "Preset",
    presets: "Presets",
    fix: "Auto-fix",
    status: "Status",
    open: "Each rule page shows a failing example, a passing example, and the configuration that turns the rule on.",
    tiers: {
      essential: ["Essential", "Correctness rules in the `essential` preset."],
      default: ["Default", "Added by `happy-path`, the default preset."],
      stricter: [
        "Stricter presets",
        "Only in `opinionated`, `nuxt` or `ecosystem`. The Presets column lists which.",
      ],
      optIn: ["Opt-in", "Not in any preset. Turn them on by name in `vue.linter.rules`."],
    },
    allLegend:
      "The Preset column names the smallest preset that turns a rule on. Rule pages list every preset. See [presets](../rules/index.md#presets) for what each one contains.",
    statuses: {
      cli: "Reported by `vp run lint`",
      library: "Experimental; not reported by the CLI yet",
      contract: "Planned; not reported yet",
    },
    crossFileUse:
      "Turn these checks on with `crossFile: true` in `vue.linter`, or run `vize lint --cross-file`. Diagnostics are shown as `vize:croquis/cf/<name>`; configure them in `vue.linter.rules` as `croquis/cf/<name>`, without `vize:`.",
    related:
      "[Rules overview](../rules/index.md) · [Rule options](./options.md) · [Migrating from ESLint](./migration.md)",
  },
  ja: {
    rule: "ルール",
    checks: "検出する内容",
    preset: "プリセット",
    presets: "プリセット",
    fix: "自動修正",
    status: "対応状況",
    open: "各ルールのページに、診断が出る例・出ない例と、そのルールを有効にする設定があります。",
    tiers: {
      essential: ["基本", "`essential` プリセットに含まれる、正しさを守るルールです。"],
      default: ["既定", "既定のプリセット `happy-path` で追加されるルールです。"],
      stricter: [
        "より厳しいプリセット",
        "`opinionated`・`nuxt`・`ecosystem` のいずれかにだけ含まれます。どれに含まれるかはプリセット列を参照してください。",
      ],
      optIn: [
        "個別に有効化",
        "どのプリセットにも含まれません。`vue.linter.rules` でルール名を指定して有効にします。",
      ],
    },
    allLegend:
      "プリセット列は、そのルールを有効にする最小のプリセットです。すべてのプリセットは各ルールのページに載っています。各プリセットの内容は[プリセット](../rules/index.md#プリセット)を参照してください。",
    statuses: {
      cli: "`vp run lint` で診断",
      library: "実験的。CLI ではまだ診断されません",
      contract: "予定。まだ診断されません",
    },
    crossFileUse:
      "`vue.linter` に `crossFile: true` を指定するか、`vize lint --cross-file` を実行すると有効になります。診断は `vize:croquis/cf/<name>` と表示されます。`vue.linter.rules` では `vize:` を除いた `croquis/cf/<name>` で設定します。",
    related:
      "[ルールの概要](../rules/index.md) · [ルールのオプション](./options.md) · [ESLint からの移行](./migration.md)",
  },
  "zh-CN": {
    rule: "规则",
    checks: "检查内容",
    preset: "预设",
    presets: "预设",
    fix: "自动修复",
    status: "状态",
    open: "每个规则页面都有会触发诊断的示例、不会触发的示例，以及启用该规则的配置。",
    tiers: {
      essential: ["基础", "`essential` 预设中的正确性规则。"],
      default: ["默认", "由默认预设 `happy-path` 添加。"],
      stricter: [
        "更严格的预设",
        "只在 `opinionated`、`nuxt` 或 `ecosystem` 中。具体是哪些见“预设”列。",
      ],
      optIn: ["按需启用", "不属于任何预设。在 `vue.linter.rules` 中按名称启用。"],
    },
    allLegend:
      "“预设”列给出启用该规则的最小预设，规则页面列出全部预设。各预设包含的内容见[预设](../rules/index.md#预设)。",
    statuses: {
      cli: "由 `vp run lint` 报告",
      library: "实验性；CLI 尚未报告",
      contract: "计划中；尚未报告",
    },
    crossFileUse:
      "在 `vue.linter` 中设置 `crossFile: true`，或运行 `vize lint --cross-file` 来启用这些检查。诊断显示为 `vize:croquis/cf/<name>`；在 `vue.linter.rules` 中去掉 `vize:`，写作 `croquis/cf/<name>`。",
    related:
      "[规则概览](../rules/index.md) · [规则选项](./options.md) · [从 ESLint 迁移](./migration.md)",
  },
  "pt-BR": {
    rule: "Regra",
    checks: "O que verifica",
    preset: "Predefinição",
    presets: "Predefinições",
    fix: "Correção automática",
    status: "Status",
    open: "Cada página de regra mostra um exemplo que falha, um que passa e a configuração que ativa a regra.",
    tiers: {
      essential: ["Essenciais", "Regras de correção da predefinição `essential`."],
      default: ["Padrão", "Adicionadas por `happy-path`, a predefinição padrão."],
      stricter: [
        "Predefinições mais rígidas",
        "Só em `opinionated`, `nuxt` ou `ecosystem`. A coluna Predefinições mostra quais.",
      ],
      optIn: [
        "Opcionais",
        "Não fazem parte de nenhuma predefinição. Ative pelo nome em `vue.linter.rules`.",
      ],
    },
    allLegend:
      "A coluna Predefinição indica a menor predefinição que ativa a regra; a página de cada regra lista todas. Veja em [predefinições](../rules/index.md#predefinições) o que cada uma contém.",
    statuses: {
      cli: "Reportada por `vp run lint`",
      library: "Experimental; a CLI ainda não reporta",
      contract: "Planejada; ainda não é reportada",
    },
    crossFileUse:
      "Ative estas verificações com `crossFile: true` em `vue.linter` ou execute `vize lint --cross-file`. Os diagnósticos aparecem como `vize:croquis/cf/<name>`; configure-os em `vue.linter.rules` como `croquis/cf/<name>`, sem `vize:`.",
    related:
      "[Visão geral das regras](../rules/index.md) · [Opções das regras](./options.md) · [Migração do ESLint](./migration.md)",
  },
  fr: {
    rule: "Règle",
    checks: "Ce qu’elle vérifie",
    preset: "Préréglage",
    presets: "Préréglages",
    fix: "Correction automatique",
    status: "État",
    open: "Chaque page de règle montre un exemple qui échoue, un exemple qui passe et la configuration qui active la règle.",
    tiers: {
      essential: ["Essentielles", "Règles de justesse du préréglage `essential`."],
      default: ["Par défaut", "Ajoutées par `happy-path`, le préréglage par défaut."],
      stricter: [
        "Préréglages plus stricts",
        "Seulement dans `opinionated`, `nuxt` ou `ecosystem`. La colonne Préréglages indique lesquels.",
      ],
      optIn: [
        "À activer",
        "Dans aucun préréglage. Activez-les par leur nom dans `vue.linter.rules`.",
      ],
    },
    allLegend:
      "La colonne Préréglage indique le plus petit préréglage qui active la règle ; la page de chaque règle les liste tous. Voir [préréglages](../rules/index.md#préréglages) pour le contenu de chacun.",
    statuses: {
      cli: "Signalée par `vp run lint`",
      library: "Expérimentale ; pas encore signalée par le CLI",
      contract: "Prévue ; pas encore signalée",
    },
    crossFileUse:
      "Activez ces vérifications avec `crossFile: true` dans `vue.linter`, ou lancez `vize lint --cross-file`. Les diagnostics s’affichent sous la forme `vize:croquis/cf/<name>` ; configurez-les dans `vue.linter.rules` sous la forme `croquis/cf/<name>`, sans `vize:`.",
    related:
      "[Vue d’ensemble des règles](../rules/index.md) · [Options des règles](./options.md) · [Migrer depuis ESLint](./migration.md)",
  },
};

const cell = (value: string) =>
  value
    .replaceAll("|", "\\|")
    .replace(/(`+)([\s\S]*?)\1|[<>]/g, (token) =>
      token === "<" ? "&lt;" : token === ">" ? "&gt;" : token,
    )
    .replace(/\s+/g, " ")
    .trim();
const link = (rule: IndexedRule, from: "." | "..") =>
  `[\`${rule.id}\`](${from === "." ? "./" : "../"}${rule.project ? "project" : "reference"}/${ruleSlug(rule.id)}.md)`;
// Rows keep the former catalogue fragments: old `page#rule-id` links land on the rule's row.
const anchor = (rule: IndexedRule) => `<span id="${ruleSlug(rule.id)}"></span>`;
const byId = (left: IndexedRule, right: IndexedRule) => left.id.localeCompare(right.id);

function smallestPreset(presets: readonly string[], locale: RuleLocale) {
  const tier = tierOf(presets);
  if (tier === "essential") return "`essential`";
  if (tier === "default") return "`happy-path`";
  return tier === "stricter" ? presetList(presets, locale) : "—";
}

function frontmatter(title: string, description: string) {
  return [
    "---",
    `title: ${JSON.stringify(title)}`,
    `description: ${JSON.stringify(description)}`,
    "---",
    "",
  ];
}

function categoryPage(group: RuleGroup, rules: readonly IndexedRule[], locale: RuleLocale) {
  const t = text[locale];
  const title = categoryTitle(group, locale);
  const selected = rules
    .filter((rule) => !rule.project && groupIncludes(group, rule.id))
    .sort(byId);
  const lines = [
    ...frontmatter(title, categoryPurpose[locale][group]),
    `# ${title}`,
    "",
    categoryPurpose[locale][group],
    "",
    t.open,
    "",
  ];
  for (const tier of tiers) {
    const members = selected.filter((rule) => tierOf(rule.presets) === tier);
    if (!members.length) continue;
    const [heading, intro] = t.tiers[tier];
    const presets = tier === "stricter";
    lines.push(
      `## ${heading} (${members.length})`,
      "",
      intro,
      "",
      `| ${t.rule} | ${t.checks}${presets ? ` | ${t.presets}` : ""} | ${t.fix} |`,
      `| --- | ---${presets ? " | ---" : ""} | --- |`,
      ...members.map(
        (rule) =>
          `| ${anchor(rule)}${link(rule, ".")} | ${cell(rule.purpose[locale])}${presets ? ` | ${presetList(rule.presets, locale)}` : ""} | ${rule.fixable ? "✓" : ""} |`,
      ),
      "",
    );
  }
  lines.push(t.related, "");
  return lines.join("\n");
}

function crossFilePage(rules: readonly IndexedRule[], locale: RuleLocale) {
  const t = text[locale];
  const title = categoryTitle("cross-file", locale);
  const selected = rules.filter((rule) => rule.project).sort(byId);
  return [
    ...frontmatter(title, categoryPurpose[locale]["cross-file"]),
    `# ${title}`,
    "",
    categoryPurpose[locale]["cross-file"],
    "",
    t.crossFileUse,
    "",
    t.open,
    "",
    `| ${t.rule} | ${t.checks} | ${t.status} |`,
    "| --- | --- | --- |",
    ...selected.map(
      (rule) =>
        `| ${anchor(rule)}${link(rule, ".")} | ${cell(rule.purpose[locale])} | ${t.statuses[rule.status ?? "cli"]} |`,
    ),
    "",
    t.related,
    "",
  ].join("\n");
}

function allRulesPage(rules: readonly IndexedRule[], locale: RuleLocale) {
  const t = text[locale];
  const title = categoryTitle("all", locale);
  const lines = [
    ...frontmatter(title, categoryPurpose[locale].all),
    `# ${title}`,
    "",
    categoryPurpose[locale].all,
    "",
    t.allLegend,
    "",
  ];
  for (const group of ruleGroups) {
    const members = rules
      .filter((rule) => !rule.project && owningRuleGroup(rule.id, false) === group)
      .sort(byId);
    lines.push(
      `## ${categoryTitle(group, locale)} (${members.length})`,
      "",
      `| ${t.rule} | ${t.checks} | ${t.preset} | ${t.fix} |`,
      "| --- | --- | --- | --- |",
      ...members.map(
        (rule) =>
          `| ${anchor(rule)}${link(rule, ".")} | ${cell(rule.purpose[locale])} | ${smallestPreset(rule.presets, locale)} | ${rule.fixable ? "✓" : ""} |`,
      ),
      "",
    );
  }
  const project = rules.filter((rule) => rule.project).sort(byId);
  lines.push(
    `## ${categoryTitle("cross-file", locale)} (${project.length})`,
    "",
    t.crossFileUse,
    "",
    `| ${t.rule} | ${t.checks} | ${t.status} |`,
    "| --- | --- | --- |",
    ...project.map(
      (rule) =>
        `| ${anchor(rule)}${link(rule, ".")} | ${cell(rule.purpose[locale])} | ${t.statuses[rule.status ?? "cli"]} |`,
    ),
    "",
    t.related,
    "",
  );
  return lines.join("\n");
}

export function writeOutput(path: string, content: string, checking: boolean) {
  if (checking) {
    if (readFileSync(path, "utf8") !== content)
      throw new Error(`Generated rule page is stale: ${path}`);
    return;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content);
}

export function generateRuleIndexes(
  root: string,
  rules: readonly IndexedRule[],
  checking: boolean,
  page: (locale: RuleLocale, path: string, content: string) => void,
) {
  for (const locale of ruleLocales) {
    for (const group of ruleGroups) page(locale, `${group}.md`, categoryPage(group, rules, locale));
    page(locale, "cross-file.md", crossFilePage(rules, locale));
    page(locale, "all.md", allRulesPage(rules, locale));
  }
}
