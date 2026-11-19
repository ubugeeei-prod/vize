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

void test("the published docs contract covers every required product surface", () => {
  const metadata = documentationContractMetadata();
  assert.equal(metadata.schemaVersion, 1);
  assert.equal(metadata.endpoint, "/vize-docs-metadata.json");
  assert.deepEqual(
    metadata.surfaces.map(({ id }) => id),
    ["compose", "ui", "fresco", "patina", "glyph", "levels", "devtools", "plugins"],
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

void test("docs builds publish a deterministic machine-readable endpoint", () => {
  const rendered = renderDocumentationContractMetadata();
  assert.deepEqual(JSON.parse(rendered), documentationContractMetadata());
  assert.equal(rendered, renderDocumentationContractMetadata());
  assert.match(documentationReleaseNote(), /8 product surfaces and 480 source-owned completion/u);

  const packageJson = JSON.parse(
    readFileSync(new URL("../../docs/package.json", import.meta.url), "utf8"),
  ) as {
    scripts: Record<string, string>;
  };
  assert.equal(
    packageJson.scripts["generate:metadata"],
    "node ./scripts/generate-contract-metadata.ts",
  );
  for (const script of ["dev", "build"]) {
    assert.match(packageJson.scripts[script] ?? "", /pnpm generate:metadata/u);
  }
  assert.match(
    readFileSync(new URL("../../docs/.gitignore", import.meta.url), "utf8"),
    /^public\/vize-docs-metadata\.json$/mu,
  );
});
