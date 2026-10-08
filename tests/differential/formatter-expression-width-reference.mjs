// One closed current CLI reference; original sixteen-case manifest stays exact.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";

const INPUT = "851c54f3ec30d67d54dc5b6369b06303f3222583da08d873c1144cf3a1a8cb52";
const OUTPUT = "b2fb731eb8b2376c016442868a4f632d62cc617c53d50e7d4ffa40a073e28b16";
const AUTHORITY =
  "tests/_fixtures/differential/formatter-regressions/expression-print-width-7876/corpus.json";
const AUTHORITY_HASH = "b986b95eecfa3f2df1d653695f27dc330f3229b22ac7c335c15a2ecb4bb44523";

export function expressionWidthReference(root, fixture) {
  if (fixture.id !== "formatter/sfc/directive-prefix-print-width") return {};
  const manifest = fs.readFileSync(
    path.join(root, "tests/_fixtures/differential/formatter/manifest.json"),
  );
  assert.equal(
    sha256(manifest),
    "6ab3a71cbb9ec75a4073a315c1f9610b12673c60ff2178517aed94cbb60621e0",
    "whole original sixteen-case manifest changed",
  );
  assert.equal(sha256(fixture.input), INPUT, "whole original deep input changed");
  assert.equal(sha256(fixture.expected), INPUT, "whole original partial reference changed");
  assert.deepEqual(fixture.argv, ["fmt", "--no-config", "--write", "App.vue"]);
  assert.equal(fixture.config.length, 0, "default original configuration changed");
  const authority = fs.readFileSync(path.join(root, AUTHORITY));
  assert.equal(sha256(authority), AUTHORITY_HASH, "closed expression-width authority changed");
  const file = fs.realpathSync(
    path.join(path.dirname(path.join(root, AUTHORITY)), "original-deep-lf.expected"),
  );
  const relative = path.relative(fs.realpathSync(root), file);
  assert(relative && !relative.startsWith("..") && !path.isAbsolute(relative));
  const currentExpected = fs.readFileSync(file);
  assert.equal(sha256(currentExpected), OUTPUT, "whole independent current reference changed");
  return {
    currentExpected,
    currentReference: {
      issue: 7876,
      authority: { path: AUTHORITY, sha256: AUTHORITY_HASH },
      inputSha256: INPUT,
      historicalExpectedSha256: INPUT,
      currentExpectedSha256: OUTPUT,
    },
  };
}
