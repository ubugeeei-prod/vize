import assert from "node:assert/strict";
import { readdir } from "node:fs/promises";
import path from "node:path";

import { test } from "vite-plus/test";
import { createSSRApp, defineComponent } from "vue";
import { renderToString } from "vue/server-renderer";

import {
  controlRuntimeFixtures,
  type RuntimeFixture,
} from "./runtime-conformance-fixtures.ts";
import { overlayRuntimeFixtures } from "./runtime-conformance-overlay-fixtures.ts";

const runtimeFixtures: readonly RuntimeFixture[] = [
  ...controlRuntimeFixtures,
  ...overlayRuntimeFixtures,
];

/** Recursively collect canonical SFC paths relative to the package source root. */
async function collectSourceSfcFiles(
  directory: string,
  relativeDirectory = "",
): Promise<readonly string[]> {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = await Promise.all(
    entries.map(async (entry): Promise<readonly string[]> => {
      const entryPath = path.join(directory, entry.name);
      const relativePath = path.join(relativeDirectory, entry.name);
      if (entry.isDirectory()) return collectSourceSfcFiles(entryPath, relativePath);
      return entry.isFile() && entry.name.endsWith(".vue") ? [relativePath] : [];
    }),
  );
  return files.flat().sort((left, right) => left.localeCompare(right));
}

/** Create an isolated application root for one renderer request. */
function createFixtureRoot(fixture: RuntimeFixture) {
  return defineComponent({
    name: `RuntimeConformance${fixture.name}`,
    setup: () => fixture.render,
  });
}

/** Render with a fresh SSR application so request-local state cannot be reused. */
async function renderFixture(fixture: RuntimeFixture): Promise<string> {
  return renderToString(createSSRApp(createFixtureRoot(fixture)));
}

test("declares an SSR and hydration fixture for every source SFC", async () => {
  const sourceFiles = await collectSourceSfcFiles(path.resolve("src"));
  const fixtureFiles = runtimeFixtures
    .map((fixture) => fixture.sourceFile)
    .sort((left, right) => left.localeCompare(right));

  assert.deepEqual(fixtureFiles, sourceFiles);
});

test("renders stable, accessible markup across isolated SSR requests", async () => {
  for (const fixture of runtimeFixtures) {
    const [left, right] = await Promise.all([renderFixture(fixture), renderFixture(fixture)]);
    assert.equal(left, right, `${fixture.name} emitted request-dependent SSR markup`);
    fixture.assertServerMarkup(left);
  }
});

test("hydrates every shipped component without warnings or node replacement", async () => {
  for (const fixture of runtimeFixtures) {
    const serverHtml = await renderFixture(fixture);
    const host = document.createElement("div");
    host.innerHTML = serverHtml;
    document.body.append(host);
    const serverRoot = host.firstElementChild;
    assert.ok(serverRoot, `${fixture.name} did not emit a root element`);

    const diagnostics: string[] = [];
    const originalWarn = console.warn;
    const originalError = console.error;
    console.warn = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
    console.error = (...values: unknown[]) => diagnostics.push(values.map(String).join(" "));
    const app = createSSRApp(createFixtureRoot(fixture));
    let mounted = false;

    try {
      app.mount(host);
      mounted = true;
      assert.ok(
        host.firstElementChild === serverRoot,
        `${fixture.name} replaced its server-rendered root during hydration`,
      );
      assert.deepEqual(diagnostics, [], `${fixture.name} emitted hydration diagnostics`);
      fixture.assertHydratedDom(host);
    } finally {
      if (mounted) app.unmount();
      host.remove();
      console.warn = originalWarn;
      console.error = originalError;
    }
  }
});
