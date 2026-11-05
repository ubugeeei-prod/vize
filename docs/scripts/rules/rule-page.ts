import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import type { CodeExample, RuleExample, RuleMetadata } from "./types.ts";
import { changedCodeLines } from "./inline-code-diff.ts";
import { exampleExplanations } from "./explanations.ts";
import { purposeJa } from "./purpose-ja.ts";
import { owningRuleGroup, categoryTitle, type RuleLocale } from "./rule-groups.ts";
import { ruleTranslator } from "./rule-translations.ts";

/** Rules with typed options: the rows of the Rule Options table, which its test keeps complete. */
export const configurableRules = new Set(
  Array.from(
    readFileSync(resolve(import.meta.dirname, "../../content/rules/options.md"), "utf8")
      .split("\n| Rule | Option shape |")[1]
      .split("\n\n")[0]
      .matchAll(/^\| `([^`]+)` \|/gm),
    (match) => match[1],
  ),
);
if (configurableRules.size === 0) throw new Error("Rule Options table has no rules");

/** Fixed page wording. Rule prose comes from the explanation tables. */
const pageText: Record<
  RuleLocale,
  {
    severity: string;
    presets: string;
    fix: string;
    options: string;
    appliesTo: string;
    yes: string;
    no: string;
    none: string;
    notForSfc: string;
    optionsLink: string;
    bad: string;
    good: string;
    configuration: string;
    configuredId: string;
    run: string;
    related: string;
    allRules: string;
    source: string;
    typeAware: string;
    unsupported: string;
    complexity: string;
    currentSupport: string;
    badDiagnostic: string;
  }
> = {
  en: {
    severity: "Default severity",
    presets: "Presets",
    fix: "Automatic fix",
    options: "Options",
    appliesTo: "Applies to",
    yes: "Yes",
    no: "No",
    none: "None",
    notForSfc: "Not implemented for SFC lint",
    optionsLink: "[Rule options](../options.md)",
    bad: "Bad",
    good: "Good",
    configuration: "Configuration (Vite+)",
    configuredId: "Configured ID (currently no SFC finding)",
    run: "Then run `vp run lint`.",
    related: "Related",
    allRules: "All rules",
    source: "Source code",
    typeAware:
      "This rule reads type information from your TypeScript project (`tsconfig.json`), so the configuration sets `typeAware: true`. `typeAware` does not turn on opt-in rules by itself.",
    unsupported:
      "Vize accepts this rule ID in configuration, but the SFC linter does not report it yet. Good shows the intended convention.",
    complexity:
      "See [complexity scoring and component boundaries](../../guide/cross-file-complexity.md) for how the example's two scores are computed.",
    currentSupport: "Current support",
    badDiagnostic: "Bad diagnostic",
  },
  ja: {
    severity: "既定の重大度",
    presets: "プリセット",
    fix: "自動修正",
    options: "オプション",
    appliesTo: "適用範囲",
    yes: "あり",
    no: "なし",
    none: "なし",
    notForSfc: "SFC lint では未対応",
    optionsLink: "[ルールのオプション](../options.md)",
    bad: "悪い例",
    good: "良い例",
    configuration: "設定（Vite+）",
    configuredId: "設定できる ID（現在の SFC lint では診断されません）",
    run: "設定したら `vp run lint` を実行します。",
    related: "関連ページ",
    allRules: "全ルール",
    source: "ソースコード",
    typeAware:
      "このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。",
    unsupported:
      "このルール ID は設定できますが、現在の SFC lint はまだ診断を出しません。良い例は意図する書き方を示します。",
    complexity:
      "例の二つの値の計算内訳は[複雑度の計算とコンポーネントの境界](../../guide/cross-file-complexity.md)を参照してください。",
    currentSupport: "現在の対応",
    badDiagnostic: "悪い例での診断",
  },
  "zh-CN": {
    severity: "默认严重程度",
    presets: "预设",
    fix: "自动修复",
    options: "选项",
    appliesTo: "适用范围",
    yes: "支持",
    no: "不支持",
    none: "无",
    notForSfc: "尚未在 SFC lint 中实现",
    optionsLink: "[规则选项](../options.md)",
    bad: "错误示例",
    good: "正确示例",
    configuration: "配置（Vite+）",
    configuredId: "可配置的 ID（当前没有 SFC 诊断）",
    run: "然后运行 `vp run lint`。",
    related: "相关页面",
    allRules: "全部规则",
    source: "源代码",
    typeAware:
      "这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。",
    unsupported:
      "Vize 接受在配置中使用这个规则 ID，但 SFC lint 目前还不会报告它。正确示例展示的是预期的写法。",
    complexity:
      "示例中两个分数的计算方式见[复杂度计分与组件边界](../../guide/cross-file-complexity.md)。",
    currentSupport: "当前支持情况",
    badDiagnostic: "错误示例的诊断",
  },
  "pt-BR": {
    severity: "Severidade padrão",
    presets: "Predefinições",
    fix: "Correção automática",
    options: "Opções",
    appliesTo: "Aplicável a",
    yes: "Sim",
    no: "Não",
    none: "Nenhuma",
    notForSfc: "Não implementada no lint de SFC",
    optionsLink: "[Opções das regras](../options.md)",
    bad: "Incorreto",
    good: "Correto",
    configuration: "Configuração (Vite+)",
    configuredId: "ID configurado (sem diagnóstico de SFC atualmente)",
    run: "Depois, execute `vp run lint`.",
    related: "Páginas relacionadas",
    allRules: "Todas as regras",
    source: "Código-fonte",
    typeAware:
      "Esta regra lê informações de tipo do seu projeto TypeScript (`tsconfig.json`), por isso a configuração define `typeAware: true`. `typeAware` sozinho não ativa regras opcionais.",
    unsupported:
      "O Vize aceita este ID de regra na configuração, mas o lint de SFC ainda não o reporta. O exemplo correto mostra a convenção pretendida.",
    complexity:
      "Veja [pontuação de complexidade e limites de componentes](../../guide/cross-file-complexity.md) para entender como as duas pontuações do exemplo são calculadas.",
    currentSupport: "Suporte atual",
    badDiagnostic: "Diagnóstico do exemplo incorreto",
  },
  fr: {
    severity: "Gravité par défaut",
    presets: "Préréglages",
    fix: "Correction automatique",
    options: "Options",
    appliesTo: "Champ d’application",
    yes: "Oui",
    no: "Non",
    none: "Aucune",
    notForSfc: "Non implémentée pour le lint des SFC",
    optionsLink: "[Options des règles](../options.md)",
    bad: "Mauvais",
    good: "Bon",
    configuration: "Configuration (Vite+)",
    configuredId: "ID configuré (aucun diagnostic SFC actuellement)",
    run: "Lancez ensuite `vp run lint`.",
    related: "Voir aussi",
    allRules: "Toutes les règles",
    source: "Code source",
    typeAware:
      "Cette règle lit les informations de type de votre projet TypeScript (`tsconfig.json`) ; la configuration définit donc `typeAware: true`. `typeAware` seul n’active pas les règles à activer explicitement.",
    unsupported:
      "Vize accepte cet identifiant de règle dans la configuration, mais le lint des SFC ne le signale pas encore. Le bon exemple montre la convention visée.",
    complexity:
      "Consultez [le calcul de la complexité et les limites des composants](../../guide/cross-file-complexity.md) pour savoir comment les deux scores de l’exemple sont calculés.",
    currentSupport: "Prise en charge actuelle",
    badDiagnostic: "Diagnostic du mauvais exemple",
  },
};

export const ruleHeadings = Object.fromEntries(
  Object.entries(pageText).map(([locale, text]) => [locale, { bad: text.bad, good: text.good }]),
) as Record<RuleLocale, { bad: string; good: string }>;

export function presetList(values: readonly string[], locale: RuleLocale) {
  return values.length
    ? values
        .map((value) => `\`${value === "general-recommended" ? "happy-path" : value}\``)
        .join(", ")
    : pageText[locale].none;
}

