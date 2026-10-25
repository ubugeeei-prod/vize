import * as fs from "fs";
import * as path from "path";
import type { ServerCandidate } from "./server-state-types.js";

export interface WorkspaceServerDiscovery {
  candidates: ServerCandidate[];
  reason?: string;
}

/** Resolve package manifests by filename; npm's exports may hide package.json. */
export function discoverWorkspaceServers(
  folders: readonly string[],
  isTrusted: boolean,
  platform: NodeJS.Platform = process.platform,
): WorkspaceServerDiscovery {
  if (!isTrusted) {
    return { candidates: [], reason: "Workspace servers skipped in Restricted Mode." };
  }
  const servers = new Map<string, ServerCandidate>();
  try {
    for (const folder of folders) {
      const packageRoot = nearestPackageRoot(folder);
      if (!packageRoot) continue;
      const serverPath = packageServer(packageRoot) ?? binServer(packageRoot, platform);
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

function nearestPackageRoot(folder: string): string | undefined {
  return ancestors(folder).find((directory) => fs.existsSync(path.join(directory, "package.json")));
}

function packageServer(packageRoot: string): string | undefined {
  for (const directory of ancestors(packageRoot)) {
    const root = path.join(directory, "node_modules", "vize");
    const manifestPath = path.join(root, "package.json");
    if (!fs.existsSync(manifestPath)) continue;
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

function binServer(packageRoot: string, platform: NodeJS.Platform): string | undefined {
  for (const directory of ancestors(packageRoot)) {
    const bin = path.join(
      directory,
      "node_modules",
      ".bin",
      platform === "win32" ? "vize.exe" : "vize",
    );
    if (!fs.existsSync(bin)) continue;
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
