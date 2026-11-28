import { packageScripts } from "./support/package-task-source.ts";
import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { test } from "node:test";

import {
  documentationReleaseNote,
  documentationContractMetadata,
  renderDocumentationContractMetadata,
} from "../../docs/scripts/generate-contract-metadata.ts";
import { COMPOSABLE_CATALOG } from "../../npm/compose/core/src/catalog.ts";
import { uiFamilyCatalog } from "../../npm/ui/src/catalog/family-catalog.ts";
import { documentationSourceContractTests } from "../../docs/scripts/check-source-contracts.ts";
import { jsrPackages } from "../../tools/support/release/jsr/packages.ts";
import { publishedCrates } from "../../tools/support/release/publish_crates/main.ts";
import { crateDocumentationInstallation } from "../../tools/support/documentation/crate-installation.ts";

void test("the published docs contract covers every required product surface", () => {
  const metadata = documentationContractMetadata();
  assert.equal(metadata.schemaVersion, 1);
  assert.equal(metadata.endpoint, "/vize-docs-metadata.json");
  assert.deepEqual(
    metadata.surfaces.map(({ id }) => id),
    ["compose", "ui", "fresco", "patina", "glyph", "davinci", "devtools", "plugins"],
  );

  for (const surface of metadata.surfaces) {
    assert.ok(surface.packageName, `${surface.id} package`);
    assert.ok(surface.documentation, `${surface.id} documentation`);
    assert.ok(surface.installation.package, `${surface.id} install command`);
    assert.ok(surface.evidence.length > 0, `${surface.id} evidence`);
    for (const path of surface.evidence) {
      assert.ok(existsSync(new URL(`../../${path}`, import.meta.url)), `${surface.id}: ${path}`);
    }
  }
});

void test("hover and completion records come from the exact UI and Compose catalogs", () => {
  const metadata = documentationContractMetadata();
  const compose = metadata.surfaces.find(({ id }) => id === "compose");
  const ui = metadata.surfaces.find(({ id }) => id === "ui");
  assert.ok(compose && "completionItems" in compose);
  assert.ok(ui && "completionItems" in ui);
  assert.deepEqual(
    compose.completionItems.map(({ label }) => label),
    COMPOSABLE_CATALOG.utilities.map(({ name }) => name),
  );
  assert.deepEqual(
    ui.completionItems.map(({ label }) => label),
    uiFamilyCatalog.map(({ title }) => title),
  );
  assert.equal(
    new Set(compose.completionItems.map(({ label }) => label)).size,
    compose.completionItems.length,
  );
  assert.equal(
    new Set(ui.completionItems.map(({ label }) => label)).size,
    ui.completionItems.length,
  );
});

void test("installation claims follow the release inventory and preserve workspace-only packages", () => {
  const metadata = documentationContractMetadata();
  for (const surface of metadata.surfaces) {
    if (!("ecosystem" in surface.installation) || surface.installation.ecosystem !== "npm")
      continue;
    const published = jsrPackages.some((entry) => entry.npmName === surface.packageName);
    assert.equal(surface.installation.distribution, published ? "registry" : "workspace");
    assert.equal(
      surface.installation.package,
      published
        ? `pnpm add ${surface.packageName}`
        : `pnpm add '${surface.packageName}@workspace:*'`,
    );
  }
  const devtools = metadata.surfaces.find(({ id }) => id === "devtools");
  assert.ok(devtools && "distribution" in devtools.installation);
  assert.equal(devtools.installation.distribution, "workspace");
  assert.match(
    readFileSync(new URL("../../npm/devtools/README.md", import.meta.url), "utf8"),
    /workspace-only and has not been published/u,
  );
});

void test("Rust installation claims follow crate release and manifest policy", () => {
  const metadata = documentationContractMetadata();
  for (const id of ["glyph", "patina"]) {
    const surface = metadata.surfaces.find((surface) => surface.id === id);
    assert.ok(surface && "ecosystem" in surface.installation);
    const installation = crateDocumentationInstallation(surface.packageName);
    assert.deepEqual(surface.installation, installation);
    assert.equal(installation.ecosystem, "cargo");
    assert.equal(
      installation.distribution,
      publishedCrates.includes(surface.packageName) ? "registry" : "workspace",
    );
  }
  assert.equal(crateDocumentationInstallation("vize_patina").package, "cargo add vize_patina");
  assert.deepEqual(crateDocumentationInstallation("vize_glyph"), {
    ecosystem: "cargo",
    distribution: "workspace",
    package: "cargo add vize_glyph --path ../vize/crates/vize_glyph",
    checkout: "../vize",
    sourcePath: "crates/vize_glyph",
  });
  const guide = readFileSync(
    new URL("../../docs/content/guide/formatter.md", import.meta.url),
    "utf8",
  );
  assert.ok(guide.includes(crateDocumentationInstallation("vize_glyph").package));
  assert.throws(() => crateDocumentationInstallation("../vize_glyph"), {
    message: "Invalid crate name: ../vize_glyph",
  });
  const index = publishedCrates.length;
  const inventory = publishedCrates as string[];
  inventory.push("vize_glyph");
  try {
    assert.throws(() => crateDocumentationInstallation("vize_glyph"), {
      message: "Release inventory contradicts crate publication policy: vize_glyph",
    });
  } finally {
    inventory.splice(index);
  }
});

void test("docs builds publish a deterministic machine-readable endpoint", () => {
  const rendered = renderDocumentationContractMetadata();
  assert.deepEqual(JSON.parse(rendered), documentationContractMetadata());
  assert.equal(rendered, renderDocumentationContractMetadata());
  assert.match(documentationReleaseNote(), /8 product surfaces and 489 source-owned completion/u);

  const packageJson = JSON.parse(
    readFileSync(new URL("../../docs/package.json", import.meta.url), "utf8"),
  ) as {
    scripts: Record<string, string>;
  };
  assert.equal(
    packageScripts("docs", packageJson.scripts)["generate:metadata"],
    "node ./scripts/generate-contract-metadata.ts",
  );
  for (const script of ["dev", "build"]) {
    const command = packageScripts("docs", packageJson.scripts)[script] ?? "";
    assert.match(command, /pnpm generate:metadata/u);
    assert.ok(command.startsWith("pnpm check:contracts && "), `${script}: check before generation`);
  }
  assert.equal(
    packageScripts("docs", packageJson.scripts)["check:contracts"],
    "node ./scripts/check-source-contracts.ts",
  );
  assert.deepEqual(
    documentationContractMetadata().acceptance.sourceContracts.tests,
    documentationSourceContractTests,
  );
  assert.match(
    readFileSync(new URL("../../docs/.gitignore", import.meta.url), "utf8"),
    /^public\/vize-docs-metadata\.json$/mu,
  );
});
