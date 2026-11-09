/**
 * Publish the source-owned documentation catalogs as one stable JSON endpoint.
 *
 * The docs build writes `public/vize-docs-metadata.json`; editors and other
 * tooling can consume the same catalogs that generate the human reference
 * pages instead of maintaining a second list of names or install commands.
 */
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";

import { COMPOSABLE_CATALOG } from "../../npm/compose/core/src/catalog.ts";
import {
  UI_FAMILY_CATALOG_SCHEMA_VERSION,
  uiFamilyCatalog,
} from "../../npm/ui/src/catalog/family-catalog.ts";

const root = resolve(import.meta.dirname, "../..");

interface PackageManifest {
  readonly name: string;
  readonly exports?: Record<string, unknown>;
  readonly metadata?: { readonly vize?: { readonly stability?: string } };
}

function manifest(relativePath: string): PackageManifest {
  return JSON.parse(readFileSync(resolve(root, relativePath), "utf8")) as PackageManifest;
}

function publicSubpaths(relativePath: string): readonly string[] {
  return Object.keys(manifest(relativePath).exports ?? {}).sort();
}

const evidence = {
  compose: [
    "npm/compose/core/src/catalog.ts",
    "tests/tooling/docs-public-api.test.ts",
    "npm/ui/scripts/generate-reference-docs.ts",
  ],
  ui: [
    "npm/ui/src/catalog/family-catalog.ts",
    "npm/ui/scripts/generate-reference-docs.ts",
    "npm/ui/scripts/check-renderers.ts",
  ],
  fresco: [
    "docs/fresco/compatibility.md",
    "tests/tooling/fresco-public-docs.test.ts",
    "tests/tooling/fresco-testing-node.test.ts",
  ],
  patina: [
    "docs/scripts/generate-patina-rules-page.ts",
    "docs/content/rules/index.md",
    ".github/workflows/pr-rust-checks.yml",
  ],
  glyph: [
    "docs/content/guide/formatter.md",
    ".github/workflows/pr-rust-checks.yml",
    "crates/vize_glyph/src/lib.rs",
  ],
  levels: [
    "docs/davinci/README.md",
    "tests/tooling/compiled-docs-inputs.test.ts",
    "docs/davinci/plan/budgets.toml",
  ],
  devtools: ["npm/devtools/README.md", "tests/tooling/docs-public-api.test.ts"],
  plugins: ["npm/plugin-sdk/README.md", "tests/tooling/docs-public-api.test.ts"],
} as const;

