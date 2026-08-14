import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { applyElkRuntimePnpmOverrides } from "../_helpers/apps.ts";
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
    ["app/pages/settings.vue", 'to="/settings/about"'],
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

test("elk dev and visual app-e2e are wired to the deterministic rendered fixture route", () => {
  const devSpec = fs.readFileSync(path.join(root, "tests/app/dev/elk.spec.ts"), "utf8");
  const visualSpec = fs.readFileSync(path.join(root, "tests/app/vrt/elk.spec.ts"), "utf8");
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(root, "tests/package.json"), "utf8"),
  ) as {
    scripts?: Record<string, string>;
  };

  assert.equal(ELK_RENDER_ROUTE, "/settings");
  assert.match(devSpec, /readElkRenderRouteSourceEvidence\(app\.cwd\)/);
  assert.match(devSpec, /ELK_MIN_RENDER_ROUTE_ELEMENTS = 100/);
  assert.match(
    devSpec,
    /ELK_RENDER_ROUTE_LINKS = \["\/settings\/interface", "\/settings\/about"\]/,
  );
  assert.doesNotMatch(devSpec, /\b(?:warmupPage|page)\.goto\(app\.url\b/);
  assert.doesNotMatch(devSpec, /verifySSRContent\(page,\s*app\.url\)/);
  assert.match(visualSpec, /readElkRenderRouteSourceEvidence\(app\.cwd\)/);
  assert.match(visualSpec, /path: ELK_RENDER_ROUTE/);
  assert.match(visualSpec, /ELK_MIN_RENDER_ROUTE_ELEMENTS = 100/);
  assert.doesNotMatch(visualSpec, /path: "\/"(?:[,}])/);
  assert.match(packageJson.scripts?.["test:dev:elk"] ?? "", /app\/dev\/elk\.spec\.ts/);
  assert.match(packageJson.scripts?.["test:dev:ci"] ?? "", /app\/dev\/elk\.spec\.ts/);
  assert.match(packageJson.scripts?.["test:vrt:elk"] ?? "", /app\/vrt\/elk\.spec\.ts/);
});

test("elk setup pins runtime overrides in the generated package manifest", (t) => {
  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-elk-overrides-"));
  t.after(() => fs.rmSync(tempDir, { recursive: true, force: true }));

  const packageJsonPath = path.join(tempDir, "package.json");
  fs.writeFileSync(
    packageJsonPath,
    JSON.stringify(
      {
        name: "elk",
        pnpm: {
          overrides: {
            "@existing/package": "1.0.0",
            vite: "^7.0.0",
          },
        },
      },
      null,
      2,
    ) + "\n",
  );

  applyElkRuntimePnpmOverrides(packageJsonPath);

  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, "utf8")) as {
    pnpm?: { overrides?: Record<string, string> };
  };
  assert.deepEqual(packageJson.pnpm?.overrides, {
    "@existing/package": "1.0.0",
    "@nuxtjs/i18n": "10.1.0",
    vite: "^8.0.0",
  });
});

function escapeRegExp(source: string): string {
  return source.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
