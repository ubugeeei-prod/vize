import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { applyElkRuntimePnpmOverrides } from "../_helpers/apps.ts";
import {
  assertElkRenderRouteAnchors,
  ELK_DEFAULT_ROUTE_MIN_ELEMENTS,
  ELK_EXPLORE_ROUTE_LINKS,
  ELK_RENDER_ROUTE,
  ELK_RENDER_ROUTE_LINKS,
  ELK_RENDER_ROUTE_MIN_ELEMENTS,
  ELK_RENDER_ROUTE_SOURCE_CONTRACTS,
  elkRequiredRouteLinks,
  elkRouteReadinessExpectation,
  elkRouteMinElements,
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
  assert.equal(ELK_RENDER_ROUTE_MIN_ELEMENTS, 100);
  assert.equal(ELK_DEFAULT_ROUTE_MIN_ELEMENTS, 60);
  assert.match(devSpec, /readElkRenderRouteSourceEvidence\(app\.cwd\)/);
  assert.match(
    devSpec,
    /const ELK_RENDER_READINESS = elkRouteReadinessExpectation\(ELK_RENDER_ROUTE\);/,
  );
  assert.match(devSpec, /links: ELK_RENDER_READINESS\.links/);
  assert.match(devSpec, /minElements: ELK_RENDER_READINESS\.minElements/);
  assert.doesNotMatch(devSpec, /\b(?:warmupPage|page)\.goto\(app\.url\b/);
  assert.doesNotMatch(devSpec, /verifySSRContent\(page,\s*app\.url\)/);
  assert.match(visualSpec, /readElkRenderRouteSourceEvidence\(app\.cwd\)/);
  assert.match(visualSpec, /path: ELK_RENDER_ROUTE/);
  assert.match(visualSpec, /const readiness = elkRouteReadinessExpectation\(route\.path\);/);
  assert.match(
    visualSpec,
    /elkRouteContentState\(page, readiness\.links, readiness\.minElements\)/,
  );
  assert.doesNotMatch(visualSpec, /path: "\/"(?:[,}])/);
  assert.match(packageJson.scripts?.["test:dev:elk"] ?? "", /app\/dev\/elk\.spec\.ts/);
  assert.match(packageJson.scripts?.["test:dev:ci"] ?? "", /app\/dev\/elk\.spec\.ts/);
  assert.match(packageJson.scripts?.["test:vrt:elk"] ?? "", /app\/vrt\/elk\.spec\.ts/);
});

test("elk visual readiness requires route-specific stable links", () => {
  assert.deepEqual(ELK_RENDER_ROUTE_LINKS, ["/settings/interface", "/settings/about"]);
  assert.deepEqual(ELK_EXPLORE_ROUTE_LINKS, ["/explore/users", "/explore/tags", "/explore/links"]);
  assert.deepEqual(elkRequiredRouteLinks(ELK_RENDER_ROUTE), [...ELK_RENDER_ROUTE_LINKS]);
  assert.deepEqual(elkRequiredRouteLinks("/explore"), [...ELK_EXPLORE_ROUTE_LINKS]);
  assert.deepEqual(elkRouteReadinessExpectation(ELK_RENDER_ROUTE), {
    links: [...ELK_RENDER_ROUTE_LINKS],
    minElements: ELK_RENDER_ROUTE_MIN_ELEMENTS,
  });
  assert.deepEqual(elkRouteReadinessExpectation("/settings?tab=interface"), {
    links: [...ELK_RENDER_ROUTE_LINKS],
    minElements: ELK_RENDER_ROUTE_MIN_ELEMENTS,
  });
  assert.deepEqual(elkRouteReadinessExpectation("/explore"), {
    links: [...ELK_EXPLORE_ROUTE_LINKS],
    minElements: ELK_DEFAULT_ROUTE_MIN_ELEMENTS,
  });
  assert.deepEqual(elkRouteReadinessExpectation("/explore?tab=users"), {
    links: [...ELK_EXPLORE_ROUTE_LINKS],
    minElements: ELK_DEFAULT_ROUTE_MIN_ELEMENTS,
  });

  for (const routePath of ["/public", "/settings/interface", "/share-target?text=hi"]) {
    assert.deepEqual(elkRequiredRouteLinks(routePath), []);
    assert.deepEqual(elkRouteReadinessExpectation(routePath), {
      links: [],
      minElements: ELK_DEFAULT_ROUTE_MIN_ELEMENTS,
    });
  }
});

test("elk visual readiness uses route-specific element thresholds", () => {
  assert.equal(elkRouteMinElements(ELK_RENDER_ROUTE), ELK_RENDER_ROUTE_MIN_ELEMENTS);

  for (const routePath of ["/explore", "/public", "/settings/interface", "/share-target?text=hi"]) {
    assert.equal(elkRouteMinElements(routePath), ELK_DEFAULT_ROUTE_MIN_ELEMENTS);
  }
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
