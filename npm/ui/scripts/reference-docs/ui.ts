/** Reference pages for `@vizejs/ui` families. */
import { readFileSync } from "node:fs";
import path from "node:path";

import type { UiFamilyCatalogEntry } from "../../src/catalog/family-catalog-types.ts";
import { featuredExamples, previewMarkup, publicExample } from "./examples.ts";
import {
  componentExports,
  extractComponentApi,
  moduleExports,
  moduleSummary,
  type ModuleExport,
} from "./extract.ts";
import {
  GENERATED_NOTICE,
  blocks,
  cell,
  emitsTable,
  embedBehavior,
  frontmatter,
  membersTable,
  propsTable,
  slotsTable,
  table,
} from "./markdown.ts";

/** Human-readable group of a family, from `src/families/<group>/…`. */
export function familyGroup(entry: UiFamilyCatalogEntry): string {
  return /^src\/families\/([^/]+)\//.exec(entry.entryFile)?.[1] ?? "other";
}

/** One-sentence summary: the entry module's TSDoc, else the catalog coverage. */
export function familySummary(packageRoot: string, entry: UiFamilyCatalogEntry): string {
  const summary = moduleSummary(path.join(packageRoot, entry.entryFile));
  if (summary !== "") return summary;
  return entry.upstreamCoverage.length === 0
    ? `Headless ${entry.title}.`
    : `Headless ${entry.title}; covers ${entry.upstreamCoverage.slice(0, 3).join(", ")}.`;
}

function exportsSection(exports: readonly ModuleExport[]): string {
  return exports
    .map((item) =>
      blocks(
        `### \`${item.name}\``,
        item.description,
        ["```ts", item.signature, "```"].join("\n"),
        ...item.examples.map((example) =>
          example.startsWith("```") ? example : ["```ts", example, "```"].join("\n"),
        ),
      ).trimEnd(),
    )
    .join("\n\n");
}

function componentSections(packageRoot: string, entry: UiFamilyCatalogEntry): string {
  const components = componentExports(path.join(packageRoot, entry.entryFile));
  return components
    .map(({ name, file }) => {
      const api = extractComponentApi(file, name);
      const heading = api.generic == null ? `### \`${name}\`` : `### \`${name}<${api.generic}>\``;
      const source = path.relative(packageRoot, file);
      return blocks(
        heading,
        `Source: \`${source}\``,
        api.props.length === 0 ? "" : `#### Props\n\n${propsTable(api.props)}`,
        api.emits.length === 0 ? "" : `#### Events\n\n${emitsTable(api.emits)}`,
        api.slots.length === 0 ? "" : `#### Slots\n\n${slotsTable(api.slots)}`,
        api.expose.length === 0 ? "" : `#### Exposed\n\n${membersTable(api.expose)}`,
      ).trimEnd();
    })
    .join("\n\n");
}

function foundationExports(packageRoot: string, entry: UiFamilyCatalogEntry): string {
  const seen = new Set<string>();
  const exports = entry.sourceFiles
    .filter((file) => file.endsWith(".ts") && !file.endsWith("-types.ts"))
    .flatMap((file) => moduleExports(path.join(packageRoot, file)))
    .filter((item) => {
      if (seen.has(item.name)) return false;
      seen.add(item.name);
      return true;
    });
  return exportsSection(exports);
}

