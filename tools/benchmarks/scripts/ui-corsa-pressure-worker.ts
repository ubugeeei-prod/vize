#!/usr/bin/env node
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { echo, calibrate, sha, stamp, sourceRoot, realGo, outputDirectory } from "./case.js";

const self = fileURLToPath(import.meta.url);
if (process.argv[2] === "--echo") await echo();
else if (process.argv[2] === "--calibrate") {
  fs.mkdirSync(outputDirectory, { recursive: true });
  fs.writeFileSync(
    path.join(outputDirectory, "calibration.json"),
    JSON.stringify(await calibrate(self)) + "\n",
  );
} else {
  const shardIndex = Number(process.argv[2]);
  const phase = process.argv[3];
  assert(Number.isInteger(shardIndex) && shardIndex >= 0 && shardIndex < 8);
  assert(phase === "observed" || phase === "control");
  const directory = path.join(outputDirectory, phase);
  fs.mkdirSync(directory, { recursive: true });
  const file = path.join(directory, "worker-" + shardIndex + ".jsonl");
  const event = (row: Record<string, unknown>) =>
    fs.appendFileSync(
      file,
      JSON.stringify({ ns: stamp(), pid: process.pid, shardIndex, phase, ...row }) + "\n",
    );
  event({ event: "worker-start", cwd: process.cwd(), sourceRoot, node: process.version });
  const native = (await import(
    pathToFileURL(path.join(sourceRoot, "npm/native/index.js")).href
  )) as {
    lintPatinaSfc(source: string, options: Record<string, unknown>): unknown;
  };
  const quality = (await import(
    pathToFileURL(path.join(sourceRoot, "npm/ui/scripts/source-quality/lint-sfc.ts")).href
  )) as {
    lintSfcFiles(
      lint: (source: string, options: Record<string, unknown>) => unknown,
      roots: string,
      shard: { index: number; total: number },
    ): Promise<Array<{ filename: string; diagnostics: unknown[] }>>;
  };
  const proxy = path.join(path.dirname(self), "rpc-proxy.js");
  let sequence = 0;
  const results = await quality.lintSfcFiles(
    (source, originalOptions) => {
      sequence++;
      const filename = originalOptions.filename;
      assert.equal(typeof filename, "string");
      const options = { ...originalOptions, corsaPath: phase === "observed" ? proxy : realGo };
      event({
        event: "call-start",
        sequence,
        filename,
        sourceBytes: Buffer.byteLength(source),
        sourceSha256: sha(source),
        options,
      });
      try {
        const result = native.lintPatinaSfc(source, options);
        const diagnostics =
          result && typeof result === "object" && "diagnostics" in result
            ? (result as { diagnostics?: unknown }).diagnostics
            : undefined;
        event({
          event: "call-end",
          sequence,
          filename,
          diagnosticsCount: Array.isArray(diagnostics) ? diagnostics.length : null,
          diagnostics: Array.isArray(diagnostics) ? diagnostics : null,
        });
        return result;
      } catch (error) {
        event({
          event: "call-threw",
          sequence,
          filename,
          error: error instanceof Error ? error.stack : String(error),
        });
        throw error;
      }
    },
    "src",
    { index: shardIndex, total: 8 },
  );
  const report = process.report.getReport() as { sharedObjects?: string[] };
  const loadedNapiPaths = (report.sharedObjects ?? []).filter((name) => name.endsWith(".node"));
  event({
    event: "worker-end",
    calls: sequence,
    files: results.map((r) => r.filename),
    diagnosticsCount: results.reduce((sum, r) => sum + r.diagnostics.length, 0),
    loadedNapiPaths,
  });
  process.exitCode = results.some((r) => r.diagnostics.length > 0) ? 1 : 0;
}
