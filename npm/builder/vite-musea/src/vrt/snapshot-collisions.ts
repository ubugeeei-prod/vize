import type { ArtFileInfo, ViewportConfig } from "../types/index.js";
import { buildSnapshotName } from "./utils.js";

interface SnapshotJob {
  art: ArtFileInfo;
  variantName: string;
  viewport: ViewportConfig;
}

/** Validate the whole batch before any worker can write a shared PNG. */
export function assertUniqueSnapshotNames(jobs: readonly SnapshotJob[]): void {
  const owners = new Map<string, string>();
  for (const { art, variantName, viewport } of jobs) {
    const name = buildSnapshotName(art.path, variantName, viewport);
    const owner = `${art.path} / ${variantName} / ${viewport.width}x${viewport.height}`;
    const previous = owners.get(name);
    if (previous !== undefined) {
      throw new Error(
        `Snapshot name collision for "${name}": ${previous}; ${owner}. ` +
          "Use distinct Art basenames, variant names, or viewport names before capturing.",
      );
    }
    owners.set(name, owner);
  }
}
