import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import { createRequire } from "node:module";
import { runInThisContext } from "node:vm";
import { consumer, output, sha256 } from "./prepare-consumer.mjs";

/** Retain full adapter packets and uncached timings for the actual host SFC. */
export async function checkPackedModules(filename, source, measurements) {
  const require = createRequire(path.join(consumer, "package.json"));
  const { compileTurbopackSfc } = require("@vizejs/turbopack-plugin/loader");
  await checkMissingParsedAttrs(require, filename);
  for (const { label, ...options } of [
    { label: "client-development", ssr: false, isProduction: false },
    { label: "server-development", ssr: true, isProduction: false },
    { label: "client-production", ssr: false, isProduction: true },
    { label: "server-production", ssr: true, isProduction: true },
    {
      label: "client-production-without-maps",
      ssr: false,
      isProduction: true,
      sourceMap: false,
    },
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

/** An older native facade must fail closed instead of silently enabling Vapor. */
async function checkMissingParsedAttrs(require, filename) {
  const entry = require.resolve("@vizejs/turbopack-plugin/loader");
  const originalRequire = createRequire(entry);
  const native = originalRequire("@vizejs/native");
  let parsed = 0;
  const legacy = {
    ...native,
    parseSfc(source, options) {
      parsed++;
      const descriptor = native.parseSfc(source, options);
      for (const script of [descriptor.script, descriptor.scriptSetup])
        if (script) delete script.attrs;
      return descriptor;
    },
  };
  const module = { exports: {} };
  const execute = runInThisContext(
    `(function(exports,require,module,__filename,__dirname){${await fs.readFile(entry, "utf8")}\n})`,
    { filename: entry },
  );
  execute(
    module.exports,
    (specifier) => (specifier === "@vizejs/native" ? legacy : originalRequire(specifier)),
    module,
    entry,
    path.dirname(entry),
  );
  for (const source of [
    "<script vapor>export default {}</script><template/>",
    '<script setup vapor="false">const n=1;</script><template/>',
    "<script setup>const n=1;</script><template/>",
  ]) {
    assert.throws(
      () =>
        module.exports.compileTurbopackSfc(filename, source, {
          root: consumer,
          ssr: false,
          isProduction: true,
          styleLanguages: ["css"],
        }),
      /native parsed script attributes are unavailable/,
    );
  }
  assert.equal(parsed, 3, "the refusal must consume the original native parser result");
}
