/**
 * Timing and output measurement.
 *
 * `hyperfine` is not in the nix devshell (`flake.nix` has no `hyperfine`), so
 * the warmup + repeated-runs + spread reporting it provides is implemented
 * here: one discarded warmup run, then N timed runs, reported as median with
 * min/max. A single run is never reported as a number.
 *
 * Output sizes are collected the way `rolldown/benchmarks` `bench.mjs`
 * collects them: walk the dist directory and bucket by extension into
 * JS / CSS / sourcemap, so a build that is fast because it emitted less is
 * visible in the same table as the time.
 */

import { spawn } from "node:child_process";
import { existsSync, readFileSync, readdirSync, rmSync, statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { join } from "node:path";

const BUILD_ONE = fileURLToPath(new URL("./build-one.mjs", import.meta.url));

function runOnce(appDir, configPath) {
  return new Promise((resolve, reject) => {
    const start = process.hrtime.bigint();
    const child = spawn(process.execPath, [BUILD_ONE, appDir, configPath], {
      cwd: appDir,
      stdio: ["ignore", "pipe", "pipe"],
    });

    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk;
    });

    child.on("error", reject);
    child.on("close", (code, signal) => {
      const wallMs = Number(process.hrtime.bigint() - start) / 1e6;
      if (code !== 0) {
        reject(
          new Error(
            `build failed (exit ${code}, signal ${signal})\n--- stderr ---\n${stderr.trim()}`,
          ),
        );
        return;
      }
      let buildMs = Number.NaN;
      const lastLine = stdout.trim().split("\n").at(-1) ?? "";
      try {
        buildMs = JSON.parse(lastLine).buildMs;
      } catch {
        // keep NaN; wall clock is the primary number
      }
      resolve({ wallMs, buildMs });
    });
  });
}

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 === 1 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

function walk(dir) {
  const files = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) {
      files.push(...walk(full));
    } else if (entry.isFile()) {
      files.push(full);
    }
  }
  return files;
}

export function collectOutput(appDir, tool) {
  const distDir = join(appDir, `dist-${tool}`);
  const sizes = { jsBytes: 0, cssBytes: 0, mapBytes: 0, otherBytes: 0, fileCount: 0 };
  if (existsSync(distDir)) {
    for (const file of walk(distDir)) {
      const { size } = statSync(file);
      sizes.fileCount += 1;
      if (file.endsWith(".map")) sizes.mapBytes += size;
      else if (file.endsWith(".js")) sizes.jsBytes += size;
      else if (file.endsWith(".css")) sizes.cssBytes += size;
      else sizes.otherBytes += size;
    }
  }

  const metricsPath = join(appDir, `.metrics-${tool}.json`);
  let moduleCount = 0;
  let modules = [];
  if (existsSync(metricsPath)) {
    const parsed = JSON.parse(readFileSync(metricsPath, "utf8"));
    moduleCount = parsed.moduleCount;
    modules = parsed.modules ?? [];
  }

  return { ...sizes, moduleCount, modules };
}

/**
 * Warmup + `runs` timed runs of one tool.
 *
 * `cold` drops the plugin's persistent caches before every run (including the
 * warmup) so cold and warm numbers are both reportable.
 */
export async function measureTool({ appDir, configPath, runs, cold, cacheDirs }) {
  const dropCaches = () => {
    if (!cold) return;
    for (const dir of cacheDirs) {
      rmSync(dir, { recursive: true, force: true });
    }
  };

  dropCaches();
  await runOnce(appDir, configPath);

  const wall = [];
  const build = [];
  for (let index = 0; index < runs; index++) {
    dropCaches();
    const result = await runOnce(appDir, configPath);
    wall.push(result.wallMs);
    build.push(result.buildMs);
  }

  return {
    runs: wall.length,
    wallMedianMs: median(wall),
    wallMinMs: Math.min(...wall),
    wallMaxMs: Math.max(...wall),
    buildMedianMs: median(build),
    samplesMs: wall,
  };
}

export { median };
