import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  assertNpmxHeadMacroAnchors,
  NPMX_HEAD_SOURCE_CONTRACTS,
  readNpmxHeadFixtureContent,
} from "../app/dev/npmx-head-contract.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function validSyntheticSources(): Record<string, string> {
  return Object.fromEntries(
    Object.entries(NPMX_HEAD_SOURCE_CONTRACTS).map(([relativePath, contract]) => [
      relativePath,
      contract.anchors.join("\n"),
    ]),
  );
}

test("npmx authored head macro contracts fail closed for every macro family", () => {
  assert.doesNotThrow(() => assertNpmxHeadMacroAnchors(validSyntheticSources()));

  for (const [relativePath, anchor] of [
    ["app/pages/package-docs/[...path].vue", "definePageMeta({"],
    ["app/app.vue", "useHead({"],
    ["app/pages/about.vue", "useSeoMeta({"],
  ] as const) {
    const sources = validSyntheticSources();
    sources[relativePath] = sources[relativePath]!.replace(anchor, "disabledMacro({");
    assert.throws(
      () => assertNpmxHeadMacroAnchors(sources),
      new RegExp(`missing npmx head macro anchor.*${anchor.replace(/[(){}]/g, "\\$&")}`),
    );
  }
});

test("npmx head fixture content comes from the pinned locale file", (t) => {
  const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-npmx-head-"));
  t.after(() => fs.rmSync(fixtureRoot, { recursive: true, force: true }));

  const localeDir = path.join(fixtureRoot, "i18n/locales");
  fs.mkdirSync(localeDir, { recursive: true });
  fs.writeFileSync(
    path.join(localeDir, "en.json"),
    JSON.stringify({
      about: {
        meta_description: "A current fixture description.",
        title: "Fixture About",
      },
      a11y: {
        title: "fixture accessibility",
        welcome: "We want {app} to stay source-owned.",
      },
      package: {
        docs: {
          og_title: "{name} - Fixture Docs",
          page_title_version: "{name} fixture docs - npmx",
        },
      },
    }),
  );

  assert.deepEqual(readNpmxHeadFixtureContent(fixtureRoot), {
    aboutDescription: "A current fixture description.",
    aboutTitle: "Fixture About",
    accessibilityDescription: "We want npmx to stay source-owned.",
    accessibilityTitle: "fixture accessibility",
    docsOgTitle: "nuxt - Fixture Docs",
    docsTitle: "nuxt@4.0.0 fixture docs - npmx",
  });
});

test("npmx head runtime oracle is wired and cannot mutate authored sources", () => {
  const spec = fs.readFileSync(path.join(root, "tests/app/dev/npmx.spec.ts"), "utf8");
  const runtimeOracle = fs.readFileSync(
    path.join(root, "tests/app/dev/npmx-head-macros.ts"),
    "utf8",
  );
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(root, "tests/package.json"), "utf8"),
  ) as {
    scripts?: Record<string, string>;
  };

  assert.match(spec, /verifyNpmxHeadMacros\(page, app\.url, app\.cwd\)/);
  assert.match(packageJson.scripts?.["test:dev:npmx"] ?? "", /app\/dev\/npmx\.spec\.ts/);
  assert.match(packageJson.scripts?.["test:dev:ci"] ?? "", /app\/dev\/npmx\.spec\.ts/);
  assert.match(runtimeOracle, /readNpmxHeadFixtureContent\(fixtureRoot\)/);
  assert.doesNotMatch(runtimeOracle, /better UX\/DX/);
  assert.doesNotMatch(
    runtimeOracle,
    /\b(?:appendFile|copyFile|rename|rm|truncate|unlink|writeFile)(?:Sync)?\b/,
    "the SEO runtime oracle must remain read-only instead of relying on cleanup",
  );
  for (const evidence of [
    "SSR request failed",
    "package-docs/nuxt/v/4.0.0",
    "package/docs/nuxt/v/4.0.0",
    "docs/nuxt/v/4.0.0",
    "https://npmx.dev/package/vue",
    "readNpmxHeadSourceEvidence(fixtureRoot)",
  ]) {
    assert.ok(runtimeOracle.includes(evidence), `missing runtime evidence: ${evidence}`);
  }
});
