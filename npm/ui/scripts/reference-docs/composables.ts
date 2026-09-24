/** Reference pages for `@vizejs/composable` entries. */
import path from "node:path";

import type {
  ComposableEntryMetadata,
  ComposableUtilityMetadata,
} from "../../../compose/core/src/catalog.ts";
import { moduleExports, moduleInterfaces } from "./extract.ts";
import { GENERATED_NOTICE, blocks, cell, frontmatter, membersTable, table } from "./markdown.ts";

/** Entries that describe the package rather than ship behavior. */
export const COMPOSABLE_REFERENCE_EXCLUDED: ReadonlySet<string> = new Set(["./catalog"]);

export function composableName(entry: ComposableEntryMetadata): string {
  return entry.subpath.slice(2);
}

function utilitiesOf(
  entry: ComposableEntryMetadata,
  utilities: readonly ComposableUtilityMetadata[],
): readonly ComposableUtilityMetadata[] {
  return utilities.filter((utility) => utility.entry === entry.subpath);
}

/** Category used to group the index (first utility's, else "other"). */
export function composableCategory(
  entry: ComposableEntryMetadata,
  utilities: readonly ComposableUtilityMetadata[],
): string {
  return utilitiesOf(entry, utilities)[0]?.category ?? "other";
}

/** Summary: TSDoc of the first documented export, else the export list. */
export function composableSummary(packageRoot: string, entry: ComposableEntryMetadata): string {
  const exports = moduleExports(path.join(packageRoot, entry.source));
  const documented = exports.find(
    (item) => item.description !== "" && entry.runtimeExports.includes(item.name),
  );
  const sentence = documented?.description.split(/(?<=\.)\s/)[0];
  return sentence ?? `Provides ${entry.runtimeExports.join(", ")}.`;
}

function list(values: readonly string[]): string {
  return values.length === 0 ? "—" : cell(values.join(", "));
}

/** Render `docs/content/guide/composables/<name>.md`. */
export function renderComposablePage(
  packageRoot: string,
  entry: ComposableEntryMetadata,
  utilities: readonly ComposableUtilityMetadata[],
): string {
  const name = composableName(entry);
  const own = utilitiesOf(entry, utilities);
  const summary = composableSummary(packageRoot, entry);
  const source = path.join(packageRoot, entry.source);
  const exports = moduleExports(source).filter((item) => entry.runtimeExports.includes(item.name));
  const interfaces = moduleInterfaces(source);
  const facts = table(
    ["", ""],
    [
      ["Package", `\`@vizejs/composable/${name}\``],
      ["Own the source", `\`vize lib pull composable:${name}\``],
      ["Runtime exports", list(entry.runtimeExports.map((item) => `\`${item}\``))],
      ["Gzip budget", `${entry.gzipBudgetBytes} B`],
    ],
  );
  const contract = table(
    [
      "Utility",
      "Category",
      "Stability",
      "SSR",
      "Hydration",
      "Cleanup",
      "Targets",
      "Host globals",
      "Uses",
    ],
    own.map((utility) => [
      `\`${utility.name}\``,
      utility.category,
      utility.stability,
      utility.ssr,
      utility.hydration,
      list(utility.cleanupOwners),
      list(utility.targets),
      list(utility.runtimeGlobals.map((global) => `\`${global}\``)),
      list(utility.dependencies.map((dependency) => `\`${dependency}\``)),
    ]),
  );
  const api = exports
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
  const types = interfaces
    .filter((item) => item.members.length > 0)
    .map((item) =>
      blocks(`### \`${item.name}\``, item.description, membersTable(item.members)).trimEnd(),
    )
    .join("\n\n");
  const usage = [
    "## Usage",
    "",
    "```ts",
    `import { ${entry.runtimeExports.join(", ")} } from "@vizejs/composable/${name}";`,
    "```",
  ].join("\n");
  return blocks(
    frontmatter(name, summary),
    GENERATED_NOTICE,
    `# ${name}`,
    summary,
    facts,
    usage,
    contract === "" ? "" : `## Runtime contract\n\n${contract}`,
    api === "" ? "" : `## API\n\n${api}`,
    types === "" ? "" : `## Types\n\n${types}`,
  );
}

/** Index body grouped by category. */
export function composableIndexRows(
  packageRoot: string,
  entries: readonly ComposableEntryMetadata[],
  utilities: readonly ComposableUtilityMetadata[],
  linkPrefix: string,
): string {
  const categories = [
    ...new Set(entries.map((entry) => composableCategory(entry, utilities))),
  ].sort();
  return categories
    .map((category) =>
      blocks(
        `## ${category}`,
        table(
          ["Entry", "Exports", "Summary"],
          entries
            .filter((entry) => composableCategory(entry, utilities) === category)
            .map((entry) => [
              `[${composableName(entry)}](${linkPrefix}${composableName(entry)}.md)`,
              list(entry.runtimeExports.map((item) => `\`${item}\``)),
              cell(composableSummary(packageRoot, entry)),
            ]),
        ),
      ).trimEnd(),
    )
    .join("\n\n");
}
