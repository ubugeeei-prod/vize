import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { sha256 } from "./providers.ts";

const bytes = fs.readFileSync(0);
const input = JSON.parse(bytes.toString("utf8"));
assert.equal(sha256(fs.readFileSync(input.addon.path)), input.addon.sha256);
const native = createRequire(import.meta.url)(input.addon.path);
const results = input.steps.map((step: { id: string; source: string; options: unknown }) => ({
  id: step.id,
  result: native.lintPatinaSfc(step.source, step.options),
}));
assert.equal(sha256(fs.readFileSync(input.addon.path)), input.addon.sha256);
process.stdout.write(
  `${JSON.stringify({
    schema: "vize.native-runtime-napi-observation",
    version: 1,
    inputSha256: sha256(bytes),
    addon: input.addon,
    source: input.source,
    results,
  })}\n`,
);
