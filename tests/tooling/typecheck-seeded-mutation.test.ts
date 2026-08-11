import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { runSeededTypecheckMutation } from "../../tools/fixtures/typecheck-seeded-mutation.mjs";
import { typecheckDependencySkip } from "./support/typecheck-dependency.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const dependencyRoot =
  process.env.VIZE_TEST_WORKSPACE_NODE_MODULES ?? path.join(root, "tests/node_modules");
const vizeBin = process.env.VIZE_TEST_BIN;
const vueTscBin = path.join(dependencyRoot, ".bin/vue-tsc");
const corsaPath = process.env.CORSA_PATH ?? path.join(dependencyRoot, ".bin/tsgo");
const skip =
  [
    typecheckDependencySkip(
      vizeBin,
      "Vize binary for seeded typecheck parity",
      "Vize binary unavailable",
    ),
    typecheckDependencySkip(
      fs.existsSync(vueTscBin),
      "vue-tsc for seeded typecheck parity",
      "vue-tsc unavailable",
    ),
    typecheckDependencySkip(
      fs.existsSync(corsaPath),
      "tsgo for seeded typecheck parity",
      "tsgo unavailable",
    ),
  ].find((reason) => reason !== false) ?? false;

test("seeded clean broken repaired SFC is exact in Vize and vue-tsc", { skip }, () => {
  const temp = fs.realpathSync(fs.mkdtempSync(path.join(os.tmpdir(), "vize-seeded-parity-")));
  const fixtureRoot = path.join(temp, "fixture");
  fs.mkdirSync(fixtureRoot);
  fs.symlinkSync(dependencyRoot, path.join(fixtureRoot, "node_modules"), "dir");
  fs.writeFileSync(
    path.join(fixtureRoot, "tsconfig.json"),
    `${JSON.stringify(
      {
        compilerOptions: {
          allowArbitraryExtensions: true,
          module: "ESNext",
          moduleResolution: "Bundler",
          skipLibCheck: true,
          strict: true,
          target: "ES2022",
          types: [],
        },
      },
      null,
      2,
    )}\n`,
  );
  try {
    const evidence = runSeededTypecheckMutation({
      fixtureRoot,
      project: {
        id: "seeded-real",
        tsconfig: "tsconfig.json",
        typecheckPerformance: { hangTimeoutMs: 30_000 },
      },
      vizeBin,
      vueTscBin,
    });
    assert.deepEqual(
      evidence.states.map((state: { state: string }) => state.state),
      ["clean", "broken", "repaired"],
    );
    assert.equal(evidence.states[1].divergence.summary.sharedCount, 1);
    assert.equal(evidence.states[1].divergence.shared[0].code, 2322);
    assert.equal(evidence.states[1].divergence.summary.falsePositiveCount, 0);
    assert.equal(evidence.states[1].divergence.summary.falseNegativeCount, 0);
    assert.equal(fs.existsSync(path.join(fixtureRoot, ".vize-typecheck-parity-seed.vue")), false);
    assert.equal(
      fs.existsSync(path.join(fixtureRoot, ".vize-typecheck-parity-seed.tsconfig.json")),
      false,
    );
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
});
