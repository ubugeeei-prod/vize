/**
 * Measure cold Vite+ rule discovery across binary identities.
 *
 * Usage: bun tools/benchmarks/scripts/vite-plus-rule-catalog-storage.ts \
 *   --runtime npm/builder/vite/src/vite-plus/runtime.ts [--samples 7] [--versions 12]
 */
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { spawnSync } from "node:child_process";

const args = process.argv.slice(2);
if (args[0] === "--child") {
  const runtime = args[1];
  assert.ok(runtime);
  const loaded = (await import(pathToFileURL(runtime).href)) as {
    availableVueRules(): Set<string>;
  };
  assert.deepEqual([...loaded.availableVueRules()], ["no-v-html"]);
  process.exit(0);
}

function option(name: string, fallback?: string): string {
  const index = args.indexOf(name);
  const value = index < 0 ? fallback : args[index + 1];
  if (value == null) throw new Error(`missing ${name}`);
  return value;
}

const runtime = path.resolve(option("--runtime"));
const samples = Number(option("--samples", "7"));
const versions = Number(option("--versions", "12"));
assert.ok(Number.isSafeInteger(samples) && samples > 0);
assert.ok(Number.isSafeInteger(versions) && versions > 0);

interface Storage {
  bytes: number;
  directories: number;
  files: number;
}

function storage(root: string): Storage {
  const result: Storage = { bytes: 0, directories: 0, files: 0 };
  const pending = [root];
  while (pending.length > 0) {
    const directory = pending.pop();
    assert.ok(directory);
    for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
      const file = path.join(directory, entry.name);
      if (entry.isDirectory()) {
        result.directories += 1;
        pending.push(file);
      } else if (entry.isFile()) {
        result.files += 1;
        result.bytes += fs.statSync(file).size;
      }
    }
  }
  return result;
}

function percentile(values: readonly number[], proportion: number): number {
  const sorted = [...values].sort((left, right) => left - right);
  return sorted[Math.ceil(sorted.length * proportion) - 1];
}

const script = path.resolve(process.argv[1]);
const elapsed: number[] = [];
const observations: { cache: Storage; temporary: Storage }[] = [];
for (let sample = 0; sample < samples; sample += 1) {
  const owner = fs.mkdtempSync(path.join(os.tmpdir(), "vize-rule-storage-bench-"));
  try {
    const project = path.join(owner, "project with spaces 界");
    const packageRoot = path.join(project, "node_modules", "vite-plus");
    const binaryRoot = path.join(packageRoot, "bin");
    const temporary = path.join(owner, "temporary");
    const cache = path.join(owner, "cache");
    const home = path.join(owner, "home");
    fs.mkdirSync(binaryRoot, { recursive: true });
    fs.mkdirSync(temporary);
    fs.mkdirSync(cache);
    fs.mkdirSync(home);
    fs.writeFileSync(path.join(project, "package.json"), '{"private":true}');
    for (let version = 0; version < versions; version += 1) {
      fs.writeFileSync(
        path.join(binaryRoot, `vp-${version}.mjs`),
        fs.readFileSync(
          new URL("./fixtures/vite-plus-rule-catalog/list-vue-lint-rules.js", import.meta.url),
          "utf8",
        ),
      );
    }
    const started = process.hrtime.bigint();
    for (let version = 0; version < versions; version += 1) {
      fs.writeFileSync(
        path.join(packageRoot, "package.json"),
        JSON.stringify({ name: "vite-plus", bin: { vp: `bin/vp-${version}.mjs` } }),
      );
      const result = spawnSync(process.execPath, [script, "--child", runtime], {
        cwd: project,
        encoding: "utf8",
        env: {
          ...process.env,
          HOME: home,
          XDG_CACHE_HOME: cache,
          TMPDIR: temporary,
          TMP: temporary,
          TEMP: temporary,
        },
      });
      assert.equal(result.status, 0, result.stderr);
    }
    elapsed.push(Number(process.hrtime.bigint() - started) / 1_000_000);
    observations.push({ cache: storage(cache), temporary: storage(temporary) });
  } finally {
    fs.rmSync(owner, { recursive: true, force: true });
  }
}

const sameStorage = (key: "cache" | "temporary") => {
  const first = observations[0][key];
  assert.ok(
    observations.every((observation) => JSON.stringify(observation[key]) === JSON.stringify(first)),
  );
  return first;
};
process.stdout.write(
  `${JSON.stringify(
    {
      runtime,
      samples,
      versions,
      wallMs: {
        p50: percentile(elapsed, 0.5),
        p95: percentile(elapsed, 0.95),
        samples: elapsed,
      },
      storage: { cache: sameStorage("cache"), temporary: sameStorage("temporary") },
    },
    null,
    2,
  )}\n`,
);
