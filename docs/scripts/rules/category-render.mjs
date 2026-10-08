import { readFileSync, writeFileSync } from "node:fs";
import { purposeJa } from "./purpose-ja.mjs";
import { resolve } from "node:path";
import { exampleLinks } from "./example-links.mjs";
/** @type {Array<[string, string, string, (name: string) => boolean]>} */
const categories = [
  ["vue", "Vue rules", "Vue ルール", (name) => name.startsWith("vue/")],
  [
    "type-and-script",
    "Type and script rules",
    "型と script のルール",
    (name) => name.startsWith("script/") || name.startsWith("type/"),
  ],
  ["html", "HTML rules", "HTML ルール", (name) => name.startsWith("html/")],
  [
    "accessibility",
    "Accessibility rules",
    "アクセシビリティ ルール",
    (name) => name.startsWith("a11y/") || name === "vue/use-unique-element-ids",
  ],
  ["ssr", "SSR rules", "SSR ルール", (name) => name.startsWith("ssr/")],
  ["petite-vue", "petite-vue rules", "petite-vue ルール", (name) => name.startsWith("petite-vue/")],
  [
    "vapor",
    "Vapor rules",
    "Vapor ルール",
    (name) =>
      name.startsWith("vapor/") ||
      ["script/no-options-api", "script/no-get-current-instance", "script/no-next-tick"].includes(
        name,
      ),
  ],
  [
    "ecosystem",
    "Ecosystem rules",
    "エコシステム ルール",
    (name) => name.startsWith("ecosystem/") || name.startsWith("nuxt/"),
  ],
  [
    "musea-and-css",
    "Musea and CSS rules",
    "Musea と CSS のルール",
    (name) => name.startsWith("musea/") || name.startsWith("css/"),
  ],
];
export function generateCategoryPages(root, rules, checking) {
  for (const [file, en, jp, select] of categories) {
    for (const locale of ["", "ja/"]) {
      const ja = Boolean(locale);
      const title = ja ? jp : en;
      const lines = [
        "---",
        `title: ${title}`,
        "---",
        "",
        `# ${title}`,
        "",
        ja
          ? "ルール名から、目的・重大度・適用範囲・設定・悪い例・良い例を確認できます。全ルールの一覧に例と現在の対応範囲を同じページでまとめています。"
          : "Follow each rule for purpose, severity, scope, configuration, and Bad/Good examples. The complete catalogue keeps all examples and current support boundaries on one page.",
        "",
        ja
          ? "Vite+ では `lint.vize.rules` に設定し、`vp run lint` を実行します。型が必要なルールや、専用ファイル・追加設定が必要なルールは個別ページの前提を確認してください。"
          : "Configure `lint.vize.rules` and run `vp run lint` with the Vite+ helper. Check each page for type-aware, filename, or additional-configuration prerequisites.",
        "",
        ja ? "| ルール | 例 | 目的 |" : "| Rule | Examples | Purpose |",
        "| --- | --- | --- |",
      ];
      for (const rule of rules
        .filter((rule) => select(rule.name))
        .sort((a, b) => a.name.localeCompare(b.name))) {
        lines.push(
          `| [\`${rule.name}\`](./all.md#${rule.name.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase()}) | ${exampleLinks("./all.md", ja, rule.name.replaceAll(/[^a-zA-Z0-9]+/g, "-").toLowerCase())} | ${(ja ? purposeJa[rule.name] : rule.description).replaceAll("|", "\\|").replaceAll("<", "&lt;").replaceAll(">", "&gt;")} |`,
        );
      }
      lines.push(
        "",
        ja
          ? "[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md)"
          : "[All rules](./all.md) · [Rule Options](./options.md) · [ESLint migration map](./migration.md) · [Project checks](./cross-file.md)",
        "",
      );
      if (file === "ecosystem")
        lines.push(
          ja
            ? "型付き Vue Router の四つの診断には、[完全なプロジェクトの例](./cross-file.md)を参照してください。"
            : "For the four typed Vue Router diagnostics, see the [complete project examples](./cross-file.md).",
          "",
        );
      if (file === "html")
        lines.push(
          ja
            ? "子の要素を合成した入れ子は [html/cross-component-nesting](./project/html-cross-component-nesting.md) の対象です。"
            : "For nesting after components are composed, see [html/cross-component-nesting](./project/html-cross-component-nesting.md).",
          "",
        );
      if (file === "vue")
        lines.push(
          ja
            ? "子への属性の継承は [vue/cross-file-attrs-fallthrough](./project/vue-cross-file-attrs-fallthrough.md) の対象です。"
            : "For attributes across imported components, see [vue/cross-file-attrs-fallthrough](./project/vue-cross-file-attrs-fallthrough.md).",
          "",
        );
      const path = resolve(root, `docs/content/${locale}rules/${file}.md`);
      const text = lines.join("\n");
      if (checking) {
        if (readFileSync(path, "utf8") !== text) throw new Error(`Stale rule category: ${path}`);
      } else writeFileSync(path, text);
    }
  }
  // Keep previous accessibility group URLs useful while all details use one route per rule.
  const groups = {
    core: [
      "alt-text",
      "anchor-has-content",
      "anchor-is-valid",
      "aria-props",
      "aria-role",
      "aria-unsupported-elements",
      "click-events-have-key-events",
      "form-control-has-label",
    ],
    structure: [
      "heading-has-content",
      "heading-levels",
      "iframe-has-title",
      "img-alt",
      "interactive-supports-focus",
      "label-has-for",
      "landmark-roles",
      "media-has-caption",
    ],
    interactions: [
      "mouse-events-have-key-events",
      "no-access-key",
      "no-aria-hidden-on-focusable",
      "no-autofocus",
      "no-distracting-elements",
      "no-i-for-icon",
      "no-redundant-roles",
      "no-refer-to-non-existent-id",
    ],
    integrity: [
      "no-role-presentation-on-focusable",
      "no-static-element-interactions",
      "placeholder-label-option",
      "role-has-required-aria-props",
      "tabindex-no-positive",
      "use-list",
    ],
  };
  for (const [group, ids] of Object.entries(groups)) {
    const text = [
      "---",
      `title: Accessibility ${group}`,
      "---",
      "",
      `# Accessibility ${group}`,
      "",
      ...ids.map((id) => `- [\`a11y/${id}\`](./reference/a11y-${id}.md)`),
      ...(group === "integrity"
        ? ["- [`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md)"]
        : []),
      "",
      "[All accessibility rules](./accessibility.md)",
      "",
    ].join("\n");
    const path = resolve(root, `docs/content/rules/accessibility-${group}.md`);
    if (checking) {
      if (readFileSync(path, "utf8") !== text)
        throw new Error(`Stale accessibility group: ${path}`);
    } else writeFileSync(path, text);
  }
}
