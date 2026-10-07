// Custody controls do not claim formatter execution.
import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  multiValueReference,
  multiValueComparisons,
} from "./support/css-multi-value-reference.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const base = path.join(root, "tests/_fixtures/differential/formatter-regressions");
const plans = ["css-continuation-indent-7915", "css-rule-layout-7926-7966"].flatMap((owner) => {
  const corpus = fs.readFileSync(path.join(base, owner, "cases.json"));
  return JSON.parse(corpus).cases.map((fixture) => {
    const input =
      owner === "css-continuation-indent-7915"
        ? Buffer.from(fixture.input)
        : fs.readFileSync(path.join(base, owner, fixture.source));
    const historical =
      owner === "css-continuation-indent-7915"
        ? Buffer.from(fixture.expected)
        : fs.readFileSync(path.join(base, owner, fixture.expected));
    return {
      owner,
      fixture,
      input,
      historical,
      corpus,
      qualification: multiValueReference(root, owner, fixture, input, historical, corpus),
    };
  });
});
const refined = plans.filter((plan) => plan.qualification.reference);

await test("12 exact current layouts retain all 29 original plans and independent stock output", () => {
  assert.equal(plans.length, 29);
  assert.equal(refined.length, 12);
  assert.equal(refined.filter((plan) => plan.owner === "css-continuation-indent-7915").length, 11);
  assert.equal(refined.filter((plan) => plan.owner === "css-rule-layout-7926-7966").length, 1);
  for (const plan of refined) {
    const comparisons = multiValueComparisons(
      plan.qualification,
      plan.historical,
      plan.qualification.expected,
    );
    assert.equal(comparisons.referenceComparison.state, "different");
    assert.equal(comparisons.currentReferenceComparison.state, "equal");
    assert.throws(() =>
      multiValueComparisons(plan.qualification, plan.historical, plan.historical),
    );
  }
  const stock = JSON.parse(
    fs.readFileSync(path.join(base, "css-continuation-indent-7915/stock-prettier.json")),
  );
  for (const row of stock.rows) {
    const options = JSON.stringify(row.options);
    const id =
      options === '{"tabWidth":2}'
        ? "original-default-control"
        : options === '{"tabWidth":4}'
          ? "original-four-spaces"
          : options === '{"useTabs":true}'
            ? "original-tabs"
            : options === '{"useTabs":true,"tabWidth":4}'
              ? "original-tabs-four"
              : "original-four-crlf";
    const plan = refined.find((plan) => plan.fixture.id === id);
    assert.equal(plan.qualification.expected.toString(), row.output);
  }
});

await test("current layout qualification rejects original byte/options/CLI/authority forgery", (t) => {
  for (const plan of refined) {
    for (const [fixture, input, historical, corpus] of [
      [plan.fixture, Buffer.from("foreign input"), plan.historical, plan.corpus],
      [plan.fixture, plan.input, plan.qualification.expected, plan.corpus],
      [plan.fixture, plan.input, plan.historical, Buffer.from("{}")],
      [
        { ...plan.fixture, options: { ...plan.fixture.options, printWidth: 200 } },
        plan.input,
        plan.historical,
        plan.corpus,
      ],
      [{ ...plan.fixture, cliExpected: [] }, plan.input, plan.historical, plan.corpus],
    ])
      assert.throws(() =>
        multiValueReference(root, plan.owner, fixture, input, historical, corpus),
      );
  }
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "css-multi-value-authority-"));
  t.after(() => fs.rmSync(temporary, { recursive: true, force: true }));
  const plan = refined[0];
  const authority = path.join(temporary, plan.qualification.reference.authority.path);
  fs.mkdirSync(path.dirname(authority), { recursive: true });
  fs.writeFileSync(authority, "{}");
  assert.throws(() =>
    multiValueReference(
      temporary,
      plan.owner,
      plan.fixture,
      plan.input,
      plan.historical,
      plan.corpus,
    ),
  );
});
