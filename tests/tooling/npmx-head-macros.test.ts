import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  assertNpmxHeadMacroAnchors,
  NPMX_HEAD_SOURCE_CONTRACTS,
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
