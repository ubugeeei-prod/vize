import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import {
  checkMeasurement,
  fixtureDigest,
  loadBudgets,
  ratchetBudgets,
  validateBudgets,
  validateMeasurement,
} from "../../tools/benchmarks/scripts/instruction-counts-lib.mjs";
import {
  formatterInstructionSuites,
  instructionSuiteSource,
  levelInstructionSuites,
} from "../../tools/benchmarks/scripts/instruction-counts-suites.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fixture = path.join(root, "tests/_fixtures/tooling/instruction-methodology-transition");
const source = "1a3996963af208c13cd732420bdee586df88bf7a";
const rebaseSource = "e40813b3306bf85e5f61d96b19ebd432cae10334";
// Reviewed 1.100.0 raises (see instruction-rust1100-transition.test.ts).
const raises1100 = {
  armature_parse_medium: 244853,
  croquis_analyze_full_large: 2789255,
  "harness-selfcheck": 15481,
};
const families = [
  {
    name: "level",
    active: "instruction-budgets.toml",
    count: 100,
    oldHash: "3c221a56f364a4aaf352729d9b9d0cf1eba2c7e4aa088174fdb6b6099bddb5aa",
    reportHash: "61803d7a6d12b6ed3e29168d4f61b91bb3bbdee7d93ae48a28364b481bb07f2c",
    rust199Hash: "ba1acd1b285b0ded41799e3d5ac59384acecb621d103f2dfa9dae5766ed6a924",
    rebaseHash: "ccb192e7b9817f2980fe216e7721a5f095d387265318b33be355eece817d6b9d",
  },
  {
    name: "formatter",
    active: "formatter-instruction-budgets.toml",
    count: 4,
    oldHash: "e49d7c38a43fcdfa3bf7114c50ef91734f355d943269d42bc1d18821d444de99",
    reportHash: "100823ff2a4cf4a4af7c493912e6653e465332c38e8a154a55e3505f3caebeb4",
    rust199Hash: "6cdf8a43941edb8a69ee05313ee1a8420a0ef03d57f1df53e90504587463c60e",
    rebaseHash: "011272dc10abdac22f8bff19a51b9047f4719b5b60ba701a6526db80be42335c",
  },
];
function packet(family) {
  const old = loadBudgets(path.join(fixture, `${family.name}-rust198.toml`));
  const registry = new Set(Object.keys(old.instruction));
  const current = loadBudgets(path.join(fixture, `${family.name}-rust199.toml`), registry);
  const raw = fs.readFileSync(path.join(fixture, `${family.name}-rust199.json`));
  const report = validateMeasurement(JSON.parse(raw), registry);
  return { old, current, report, registry };
}
// The committed bench sources whose bytes pin a family's synthetic inputs.
function benchSources(family) {
  const suites =
    family.name === "level" ? levelInstructionSuites(root) : formatterInstructionSuites();
  return suites.map((suite) => instructionSuiteSource(root, suite));
}
// Active rows after the archive: a cap may only fall (except a reviewed 1.100.0
// raise, to at most its reviewed count), and an input identity may change only
// to the digest of the one committed bench that declares the probe (a bench
// edit re-pins its synthetic input).
function verifyActiveRows(family, active, archived) {
  for (const [id, row] of Object.entries(archived)) {
    const now = active[id];
    assert.ok(now, `${id}: active row removed`);
    assert.equal(now.fixture, row.fixture, `${id}: active fixture changed`);
    assert.equal(now.window, row.window, `${id}: active window changed`);
    assert.ok(
      now.instructions <= Math.max(row.instructions, raises1100[id] ?? 0),
      `${id}: active cap raised`,
    );
    if (now.fixture_sha256 !== row.fixture_sha256) {
      const declaring = benchSources(family).filter((source) =>
        fs.readFileSync(source, "utf8").includes(`"${id}"`),
      );
      assert.equal(declaring.length, 1, `${id}: identity changed without one declaring bench`);
      assert.equal(
        now.fixture_sha256,
        fixtureDigest(root, now.fixture, declaring[0]),
        `${id}: identity changed without its committed bench source`,
      );
    }
  }
}
// 1.99.0 registry, archived 1.99.1 registry and its three-run same-input evidence.
// The archive replays under the 1.99.0 rows.
function rebase(family) {
  const { current: old, registry } = packet(family);
  const archived = loadBudgets(path.join(fixture, `${family.name}-rust1991.toml`));
  const current = validateBudgets(
    { ...archived, instruction: structuredClone(old.instruction) },
    registry,
  );
  const raw = fs.readFileSync(path.join(fixture, `${family.name}-rust1991.json`));
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
    assert.equal(report.methodology.rustc, current.methodology.rustc);
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
    assert.throws(() => ratchetBudgets(old, current), /unauthorized forward instruction compiler/);
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

  void test(`${family.name}: CLI accepts the current registry and refuses historical reports`, () => {
    const mode = family.name === "formatter" ? ["--formatter"] : [];
    const verified = cli(
      family.name === "level"
        ? [...mode, "--verify-budgets"]
        : [
            ...mode,
            "--verify-budgets",
            "--base-budgets",
            path.join(fixture, `${family.name}-rust1991.toml`),
          ],
    );
    assert.equal(verified.status, 0, verified.stderr);
    assert.match(
      verified.stdout,
      new RegExp(
        `${family.name === "level" ? 101 : family.count} pinned ceilings and ratchet verified`,
      ),
    );
    const replay = cli([
      ...mode,
      "--check",
      "--measurement",
      path.join(fixture, `${family.name}-rust1100.json`),
    ]);
    assert.equal(replay.status, 1);
    assert.match(replay.stderr, /measurement source is not current checkout/);
    const historical = cli([
      ...mode,
      "--check",
      "--measurement",
      path.join(fixture, `${family.name}-rust1991.json`),
    ]);
    assert.equal(historical.status, 1);
    assert.match(
      historical.stderr,
      family.name === "level"
        ? /missing \[s1_to_s2_emit_helper_rich_large\]/
        : /instruction budget compiler mismatch/,
    );
  });

  void test(`${family.name}: the 1.99.1 re-baseline keeps every 1.99.0 row, input and cap`, () => {
    assert.equal(
      digest(fs.readFileSync(path.join(fixture, `${family.name}-rust199.toml`))),
      family.rust199Hash,
    );
    assert.equal(
      digest(fs.readFileSync(path.join(fixture, `${family.name}-rust1991.json`))),
      family.rebaseHash,
    );
    const { old, current, report } = rebase(family);
    assert.deepEqual(current.instruction, old.instruction);
    assert.equal(current.source_commit, rebaseSource);
    assert.equal(current.recorded_run, report.recorded_run);
    ratchetBudgets(current, old);
    // The archived producer was the 1.99.1 checkout, not our HEAD.
    assert.equal(report.source_commit, rebaseSource);
    checkMeasurement(report, current);
    const id = Object.keys(old.instruction)[0];
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

  void test(`${family.name}: active rows may only lower caps or re-pin an input to its committed bench`, () => {
    const { current: old } = packet(family);
    const active = loadBudgets(path.join(root, "docs/davinci/plan", family.active)).instruction;
    verifyActiveRows(family, active, old.instruction);
    const ids = Object.keys(old.instruction);
    const check = (change) => {
      const rows = structuredClone(active);
      change(rows);
      return () => verifyActiveRows(family, rows, old.instruction);
    };
    const [first] = ids;
    check((rows) => {
      rows[first].instructions -= 1;
    })();
    assert.throws(
      check((rows) => {
        rows[first].instructions = old.instruction[first].instructions + 1;
      }),
      /active cap raised/,
    );
    assert.throws(
      check((rows) => {
        delete rows[first];
      }),
      /active row removed/,
    );
    for (const id of ids) {
      assert.throws(
        check((rows) => {
          rows[id].fixture_sha256 = "0".repeat(64);
        }),
        /identity changed without/,
        id,
      );
    }
    if (family.name !== "level") return;
    // A re-pin must match the bench that declares the probe, not any bench.
    const id = "patina_jsx_markup_one_root";
    const other = benchSources(family).find(
      (source) => !fs.readFileSync(source, "utf8").includes(`"${id}"`),
    );
    assert.throws(
      check((rows) => {
        rows[id].fixture_sha256 = fixtureDigest(root, rows[id].fixture, other);
      }),
      /identity changed without its committed bench source/,
    );
  });

  void test(`${family.name}: the 1.99.1 re-baseline pins compiler, method and both provenances`, () => {
    const { old, current } = rebase(family);
    for (const key of Object.keys(old.methodology).filter((item) => item !== "rustc")) {
      const candidate = structuredClone(current);
      candidate.methodology[key] += "-changed";
      assert.throws(() => ratchetBudgets(candidate, old), /preserve every row, input identity/);
    }
    // No step may be skipped: a 1.99.0 base admits only 1.99.1.
    for (const compiler of ["rustc 1.99.1 (wrong build)", "rustc 1.100.0 (0ef1bc386 2026-11-09)"]) {
      const candidate = structuredClone(current);
      candidate.methodology.rustc = compiler;
      assert.throws(
        () => ratchetBudgets(candidate, old),
        /unauthorized forward instruction compiler/,
      );
    }
    for (const key of ["source_commit", "recorded_run"]) {
      const value =
        key === "source_commit"
          ? "a".repeat(40)
          : "https://github.com/ubugeeei-prod/vize/actions/runs/1";
      const base = structuredClone(old);
      base[key] = value;
      assert.throws(() => ratchetBudgets(current, base), /unauthenticated 1.99.0 instruction/);
      const candidate = structuredClone(current);
      candidate[key] = value;
      assert.throws(() => ratchetBudgets(candidate, old), /preserve every row, input identity/);
    }
  });
}
