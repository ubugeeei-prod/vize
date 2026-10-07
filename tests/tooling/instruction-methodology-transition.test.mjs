import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  checkMeasurement,
  loadBudgets,
  ratchetBudgets,
  validateMeasurement,
} from "../../tools/benchmarks/scripts/instruction-counts-lib.mjs";
import {
  authenticateInstructionMeasurementSource,
  verifyRepositoryInstructionBudgets,
} from "../../tools/benchmarks/scripts/instruction-methodology-transition.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/tooling/instruction-methodology-transition");
const source = "1a3996963af208c13cd732420bdee586df88bf7a";
const context = { sourceCommit: source, toolchain: "1.99.0" };
const families = [
  {
    name: "level",
    active: "instruction-budgets.toml",
    count: 100,
    oldHash: "3c221a56f364a4aaf352729d9b9d0cf1eba2c7e4aa088174fdb6b6099bddb5aa",
    reportHash: "61803d7a6d12b6ed3e29168d4f61b91bb3bbdee7d93ae48a28364b481bb07f2c",
  },
  {
    name: "formatter",
    active: "formatter-instruction-budgets.toml",
    count: 4,
    oldHash: "e49d7c38a43fcdfa3bf7114c50ef91734f355d943269d42bc1d18821d444de99",
    reportHash: "100823ff2a4cf4a4af7c493912e6653e465332c38e8a154a55e3505f3caebeb4",
  },
];
function packet(family) {
  const old = loadBudgets(path.join(fixture, `${family.name}-rust198.toml`));
  const registry = new Set(Object.keys(old.instruction));
  const current = loadBudgets(path.join(root, "docs/davinci/plan", family.active), registry);
  const raw = fs.readFileSync(path.join(fixture, `${family.name}-rust199.json`));
  const report = validateMeasurement(JSON.parse(raw), registry);
  return { old, current, report, registry };
}
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const cli = (args) =>
  spawnSync(process.execPath, ["tools/benchmarks/scripts/instruction-counts.mjs", ...args], {
    cwd: root,
    encoding: "utf8",
  });

