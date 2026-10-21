import * as fs from "fs";
import * as path from "path";

export interface ServerCachePruneResult {
  removed: string[];
  failed: Array<{ version: string; error: string }>;
}

/**
 * Keep the current downloaded server and the newest previous version for
 * rollback. Non-directory entries are never touched.
 */
export async function pruneDownloadedServerVersions(
  serversDir: string,
  currentVersion: string,
): Promise<ServerCachePruneResult> {
  let entries: fs.Dirent[];
  try {
    entries = await fs.promises.readdir(serversDir, { withFileTypes: true });
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code === "ENOENT") {
      return { removed: [], failed: [] };
    }
    throw error;
  }

  const previous = await Promise.all(
    entries
      .filter((entry) => entry.isDirectory() && entry.name !== currentVersion)
      .map(async (entry) => ({
        version: entry.name,
        mtimeMs: (await fs.promises.stat(path.join(serversDir, entry.name))).mtimeMs,
      })),
  );
  previous.sort(
    (left, right) => right.mtimeMs - left.mtimeMs || left.version.localeCompare(right.version),
  );

  const result: ServerCachePruneResult = { removed: [], failed: [] };
  for (const entry of previous.slice(1)) {
    try {
      await fs.promises.rm(path.join(serversDir, entry.version), {
        force: true,
        recursive: true,
      });
      result.removed.push(entry.version);
    } catch (error) {
      result.failed.push({ version: entry.version, error: String(error) });
    }
  }
  return result;
}
