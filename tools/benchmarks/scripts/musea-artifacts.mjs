/**
 * Content-addressed runtime snapshot for the Musea benchmark lane.
 *
 * ESM bundles are location-sensitive: relative chunks and `import.meta.url`
 * resource lookups both depend on the package layout. The snapshot therefore
 * preserves each path below its package root instead of inserting a hash
 * directory below `dist`. A fresh worker imports this snapshot and
 * forces `@vizejs/native` to load the copied binding, so the recorded files are
 * the files that execute.
 */

import { createHash, randomUUID } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  readdirSync,
  realpathSync,
  renameSync,
  rmSync,
} from "node:fs";
import { createRequire } from "node:module";
import { basename, dirname, join, relative } from "node:path";

import { fileSha256 } from "./benchmark-binary.mjs";

const LEGACY_PIN_DIRECTORY = ".musea-benchmark-runtime";
const SNAPSHOT_CACHE = join("node_modules", ".cache", "vize-musea-benchmark");
const NATIVE_RUNTIME_FILES = ["index.js", "native-binding.js", "native-targets.js", "package.json"];

function byCodeUnit(a, b) {
  if (a === b) return 0;
  return a < b ? -1 : 1;
}

function requireBuiltModule(path, buildTask, packageName) {
  if (!existsSync(path)) {
    throw new Error(
      `${packageName} build not found: ${path}. Run vp run --workspace-root ${buildTask} first.`,
    );
  }
  return path;
}

function runtimeFiles(directory) {
  const found = [];
  const visit = (current) => {
    const entries = readdirSync(current, { withFileTypes: true }).sort((a, b) =>
      byCodeUnit(a.name, b.name),
    );
    for (const entry of entries) {
      const path = join(current, entry.name);
      if (entry.isDirectory() && entry.name !== LEGACY_PIN_DIRECTORY) visit(path);
      else if (
        entry.isFile() &&
        !entry.name.endsWith(".map") &&
        !/\.d\.(?:cts|mts|ts)$/.test(entry.name)
      ) {
        found.push(path);
      }
    }
  };
  visit(directory);
  return found;
}

function identify(path) {
  const sha256 = fileSha256(path);
  if (sha256 == null) {
    throw new Error(`musea-artifacts: cannot hash ${path}; refusing to measure it`);
  }
  return sha256;
}

function resolveNativeBinding(nativeDir) {
  let bindings = [];
  try {
    bindings = readdirSync(nativeDir)
      .filter((entry) => entry.endsWith(".node"))
      .sort(byCodeUnit);
  } catch {
    bindings = [];
  }
  if (bindings.length !== 1) {
    throw new Error(
      `@vizejs/native must have exactly one local binding in ${nativeDir}, found ${bindings.length}. Run vp run --workspace-root build:native:test first.`,
    );
  }
  return join(nativeDir, bindings[0]);
}

function sourceManifest(rootDir) {
  const pluginEntry = requireBuiltModule(
    join(rootDir, "npm", "builder", "vite-musea", "dist", "index.mjs"),
    "build:nuxt-stack",
    "@vizejs/vite-plugin-musea",
  );
  const nuxtEntry = requireBuiltModule(
    join(rootDir, "npm", "framework", "musea-nuxt", "dist", "index.mjs"),
    "build:nuxt-stack",
    "@vizejs/musea-nuxt",
  );
  const nativeDir = join(rootDir, "npm", "native");
  const nativeBinding = resolveNativeBinding(nativeDir);
  const manifest = [];

  const addTree = (entry, label, group) => {
    const sourceRoot = dirname(entry);
    for (const source of runtimeFiles(sourceRoot)) {
      const name = relative(sourceRoot, source);
      manifest.push({
        label: source === entry ? label : `${label}:${name}`,
        group,
        source,
        snapshotName: join("dist", name),
      });
    }
  };
  addTree(pluginEntry, "museaPlugin", "plugin");
  addTree(nuxtEntry, "museaNuxt", "nuxt");

  for (const name of NATIVE_RUNTIME_FILES) {
    const source = join(nativeDir, name);
    if (!existsSync(source)) {
      throw new Error(`@vizejs/native runtime file not found: ${source}`);
    }
    manifest.push({
      label: `nativeLoader:${name}`,
      group: "plugin",
      source,
      snapshotName: join("node_modules", "@vizejs", "native", name),
    });
  }
  manifest.push({
    label: "native",
    group: "plugin",
    source: nativeBinding,
    snapshotName: join("node_modules", "@vizejs", "native", basename(nativeBinding)),
  });
  return manifest.sort((a, b) => byCodeUnit(a.label, b.label));
}

