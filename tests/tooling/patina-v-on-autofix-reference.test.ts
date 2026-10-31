import assert from "node:assert/strict";
import { test } from "node:test";
import { asRecord } from "../../tools/support/json-value.ts";
import {
  cases,
  provider,
  referenceFor,
  sourceFor,
} from "./support/patina-v-on-autofix-contract.ts";

test("twenty-two whole official autofix outcomes retain false defaults, exact edits and Unicode identity frontier", () => {
  const reference = provider();
  for (const row of cases) {
    const source = sourceFor(row),
      pinned = referenceFor(row);
    const options = { filename: row.file.replace(/\.txt$/u, "") };
    const config = reference.config(row);
    assert.deepEqual(reference.linter.verify(source, config, options), pinned.messages, row.id);
    const fixed = reference.linter.verifyAndFix(source, config, options);
    assert.deepEqual(fixed, pinned.fixResult, row.id);
    const output = asRecord(fixed).output;
    assert.equal(typeof output, "string");
    assert.deepEqual(
      reference.linter.verify(output as string, config, options),
      pinned.requery,
      row.id,
    );
    assert.deepEqual(
      reference.linter.verifyAndFix(output as string, config, options),
      pinned.secondFixResult,
      row.id,
    );
  }
  const identity = referenceFor(cases.find((row) => row.id === "Unicode.vue/never-true")!);
  assert.equal(asRecord(identity.secondFixResult).fixed, true);
  assert.equal(asRecord(identity.fixResult).output, asRecord(identity.secondFixResult).output);
});
