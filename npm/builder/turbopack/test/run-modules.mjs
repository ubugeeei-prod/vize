import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { createRequire } from "node:module";
import { consumer, output, sha256 } from "./prepare-consumer.mjs";

/** Retain full adapter packets and uncached timings for the actual host SFC. */
export async function checkPackedModules(filename, source, measurements) {
  const require = createRequire(path.join(consumer, "package.json"));
  const { compileTurbopackSfc } = require("@vizejs/turbopack-plugin/loader");
  for (const [label, options] of [
    ["client-development", { ssr: false, isProduction: false }],
    ["server-development", { ssr: true, isProduction: false }],
    ["client-production", { ssr: false, isProduction: true }],
    ["server-production", { ssr: true, isProduction: true }],
    ["client-production-without-maps", { ssr: false, isProduction: true, sourceMap: false }],
  ]) {
    const samplesMs = [];
    let baseline;
    for (let sample = 0; sample < 5; sample++) {
      const start = performance.now();
      const packet = compileTurbopackSfc(filename, source, {
        root: consumer,
        styleLanguages: ["css", "scss"],
        ...options,
      });
      samplesMs.push(performance.now() - start);
      const bytes = JSON.stringify(packet, null, 2) + "\n";
      if (baseline !== undefined)
        assert.equal(bytes, baseline, `${label}: full module packet must be deterministic`);
      else baseline = bytes;
      if (options.sourceMap === false) assert.equal(packet.map, null);
    }
    await fs.writeFile(path.join(output, `module-${label}.json`), baseline);
    measurements.push({
      phase: `module-${label}`,
      samplesMs,
      sourceBytes: Buffer.byteLength(source),
      resultBytes: Buffer.byteLength(baseline),
      sha256: sha256(baseline),
    });
  }
}
