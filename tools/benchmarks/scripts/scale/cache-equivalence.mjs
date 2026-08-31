#!/usr/bin/env node
/**
 * Cold vs warm output equivalence for the persistent pre-compile cache.
 *
 * The build benchmark reports warm numbers by construction: the warmup run
 * populates `node_modules/.vize/vite-precompile/`, so every timed run after it
 * restores from disk instead of compiling. That makes the cache load-bearing for
 * the headline number, and makes a stale entry the most expensive possible bug:
 * at 10k SFCs nobody finds it by reading output.
 *
 * So assert it. Build with the cache dropped, hash every emitted file, build
 * again with the cache in place, and require the same set of `(name, sha256)`
 * pairs. Rollup names chunks by content hash, so an identical name set is
 * already strong; the digests make it exact.
 *
 * Usage: node tools/benchmarks/scripts/scale/cache-equivalence.mjs <appDir> [tool=vize]
 * Exit 0 when cold and warm agree, 1 when they do not.
 */

import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, rmSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join, relative } from "node:path";

const BUILD_ONE = fileURLToPath(new URL("./build-one.mjs", import.meta.url));

function walk(dir, base = dir) {
  const files = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) files.push(...walk(full, base));
    else if (entry.isFile()) files.push(relative(base, full));
  }
  return files;
}

function fingerprint(distDir) {
  if (!existsSync(distDir)) {
    return new Map();
  }
  const digests = new Map();
  for (const name of walk(distDir).sort()) {
    digests.set(
      name,
      createHash("sha256")
        .update(readFileSync(join(distDir, name)))
        .digest("hex"),
    );
  }
  return digests;
}

function build(appDir, configPath) {
  const result = spawnSync(process.execPath, [BUILD_ONE, appDir, configPath], {
    cwd: appDir,
    stdio: ["ignore", "pipe", "pipe"],
    encoding: "utf8",
  });
  if (result.status !== 0) {
    throw new Error(`build failed (exit ${result.status})\n${result.stderr ?? ""}`);
  }
}

export function compareColdAndWarm(appDir, tool = "vize") {
  const configPath = join(appDir, `vite.${tool}.config.mjs`);
  const distDir = join(appDir, `dist-${tool}`);
  const cacheDir = join(appDir, "node_modules", ".vize");

  rmSync(cacheDir, { recursive: true, force: true });
  build(appDir, configPath);
  const cold = fingerprint(distDir);
  const cacheWritten = existsSync(cacheDir);

  build(appDir, configPath);
  const warm = fingerprint(distDir);

  const differences = [];
  for (const [name, digest] of cold) {
    if (!warm.has(name)) differences.push({ name, kind: "missing-when-warm" });
    else if (warm.get(name) !== digest) differences.push({ name, kind: "content-differs" });
  }
  for (const name of warm.keys()) {
    if (!cold.has(name)) differences.push({ name, kind: "extra-when-warm" });
  }

  return { coldFileCount: cold.size, warmFileCount: warm.size, cacheWritten, differences };
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const [appDir, tool = "vize"] = process.argv.slice(2);
  if (!appDir) {
    console.error("usage: node tools/benchmarks/scripts/scale/cache-equivalence.mjs <appDir> [tool]");
    process.exit(2);
  }

  const report = compareColdAndWarm(appDir, tool);
  console.log(`tool           : ${tool}`);
  console.log(`cache written  : ${report.cacheWritten}`);
  console.log(`cold dist files: ${report.coldFileCount}`);
  console.log(`warm dist files: ${report.warmFileCount}`);
  if (report.differences.length === 0) {
    console.log("result         : cold and warm builds are byte-identical");
    process.exit(0);
  }
  console.log(`result         : ${report.differences.length} difference(s)`);
  for (const difference of report.differences.slice(0, 20)) {
    console.log(`  - ${difference.kind}: ${difference.name}`);
  }
  process.exit(1);
}