/** Render `docs/content/guide/ui/<name>.md`. */
export function renderUiFamilyPage(packageRoot: string, entry: UiFamilyCatalogEntry): string {
  const summary = familySummary(packageRoot, entry);
  const subpath = entry.packageSubpath === "." ? "" : entry.packageSubpath.slice(1);
  const components = componentExports(path.join(packageRoot, entry.entryFile));
  const importNames = components.map((component) => component.name);
  const facts = table(
    ["", ""],
    [
      ["Package", `\`@vizejs/ui${subpath}\``],
      ["Maturity", entry.maturity],
      ["Own the source", `\`vize lib pull ${entry.canonicalName}\``],
      [
        "Requires",
        entry.dependencies.length === 0
          ? "—"
          : entry.dependencies.map((name) => `[${name}](./${name}.md)`).join(", "),
      ],
      ["Aliases", entry.aliases.length === 0 ? "—" : cell(entry.aliases.join(", "))],
      [
        "Covers",
        entry.upstreamCoverage.length === 0 ? "—" : cell(entry.upstreamCoverage.join(", ")),
      ],
    ],
  );
  const usage =
    importNames.length === 0
      ? blocks(
          "## Minimal setup",
          "Use a Vue 3.5+ project. Install the package, then import the functions and types documented in the API below from this entry.",
          "```bash\nvp install @vizejs/ui\n```",
          `Public entry: \`@vizejs/ui${subpath}\`. See [Vite+ integration](../vite-plus.md) for project setup.`,
        )
      : [
          "## Minimal setup",
          "",
          "Use a Vue 3.5+ project with Vue SFC compilation configured (see [Vite+ integration](../vite-plus.md)).",
          "",
          "```bash",
          "vp install @vizejs/ui",
          "```",
          "",
          'Add the public import below to `<script setup lang="ts">`. Components provide behavior without styles; the preview uses optional [Paper styles](../ui-styles.md).',
          "",
          "```ts",
          `import { ${importNames.join(", ")} } from "@vizejs/ui${subpath}";`,
          "```",
          "",
          `Or copy the source into your project with \`vize lib pull ${entry.canonicalName}\` (see [Source Distribution](../lib-pull.md)).`,
        ].join("\n");
  const example = publicExample(packageRoot, entry);
  const api =
    components.length > 0
      ? componentSections(packageRoot, entry)
      : foundationExports(packageRoot, entry);
  const behavior = embedBehavior(
    readFileSync(path.join(packageRoot, entry.behaviorContract), "utf8"),
  );
  return blocks(
    frontmatter(entry.title, summary),
    GENERATED_NOTICE,
    `# ${entry.title}`,
    summary,
    facts,
    usage,
    example == null
      ? ""
      : blocks(
          "## Try it",
          "This is the basic example maintained with the component source, using the public package imports. The preview adds the optional Paper styles.",
          previewMarkup(
            entry.canonicalName,
            entry.title,
            (featuredExamples as readonly string[]).includes(entry.canonicalName),
          ),
          "## Copy the example",
          ["```vue", example.trim(), "```"].join("\n"),
        ),
    api === "" ? "" : `## API\n\n${api}`,
    `## Behavior\n\n${behavior}`,
  );
}

/** Rows of the UI index: group, link, maturity, summary. */
export function uiIndexRows(
  packageRoot: string,
  entries: readonly UiFamilyCatalogEntry[],
  linkPrefix: string,
  locale = "en",
): string {
  const names: Record<string, readonly [string, string]> = {
    actions: ["Run actions", "操作を実行する"],
    form: ["Collect and validate input", "入力・検証"],
    selection: ["Choose values", "値を選ぶ"],
    navigation: ["Move through content", "コンテンツを移動する"],
    overlays: ["Show contextual content", "補足・確認を表示する"],
    disclosure: ["Reveal details", "詳細を開く"],
    feedback: ["Show status and progress", "状態・進捗を伝える"],
    layout: ["Arrange a page", "画面を組み立てる"],
    data: ["Explore data", "データを見る"],
    charts: ["Visualize data", "データを可視化する"],
    "date-time": ["Pick dates and times", "日時を選ぶ"],
    editor: ["Edit content", "コンテンツを編集する"],
    media: ["Present media", "メディアを表示する"],
    typography: ["Format text", "文字を整える"],
    accessibility: ["Support accessible interactions", "操作のアクセシビリティ"],
    interaction: ["Handle gestures and input", "ジェスチャー・操作"],
    foundations: ["Compose the foundations", "基本の振る舞いを組み合わせる"],
    i18n: ["Localize an interface", "インターフェースを翻訳する"],
  };
  const groups = [...new Set(entries.map(familyGroup))].sort();
  return groups
    .map((group) =>
      blocks(
        `### ${names[group]?.[locale === "ja" ? 1 : 0] ?? group}`,
        table(
          ["Family", "Maturity", "Summary"],
          entries
            .filter((entry) => familyGroup(entry) === group)
            .map((entry) => [
              `[${entry.title}](${linkPrefix}${entry.canonicalName}.md)`,
              entry.maturity,
              cell(familySummary(packageRoot, entry)),
            ]),
        ),
      ).trimEnd(),
    )
    .join("\n\n");
}
