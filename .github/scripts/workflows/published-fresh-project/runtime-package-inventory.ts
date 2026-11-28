import { createHash } from "node:crypto";
import { constants } from "node:fs";
import { lstat, open, opendir, realpath, stat } from "node:fs/promises";
import { join } from "node:path";

interface Limits {
  maximumEntries?: number;
  maximumManifests?: number;
  maximumManifestBytes?: number;
  maximumTotalManifestBytes?: number;
}

interface ManifestSource {
  path: string;
  bytes: number;
  sha256: string;
  source: string;
  name: string | null;
  version: string | null;
}

const defaults = {
  maximumEntries: 200_000,
  maximumManifests: 10_000,
  maximumManifestBytes: 1_048_576,
  maximumTotalManifestBytes: 67_108_864,
};
const compare = (left: string, right: string) => (left < right ? -1 : Number(left > right));

/**
 * Observe actual manifest bytes and direct resolutions after installation.
 * Aliases and directory cycles share physical visits; broken links, unreadable
 * or invalid manifests, special files and exhausted budgets refuse a result.
 * Defaults allow 200,000 entries, 10,000 manifests, 1 MiB per manifest and
 * 64 MiB of manifest bytes. Overrides may only tighten these limits.
 */
export async function collectRuntimePackageInventory(
  server: string,
  expectedRuntimeManifest: string,
  options: Limits = {},
) {
  const limits = { ...defaults };
  for (const key of Object.keys(limits) as Array<keyof Limits>) {
    const requested = options[key];
    const value = requested === undefined ? limits[key] : requested;
    if (!Number.isSafeInteger(value) || value <= 0 || value > defaults[key])
      throw new Error(`Invalid inventory limit: ${key}`);
    limits[key] = value;
  }
  let entries = 0;
  let totalBytes = 0;
  const manifests = new Map<string, ManifestSource>();
  const packages = new Map<string, Set<string>>();

  async function observe(file: string, installed: boolean) {
    const physical = await realpath(file);
    let manifest = manifests.get(physical);
    if (manifest === undefined) {
      if (manifests.size >= limits.maximumManifests)
        throw new Error("Runtime manifest count exceeds its bound");
      const handle = await open(physical, constants.O_RDONLY | constants.O_NONBLOCK);
      let data: Buffer;
      try {
        const before = await handle.stat();
        if (!before.isFile()) throw new Error(`Runtime manifest is not a regular file: ${file}`);
        if (before.size > limits.maximumManifestBytes)
          throw new Error(`Runtime manifest exceeds its byte bound: ${file}`);
        const chunks: Buffer[] = [];
        let bytes = 0;
        for (;;) {
          const buffer = Buffer.alloc(Math.min(65_536, limits.maximumManifestBytes - bytes + 1));
          const read = await handle.read(buffer, 0, buffer.length, null);
          if (read.bytesRead === 0) break;
          bytes += read.bytesRead;
          if (bytes > limits.maximumManifestBytes)
            throw new Error(`Runtime manifest exceeds its byte bound: ${file}`);
          if (totalBytes + bytes > limits.maximumTotalManifestBytes)
            throw new Error("Runtime manifest bytes exceed their aggregate bound");
          chunks.push(buffer.subarray(0, read.bytesRead));
        }
        data = Buffer.concat(chunks);
      } finally {
        await handle.close();
      }
      totalBytes += data.length;
      const source = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(data);
      const value: unknown = JSON.parse(source);
      const object =
        value !== null && typeof value === "object" && !Array.isArray(value)
          ? (value as Record<string, unknown>)
          : {};
      manifest = {
        path: physical,
        bytes: data.length,
        sha256: createHash("sha256").update(data).digest("hex"),
        source,
        name: typeof object.name === "string" ? object.name : null,
        version: typeof object.version === "string" ? object.version : null,
      };
      manifests.set(physical, manifest);
    }
    if (installed && manifest.name !== null && manifest.version !== null) {
      const versions = packages.get(manifest.name) ?? new Set<string>();
      versions.add(manifest.version);
      packages.set(manifest.name, versions);
    }
    return manifest;
  }

  const runtimeManifest = await observe(join(server, "package.json"), false);
  if (runtimeManifest.source !== expectedRuntimeManifest)
    throw new Error("Runtime manifest changed after capture");
  const runtime: unknown = JSON.parse(runtimeManifest.source);
  const dependencies =
    runtime !== null && typeof runtime === "object" && !Array.isArray(runtime)
      ? (runtime as Record<string, unknown>).dependencies
      : undefined;
  if (
    dependencies === null ||
    typeof dependencies !== "object" ||
    Array.isArray(dependencies) ||
    Object.keys(dependencies).length === 0
  )
    throw new Error("Deployment graph is empty or invalid");

  const modules = join(server, "node_modules");
  const direct: Array<{ name: string; version: string; manifest: string }> = [];
  for (const [name, version] of Object.entries(dependencies as Record<string, unknown>).sort(
    ([a], [b]) => compare(a, b),
  )) {
    if (++entries > limits.maximumEntries)
      throw new Error("Runtime inventory entries exceed their bound");
    const segments = name.split("/");
    if (
      name.includes("\\") ||
      name.includes("\0") ||
      segments.some((part) => !part || part === "." || part === "..") ||
      segments.length !== (name.startsWith("@") ? 2 : 1)
    )
      throw new Error(`Invalid runtime dependency name: ${name}`);
    if (typeof version !== "string" || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version))
      throw new Error(`Runtime dependency is not exact: ${name}`);
    const manifest = await observe(join(modules, name, "package.json"), true);
    if (manifest.name !== name || manifest.version !== version)
      throw new Error(`Runtime direct resolution differs: ${name}`);
    direct.push({ name, version, manifest: manifest.path });
  }

  const pending = [modules];
  const visited = new Set<string>();
  while (pending.length > 0) {
    const directory = await realpath(pending.pop()!);
    if (visited.has(directory)) continue;
    visited.add(directory);
    for await (const entry of await opendir(directory)) {
      if (++entries > limits.maximumEntries)
        throw new Error("Runtime inventory entries exceed their bound");
      const file = join(directory, entry.name);
      const physical = await lstat(file);
      const target = physical.isSymbolicLink() ? await stat(file) : physical;
      if (target.isDirectory()) pending.push(file);
      else if (target.isFile()) {
        if (entry.name === "package.json") await observe(file, true);
      } else throw new Error(`Unsupported runtime filesystem entry: ${file}`);
    }
  }
  return {
    runtimeManifest,
    direct,
    manifests: [...manifests.values()].sort((a, b) => compare(a.path, b.path)),
    packages: Object.fromEntries(
      [...packages]
        .sort(([a], [b]) => a.localeCompare(b))
        .map(([name, versions]) => [name, [...versions].sort()]),
    ),
  };
}
