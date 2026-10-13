#!/usr/bin/env node
/**
 * One production build, in a fresh process.
 *
 * The harness spawns this once per benchmark run so the measured wall clock is
 * process-level, the way `hyperfine 'node --run build:<tool>'` measures the
 * reference. Node startup is therefore included, identically for both tools.
 *
 * Usage: node build-one.mjs <appDir> <configPath>
 * Prints one JSON line: `{"buildMs":<number>}` (in-process build time, so the
 * harness can report it alongside the process wall clock).
 */

import { performance } from "node:perf_hooks";

const [appDir, configPath] = process.argv.slice(2);

if (!appDir || !configPath) {
  console.error("usage: node build-one.mjs <appDir> <configPath>");
  process.exit(2);
}

const { build } = await import("vite");

const start = performance.now();
await build({ configFile: configPath, root: appDir, logLevel: "silent" });
const buildMs = performance.now() - start;

process.stdout.write(`${JSON.stringify({ buildMs })}\n`);
