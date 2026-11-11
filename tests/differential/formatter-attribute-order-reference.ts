// #12826 sorts static-argument bindings and custom directives by name; only a
// bare v-bind/v-on object or a dynamic argument bounds a run. Closed original
// cases keep their historical bytes and mismatch; these are the current bytes.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { sha256 } from "./manifest.mjs";
import { assertClosedFormatterManifest } from "./formatter-sole-child-width-reference.mjs";

const DIRECTORY = "tests/_fixtures/differential/formatter-regressions/attribute-order-12826";
type Pinned = { input: string; historical: string; current: string };
type Fixture = {
  id: string;
  input: Buffer;
  expected: Buffer;
  argv: string[];
  config: { length: number };
};

const REFERENCES = new Map<string, Pinned>([
  [
    "formatter/sfc/directive-expression-indent-width",
    {
      input: "5f18216d74943dd4c4363314554031701b69082214b1aa79a376dd2083266e59",
      historical: "5f18216d74943dd4c4363314554031701b69082214b1aa79a376dd2083266e59",
      current: "431c344fddc54f613cccedd98d324453b7d65b9c80e6017c91ed8bf6d2b72b9e",
    },
  ],
  [
    "formatter/sfc/l1-directive-classification",
    {
      input: "cdbdabcd8e44d223982c2834c197623715f1b2f9d5c7d53652499df92807abc5",
      historical: "dc7abe7c89c59653f849b7f474bc849d4a2a91a6a6e2363247fde47d3087b98d",
      current: "e0113fce08cff23ba57ea650741fd7864b15469253ff346b0259942ebbe0c7a4",
    },
  ],
]);

export function attributeOrderCurrentReference(root: string, fixture: Fixture) {
  const pinned = REFERENCES.get(fixture.id);
  if (!pinned) return {};
  assertClosedFormatterManifest(root);
  assert.equal(sha256(fixture.input), pinned.input, "whole original input changed");
  assert.equal(sha256(fixture.expected), pinned.historical, "whole historical reference changed");
  assert.deepEqual(fixture.argv, ["fmt", "--no-config", "--write", "App.vue"]);
  assert.equal(fixture.config.length, 0, "default original configuration changed");
  const relative = `${DIRECTORY}/${fixture.id.slice("formatter/sfc/".length)}.current.expected.txt`;
  const file = fs.realpathSync(path.join(root, relative));
  const contained = path.relative(fs.realpathSync(root), file);
  assert(contained && !contained.startsWith("..") && !path.isAbsolute(contained));
  const currentExpected = fs.readFileSync(file);
  assert.equal(sha256(currentExpected), pinned.current, "whole current reference changed");
  assert(!currentExpected.equals(fixture.expected), "historical mismatch remains observable");
  return {
    currentExpected,
    currentReference: {
      issue: 12826,
      path: relative,
      inputSha256: pinned.input,
      historicalExpectedSha256: pinned.historical,
      currentExpectedSha256: pinned.current,
    },
  };
}
