import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { generateCategoryPages } from "./category-render.mjs";
import { generateProjectPages } from "./project-render.mjs";
import { ruleExamples } from "./examples.mjs";
import { exampleExplanations, validateExampleExplanations } from "./explanations.mjs";
import { migrationPage } from "./migration.mjs";
import { purposeJa } from "./purpose-ja.mjs";
import { exampleLinks } from "./example-links.mjs";
import { projectIndex } from "./project-index.mjs";

export const configurableRules = new Set([
  "html/no-empty-palpable-content",
  "musea/prefer-design-tokens",
  "script/custom-event-name-casing",
  "script/define-props-destructuring",
  "script/no-restricted-globals",
  "script/no-restricted-members",
  "type/strict-boolean-expressions",
  "vue/attribute-hyphenation",
  "vue/component-name-in-template-casing",
  "vue/html-self-closing",
  "vue/no-mutating-props",
  "vue/require-component-registration",
  "vue/sfc-element-order",
  "vue/v-on-event-hyphenation",
]);

export function generateRulePages({
  workspaceRoot,
  rules,
  groupedRules,
  sortedCategories,
  categoryLabels,
}) {
  const checking = process.argv.includes("--check");
  validateExampleExplanations(rules);
  generateProjectPages(workspaceRoot, checking);
  generateCategoryPages(workspaceRoot, rules, checking);
  for (const locale of ["", "ja/"]) {
    const directory = resolve(workspaceRoot, `docs/content/${locale}rules`);
    if (!checking) mkdirSync(resolve(directory, "reference"), { recursive: true });
    const ja = locale === "ja/";
    output(
      resolve(directory, "migration.md"),
      migrationPage(workspaceRoot, new Set(rules.map((rule) => rule.name)), ja),
      checking,
    );
    const lines = [
      "---",
      `title: ${ja ? "全 lint ルール" : "All lint rules"}`,
      "---",
      "",
      `# ${ja ? "全 lint ルール" : "All lint rules"}`,
      "",
      ja
        ? `現在のソース カタログにある ${rules.length} 項目の一覧です。未対応の範囲は個別ページに明記しています。ルール名から、目的・適用範囲・設定・悪い例・良い例を確認できます。`
        : `All ${rules.length} source catalog entries, including explicitly marked support gaps. Follow a rule name for its purpose, scope, configuration, and Bad/Good examples.`,
      "",
      ja
        ? "Vite+ では `@vizejs/vite-plugin/vite-plus` の `defineConfig` を使い、`lint.vize.rules` に指定します。`vp run lint` で Vize と Oxlint の lint を実行します。"
        : "With Vite+, import `defineConfig` from `@vizejs/vite-plugin/vite-plus`, configure `lint.vize.rules`, and run `vp run lint` for Vize and Oxlint diagnostics.",
      "",
      ja
        ? "既定の重大度は実装の値です。プロジェクトでは `off` / `warn` / `error` に変更できます。Options は [ルール オプション](./options.md)、複数ファイルの検査は [Cross-file ルール](./cross-file.md) を参照してください。"
        : "Severity is the implementation default; override it with `off`, `warn`, or `error`. See [Rule Options](./options.md) for rule-option support and [Cross-file rules](./cross-file.md) for project-graph findings.",
      "",
      ja
        ? "プリセットが `_none_` のルールは明示的な有効化または追加の設定が必要です。"
        : "`_none_` means explicit enablement or host configuration is required. `general-recommended` is displayed as `happy-path`.",
      "",
      ja
        ? "[ESLint からのルール移行対応表](./migration.md)で対応名と未実装の範囲を確認できます。"
        : "See the [ESLint migration map](./migration.md) for rule IDs, differences, and unsupported mappings.",
      "",
      `## ${ja ? "カテゴリ" : "Categories"}`,
      "",
      ja ? "| カテゴリ | ルール数 |" : "| Category | Rules |",
      "| --- | ---: |",
    ];
    for (const category of sortedCategories) {
      const label = categoryLabels[category] ?? category;
      const count = groupedRules.get(category).length;
      lines.push(`| [${label}](#${slug(`${label} ${count}`)}) | ${count} |`);
    }
    for (const category of sortedCategories) {
      const group = groupedRules.get(category);
      const label = categoryLabels[category] ?? category;
      lines.push(
        "",
        `## ${label} (${group.length})`,
        "",
        ja
          ? "| ルール | 例 | 重大度 | プリセット | 自動修正 | オプション | 実装 | 目的 |"
          : "| Rule | Examples | Severity | Presets | Fixable | Options | Implementation | Description |",
        "| --- | --- | --- | --- | --- | --- | --- | --- |",
      );
      for (const rule of group) {
        const path = `${slug(rule.name)}.md`;
        const example = ruleExamples(workspaceRoot, rule);
        if (!purposeJa[rule.name]) throw new Error(`Missing Japanese purpose for ${rule.name}`);
        lines.push(
          `| [\`${rule.name}\`](./reference/${path}) | ${exampleLinks(`./reference/${path}`, ja)} | \`${rule.defaultSeverity}\` | ${presets(rule.presets)} | ${rule.fixable ? (ja ? "あり" : "Yes") : ja ? "なし" : "No"} | ${configurableRules.has(rule.name) ? "[`ruleOptions`](./options.md)" : ja ? "なし" : "No"} | ${implementation(rule)} | ${cell(ja ? purposeJa[rule.name] : rule.description)} |`,
        );
        output(resolve(directory, "reference", path), detail(rule, example, ja), checking);
      }
    }
    lines.push(...projectIndex(workspaceRoot, ja));
    output(resolve(directory, "all.md"), `${lines.join("\n")}\n`, checking);
  }
}

