import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { readFileSync } from "node:fs";

const fixture = JSON.parse(process.argv[2]);

const { loadFrozenTemplateCompiler } = await import(fixture.compilerModule);
const provider = JSON.parse(readFileSync(fixture.boundaryFile, "utf8")).compilerProvider;
const foreignRequire = createRequire(process.env.VIZE_LEGACY_API_VUE_MANIFEST);
const installed = foreignRequire("vue/package.json").version;
const runtimeVersion = foreignRequire("vue").version;
assert.equal(runtimeVersion, installed);
assert.notEqual(installed, provider.vue, "the control must install a genuinely different Vue");
assert.throws(
  () => loadFrozenTemplateCompiler("template-boundaries.json", provider, "dom"),
  assert.AssertionError,
);
process.stdout.write(
  JSON.stringify({ installed, runtimeVersion, expected: provider.vue, refused: true }),
);
