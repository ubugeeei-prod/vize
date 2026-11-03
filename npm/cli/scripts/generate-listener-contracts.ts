// Publish the canonical listener contracts as `vize/virtual-ts-listeners`,
// the module hoisted content-mapper output imports them from.
// `--check` fails on a stale copy instead of writing it.
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { format } from "oxfmt";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const canonical = readFileSync(
  path.join(
    root,
    "crates/vize_canon/src/virtual_ts/helpers/declarations/listener-contracts.d.ts.txt",
  ),
  "utf8",
);
const prefix = 'declare module "vize:virtual-ts-listeners" {\n';
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
const outputs = new Map([["npm/cli/virtual-ts-listeners.d.ts", published.code]]);

for (const [relativePath, contents] of outputs) {
  const target = path.join(root, relativePath);
  if (process.argv.includes("--check")) {
    assert.equal(readFileSync(target, "utf8"), contents, `${relativePath} is stale`);
  } else {
    writeFileSync(target, contents);
  }
}
