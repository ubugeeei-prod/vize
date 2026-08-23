import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { isolateUniqueLocalTypePackages } from "../../tools/fixtures/typecheck-baseline-isolation-unique.mjs";
import { pluginPackageNamesFromConfigs } from "../../tools/fixtures/typecheck-baseline-isolation-plugins.mjs";

/**
 * Some Vue language-plugin configs use the TypeScript `{ name }` object shape
 * instead of a string or `[name, options]` tuple (#4461). Unique isolation
 * must still extract the package and link the fixture copy.
 */

function scaffold() {
  const outer = fs.realpathSync(
    fs.mkdtempSync(path.join(os.tmpdir(), "vize-isolation-vue-plugin-objects-")),
  );
  const fixtureRoot = path.join(outer, "fixture");
  fs.mkdirSync(fixtureRoot, { recursive: true });
  const ancestor = path.join(outer, "node_modules", "@vue", "language-plugin-pug");
  fs.mkdirSync(ancestor, { recursive: true });
  fs.writeFileSync(path.join(ancestor, "package.json"), `{"name":"@vue/language-plugin-pug"}\n`);
  return { fixtureRoot, outer };
}

function writeStoreCopy(fixtureRoot: string) {
  const packageRoot = path.join(
    fixtureRoot,
    "node_modules",
    ".pnpm",
    "@vue+language-plugin-pug@1.8.27",
    "node_modules",
    "@vue",
    "language-plugin-pug",
  );
  fs.mkdirSync(packageRoot, { recursive: true });
  fs.writeFileSync(path.join(packageRoot, "package.json"), `{"name":"@vue/language-plugin-pug"}\n`);
}

test("vueCompilerOptions.plugins objects contribute the name field", () => {
  assert.deepEqual(
    pluginPackageNamesFromConfigs([
      {
        vueCompilerOptions: {
          plugins: [
            { name: "@vue/language-plugin-pug" },
            { name: "../node_modules/@vue/language-plugin-pug" },
          ],
        },
      },
    ]),
    ["@vue/language-plugin-pug"],
  );
});

test("unique isolation links a vue plugin object with a relative node_modules name", () => {
  const { fixtureRoot, outer } = scaffold();
  try {
    writeStoreCopy(fixtureRoot);
    const configPath = path.join(fixtureRoot, ".nuxt", "tsconfig.json");
    fs.mkdirSync(path.dirname(configPath), { recursive: true });
    fs.writeFileSync(
      configPath,
      `${JSON.stringify({
        vueCompilerOptions: {
          plugins: [{ name: "../node_modules/@vue/language-plugin-pug" }],
        },
      })}\n`,
    );
    assert.deepEqual(isolateUniqueLocalTypePackages(fixtureRoot, configPath), [
      {
        name: "@vue/language-plugin-pug",
        target:
          "node_modules/.pnpm/@vue+language-plugin-pug@1.8.27/node_modules/@vue/language-plugin-pug",
      },
    ]);
  } finally {
    fs.rmSync(outer, { recursive: true, force: true });
  }
});