/** English scope; other locales translate this exact text. */
export function scopeLabel(rule: RuleMetadata, example: RuleExample, ja: boolean) {
  if (example.standaloneScript)
    return ja ? "Nuxt 設定ファイル（nuxt.config.ts）" : "Nuxt configuration files (nuxt.config.ts)";
  if (rule.name.startsWith("petite-vue/"))
    return ja
      ? "petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。"
      : "HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.";
  if (rule.metaType === "CssRuleMeta")
    return ja
      ? "プレーン CSS の style ブロック（scss・sass・less・stylus のブロックは対象外）"
      : "Plain CSS style blocks (scss, sass, less and stylus blocks are skipped)";
  if (rule.metaType === "MuseaRuleMeta")
    return ja
      ? "Musea の .art.vue ファイルの art / variant / style"
      : "Musea .art.vue art, variant, and style blocks";
  if (rule.name.startsWith("type/"))
    return ja
      ? "Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。"
      : "Type information in Vue SFC scripts and templates, for the constructs shown below";
  if (
    ["script/no-options-api", "script/no-get-current-instance", "script/no-next-tick"].includes(
      rule.name,
    )
  )
    return ja
      ? "Vapor を想定した script 検査。明示的に有効にすると通常の script でも同じ禁止を適用します。"
      : "Vapor-oriented script checks; explicit enablement also applies the restriction to ordinary scripts";
  if (rule.metaType === "ScriptRuleMeta")
    return ja
      ? "Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。"
      : "JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form";
  return ja
    ? "Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。"
    : "Vue SFC templates and blocks, with script context where the rule requires it";
}

