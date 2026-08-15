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
  elkRouteReadinessState,
  elkRouteMinElements,
} from "../app/dev/elk-route-contract.ts";
import { MOBILE_VIEWPORT, elkVisualRoutes } from "../app/vrt/elk-routes.ts";

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
    ["app/pages/[[server]]/explore.vue", "disabled: !isHydrated.value || !currentUser.value"],
    ["app/pages/share-target.vue", "if (!useAppConfig().pwaEnabled)"],
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
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(root, "tests/package.json"), "utf8"),
  ) as {
    scripts?: Record<string, string>;
  };

  assert.equal(ELK_RENDER_ROUTE, "/settings");
  assert.equal(ELK_RENDER_ROUTE_MIN_ELEMENTS, 100);
  assert.equal(ELK_DEFAULT_ROUTE_MIN_ELEMENTS, 60);

  // The visual suite opens the deterministic render route first (it doubles as
  // the warm-up route) and must never screenshot the non-deterministic root.
  assert.equal(elkVisualRoutes[0]?.path, ELK_RENDER_ROUTE);
  assert.deepEqual(
    elkVisualRoutes.filter((route) => route.path === "/"),
    [],
  );
  assert.deepEqual(
    elkVisualRoutes.filter((route) => route.path === ELK_RENDER_ROUTE).map((route) => route.name),
    ["settings-shell", "settings-shell-mobile"],
  );
  assert.deepEqual(
    elkVisualRoutes.filter((route) => route.viewport).map((route) => route.viewport),
    [MOBILE_VIEWPORT],
  );
  assert.deepEqual(
    elkVisualRoutes.filter((route) => route.path.startsWith("/share-target")),
    [],
  );
  assert.equal(new Set(elkVisualRoutes.map((route) => route.name)).size, elkVisualRoutes.length);

  assert.match(packageJson.scripts?.["test:dev:elk"] ?? "", /app\/dev\/elk\.spec\.ts/);
  assert.match(packageJson.scripts?.["test:dev:ci"] ?? "", /app\/dev\/elk\.spec\.ts/);
  assert.match(packageJson.scripts?.["test:vrt:elk"] ?? "", /app\/vrt\/elk\.spec\.ts/);
});

test("elk readiness gating consumes the route-specific links and thresholds", () => {
  const renderReady = {
    elementCount: ELK_RENDER_ROUTE_MIN_ELEMENTS,
    missingLinks: [],
    rootFound: true,
  };

  assert.equal(elkRouteReadinessState(ELK_RENDER_ROUTE, renderReady), "ready");
  assert.equal(
    elkRouteReadinessState(ELK_RENDER_ROUTE, {
      ...renderReady,
      elementCount: ELK_RENDER_ROUTE_MIN_ELEMENTS - 1,
    }),
    `incomplete:elements=${ELK_RENDER_ROUTE_MIN_ELEMENTS - 1}:missing=`,
  );
  assert.equal(
    elkRouteReadinessState(ELK_RENDER_ROUTE, {
      ...renderReady,
      missingLinks: [ELK_RENDER_ROUTE_LINKS[1]],
    }),
    `incomplete:elements=${ELK_RENDER_ROUTE_MIN_ELEMENTS}:missing=/settings/about`,
  );
  assert.equal(
    elkRouteReadinessState(ELK_RENDER_ROUTE, { ...renderReady, rootFound: false }),
    "missing-root",
  );

  // Non-settings routes stay on the bounded default threshold and never gate on
  // the pinned settings navigation links.
  for (const routePath of ["/public", "/settings/interface"]) {
    assert.equal(
      elkRouteReadinessState(routePath, {
        elementCount: ELK_DEFAULT_ROUTE_MIN_ELEMENTS,
        missingLinks: elkRequiredRouteLinks(routePath),
        rootFound: true,
      }),
      "ready",
    );
    assert.equal(
      elkRouteReadinessState(routePath, {
        elementCount: ELK_DEFAULT_ROUTE_MIN_ELEMENTS - 1,
        missingLinks: [],
        rootFound: true,
      }),
      `incomplete:elements=${ELK_DEFAULT_ROUTE_MIN_ELEMENTS - 1}:missing=`,
    );
  }

  // Every configured visual route resolves to a threshold the suite can reach,
  // and gates only on the links that route actually renders.
  for (const route of elkVisualRoutes) {
    const readiness = elkRouteReadinessExpectation(route.path);
    const missing = readiness.links.length === 0 ? undefined : readiness.links.join(",");
    assert.equal(
      elkRouteReadinessState(route.path, {
        elementCount: readiness.minElements,
        missingLinks: [],
        rootFound: true,
      }),
      "ready",
    );
    assert.equal(
      elkRouteReadinessState(route.path, {
        elementCount: readiness.minElements,
        missingLinks: readiness.links,
        rootFound: true,
      }),
      missing === undefined
        ? "ready"
        : `incomplete:elements=${readiness.minElements}:missing=${missing}`,
    );
  }
});

test("elk visual readiness requires route-specific stable links", () => {
  assert.deepEqual(ELK_RENDER_ROUTE_LINKS, ["/settings/interface", "/settings/about"]);
  assert.deepEqual(ELK_EXPLORE_ROUTE_LINKS, ["/explore/tags", "/explore/links"]);
  assert.deepEqual(elkRequiredRouteLinks(ELK_RENDER_ROUTE), [...ELK_RENDER_ROUTE_LINKS]);
  assert.deepEqual(elkRequiredRouteLinks("/explore"), [...ELK_EXPLORE_ROUTE_LINKS]);
  assert.ok(!elkRequiredRouteLinks("/explore").includes("/explore/users"));
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

  for (const routePath of ["/public", "/settings/interface"]) {
    assert.deepEqual(elkRequiredRouteLinks(routePath), []);
    assert.deepEqual(elkRouteReadinessExpectation(routePath), {
      links: [],
      minElements: ELK_DEFAULT_ROUTE_MIN_ELEMENTS,
    });
  }
});

test("elk visual readiness uses route-specific element thresholds", () => {
  assert.equal(elkRouteMinElements(ELK_RENDER_ROUTE), ELK_RENDER_ROUTE_MIN_ELEMENTS);

  for (const routePath of ["/explore", "/public", "/settings/interface"]) {
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
