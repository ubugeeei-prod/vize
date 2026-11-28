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

import { documentationSourceContractTests } from "./check-source-contracts.ts";
import { jsrPackages } from "../../tools/support/release/jsr/packages.ts";
import { crateDocumentationInstallation } from "../../tools/support/documentation/crate-installation.ts";

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

function npmInstallation(packageName: string) {
  const published = jsrPackages.some((entry) => entry.npmName === packageName);
  return {
    ecosystem: "npm",
    distribution: published ? "registry" : "workspace",
    package: published ? `pnpm add ${packageName}` : `pnpm add '${packageName}@workspace:*'`,
  } as const;
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
    "npm/fresco/package.json",
    "npm/fresco/src/index.ts",
    "docs/fresco/compatibility.md",
    "tests/tooling/docs-public-api.test.ts",
    "tests/tooling/fresco-public-docs.test.ts",
    "tests/tooling/fresco-prop-defaults.test.ts",
    "tests/tooling/fresco-testing-node.test.ts",
  ],
  patina: [
    "docs/scripts/generate-patina-rules-page.ts",
    "docs/content/rules/index.md",
    ".github/workflows/pr-rust-checks.yml",
    "crates/vize_patina/Cargo.toml",
    "tools/support/documentation/crate-installation.ts",
    "tools/support/release/publish_crates/main.ts",
  ],
  glyph: [
    "docs/content/guide/formatter.md",
    ".github/workflows/pr-rust-checks.yml",
    "crates/vize_glyph/src/lib.rs",
    "crates/vize_glyph/Cargo.toml",
    "tools/support/documentation/crate-installation.ts",
    "tools/support/release/publish_crates/main.ts",
  ],
  davinci: [
    "docs/davinci/README.md",
    "tests/tooling/compiled-docs-inputs.test.ts",
    "docs/davinci/plan/budgets.toml",
  ],
  devtools: [
    "npm/devtools/README.md",
    "tests/tooling/docs-public-api.test.ts",
    "tools/support/release/jsr/packages.ts",
    "tests/tooling/docs-plugin-examples.test.ts",
    "tests/tooling/davinci-plugin-doc-examples.test.ts",
  ],
  plugins: [
    "npm/plugin-sdk/README.md",
    "tests/tooling/docs-public-api.test.ts",
    "tests/tooling/docs-plugin-examples.test.ts",
    "tests/tooling/davinci-plugin-doc-examples.test.ts",
  ],
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
        installation: npmInstallation(fresco.name),
        stability: fresco.metadata?.vize?.stability ?? "unknown",
        publicSubpaths: publicSubpaths("npm/fresco/package.json"),
        evidence: evidence.fresco,
      },
      {
        id: "patina",
        title: "Patina",
        packageName: "vize_patina",
        documentation: "/rules/",
        installation: crateDocumentationInstallation("vize_patina"),
        evidence: evidence.patina,
      },
      {
        id: "glyph",
        title: "Glyph",
        packageName: "vize_glyph",
        documentation: "/guide/formatter.html",
        installation: crateDocumentationInstallation("vize_glyph"),
        evidence: evidence.glyph,
      },
      {
        id: "davinci",
        title: "Davinci",
        packageName: "@vizejs/native",
        documentation: "https://github.com/ubugeeei-prod/vize/tree/main/docs/davinci",
        installation: { package: "pnpm add @vizejs/native" },
        evidence: evidence.davinci,
      },
      {
        id: "devtools",
        title: "Devtools",
        packageName: devtools.name,
        documentation: "https://github.com/ubugeeei-prod/vize/tree/main/npm/devtools/README.md",
        installation: npmInstallation(devtools.name),
        stability: devtools.metadata?.vize?.stability ?? "unknown",
        publicSubpaths: publicSubpaths("npm/devtools/package.json"),
        evidence: evidence.devtools,
      },
      {
        id: "plugins",
        title: "Plugin SDK",
        packageName: plugins.name,
        documentation: "https://github.com/ubugeeei-prod/vize/tree/main/npm/plugin-sdk/README.md",
        installation: npmInstallation(plugins.name),
        stability: plugins.metadata?.vize?.stability ?? "unknown",
        publicSubpaths: publicSubpaths("npm/plugin-sdk/package.json"),
        evidence: evidence.plugins,
      },
    ],
    acceptance: {
      sourceContracts: {
        command: "pnpm --dir docs check:contracts",
        tests: documentationSourceContractTests,
      },
      examples: {
        format: "vp check",
        lint: "vp check",
        typecheck: "docs UI preview and public API checks",
        render: "docs build and browser preview checks",
        rust: "cargo test --locked --profile ci -p vize_glyph -p vize_patina --doc",
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
