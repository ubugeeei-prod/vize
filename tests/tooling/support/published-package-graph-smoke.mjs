import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { installPackageGraph } from "../../../tools/support/release/lint_reports/package_graph.mjs";

// Actual public npm archives, without source/native overrides or postinstall.
const root = fs.realpathSync(
  fs.mkdtempSync(path.join(process.env.RUNNER_TEMP ?? os.tmpdir(), "published-package-graph-")),
);
const consumer = path.join(root, "consumer");
const evidence = path.join(root, "evidence");
fs.mkdirSync(consumer);
fs.mkdirSync(evidence);
console.log(root);
const result = await installPackageGraph(consumer, evidence, {
  "@parcel/watcher-wasm": "2.6.0",
  "@isaacs/cliui": "8.0.2",
  "string-width-cjs": "npm:string-width@4.2.3",
});
const bundled = result.records.find((record) => record.metadata.name === "napi-wasm");
assert.equal(bundled.metadata.version, "1.1.0");
assert.equal(bundled.bundledIn.location, "node_modules/@parcel/watcher-wasm");
assert.equal(bundled.members.length, 4);
const alias = result.records.find((record) => record.location === "node_modules/string-width-cjs");
assert.equal(alias.metadata.name, "string-width");
assert.equal(alias.metadata.version, "4.2.3");
const transitive = result.records.find(
  (record) => record.location === "node_modules/strip-ansi-cjs",
);
assert.equal(transitive.metadata.name, "strip-ansi");
result.assertDependencies();
console.log(
  JSON.stringify({
    records: result.records.length,
    bundled: bundled.metadata,
    alias: alias.metadata.name,
  }),
);
