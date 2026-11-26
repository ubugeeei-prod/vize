import * as fs from "fs";
import * as path from "path";
import type { ServerCandidate } from "./server-state-types.js";

export interface WorkspaceServerDiscovery {
  candidates: ServerCandidate[];
  reason?: string;
}

export const RESTRICTED_MODE_WORKSPACE_SERVER_REASON =
  "Workspace servers skipped in Restricted Mode.";

/** Resolve package manifests by filename; npm's exports may hide package.json. */
type Exists = (file: string) => boolean;

/** Every path discovery may check, so a caller can test them all at once. */
export function workspaceServerProbePaths(
  folders: readonly string[],
  platform: NodeJS.Platform = process.platform,
): string[] {
  const bin = platform === "win32" ? "vize.exe" : "vize";
  return [
    ...new Set(
      folders.flatMap((folder) =>
        searchDirectories(folder, folders).flatMap((directory) => [
          path.join(directory, "package.json"),
          path.join(directory, "node_modules", "vize", "package.json"),
          path.join(directory, "node_modules", ".bin", bin),
        ]),
      ),
    ),
  ];
}

export function discoverWorkspaceServers(
  folders: readonly string[],
  isTrusted: boolean,
  platform: NodeJS.Platform = process.platform,
  exists: Exists = fs.existsSync,
): WorkspaceServerDiscovery {
  if (!isTrusted) {
    return { candidates: [], reason: RESTRICTED_MODE_WORKSPACE_SERVER_REASON };
  }
  const servers = new Map<string, ServerCandidate>();
  try {
    for (const folder of folders) {
      const directories = searchDirectories(folder, folders);
      const packageRoot = directories.findIndex((directory) =>
        exists(path.join(directory, "package.json")),
      );
      if (packageRoot < 0) continue;
      // The package's own directory first, then each parent that may hoist it.
      const installRoots = directories.slice(packageRoot);
      const serverPath =
        packageServer(installRoots, exists) ?? binServer(installRoots, platform, exists);
      if (serverPath) {
        const realPath = fs.realpathSync(serverPath);
        // realpath resolves links, but its drive-letter casing may follow the
        // caller. Preserve every other byte: NTFS directories may distinguish case.
        const identity =
          platform === "win32"
            ? realPath.replace(/^[A-Z]:/, (drive) => drive.toLowerCase())
            : realPath;
        servers.set(identity, { path: realPath, source: "workspace" });
      }
    }
  } catch (error) {
    return { candidates: [], reason: `Workspace server resolution failed: ${String(error)}` };
  }
  // One LanguageClient serves all folders. Never silently apply one project's
  // engine to another, even when two distinct builds report the same version.
  if (servers.size > 1) {
    return {
      candidates: [],
      reason: `Workspace folders use different Vize installations: ${[...servers.values()].map((server) => server.path).join(", ")}. Use separate windows or vize.serverPath to select one explicitly.`,
    };
  }
  return { candidates: [...servers.values()] };
}

/**
 * The folder and its ancestors, ending at the outermost workspace folder that
 * contains it. Trusting a folder says nothing about its parents, and a shared
 * parent such as a temporary directory holds other people's files.
 */
function searchDirectories(folder: string, folders: readonly string[]): string[] {
  const directories = ancestors(folder);
  const roots = folders.map((root) => path.resolve(root));
  let outermost = 0;
  for (const [index, directory] of directories.entries()) {
    if (roots.some((root) => path.relative(root, directory) === "")) outermost = index;
  }
  return directories.slice(0, outermost + 1);
}

function packageServer(directories: readonly string[], exists: Exists): string | undefined {
  for (const directory of directories) {
    const root = path.join(directory, "node_modules", "vize");
    const manifestPath = path.join(root, "package.json");
    if (!exists(manifestPath)) continue;
    const manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8")) as {
      name?: unknown;
      bin?: string | { vize?: unknown };
    };
    const bin = typeof manifest.bin === "string" ? manifest.bin : manifest.bin?.vize;
    if (manifest.name !== "vize" || typeof bin !== "string") {
      throw new Error(`No Vize bin entry in ${manifestPath}`);
    }
    const executable = path.resolve(root, bin);
    if (!fs.statSync(executable).isFile()) throw new Error(`Vize bin is not a file: ${executable}`);
    return executable;
  }
  return undefined;
}

function binServer(
  directories: readonly string[],
  platform: NodeJS.Platform,
  exists: Exists,
): string | undefined {
  for (const directory of directories) {
    const bin = path.join(
      directory,
      "node_modules",
      ".bin",
      platform === "win32" ? "vize.exe" : "vize",
    );
    if (!exists(bin)) continue;
    const resolved = fs.realpathSync(bin);
    const descriptor = fs.openSync(resolved, "r");
    const prefix = Buffer.alloc(160);
    try {
      fs.readSync(descriptor, prefix, 0, prefix.length, 0);
    } finally {
      fs.closeSync(descriptor);
    }
    if (
      prefix.subarray(0, 2).toString() === "#!" &&
      !/^#![^\r\n]*\bnode(?:\s|$)/.test(prefix.toString())
    ) {
      throw new Error(
        `Cannot resolve npm shell wrapper ${bin}; reinstall the workspace vize package.`,
      );
    }
    return resolved;
  }
  return undefined;
}

function ancestors(start: string): string[] {
  const directories: string[] = [];
  let current = path.resolve(start);
  while (true) {
    directories.push(current);
    const parent = path.dirname(current);
    if (parent === current) return directories;
    current = parent;
  }
}

/** Tests every path at once; a serial existsSync per path is one round trip each on 9P. */
export async function existingPaths(paths: readonly string[]): Promise<(file: string) => boolean> {
  const present = new Set<string>();
  await Promise.all(
    paths.map((file) =>
      fs.promises.access(file).then(
        () => present.add(file),
        () => undefined,
      ),
    ),
  );
  return (file) => present.has(file);
}
