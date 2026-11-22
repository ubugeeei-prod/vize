import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const require = createRequire(import.meta.url);
const sass = require("sass");
const rows = JSON.parse(fs.readFileSync(0, "utf8"));
const runner = fileURLToPath(
  new URL("../../../vize/src/commands/build/runner/preprocess.mjs", import.meta.url),
);
const jobs = rows.map(({ name, source }) => ({ file: resolve(name), lang: "scss", source }));
const stock = spawnSync(process.execPath, [runner], {
  input: JSON.stringify(jobs),
  encoding: "utf8",
  maxBuffer: 32 << 20,
});
assert.equal(stock.status, 0, stock.stderr);
const output = JSON.parse(stock.stdout);
assert.equal(output.length, rows.length);
for (const [index, row] of rows.entries()) {
  assert.deepEqual(output[index], { css: row.css }, JSON.stringify(row.source));
}
if (process.argv.includes("--correctness-only")) {
  process.stdout.write(JSON.stringify({ checked: rows.length }));
  process.exit(0);
}
const medians = [];
for (const row of rows) {
  const options = {
    syntax: "scss",
    url: pathToFileURL(resolve(row.name)),
    loadPaths: [dirname(resolve(row.name))],
  };
  assert.equal(row.css, sass.compileString(row.source, options).css, row.name);
  const samples = [];
  for (let i = 0; i < 35; i++) {
    const start = process.hrtime.bigint();
    const css = sass.compileString(row.source, options).css;
    const ns = Number(process.hrtime.bigint() - start);
    assert.equal(css, row.css, row.name);
    if (i >= 4) samples.push(ns);
  }
  samples.sort((left, right) => left - right);
  medians.push(samples[15]);
}
process.stdout.write(
  JSON.stringify({ checked: rows.length, medians, version: sass.info.split("\n")[0] }),
);