const cjk = (locale: RuleLocale) => locale === "ja" || locale === "zh-CN";
const sentence = (text: string, locale: RuleLocale) =>
  /[.。!?！？:：]$/.test(text) ? text : `${text}${cjk(locale) ? "。" : "."}`;
const colon = (locale: RuleLocale) => (cjk(locale) ? "：" : locale === "fr" ? " : " : ": ");
const escapeTags = (value: string) =>
  String(value).replace(/(`+)([\s\S]*?)\1|[<>]/g, (token) =>
    token === "<" ? "&lt;" : token === ">" ? "&gt;" : token,
  );
const cell = (value: string) => escapeTags(value).replaceAll("|", "\\|");

export const ruleSlug = (id: string) =>
  id
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");

/** Executed examples exactly as the linter test reads them. */
export function pageExamples(example: RuleExample): Record<"bad" | "good", CodeExample> {
  const strip = ({ language, source }: CodeExample): CodeExample =>
    example.standaloneScript
      ? {
          language: "ts",
          source: source.replace(/^<script[^>]*>\n/, "").replace(/\n<\/script>$/, ""),
        }
      : { language, source };
  return { bad: strip(example.bad), good: strip(example.good) };
}

export function ruleConfiguration(rule: RuleMetadata, example: RuleExample) {
  return {
    preset: "incremental",
    rules: { [rule.name]: rule.defaultSeverity === "warning" ? "warn" : "error" },
    ...(example.typeAware || rule.name.startsWith("type/") ? { typeAware: true } : {}),
    ...(example.ruleOptions ? { ruleOptions: { [rule.name]: example.ruleOptions } } : {}),
  };
}

interface RuleCopy {
  purpose: string;
  bad: string;
  good: string;
  scope: string;
  note?: string;
}

function ruleCopy(rule: RuleMetadata, example: RuleExample, locale: RuleLocale): RuleCopy {
  const explanation = exampleExplanations.get(rule.name);
  if (!explanation) throw new Error(`Missing example explanation: ${rule.name}`);
  if (locale === "en" || locale === "ja") {
    const ja = locale === "ja";
    const purpose = ja ? purposeJa[rule.name] : rule.description;
    if (!purpose) throw new Error(`Missing ${locale} purpose for ${rule.name}`);
    return {
      purpose,
      bad: explanation.bad[locale],
      good: explanation.good[locale],
      scope: scopeLabel(rule, example, ja),
      note: ja ? example.noteJa : example.note,
    };
  }
  const t = ruleTranslator(rule.name, locale, false);
  return {
    purpose: t.purpose,
    bad: t.bad,
    good: t.good,
    scope: t.line(scopeLabel(rule, example, false)),
    note: example.note && t.block(example.note),
  };
}

export function renderRulePage(rule: RuleMetadata, example: RuleExample, locale: RuleLocale) {
  const text = pageText[locale];
  const copy = ruleCopy(rule, example, locale);
  const config = ruleConfiguration(rule, example);
  const examples = pageExamples(example);
  const changes = changedCodeLines(examples.bad.source, examples.good.source);
  const fix = example.availability ? text.notForSfc : rule.fixable ? text.yes : text.no;
  const options = configurableRules.has(rule.name) ? text.optionsLink : text.none;
  const group = owningRuleGroup(rule.name, false);
  const lines = [
    "---",
    `title: "${rule.name}"`,
    `description: ${JSON.stringify(sentence(copy.purpose, locale))}`,
    "---",
    "",
    `# \`${rule.name}\``,
    "",
    escapeTags(sentence(copy.purpose, locale)),
    "",
    `| ${text.severity} | ${text.presets} | ${text.fix} | ${text.options} |`,
    "| --- | --- | --- | --- |",
    `| \`${rule.defaultSeverity}\` | ${presetList(rule.presets, locale)} | ${cell(fix)} | ${options} |`,
    "",
    `${text.appliesTo}${colon(locale)}${sentence(copy.scope, locale)}`,
    "",
  ];
  if (example.availability)
    lines.push(
      "> [!WARNING]",
      // The linter test keys the audited unsupported boundary on the English line.
      `> ${text.currentSupport}${colon(locale)}\`${example.availability}\`${cjk(locale) ? "。" : "."} ${text.unsupported}`,
      "",
    );
  if (example.badDiagnostic)
    lines.push(`${text.badDiagnostic}${colon(locale)}\`${example.badDiagnostic}\``, "");
  if (copy.note) lines.push(copy.note, "");
  if (rule.name === "vue/max-template-complexity") lines.push(text.complexity, "");
  if (config.typeAware) lines.push(text.typeAware, "");
  for (const key of ["bad", "good"] as const) {
    const { language, source } = examples[key];
    const changed = key === "bad" ? changes.removed : changes.added;
    const annotation = changed.length
      ? ` annotate="${key === "bad" ? "remove" : "add"}:${changed.join(",")}"`
      : "";
    lines.push(`## ${text[key]}`, "", copy[key], "");
    const filename = example[`${key}Filename`] ?? example.filename;
    if (filename) lines.push(`\`${filename}\``, "");
    lines.push(`\`\`\`${language}${annotation}`, source, "```", "");
  }
  if (locale === "ja" && example.availability)
    lines.push('<span id="設定できる-id-現在の-sfc-検出なし"></span>', "");
  lines.push(
    `## ${example.availability ? text.configuredId : text.configuration}`,
    "",
    "```ts",
    'import { defineConfig } from "@vizejs/vite-plugin/vite-plus";',
    "",
    "export default defineConfig({",
    "  vue: {",
    `    linter: ${JSON.stringify(config, null, 2).split("\n").join("\n    ")},`,
    "  },",
    "});",
    "```",
    "",
    text.run,
    "",
    `## ${text.related}`,
    "",
    `- [${categoryTitle(group, locale)}](../${group}.md)`,
    ...(configurableRules.has(rule.name) ? [`- ${text.optionsLink}`] : []),
    `- [${text.allRules}](../all.md)`,
    `- [${text.source}](https://github.com/ubugeeei-prod/vize/blob/main/${rule.implementationPath}#L${rule.implementationLine})`,
    "",
  );
  return lines.join("\n");
}
