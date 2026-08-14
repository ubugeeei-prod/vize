import fs from "node:fs";
import path from "node:path";

export const ELK_RENDER_ROUTE = "/settings";

export const ELK_RENDER_ROUTE_LINKS = ["/settings/interface", "/settings/about"] as const;

export const ELK_EXPLORE_ROUTE_LINKS = [
  "/explore/users",
  "/explore/tags",
  "/explore/links",
] as const;

export const ELK_RENDER_ROUTE_MIN_ELEMENTS = 100;

export const ELK_DEFAULT_ROUTE_MIN_ELEMENTS = 60;

// Only the deterministic render route ships the pinned settings navigation, so
// other routes must not gate readiness on those links.
export function elkRequiredRouteLinks(routePath: string): string[] {
  const pathname = elkRoutePathname(routePath);
  if (pathname === ELK_RENDER_ROUTE) return [...ELK_RENDER_ROUTE_LINKS];
  if (pathname === "/explore") return [...ELK_EXPLORE_ROUTE_LINKS];
  return [];
}

export function elkRouteMinElements(routePath: string): number {
  return elkRoutePathname(routePath) === ELK_RENDER_ROUTE
    ? ELK_RENDER_ROUTE_MIN_ELEMENTS
    : ELK_DEFAULT_ROUTE_MIN_ELEMENTS;
}

export interface ElkRouteReadinessExpectation {
  links: string[];
  minElements: number;
}

export function elkRouteReadinessExpectation(routePath: string): ElkRouteReadinessExpectation {
  return {
    links: elkRequiredRouteLinks(routePath),
    minElements: elkRouteMinElements(routePath),
  };
}

export interface ElkRouteObservation {
  elementCount: number;
  missingLinks: readonly string[];
  rootFound: boolean;
}

// Shared by the dev and visual specs so readiness gating is a single behavior
// that can be exercised without a browser.
export function elkRouteReadinessState(
  routePath: string,
  observation: ElkRouteObservation,
): string {
  if (!observation.rootFound) {
    return "missing-root";
  }

  const { minElements } = elkRouteReadinessExpectation(routePath);
  if (observation.elementCount >= minElements && observation.missingLinks.length === 0) {
    return "ready";
  }

  const missing = observation.missingLinks.join(",");
  return `incomplete:elements=${observation.elementCount}:missing=${missing}`;
}

function elkRoutePathname(routePath: string): string {
  return routePath.split("?", 1)[0] || "/";
}

export const ELK_RENDER_ROUTE_SOURCE_CONTRACTS = {
  "app/pages/index.vue": {
    description: "root route is an empty auth middleware handoff",
    anchors: ["middleware: 'auth'", "<template>\n  <div />\n</template>"],
  },
  "app/pages/settings.vue": {
    description: "render route exposes the stable settings navigation shell",
    anchors: ["wideLayout: true", "<SettingsItem", 'to="/settings/about"', "<NuxtPage"],
  },
  "app/layouts/default.vue": {
    description: "render route exercises the normal Elk layout/navigation shell",
    anchors: ["<NavSide command", "<slot />", "<NavBottom"],
  },
} as const satisfies Record<string, { description: string; anchors: readonly string[] }>;

export interface ElkRenderRouteSourceEvidence {
  files: Array<{
    relativePath: string;
    sizeBytes: number;
  }>;
}

export function assertElkRenderRouteAnchors(sources: Record<string, string>): void {
  for (const [relativePath, contract] of Object.entries(ELK_RENDER_ROUTE_SOURCE_CONTRACTS)) {
    const source = sources[relativePath];
    if (source == null) {
      throw new Error(`missing Elk render route source: ${relativePath}`);
    }

    for (const anchor of contract.anchors) {
      if (!source.includes(anchor)) {
        throw new Error(
          `missing Elk render route anchor in ${relativePath}: ${anchor} (${contract.description})`,
        );
      }
    }
  }
}

export function readElkRenderRouteSourceEvidence(
  fixtureRoot: string,
): ElkRenderRouteSourceEvidence {
  const sources: Record<string, string> = {};
  const files: ElkRenderRouteSourceEvidence["files"] = [];

  for (const relativePath of Object.keys(ELK_RENDER_ROUTE_SOURCE_CONTRACTS).sort()) {
    const source = fs.readFileSync(path.join(fixtureRoot, relativePath), "utf8");
    sources[relativePath] = source;
    files.push({
      relativePath,
      sizeBytes: Buffer.byteLength(source),
    });
  }

  assertElkRenderRouteAnchors(sources);

  return { files };
}
