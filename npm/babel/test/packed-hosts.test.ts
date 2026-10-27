import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  hostSpecs,
  json,
  prepareHost,
  settings,
  type HostSettings,
  type HostSpec,
} from "./hosts/environment.ts";
import type { WholeError } from "./host-types.ts";

const runner = pathToFileURL(
  path.join(path.dirname(fileURLToPath(import.meta.url)), "hosts/runner.ts"),
).href;
let configured: HostSettings | undefined;
const prepared = new Map<number, ReturnType<typeof prepareHost>>();
function context(spec: HostSpec) {
  configured ??= settings();
  if (!prepared.has(spec.major)) prepared.set(spec.major, prepareHost(spec, configured));
  return { config: configured, host: prepared.get(spec.major)! };
}
function worker(spec: HostSpec, format: "esm" | "cjs", phase: "cold" | "positive") {
  const { config, host } = context(spec);
  const name = "babel" + spec.major + "-" + format + "-" + phase;
  const report = path.join(config.output, name + ".json");
  const input = { ...config, host, format, phase, report };
  const inputFile = path.join(config.output, name + ".input.json");
  json(inputFile, input);
  const load =
    "{ requireHost, core: () => import('@babel/core'), vize: () => " +
    (format === "esm"
      ? "import('@vizejs/babel-plugin')"
      : "Promise.resolve(requireHost('@vizejs/babel-plugin'))") +
    ", react: () => import('@babel/plugin-transform-react-jsx'), typescript: () => import('@babel/plugin-transform-typescript'), modules: () => import('@babel/plugin-transform-modules-commonjs'), decorators: () => import('@babel/plugin-proposal-decorators'), classProperties: () => import('@babel/plugin-transform-class-properties'), mapping: () => import('@jridgewell/gen-mapping'), trace: () => import('@jridgewell/trace-mapping'), vue: () => import('vue'), renderer: () => import('@vue/server-renderer') }";
  const body =
    "const fs = requireHost('node:fs'); const { runHost } = await import(" +
    JSON.stringify(runner) +
    "); await runHost(JSON.parse(fs.readFileSync(process.argv[2])), " +
    load +
    ");";
  const entry = path.join(
    host.root,
    "node_modules/.vize-babel-host-" + format + (format === "esm" ? ".mjs" : ".cjs"),
  );
  fs.writeFileSync(
    entry,
    format === "esm"
      ? "import { createRequire } from 'node:module'; const requireHost = createRequire(import.meta.url); " +
          body
      : "const requireHost = require; (async () => { " +
          body +
          " })().catch(error => { console.error(error); process.exitCode = 1; });",
  );
  const result = spawnSync(process.execPath, [entry, inputFile], {
    cwd: host.root,
    env: process.env,
    maxBuffer: 64 * 1024 * 1024,
  });
  fs.writeFileSync(
    path.join(config.output, name + ".stdout.raw"),
    result.stdout ?? Buffer.alloc(0),
  );
  fs.writeFileSync(
    path.join(config.output, name + ".stderr.raw"),
    result.stderr ?? Buffer.alloc(0),
  );
  json(path.join(config.output, name + ".process.json"), {
    argv: [process.execPath, entry, inputFile],
    status: result.status,
    signal: result.signal,
    error: result.error ? String(result.error) : null,
  });
  assert.equal(result.status, 0, name + ": " + report + "\n" + String(result.stderr));
  assert.equal(result.signal, null);
  assert.equal(result.stderr.length, 0, name + " has no unrecorded warnings/errors");
  const observed = JSON.parse(fs.readFileSync(report, "utf8")) as {
    fatal: WholeError | null;
    observations: { assertion: WholeError | null }[];
  };
  assert.equal(observed.fatal, null);
  assert(observed.observations.length > 0);
  assert(observed.observations.every((item) => item.assertion === null));
}

void test("packed cold consumers preserve plain and scoped host results without native resolution", () => {
  const failures = [];
  for (const spec of hostSpecs)
    for (const format of ["esm", "cjs"] as const) {
      try {
        worker(spec, format, "cold");
      } catch (error) {
        failures.push((error as Error).message);
      }
    }
  assert.deepEqual(failures, [], "every cold host pair must pass; all failures are retained");
});
for (const spec of hostSpecs)
  for (const format of ["esm", "cjs"] as const) {
    void test("Babel " + spec.version + " actual packed " + format + " host contracts", () =>
      worker(spec, format, "positive"),
    );
  }
