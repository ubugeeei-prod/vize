import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { test } from "node:test";
import { parse } from "yaml";

const read = (file: string) => fs.readFileSync(new URL(file, import.meta.url), "utf8");
const originalBytes = read("./fixtures/real-project-matrix-original-67e3.yml");
assert.equal(
  createHash("sha256").update(originalBytes).digest("hex"),
  "3f5650d943dfbb73d9cea41782499c6225938d7ce4330549d4c27f7f90ea267c",
);
const prefix =
  "node tests/differential/build-receipt.ts\n" +
  'node tests/tooling/support/lsp/verify-session-captures.ts snapshot "$FIXTURE_REPORT_DIR/lsp-wire-before.json"\n' +
  "printf '%s\\n' 'Lifecycle timings include passive raw RPC recording; they are not paired performance acceptance.' > \"$FIXTURE_REPORT_DIR/lsp-wire-timing-policy.txt\"\n";

test("passive native LSP capture retains the complete original Matrix workflow", () => {
  const original = parse(originalBytes);
  const current = parse(read("../../.github/workflows/real-project-matrix.yml"));
  const steps = current.jobs["real-project-matrix"].steps;
  const oldSteps = original.jobs["real-project-matrix"].steps;
  const lsp = steps.find((step) => step.id === "lsp");
  const oldLsp = oldSteps.find((step) => step.id === "lsp");
  assert.ok(lsp && oldLsp);
  assert.deepEqual(lsp.env, {
    ...oldLsp.env,
    VIZE_LSP_REQUIRE_SOURCE_BUILD: "1",
    VIZE_LSP_BUILD_RECIPE: "cargo build --profile ci -p vize",
  });
  assert.equal(lsp.run, prefix + oldLsp.run);
  delete lsp.env.VIZE_LSP_REQUIRE_SOURCE_BUILD;
  delete lsp.env.VIZE_LSP_BUILD_RECIPE;
  lsp.run = lsp.run.slice(prefix.length);

  const integrity = steps.findIndex((step) => step.name === "Validate source-bound LSP wire");
  assert.equal(integrity, steps.findIndex((step) => step.id === "lsp") + 1);
  assert.deepEqual(steps.splice(integrity, 1)[0], {
    name: "Validate source-bound LSP wire",
    if: "${{ always() }}",
    run: 'node tests/tooling/support/lsp/verify-session-captures.ts verify "$FIXTURE_REPORT_DIR/lsp-wire-before.json" "$FIXTURE_REPORT_DIR/lsp-wire-integrity.json"',
  });

  const index = steps.findIndex((step) => step.name === "Upload source-bound LSP wire evidence");
  assert.ok(index >= 0);
  const [wire] = steps.splice(index, 1);
  assert.deepEqual(wire, {
    name: "Upload source-bound LSP wire evidence",
    if: "${{ always() }}",
    uses: "actions/upload-artifact@043fb46d1a93c77aae656e7c1c64a875d1fc6a0a",
    with: {
      name: "real-project-lsp-wire-${{ matrix.shard }}-${{ github.sha }}",
      path: "target/differential/lsp-sessions\n" + "target/ci/vize.differential-build.json\n",
      "if-no-files-found": "error",
      "retention-days": 30,
    },
  });
  // The release-gated title is the only other change: it names the exact
  // release dispatch beside the original enforced and observational titles.
  assert.match(
    original["run-name"],
    /'enforced' \|\| 'observational' \}\} @ \$\{\{ github\.sha \}\}$/,
  );
  assert.match(
    current["run-name"],
    /'enforced' \|\|[\s\S]*'release-gated'\) \|\| 'observational' \}\} @ \$\{\{ github\.sha \}\}$/,
  );
  current["run-name"] = original["run-name"];
  assert.deepEqual(current, original);
});
