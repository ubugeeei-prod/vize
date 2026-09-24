/** Reference pages for `@vizejs/ui` families. */
import { readFileSync } from "node:fs";
import path from "node:path";

import type { UiFamilyCatalogEntry } from "../../src/catalog/family-catalog-types.ts";
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
      ? ""
      : [
          "## Usage",
          "",
          "```ts",
          `import { ${importNames.join(", ")} } from "@vizejs/ui${subpath}";`,
          "```",
          "",
          `Or copy the source into your project with \`vize lib pull ${entry.canonicalName}\` (see [Source Distribution](../lib-pull.md)).`,
        ].join("\n");
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
    api === "" ? "" : `## API\n\n${api}`,
    `## Behavior\n\n${behavior}`,
  );
}

/** Rows of the UI index: group, link, maturity, summary. */
export function uiIndexRows(
  packageRoot: string,
  entries: readonly UiFamilyCatalogEntry[],
  linkPrefix: string,
): string {
  const groups = [...new Set(entries.map(familyGroup))].sort();
  return groups
    .map((group) =>
      blocks(
        `## ${group}`,
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
