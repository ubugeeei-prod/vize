import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { captureAuthoredOracle } from "./support/n8n-cli-config-oracle.mjs";
import { projection, root } from "./support/n8n-cli-config-inputs.mjs";

const relative = "tests/_fixtures/differential/lint/component-registration-context-8142";
const fixture = JSON.parse(fs.readFileSync(path.join(root, relative, "cases.json"), "utf8"));
const expected = JSON.parse(fs.readFileSync(path.join(root, relative, "independent.json"), "utf8"));
const sha256 = (source) => createHash("sha256").update(source).digest("hex");

await test("explicit component context retains all complete original 51-rule vectors", async () => {
  const evidence = { records: [], qualified: 0 };
  const output = path.join(root, "target/differential/component-registration-context-oracle.json");
  fs.mkdirSync(path.dirname(output), { recursive: true });
  const persist = () => fs.writeFileSync(output, JSON.stringify(evidence, null, 2) + "\n");
  persist();
  assert.equal(fixture.cases.length, 30);
  assert.equal(expected.records.length, 60);
  for (const input of fixture.cases) {
    assert.equal(sha256(input.source), input.sourceSha256, input.id);
    const observations = [];
    for (const repeat of [0, 1]) {
      const actual = await captureAuthoredOracle({
        benchmarkManifest:
          process.env.VIZE_SLOT_VALIDITY_ORACLE_MANIFEST ??
          path.join(root, "tools/benchmarks/scripts/package.json"),
        fixture: { cases: [input], optionControls: [] },
        baseRuleOptions: {
          ...projection.linter.ruleOptions,
          "vue/require-component-registration": input.referenceOptions,
        },
        officialVueBase: true,
        filePrefix: relative,
        onRecorded: (packet) => {
          evidence.current = { id: input.id, repeat, packet };
          persist();
        },
      });
      evidence.records.push({ id: input.id, repeat, capture: actual.recorded });
      persist();
      assert.equal(actual.raw.loadError, undefined, input.id);
      const golden = expected.records.find(
        (record) => record.id === input.id && record.repeat === repeat,
      );
      assert.ok(golden, input.id);
      assert.deepEqual(actual.recorded, golden.capture, input.id);
      assert.equal(actual.recorded.captures.length, 1);
      assert.equal(actual.recorded.captures[0].error, undefined, input.id);
      assert.equal(Object.keys(actual.recorded.configurations[0].rules).length, 51);
      observations.push(actual.recorded);
      evidence.qualified++;
      persist();
    }
    assert.deepEqual(observations[0], observations[1], input.id);
  }
  assert.equal(evidence.qualified, 60);
});