function detail(rule, example, ja) {
  const label = (en, japanese) => (ja ? japanese : en);
  const config = {
    preset: "incremental",
    rules: { [rule.name]: rule.defaultSeverity === "warning" ? "warn" : "error" },
    ...(example.typeAware || rule.name.startsWith("type/") ? { typeAware: true } : {}),
    ...(example.ruleOptions ? { ruleOptions: { [rule.name]: example.ruleOptions } } : {}),
  };
  const scope = scopeLabel(rule, example, ja);
  const options = configurableRules.has(rule.name)
    ? label(
        "See [typed options and defaults](../options.md).",
        "[型付きオプションと既定値](../options.md)を参照してください。",
      )
    : label(
        "No rule-specific options. Severity and preset selection are configurable.",
        "ルール固有のオプションはありません。重大度とプリセットは設定できます。",
      );
  const lines = [
    "---",
    `title: "${rule.name}"`,
    "---",
    "",
    `# \`${rule.name}\``,
    "",
    ja ? purposeJa[rule.name] : rule.description,
    "",
    exampleLinks("", ja),
    "",
    `${label("Default severity", "既定の重大度")}: \`${rule.defaultSeverity}\`  `,
    `${label("Presets", "プリセット")}: ${presets(rule.presets)}  `,
    `${label("Automatic fix", "自動修正")}: ${example.availability ? label("Not implemented for SFC lint", "SFC lint では未対応") : rule.fixable ? label("Available for supported findings", "対応する検出で利用可能") : label("None; review the suggested change", "なし。修正内容を確認してください")}  `,
    `${label("Applies to", "適用範囲")}: ${scope}  `,
    `${label("Options", "オプション")}: ${options}`,
    "",
  ];
  if (example.availability)
    lines.push(`${label("Current support", "現在の対応")}: \`${example.availability}\``, "");
  if (example.badDiagnostic)
    lines.push(`${label("Bad diagnostic", "悪い例での診断")}: \`${example.badDiagnostic}\``, "");
  const note = ja ? example.noteJa : example.note;
  if (note) lines.push(note, "");
  if (rule.name === "vue/max-template-complexity")
    lines.push(
      label(
        "See [complexity scoring and component boundaries](../../guide/cross-file-complexity.md) for the contributions behind the example's two scores.",
        "例の二つの値の計算内訳は[複雑度の計算とコンポーネントの境界](../../guide/cross-file-complexity.md)を参照してください。",
      ),
      "",
    );
  if (config.typeAware)
    lines.push(
      label(
        "Type-aware checks use the native Corsa runtime and the TypeScript project. `typeAware` alone does not enable an opt-in rule.",
        "型を使う検査には Corsa と TypeScript プロジェクトが必要です。typeAware だけでは opt-in ルールは有効になりません。",
      ),
      "",
    );
  lines.push(
    `## ${example.availability ? label("Configured ID (currently no SFC finding)", "設定できる ID（現在の SFC 検出なし）") : label("Configuration (Vite+)", "設定（Vite+）")}`,
    "",
    "```ts",
    'import { defineConfig } from "@vizejs/vite-plugin/vite-plus";',
    "",
    "export default defineConfig({",
    "  lint: {",
    `    vize: ${JSON.stringify(config, null, 2).split("\n").join("\n    ")},`,
    "  },",
    "});",
    "```",
    "",
    "```sh",
    "vp run lint",
    "```",
    "",
  );
  for (const [key, title] of [
    ["bad", label("Bad", "悪い")],
    ["good", label("Good", "良い")],
  ]) {
    let { language, source } = example[key];
    if (example.standaloneScript) {
      language = "ts";
      source = source.replace(/^<script[^>]*>\n/, "").replace(/\n<\/script>$/, "");
    }
    lines.push(`## ${title}`, "", exampleExplanations.get(rule.name)[key][ja ? "ja" : "en"], "");
    const filename = example[`${key}Filename`] ?? example.filename;
    if (filename) lines.push(`\`${filename}\``, "");
    lines.push(`\`\`\`${language}`, source, "```", "");
  }
  lines.push(
    label(
      "Good avoids this rule's finding under the configuration above; other rules may still report diagnostics.",
      "良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。",
    ),
    "",
    `[${label("Implementation", "実装")}](https://github.com/ubugeeei-prod/vize/blob/main/${rule.implementationPath}#L${rule.implementationLine}) · [${label("All rules", "全ルール")}](../all.md)`,
    "",
  );
  return lines.join("\n");
}