for (const family of families) {
  void test(`${family.name}: whole original and actual archived evidence stays byte-exact`, () => {
    assert.equal(
      digest(fs.readFileSync(path.join(fixture, `${family.name}-rust198.toml`))),
      family.oldHash,
    );
    assert.equal(
      digest(fs.readFileSync(path.join(fixture, `${family.name}-rust199.json`))),
      family.reportHash,
    );
    const { old, current, report, registry } = packet(family);
    assert.equal(registry.size, family.count);
    assert.deepEqual(current.instruction, old.instruction);
    assert.equal(current.source_commit, source);
    assert.equal(current.recorded_run, report.recorded_run);
    ratchetBudgets(current, old);
    authenticateInstructionMeasurementSource(report, context);
    checkMeasurement(report, current);
    // This explicit context authenticates the archived producer, not our HEAD.
    assert.equal(report.source_commit, source);
    for (const run of report.runs) {
      assert.equal(Object.keys(run).length, family.count);
      for (const [id, row] of Object.entries(run)) {
        for (const key of ["fixture", "fixture_sha256", "window"]) {
          assert.equal(row[key], old.instruction[id][key], `${id}/${key}`);
        }
        assert.ok(row.instructions <= old.instruction[id].instructions, id);
      }
    }
  });

  void test(`${family.name}: each original input identity and every cap remain immutable in migration`, () => {
    const { old, current } = packet(family);
    for (const id of Object.keys(old.instruction)) {
      for (const key of ["fixture", "fixture_sha256", "window", "instructions"]) {
        const candidate = structuredClone(current);
        candidate.instruction[id][key] =
          key === "instructions"
            ? candidate.instruction[id][key] + 1
            : key === "fixture_sha256"
              ? "0".repeat(64)
              : `${candidate.instruction[id][key]}-changed`;
        assert.throws(
          () => ratchetBudgets(candidate, old),
          /preserve every original cap and identity/,
          `${id}/${key}`,
        );
        const forgedBase = structuredClone(old);
        forgedBase.instruction[id][key] = candidate.instruction[id][key];
        assert.throws(
          () => ratchetBudgets(candidate, forgedBase),
          /original instruction contract changed/,
          `${id}/${key}/both`,
        );
      }
    }
    const lowered = structuredClone(current);
    lowered.instruction[Object.keys(old.instruction)[0]].instructions -= 1;
    assert.throws(() => ratchetBudgets(lowered, old), /preserve every original cap and identity/);
    // Once the same methodology is established, the original downward ratchet still works.
    ratchetBudgets(lowered, current);
  });

  void test(`${family.name}: missing, foreign and extra rows cannot replace a complete inventory`, () => {
    const { old, current } = packet(family);
    const id = Object.keys(old.instruction)[0];
    const missing = structuredClone(current);
    delete missing.instruction[id];
    assert.throws(() => ratchetBudgets(missing, old), /preserve every original cap and identity/);
    const bothMissing = structuredClone(old);
    delete bothMissing.instruction[id];
    assert.throws(() => ratchetBudgets(missing, bothMissing), /original inventory changed/);
    const extra = structuredClone(current);
    extra.instruction.foreign_probe = structuredClone(current.instruction[id]);
    assert.throws(() => ratchetBudgets(extra, old), /preserve every original cap and identity/);
    const foreign = packet(families.find((item) => item.name !== family.name));
    assert.throws(
      () => ratchetBudgets(foreign.current, old),
      /preserve every original cap and identity/,
    );
  });

  void test(`${family.name}: every non-rustc method field and literal compiler direction are pinned`, () => {
    const { old, current } = packet(family);
    for (const key of Object.keys(old.methodology).filter((item) => item !== "rustc")) {
      const base = structuredClone(old);
      const candidate = structuredClone(current);
      base.methodology[key] += "-changed";
      candidate.methodology[key] += "-changed";
      assert.throws(
        () => ratchetBudgets(candidate, base),
        /original instruction contract changed/,
        key,
      );
      assert.throws(
        () => ratchetBudgets(candidate, old),
        /preserve every original cap and identity/,
        key,
      );
    }
    assert.throws(() => ratchetBudgets(old, current), /unauthorized original instruction compiler/);
    for (const compiler of [
      "rustc 1.100.0 (future)",
      "rustc 1.99.0 (wrong build)",
      "rustc newer",
    ]) {
      const candidate = structuredClone(current);
      candidate.methodology.rustc = compiler;
      assert.throws(
        () => ratchetBudgets(candidate, old),
        /unauthorized forward instruction compiler/,
      );
    }
    const older = structuredClone(old);
    older.methodology.rustc = "rustc 1.97.0 (old)";
    assert.throws(
      () => ratchetBudgets(current, older),
      /unauthorized original instruction compiler/,
    );
  });

  void test(`${family.name}: complete baseline and refreshed registry provenance cannot be substituted`, () => {
    const { old, current } = packet(family);
    for (const key of ["source_commit", "recorded_run"]) {
      const value =
        key === "source_commit"
          ? "a".repeat(40)
          : "https://github.com/ubugeeei-prod/vize/actions/runs/1";
      const base = structuredClone(old);
      base[key] = value;
      assert.throws(
        () => ratchetBudgets(current, base),
        /unauthenticated original instruction provenance/,
      );
      const candidate = structuredClone(current);
      candidate[key] = value;
      assert.throws(
        () => ratchetBudgets(candidate, old),
        /preserve every original cap and identity/,
      );
    }
    const extra = { ...current, unchecked_transition: true };
    assert.throws(() => ratchetBudgets(extra, old), /preserve every original cap and identity/);
  });

  void test(`${family.name}: current measurement requires exact checkout and pinned Rust, preserving regression refusal`, () => {
    const { current, report } = packet(family);
    authenticateInstructionMeasurementSource(report, context);
    assert.throws(
      () =>
        authenticateInstructionMeasurementSource(report, {
          ...context,
          sourceCommit: "a".repeat(40),
        }),
      /not current checkout/,
    );
    assert.throws(
      () => authenticateInstructionMeasurementSource(report, { ...context, sourceCommit: "main" }),
      /invalid current checkout identity/,
    );
    for (const toolchain of ["1.98.0", "1.100.0", "stable"]) {
      assert.throws(
        () => authenticateInstructionMeasurementSource(report, { ...context, toolchain }),
        /must be 1.99.0/,
      );
    }
    const wrongCompiler = structuredClone(report);
    wrongCompiler.methodology.rustc = "rustc 1.99.0 (wrong build)";
    assert.throws(
      () => authenticateInstructionMeasurementSource(wrongCompiler, context),
      /measurement compiler mismatch/,
    );
    const id = Object.keys(current.instruction)[0];
    const regression = structuredClone(report);
    for (const run of regression.runs)
      run[id].instructions = current.instruction[id].instructions + 1;
    assert.throws(() => checkMeasurement(regression, current), /instruction budget exceeded/);
  });

  void test(`${family.name}: CLI accepts the authorized ratchet and refuses historical reports as current source`, () => {
    const mode = family.name === "formatter" ? ["--formatter"] : [];
    const verified = cli([
      ...mode,
      "--verify-budgets",
      "--base-budgets",
      path.join(fixture, `${family.name}-rust198.toml`),
    ]);
    assert.equal(verified.status, 0, verified.stderr);
    assert.match(
      verified.stdout,
      new RegExp(`${family.count} pinned ceilings and ratchet verified`),
    );
    const replay = cli([
      ...mode,
      "--check",
      "--measurement",
      path.join(fixture, `${family.name}-rust199.json`),
    ]);
    assert.equal(replay.status, 1);
    assert.match(replay.stderr, /measurement source is not current checkout/);
  });
}

void test("repository source pin remains strict independently of an archived report", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "instruction-source-pin-"));
  try {
    const { current } = packet(families[0]);
    for (const channel of ["1.98.0", "1.100.0", "stable"]) {
      fs.writeFileSync(
        path.join(temporary, "rust-toolchain.toml"),
        `[toolchain]\nchannel = "${channel}"\n`,
      );
      assert.throws(() => verifyRepositoryInstructionBudgets(current, temporary), /must be 1.99.0/);
    }
    fs.writeFileSync(
      path.join(temporary, "rust-toolchain.toml"),
      '[toolchain]\nchannel = "1.99.0"\n',
    );
    verifyRepositoryInstructionBudgets(current, temporary);
    const wrong = structuredClone(current);
    wrong.methodology.rustc = "rustc 1.98.0 (88d9e12ae 2026-08-18)";
    assert.throws(
      () => verifyRepositoryInstructionBudgets(wrong, temporary),
      /budget compiler mismatch/,
    );
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
