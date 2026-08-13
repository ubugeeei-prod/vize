import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  assertElkRenderRouteAnchors,
  ELK_RENDER_ROUTE,
  ELK_RENDER_ROUTE_SOURCE_CONTRACTS,
} from "../app/dev/elk-route-contract.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");

function validSyntheticSources(): Record<string, string> {
  return Object.fromEntries(
    Object.entries(ELK_RENDER_ROUTE_SOURCE_CONTRACTS).map(([relativePath, contract]) => [
      relativePath,
      contract.anchors.join("\n"),
    ]),
  );
}

test("elk render route source contract fails closed for each fixture anchor", () => {
  assert.doesNotThrow(() => assertElkRenderRouteAnchors(validSyntheticSources()));

  for (const [relativePath, anchor] of [
    ["app/pages/index.vue", "middleware: 'auth'"],
    ["app/pages/settings/about/index.vue", 'text="GitHub"'],
    ["app/layouts/default.vue", "<NavSide command"],
  ] as const) {
    const sources = validSyntheticSources();
    sources[relativePath] = sources[relativePath]!.replace(anchor, "removed");
    assert.throws(
      () => assertElkRenderRouteAnchors(sources),
      new RegExp(`missing Elk render route anchor.*${escapeRegExp(anchor)}`),
    );
  }
});

test("elk dev app-e2e is wired to the deterministic rendered fixture route", () => {
  const spec = fs.readFileSync(path.join(root, "tests/app/dev/elk.spec.ts"), "utf8");
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(root, "tests/package.json"), "utf8"),
  ) as {
    scripts?: Record<string, string>;
  };

  assert.equal(ELK_RENDER_ROUTE, "/settings/about");
  assert.match(spec, /readElkRenderRouteSourceEvidence\(app\.cwd\)/);
  assert.match(spec, /await expect\(mountEl\)\.toContainText\("GitHub"\)/);
  assert.doesNotMatch(spec, /\b(?:warmupPage|page)\.goto\(app\.url\b/);
  assert.doesNotMatch(spec, /verifySSRContent\(page,\s*app\.url\)/);
  assert.match(packageJson.scripts?.["test:dev:elk"] ?? "", /app\/dev\/elk\.spec\.ts/);
  assert.match(packageJson.scripts?.["test:dev:ci"] ?? "", /app\/dev\/elk\.spec\.ts/);
});

function escapeRegExp(source: string): string {
  return source.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
