// Publish the canonical listener contracts as the content mapper's
// `virtual-ts-listeners.d.ts`; `virtual-ts.d.*` re-export it. `--check`
// fails when the published copy is stale instead of rewriting it.
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { format } from "oxfmt";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const canonical = readFileSync(
  path.join(root, "crates/vize_canon/src/virtual_ts/helpers/declarations/listener-contracts.d.ts.txt"),
  "utf8",
);
const prefix = 'declare module "vize:virtual-ts" {\n';
assert.ok(canonical.startsWith(prefix) && canonical.endsWith("\n}\n"), "module wrapper");
const body = canonical.slice(prefix.length, -"}\n".length);
const published = await format(
  "virtual-ts-listeners.d.ts",
  [
    "// Generated from the canonical listener contracts. Do not edit.",
    'import type { $IsAny } from "./generated/content-mapper-types.js" with { "resolution-mode": "import" };',
    "",
    body,
  ].join("\n"),
  { printWidth: 100 },
);
assert.deepEqual(published.errors, [], "canonical listener contracts must parse");
const target = path.join(root, "npm/cli/virtual-ts-listeners.d.ts");
if (process.argv.includes("--check")) {
  assert.equal(readFileSync(target, "utf8"), published.code, "virtual-ts-listeners.d.ts is stale");
} else {
  writeFileSync(target, published.code);
}
