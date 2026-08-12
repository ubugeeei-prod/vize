import fs from "node:fs";
import path from "node:path";

/**
 * Locates the manifest of the vue-tsc package a bin entry ultimately executes.
 * pnpm writes `node_modules/.bin/vue-tsc` either as a store symlink or as a
 * cmd-shim script naming its target only in the script body, and the version pin
 * must read the same manifest either way. Slicing the bin path breaks on the
 * shim — that path holds no `vue-tsc` segment — so follow the shim, then walk up
 * to the nearest manifest that declares vue-tsc. Anchoring on the manifest
 * `name` also keeps an enclosing workspace manifest from supplying the version.
 */
export function resolveVueTscManifestPath(binaryPath: string): string | undefined {
  for (const candidate of [readShimTarget(binaryPath), binaryPath]) {
    if (candidate == null) continue;
    let resolved: string;
    try {
      resolved = fs.realpathSync(candidate);
    } catch {
      continue;
    }
    for (let directory = path.dirname(resolved); ;) {
      const manifestPath = path.join(directory, "package.json");
      if (readPackageName(manifestPath) === "vue-tsc") return manifestPath;
      const parent = path.dirname(directory);
      if (parent === directory) break;
      directory = parent;
    }
  }
  return undefined;
}

/** Absolute target of a cmd-shim script, or undefined when not a shim. */
function readShimTarget(binaryPath: string): string | undefined {
  let content: string;
  try {
    content = fs.readFileSync(binaryPath, "utf8");
  } catch {
    return undefined;
  }
  const declared = /^#\s*cmd-shim-target=(.+)$/m.exec(content)?.[1]?.trim();
  if (declared != null && declared !== "") return declared;
  // Shims without that marker name the target relative to the bin directory, in
  // their `exec node "$basedir/../<pkg>/bin/<entry>"` line. Requiring the `../`
  // skips the `"$basedir/node"` interpreter argument on the same line.
  const relative = /"\$basedir\/(\.\.\/[^"]+)"/.exec(content)?.[1];
  return relative == null ? undefined : path.join(path.dirname(binaryPath), relative);
}

function readPackageName(manifestPath: string): string | undefined {
  try {
    const parsed = JSON.parse(fs.readFileSync(manifestPath, "utf8")) as { name?: unknown };
    return typeof parsed.name === "string" ? parsed.name : undefined;
  } catch {
    return undefined;
  }
}
