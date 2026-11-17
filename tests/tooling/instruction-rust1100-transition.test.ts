import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  checkMeasurement,
  loadBudgets,
  ratchetBudgets,
  validateBudgets,
  validateMeasurement,
} from "../../tools/benchmarks/scripts/instruction-counts-lib.mjs";
import {
  authenticateInstructionMeasurementSource,
  verifyRepositoryInstructionBudgets,
} from "../../tools/benchmarks/scripts/instruction-methodology-transition.mjs";

// The reviewed Rust 1.99.1 to 1.100.0 instruction re-baseline. The 1.98, 1.99.0
// and 1.99.1 steps stay in instruction-methodology-transition.test.mjs.
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/tooling/instruction-methodology-transition");
const rebaseSource = "e40813b3306bf85e5f61d96b19ebd432cae10334";
const rust1100Source = "aa9ab054fd272578f142dfd2c789d0e1869c7b86";
const context = { sourceCommit: rust1100Source, toolchain: "1.100.0" };
// Reviewed compiler-explained caps: under cap on 1.99.1 at the same source
// (run 43352512122), over it only on 1.100.0 (run 43351845175).
const raises1100: Record<string, number> = {
  armature_parse_medium: 244853,
  croquis_analyze_full_large: 2789255,
  "harness-selfcheck": 15481,
};
const families = [
  {
    name: "level",
    active: "instruction-budgets.toml",
    rust1991Hash: "f9e6751308604dc6f99dc8d686169a4a83c3e7e75eeb17c81febdce46949ac13",
    rust1100Hash: "4061584b8044b5c71b6b553e0e81e2161cccfc147774ed4c6018d448778f18bc",
  },
  {
    name: "formatter",
    active: "formatter-instruction-budgets.toml",
    rust1991Hash: "2c7104fe8349d316a2f0ba098637dd907d9f796fd37d009da031a5163abe6acc",
    rust1100Hash: "83c1e85383248600f006855cba8f8f53ea64213b70a6bedcc3211b7cddc7f233",
  },
];
type Family = (typeof families)[number];
const digest = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
// Archived 1.99.1 registry, committed 1.100.0 registry and its three-run same-input
// evidence. The archive replays under its own rows; active rows keep every
// fixture and window, and a cap may only fall or take its reviewed raise.
function rebase1100(family: Family) {
  const old = loadBudgets(path.join(fixture, `${family.name}-rust1991.toml`));
  const registry = new Set(Object.keys(old.instruction));
  const active = loadBudgets(path.join(root, "docs/davinci/plan", family.active));
  for (const [id, row] of Object.entries<any>(old.instruction)) {
    const now = active.instruction[id];
    assert.ok(now, `${id}: active row removed`);
    assert.equal(now.fixture, row.fixture, `${id}: active fixture changed`);
    assert.equal(now.window, row.window, `${id}: active window changed`);
    assert.ok(
      now.instructions <= Math.max(row.instructions, raises1100[id] ?? 0),
      `${id}: active cap raised`,
    );
  }
  const instruction = structuredClone(old.instruction);
  for (const [id, cap] of Object.entries(raises1100))
    if (instruction[id]) instruction[id].instructions = cap;
  const current = validateBudgets({ ...active, instruction }, registry);
  const raw = fs.readFileSync(path.join(fixture, `${family.name}-rust1100.json`), "utf8");
  const report = validateMeasurement(JSON.parse(raw), registry);
  return { old, current, report, registry };
}
// The archived 1.99.0 registry and its 1.99.1 re-baseline under the 1.99.0 rows.
function rebase1991(family: Family) {
  const rust1990 = loadBudgets(path.join(fixture, `${family.name}-rust199.toml`));
  const registry = new Set(Object.keys(rust1990.instruction));
  const archived = loadBudgets(path.join(fixture, `${family.name}-rust1991.toml`));
  const rust1991 = validateBudgets(
    { ...archived, instruction: structuredClone(rust1990.instruction) },
    registry,
  );
  return { rust1990, rust1991 };
}

