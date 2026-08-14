import fs from "node:fs";
import path from "node:path";

export const ELK_RENDER_ROUTE = "/settings";

export const ELK_RENDER_ROUTE_LINKS = ["/settings/interface", "/settings/about"] as const;

// Only the deterministic render route ships the pinned settings navigation, so
// other routes must not gate readiness on those links.
export function elkRequiredRouteLinks(routePath: string): string[] {
  return routePath === ELK_RENDER_ROUTE ? [...ELK_RENDER_ROUTE_LINKS] : [];
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