function identifyManifest(manifest) {
  const graph = createHash("sha256");
  for (const artifact of manifest) {
    artifact.sha256 = identify(artifact.source);
    graph.update(
      `${artifact.group}:${Buffer.byteLength(artifact.snapshotName)}:${artifact.snapshotName}:${artifact.sha256}\n`,
    );
  }
  return graph.digest("hex");
}

/** Install one immutable file without truncating a copy another worker can read. */
function installPinnedCopy(artifact, measuredPath) {
  if (existsSync(measuredPath)) {
    if (identify(measuredPath) !== artifact.sha256) {
      throw new Error(`musea-artifacts: corrupt pinned copy at ${measuredPath}`);
    }
    return;
  }

  mkdirSync(dirname(measuredPath), { recursive: true });
  const temporary = `${measuredPath}.tmp-${process.pid}-${randomUUID()}`;
  try {
    copyFileSync(artifact.source, temporary);
    if (identify(temporary) !== artifact.sha256) {
      throw new Error(
        `musea-artifacts: ${artifact.label} changed while it was pinned; refusing to measure it`,
      );
    }
    try {
      renameSync(temporary, measuredPath);
    } catch (error) {
      if (!existsSync(measuredPath) || identify(measuredPath) !== artifact.sha256) throw error;
    }
  } finally {
    rmSync(temporary, { force: true });
  }
}

/** Resolve and pin both package runtimes plus the exact native loader and binding. */
export function resolveMuseaArtifacts(rootDir) {
  const manifest = sourceManifest(rootDir);
  const identity = identifyManifest(manifest);
  const snapshotRoots = {
    plugin: join(rootDir, "npm", "builder", "vite-musea", SNAPSHOT_CACHE, identity, "package"),
    nuxt: join(rootDir, "npm", "framework", "musea-nuxt", SNAPSHOT_CACHE, identity, "package"),
  };
  const artifacts = {};

  for (const artifact of manifest) {
    if (identify(artifact.source) !== artifact.sha256) {
      throw new Error(
        `musea-artifacts: ${artifact.label} changed while its snapshot was prepared; refusing to measure it`,
      );
    }
    const measuredPath = join(snapshotRoots[artifact.group], artifact.snapshotName);
    installPinnedCopy(artifact, measuredPath);
    artifacts[artifact.label] = {
      source: artifact.source,
      measuredPath,
      sha256: artifact.sha256,
      pinned: true,
    };
  }
  return artifacts;
}

/** Prove the snapshotted loader will select the snapshotted local binding first. */
export function assertMuseaNativeSelection(artifacts) {
  const loader = artifacts["nativeLoader:native-targets.js"]?.measuredPath;
  const binding = artifacts.native?.measuredPath;
  const nativeRequire = createRequire(loader);
  const { nativeTargets } = nativeRequire("./native-targets.js");
  if (typeof nativeTargets !== "function") {
    throw new Error("musea-artifacts: snapshotted native loader has no nativeTargets function");
  }
  const expected = nativeTargets([]).map((target) => `vize-vitrine.${target}.node`);
  if (!expected.includes(basename(binding))) {
    throw new Error(
      `musea-artifacts: ${basename(binding)} is not selected by the snapshotted native loader (expected one of ${expected.join(", ")})`,
    );
  }
}

/** Prove Node actually loaded the content-addressed binding, not an optional fallback. */
export function assertMuseaNativeLoaded(artifacts) {
  const loader = artifacts["nativeLoader:native-targets.js"]?.measuredPath;
  const binding = artifacts.native?.measuredPath;
  const nativeRequire = createRequire(loader);
  if (nativeRequire.cache[realpathSync(binding)] == null) {
    throw new Error(
      `musea-artifacts: snapshotted native binding was not loaded from ${binding}; refusing to publish a timing`,
    );
  }
}

/** Fail if either a source artifact or the exact snapshot being measured moved. */
export function assertMuseaArtifactsUnchanged(artifacts) {
  for (const [label, artifact] of Object.entries(artifacts)) {
    for (const [kind, path] of [
      ["source", artifact.source],
      ["measured copy", artifact.measuredPath],
    ]) {
      const current = fileSha256(path);
      if (current !== artifact.sha256) {
        throw new Error(
          `musea-artifacts: ${label} ${kind} changed during the run (${artifact.sha256} -> ${current ?? "missing"}); refusing to publish a timing`,
        );
      }
    }
  }
}