/** Complete serializable contract published by the documentation site. */
export function documentationContractMetadata() {
  const fresco = manifest("npm/fresco/package.json");
  const devtools = manifest("npm/devtools/package.json");
  const plugins = manifest("npm/plugin-sdk/package.json");

  return {
    schemaVersion: 1,
    endpoint: "/vize-docs-metadata.json",
    generatedFrom: [
      "npm/compose/core/src/catalog.ts",
      "npm/ui/src/catalog/family-catalog.ts",
      "package export maps",
      "source-owned documentation gates",
    ],
    surfaces: [
      {
        id: "compose",
        title: "Compose",
        packageName: COMPOSABLE_CATALOG.packageName,
        documentation: "/guide/composables/",
        installation: {
          package: `pnpm add ${COMPOSABLE_CATALOG.packageName}`,
          source: COMPOSABLE_CATALOG.sourceInstallation.specifier,
        },
        evidence: evidence.compose,
        catalog: COMPOSABLE_CATALOG,
        completionItems: COMPOSABLE_CATALOG.utilities.map((item) => ({
          label: item.name,
          detail: `${COMPOSABLE_CATALOG.packageName}${item.entry.slice(1)}`,
          documentation: `/guide/composables/${item.entry.slice(2)}.html`,
          stability: item.stability,
          targets: item.targets,
        })),
      },
      {
        id: "ui",
        title: "UI",
        packageName: "@vizejs/ui",
        documentation: "/guide/ui/",
        installation: {
          package: "pnpm add @vizejs/ui",
          source: "vize lib pull <family>",
        },
        evidence: evidence.ui,
        catalog: {
          schemaVersion: UI_FAMILY_CATALOG_SCHEMA_VERSION,
          families: uiFamilyCatalog,
        },
        completionItems: uiFamilyCatalog.map((family) => ({
          label: family.title,
          detail: `@vizejs/ui${family.packageSubpath.slice(1)}`,
          documentation: `/guide/ui/${family.canonicalName}.html`,
          aliases: family.aliases,
          maturity: family.maturity,
          sourceInstall: `vize lib pull ${family.canonicalName}`,
        })),
      },
      {
        id: "fresco",
        title: "Fresco",
        packageName: fresco.name,
        documentation: "https://github.com/ubugeeei-prod/vize/tree/main/docs/fresco",
        installation: { package: `pnpm add ${fresco.name}` },
        stability: fresco.metadata?.vize?.stability ?? "unknown",
        publicSubpaths: publicSubpaths("npm/fresco/package.json"),
        evidence: evidence.fresco,
      },
      {
        id: "patina",
        title: "Patina",
        packageName: "vize_patina",
        documentation: "/rules/",
        installation: { package: "cargo add vize_patina" },
        evidence: evidence.patina,
      },
      {
        id: "glyph",
        title: "Glyph",
        packageName: "vize_glyph",
        documentation: "/guide/formatter.html",
        installation: { package: "cargo add vize_glyph" },
        evidence: evidence.glyph,
      },
      {
        id: "levels",
        title: "Levels",
        packageName: "@vizejs/native",
        documentation: "https://github.com/ubugeeei-prod/vize/tree/main/docs/davinci",
        installation: { package: "pnpm add @vizejs/native" },
        evidence: evidence.levels,
      },
      {
        id: "devtools",
        title: "Devtools",
        packageName: devtools.name,
        documentation: "https://github.com/ubugeeei-prod/vize/tree/main/npm/devtools/README.md",
        installation: { package: `pnpm add ${devtools.name}` },
        stability: devtools.metadata?.vize?.stability ?? "unknown",
        publicSubpaths: publicSubpaths("npm/devtools/package.json"),
        evidence: evidence.devtools,
      },
      {
        id: "plugins",
        title: "Plugin SDK",
        packageName: plugins.name,
        documentation: "https://github.com/ubugeeei-prod/vize/tree/main/npm/plugin-sdk/README.md",
        installation: { package: `pnpm add ${plugins.name}` },
        stability: plugins.metadata?.vize?.stability ?? "unknown",
        publicSubpaths: publicSubpaths("npm/plugin-sdk/package.json"),
        evidence: evidence.plugins,
      },
    ],
    acceptance: {
      examples: {
        format: "vp check",
        lint: "vp check",
        typecheck: "docs UI preview and public API checks",
        render: "docs build and browser preview checks",
      },
      releaseEvidence: {
        workflow: ".github/workflows/release.yml",
        notesGenerator: "tools/support/release/github_release_notes.ts",
        source: "exact merged revision",
      },
    },
  } as const;
}

/** Stable pretty-printed wire representation used by docs builds and tests. */
export function renderDocumentationContractMetadata(): string {
  return `${JSON.stringify(documentationContractMetadata(), null, 2)}\n`;
}

/** Release-note section backed by the same catalog rendered by the docs build. */
export function documentationReleaseNote(): string {
  const metadata = documentationContractMetadata();
  const completionItems = metadata.surfaces.reduce(
    (total, surface) => total + ("completionItems" in surface ? surface.completionItems.length : 0),
    0,
  );
  return [
    "## Documentation",
    "",
    `The [machine-readable docs contract](https://vizejs.dev${metadata.endpoint}) publishes ${metadata.surfaces.length} product surfaces and ${completionItems} source-owned completion records from this release's verified catalogs.`,
  ].join("\n");
}

function main(): void {
  const output = resolve(root, "docs/public/vize-docs-metadata.json");
  writeFileSync(output, renderDocumentationContractMetadata());
  process.stdout.write(`wrote ${output}\n`);
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) main();