function scopeLabel(rule, example, ja) {
  if (example.standaloneScript)
    return ja ? "Nuxt 設定ファイル（nuxt.config.ts）" : "Nuxt configuration files (nuxt.config.ts)";
  if (rule.name.startsWith("petite-vue/"))
    return ja
      ? "petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。"
      : "HTML documents detected as petite-vue; ordinary Vue SFCs are outside this rule's scope.";
  if (rule.metaType === "CssRuleMeta")
    return ja ? "SFC の style ブロック内の CSS" : "CSS inside SFC style blocks";
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
      ? "Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。"
      : "JS/TS scripts in Vue SFCs; examples show the relevant Options API or script setup form";
  return ja
    ? "Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。"
    : "Vue SFC templates and blocks, with script context where the rule requires it";
}

function presets(values) {
  return values.length
    ? values
        .map((value) => `\`${value === "general-recommended" ? "happy-path" : value}\``)
        .join(", ")
    : "_none_";
}
function implementation(rule) {
  return `[source](https://github.com/ubugeeei-prod/vize/blob/main/${rule.implementationPath}#L${rule.implementationLine})`;
}
function slug(value) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
}
function cell(value) {
  return String(value)
    .replaceAll("|", "\\|")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replace(/\s+/g, " ")
    .trim();
}
function output(path, content, checking) {
  if (checking) {
    if (readFileSync(path, "utf8") !== content)
      throw new Error(`Generated rule reference is stale: ${path}`);
  } else writeFileSync(path, content);
}
