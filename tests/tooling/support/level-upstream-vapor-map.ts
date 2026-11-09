import assert from "node:assert/strict";
import process from "node:process";
import { officialCompilerVapor } from "./vue-vapor-release.mjs";

const chunks = [];
for await (const chunk of process.stdin) chunks.push(chunk);
const { source, filename } = JSON.parse(Buffer.concat(chunks).toString("utf8"));
assert.equal(typeof source, "string");
assert.ok(source.length > 0);
assert.equal(typeof filename, "string");
assert.ok(filename.length > 0);

const result = officialCompilerVapor.compile(source, {
  mode: "module",
  prefixIdentifiers: true,
  sourceMap: true,
  filename,
});
assert.ok(result.map);
process.stdout.write(`${JSON.stringify(result.map)}\n`);
