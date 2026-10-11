import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  previousCapture7502,
  previousOutputSha7502,
  previousOutputUrl7502,
  hash7502,
  unchangedRows7502,
  transition7502,
  requireReviewed7502,
} from "./support/native-attribute-values-7502-inputs.ts";
import { validateCapture7502 } from "./support/native-attribute-values-7502-oracle.ts";
import { historyCommands7502 } from "./support/native-attribute-values-7502-history.ts";

await test("Vapor class successor changes only two results and retains every whole other field", () => {
  assert.equal(hash7502(readFileSync(previousOutputUrl7502)), previousOutputSha7502);
  const prior = previousCapture7502();
  assert.throws(() => validateCapture7502(prior), assert.AssertionError);
  const relabelled = structuredClone(prior);
  relabelled.version = 3;
  relabelled.transition = transition7502;
  relabelled.summary = {
    fixtures: 14,
    outcomes: 84,
    positive: 78,
    lowerRefusals: 6,
    targetRefusals: 0,
  };
  assert.throws(() => validateCapture7502(relabelled), assert.AssertionError);
  const packet = structuredClone(prior);
  packet.transition = transition7502;
  // Synthetic results exercise only the source law, never native/output acceptance.
  const changed = packet.rows.filter(
    (row: any) => row.id === transition7502.input && row.target === "vapor",
  );
  assert.equal(changed.length, 2);
  for (const row of changed) row.result = { synthetic: "source-law-only" };
  unchangedRows7502(packet);
  for (const [index, row] of packet.rows.entries()) {
    if (row.id === transition7502.input && row.target === "vapor") continue;
    const counterfactual = structuredClone(packet);
    counterfactual.rows[index].observation.file.nodeCount++;
    assert.throws(() => unchangedRows7502(counterfactual), assert.AssertionError);
  }
  for (const row of changed) {
    for (const key of Object.keys(row).filter((key) => key !== "result")) {
      const counterfactual = structuredClone(packet);
      const target = counterfactual.rows.find(
        (candidate: any) =>
          candidate.id === row.id &&
          candidate.target === row.target &&
          candidate.linkMode === row.linkMode,
      );
      target[key] = { forged: true };
      assert.throws(() => unchangedRows7502(counterfactual), assert.AssertionError);
    }
  }
});

await test("the actual original L3 class refusal is an additional mandatory pinned process", () => {
  const commands = historyCommands7502("/baseline", "/history");
  assert.equal(commands.length, 13);
  assert.deepEqual(commands.at(-1), [
    "cargo",
    [
      "test",
      "--locked",
      "--profile",
      "ci",
      "-p",
      "vize_l3",
      "--lib",
      "decision::vapor::tests::actual_whitespace_headers_tags_and_dynamic_ops_refuse_before_output",
      "--",
      "--exact",
      "--nocapture",
    ],
  ]);
});

await test("current whole V3 output is deliberately unqualified until independently frozen", () => {
  const fixture = JSON.parse(
    readFileSync(
      new URL(
        "../../crates/vize_atelier_sfc/tests/fixtures/native_attribute_values_7502/reviewed_output_v3.json",
        import.meta.url,
      ),
      "utf8",
    ),
  );
  assert.equal(fixture.version, 3);
  if (fixture.state === "unfrozen") {
    assert.equal(fixture.capture, null);
    assert.throws(() => requireReviewed7502(previousCapture7502(), fixture), /remain unfrozen/);
  } else assert.equal(fixture.state, "reviewed");
});
