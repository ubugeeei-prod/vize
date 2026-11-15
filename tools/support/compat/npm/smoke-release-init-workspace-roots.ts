import assert from "node:assert/strict";

import { PACKAGE_MANAGERS, type ProjectManifest } from "./smoke-release-init-managers.ts";

export const WORKSPACE_PACKAGE_MANAGERS = ["pnpm", "bun"] as const;
export type WorkspacePackageManager = (typeof WORKSPACE_PACKAGE_MANAGERS)[number];

/** Materializes one workspace root without changing the historical pnpm bytes. */
export function workspaceRootFiles(
  managerId: WorkspacePackageManager,
  redirects: Readonly<Record<string, string>>,
): Record<string, string> {
  const manager = PACKAGE_MANAGERS[managerId];
  const manifest: ProjectManifest = {
    name: managerId === "pnpm" ? "vize-fresh-workspace" : "vize-fresh-bun-workspace",
    private: true,
    ...(managerId === "bun" ? { workspaces: ["packages/*"] } : {}),
  };
  const redirected = { ...manager.redirect(manifest, redirects) };
  if (managerId === "pnpm") {
    const overrides = redirected["pnpm-workspace.yaml"];
    assert.equal(typeof overrides, "string", "pnpm workspace overrides are missing");
    redirected["pnpm-workspace.yaml"] = ["packages:", '  - "packages/*"', overrides].join("\n");
  }
  return {
    "package.json": `${JSON.stringify(manifest, null, 2)}\n`,
    ...redirected,
  };
}
