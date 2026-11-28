/** One fresh process and one global Rayon pool; explicit per-call pools are excluded. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath } from "node:url";

const hash = (value) => createHash("sha256").update(value).digest("hex");

/** The timed section includes argument and result conversion and window retirement. */
export function measureNative(native, files, chunkSize) {
  const packets = [];
  let calls = 0;
  const startCpu = process.cpuUsage();
  const start = performance.now();
  const window = native.beginTypeResolutionBatch();
  try {
    for (const ssr of [false, true, false]) {
      const results = [];
      for (let offset = 0; offset < files.length; offset += chunkSize) {
        const packet = native.compileSfcBatchWithResults(files.slice(offset, offset + chunkSize), {
          ssr,
          inlineTemplate: true,
          isProd: true,
          vueVersion: "3",
          includeSourceMap: true,
        });
        calls++;
        assert.equal(packet.failedCount, 0);
        assert.equal(packet.successCount, Math.min(chunkSize, files.length - offset));
        results.push(...packet.results);
      }
      packets.push(results);
    }
  } finally {
    window.close();
  }
  const wallMs = performance.now() - start;
  const cpu = process.cpuUsage(startCpu);
  return {
    wallMs,
    cpuMs: (cpu.user + cpu.system) / 1000,
    calls,
    packets,
    rssBytes: process.memoryUsage().rss,
    maxRssKiB: process.resourceUsage().maxRSS,
  };
}

function validate(packets, files, expectedDependencies) {
  const complete = createHash("sha256");
  assert.equal(packets.length, 3);
  for (const results of packets) {
    assert.equal(results.length, files.length);
    for (const [index, result] of results.entries()) {
      assert.equal(result.path, files[index].path);
      assert.deepEqual(result.errors, []);
      assert.deepEqual(result.warnings, []);
      assert.equal(typeof result.code, "string");
      assert(result.code.length > 0);
      assert(result.map, "real generated source maps must be present");
      assert.deepEqual(result.dependencies ?? [], expectedDependencies);
      complete.update(JSON.stringify(result));
      complete.update("\n");
    }
  }
  // Retain the complete first component and hash every complete result, including
  // dependency names, diagnostics and maps. No payload is dropped for measurement.
  return {
    sha256: complete.digest("hex"),
    first: packets.map((results) => results[0]),
    files: packets.map((results) =>
      results.map((result) => ({
        path: result.path,
        sha256: hash(JSON.stringify(result)),
      })),
    ),
  };
}

export function freshness(native, root) {
  const filename = path.join(root, "freshness.vue");
  const source =
    '<script setup lang="ts">import type { Model0 } from "./src/models/Model0"; defineProps<Model0>();</script><template><div /></template>';
  const file = path.join(root, "src/models/Model0.ts");
  const original = fs.readFileSync(file, "utf8");
  const originalStamp = fs.statSync(file);
  // Normalize to an exactly representable second before the first compile.
  // Passing a Date back to utimes would discard submillisecond precision and
  // could make a metadata-only cache observe a different timestamp.
  const second = Math.floor(originalStamp.mtimeMs / 1000);
  fs.utimesSync(file, second, second);
  const stamp = fs.statSync(file, { bigint: true });
  const compile = () => {
    const window = native.beginTypeResolutionBatch();
    try {
      const result = native.compileSfcBatchWithResults([{ path: filename, source }], {
        mode: "function",
        vueVersion: "3",
      }).results[0];
      assert.deepEqual(result.errors, []);
      return result;
    } finally {
      window.close();
    }
  };
  const before = compile();
  assert.match(before.code, /field0:\s*\{\s*type:\s*String/u);
  const changed = original.replace("string", "number");
  assert.notEqual(changed, original);
  assert.equal(Buffer.byteLength(changed), Buffer.byteLength(original));
  fs.writeFileSync(file, changed);
  fs.utimesSync(file, second, second);
  const editedStamp = fs.statSync(file, { bigint: true });
  assert.equal(editedStamp.mtimeNs, stamp.mtimeNs, "the edit retains the exact timestamp");
  assert.equal(editedStamp.size, stamp.size);
  const after = compile();
  assert.match(after.code, /field0:\s*\{\s*type:\s*Number/u);
  fs.renameSync(path.join(root, "src/models"), path.join(root, "src/retired-models"));
  fs.mkdirSync(path.join(root, "src/models"));
  fs.writeFileSync(file, original.replace("string", "boolean"));
  const remounted = compile();
  assert.match(remounted.code, /field0:\s*\{\s*type:\s*Boolean/u);
  fs.rmSync(path.join(root, "src/models"), { recursive: true });
  fs.renameSync(path.join(root, "src/retired-models"), path.join(root, "src/models"));
  fs.writeFileSync(file, original);
  fs.utimesSync(file, originalStamp.atime, originalStamp.mtime);
  return {
    before,
    after,
    remounted,
    sameLength: true,
    restoredMtimeNs: stamp.mtimeNs.toString(),
    editedMtimeNs: editedStamp.mtimeNs.toString(),
  };
}

function measureVue(compiler, files) {
  const products = [];
  const startCpu = process.cpuUsage();
  const start = performance.now();
  for (const ssr of [false, true, false]) {
    const results = [];
    for (const file of files) {
      const parsed = compiler.parse(file.source, { filename: file.path, sourceMap: true });
      assert.deepEqual(parsed.errors, []);
      const script = compiler.compileScript(parsed.descriptor, {
        id: "benchmark",
        isProd: true,
        sourceMap: true,
        inlineTemplate: true,
        templateOptions: { ssr },
      });
      results.push({ code: script.content, map: script.map });
    }
    products.push(results);
  }
  const wallMs = performance.now() - start;
  const cpu = process.cpuUsage(startCpu);
  return { wallMs, cpuMs: (cpu.user + cpu.system) / 1000, sha256: hash(JSON.stringify(products)) };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const [kind, provider, fixture, chunk, output] = process.argv.slice(2);
  assert(kind && provider && fixture && chunk && output);
  const files = fs
    .readdirSync(path.join(fixture, "src/components"))
    .sort()
    .map((name) => {
      const filename = path.join(fixture, "src/components", name);
      return { path: filename, source: fs.readFileSync(filename, "utf8") };
    });
  assert.equal(files.length, 1000);
  const require = createRequire(import.meta.url);
  if (kind === "vue") {
    const compiler = require(path.join(provider, "node_modules/@vue/compiler-sfc"));
    compiler.registerTS(() => require(path.join(provider, "node_modules/typescript")));
    fs.writeFileSync(
      output,
      JSON.stringify({ cold: measureVue(compiler, files), warm: measureVue(compiler, files) }),
    );
  } else {
    assert.equal(kind, "native");
    const native = require(provider);
    const expectedDependencies = JSON.parse(
      fs.readFileSync(path.join(fixture, "dependencies.json"), "utf8"),
    );
    const phases = {};
    for (const phase of ["cold", "warm"]) {
      const measured = measureNative(native, files, Number(chunk));
      phases[phase] = {
        ...measured,
        packets: undefined,
        output: validate(measured.packets, files, expectedDependencies),
      };
    }
    assert.equal(phases.cold.output.sha256, phases.warm.output.sha256);
    fs.writeFileSync(
      output,
      JSON.stringify({
        bindingSha256: hash(fs.readFileSync(provider)),
        buildToken: native.getNativeBuildToken(),
        threads: process.env.RAYON_NUM_THREADS,
        ...phases,
        freshness: freshness(native, fixture),
      }),
    );
  }
}