for (const family of families) {
  void test(`${family.name}: current measurement requires exact checkout and pinned Rust, preserving regression refusal`, () => {
    const { current, report } = rebase1100(family);
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
    for (const toolchain of ["1.98.0", "1.99.0", "1.99.1", "1.101.0", "stable"]) {
      assert.throws(
        () => authenticateInstructionMeasurementSource(report, { ...context, toolchain }),
        /must be 1.100.0/,
      );
    }
    const wrongCompiler = structuredClone(report);
    wrongCompiler.methodology.rustc = "rustc 1.100.0 (wrong build)";
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

  void test(`${family.name}: the 1.100.0 re-baseline keeps every 1.99.1 row, input and cap`, () => {
    assert.equal(
      digest(fs.readFileSync(path.join(fixture, `${family.name}-rust1991.toml`))),
      family.rust1991Hash,
    );
    assert.equal(
      digest(fs.readFileSync(path.join(fixture, `${family.name}-rust1100.json`))),
      family.rust1100Hash,
    );
    const { old, current, report, registry } = rebase1100(family);
    assert.equal(old.source_commit, rebaseSource);
    for (const [id, row] of Object.entries<{ instructions: number }>(old.instruction)) {
      const raised = raises1100[id];
      assert.equal(current.instruction[id].instructions, raised ?? row.instructions, id);
      if (raised) assert.ok(raised > row.instructions, `${id}: reviewed raise`);
    }
    // Every reviewed raise is exactly the identical three-run 1.100.0 count.
    for (const run of report.runs)
      for (const [id, cap] of Object.entries(raises1100))
        if (run[id]) assert.equal(run[id].instructions, cap, id);
    assert.equal(current.source_commit, rust1100Source);
    assert.equal(current.recorded_run, report.recorded_run);
    ratchetBudgets(current, old);
    authenticateInstructionMeasurementSource(report, context);
    checkMeasurement(report, current);
    for (const run of report.runs) assert.equal(Object.keys(run).length, registry.size);
    const id = Object.keys(old.instruction).find((item) => !(item in raises1100));
    assert.ok(id);
    const lowered = structuredClone(current);
    lowered.instruction[id].instructions -= 1;
    ratchetBudgets(lowered, old);
    const raised = structuredClone(current);
    raised.instruction[id].instructions += 1;
    assert.throws(() => ratchetBudgets(raised, old), /instruction budget loosened/);
    for (const key of ["fixture", "fixture_sha256", "window"]) {
      const changed = structuredClone(current);
      changed.instruction[id][key] += "-changed";
      assert.throws(() => ratchetBudgets(changed, old), /preserve every row, input identity/);
    }
    const missing = structuredClone(current);
    delete missing.instruction[id];
    assert.throws(() => ratchetBudgets(missing, old), /preserve every row, input identity/);
    const extra = structuredClone(current);
    extra.instruction.foreign_probe = structuredClone(current.instruction[id]);
    assert.throws(() => ratchetBudgets(extra, old), /preserve every row, input identity/);
  });

  void test(`${family.name}: only the reviewed rows rise, only to their 1.100.0 counts, only once`, () => {
    const { old, current } = rebase1100(family);
    const listed = Object.keys(raises1100).filter((id) => id in old.instruction);
    assert.equal(listed.length, family.name === "level" ? 3 : 0);
    for (const id of listed) {
      const beyond = structuredClone(current);
      beyond.instruction[id].instructions += 1;
      assert.throws(() => ratchetBudgets(beyond, old), /instruction budget loosened/, id);
      const below = structuredClone(current);
      below.instruction[id].instructions -= 1;
      ratchetBudgets(below, old);
      // Once on 1.100.0 the ordinary downward ratchet applies to the raised cap.
      assert.throws(() => ratchetBudgets(beyond, current), /instruction budget loosened/, id);
    }
    // No other step admits these raises.
    const { rust1990, rust1991 } = rebase1991(family);
    for (const id of listed) {
      const forged = structuredClone(rust1991);
      forged.instruction[id].instructions = raises1100[id];
      assert.throws(() => ratchetBudgets(forged, rust1990), /instruction budget loosened/, id);
    }
  });

  void test(`${family.name}: the 1.100.0 re-baseline pins compiler, method and both provenances`, () => {
    const { old, current } = rebase1100(family);
    for (const key of Object.keys(old.methodology).filter((item) => item !== "rustc")) {
      const candidate = structuredClone(current);
      candidate.methodology[key] += "-changed";
      assert.throws(() => ratchetBudgets(candidate, old), /preserve every row, input identity/);
    }
    for (const compiler of ["rustc 1.100.0 (wrong build)", "rustc 1.101.0 (future)"]) {
      const candidate = structuredClone(current);
      candidate.methodology.rustc = compiler;
      assert.throws(
        () => ratchetBudgets(candidate, old),
        /unauthorized forward instruction compiler/,
      );
    }
    // A 1.100.0 registry cannot step backwards to 1.99.1.
    assert.throws(() => ratchetBudgets(old, current), /unauthorized original instruction compiler/);
    for (const key of ["source_commit", "recorded_run"]) {
      const value =
        key === "source_commit"
          ? "a".repeat(40)
          : "https://github.com/ubugeeei-prod/vize/actions/runs/1";
      const base = structuredClone(old);
      base[key] = value;
      assert.throws(() => ratchetBudgets(current, base), /unauthenticated 1.99.1 instruction/);
      const candidate = structuredClone(current);
      candidate[key] = value;
      assert.throws(() => ratchetBudgets(candidate, old), /preserve every row, input identity/);
    }
  });
}

void test("repository source pin remains strict independently of an archived report", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "instruction-source-pin-"));
  try {
    const { current } = rebase1100(families[0]);
    for (const channel of ["1.98.0", "1.99.0", "1.99.1", "1.101.0", "stable"]) {
      fs.writeFileSync(
        path.join(temporary, "rust-toolchain.toml"),
        `[toolchain]\nchannel = "${channel}"\n`,
      );
      assert.throws(
        () => verifyRepositoryInstructionBudgets(current, temporary),
        /must be 1.100.0/,
      );
    }
    fs.writeFileSync(
      path.join(temporary, "rust-toolchain.toml"),
      '[toolchain]\nchannel = "1.100.0"\n',
    );
    verifyRepositoryInstructionBudgets(current, temporary);
    const wrong = structuredClone(current);
    wrong.methodology.rustc = "rustc 1.99.1 (58cf8f9c3 2026-10-13)";
    assert.throws(
      () => verifyRepositoryInstructionBudgets(wrong, temporary),
      /budget compiler mismatch/,
    );
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

void test("measurements read only the contributor toolchain pin", () => {
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "instruction-measurement-pin-"));
  try {
    const { current } = rebase1100(families[0]);
    fs.writeFileSync(
      path.join(temporary, "rust-toolchain.toml"),
      '[toolchain]\nchannel = "1.100.0"\n',
    );
    // A retired separate measurement manifest cannot select another compiler.
    const retired = path.join(temporary, "rust-measurement-toolchain.toml");
    fs.writeFileSync(retired, '[toolchain]\nchannel = "1.99.1"\n');
    verifyRepositoryInstructionBudgets(current, temporary);
    fs.writeFileSync(path.join(temporary, "rust-toolchain.toml"), "invalid TOML\n");
    assert.throws(() => verifyRepositoryInstructionBudgets(current, temporary));
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});
