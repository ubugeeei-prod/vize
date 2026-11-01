import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import {
  selectCandidatePlan,
  validateCandidatePlan,
  type CandidatePlan,
  type ReadyCandidate,
} from "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-plan.ts";

const literalPlan = () =>
  JSON.parse(
    fs.readFileSync(
      new URL(
        "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-cases.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as CandidatePlan;
const pending = (): CandidatePlan => ({
  schemaVersion: 1,
  optionsFix: null,
  historicalSourceFix: "0eb2f8db16f6541cd422ee610ab3e554bf041e0d",
  cases: ["type-work", "route-move"].map((kind) => ({
    kind: kind as "type-work" | "route-move",
    state: "pending",
    sources: null,
    allocation: null,
    sourceCi: null,
  })),
});
const id = (letter: string) => letter.repeat(40);
function ready() {
  const plan = pending();
  plan.optionsFix = { sha: id("a"), tree: id("b") };
  const pin = { bytes: 1, sha256: "a".repeat(64), gitBlob: id("a") };
  plan.cases[0] = {
    kind: "type-work",
    state: "ready",
    sources: {
      kind: "type-work",
      base: { sha: id("c"), tree: id("d") },
      head: { sha: id("e"), tree: id("f") },
      containing: { optionsFix: id("a"), historicalSourceFix: plan.historicalSourceFix },
      changed: [],
      cargo: [],
      protocol: [],
    },
    allocation: { base: { cli: pin, execution: pin }, head: { cli: pin, execution: pin } },
    sourceCi: {
      headSha: id("e"),
      runId: "1",
      executedSourceSha: id("e"),
      executedTreeSha: id("f"),
      requiredChecks: "success",
    },
  };
  return plan;
}
test("synthetic pending plan has null unknowns and refuses any candidate before source acquisition/build", () => {
  assert.doesNotThrow(() => validateCandidatePlan(pending()));
  assert.throws(() => selectCandidatePlan(pending(), id("c"), id("e"), "2500"), /pending/u);
  const legacy = selectCandidatePlan(
    pending(),
    "e57fd021bf08b197956807eb122941c9cd13b056",
    "d440bf04becabbb71d916915c640d39c8d8c6450",
    "2500",
  );
  assert.deepEqual(legacy, { route: "legacy" });
  assert.throws(() => selectCandidatePlan(pending(), id("c"), id("e"), "10000"));
});
test("each source-bound controller case selects only its own exact pair", () => {
  const verify = (plan: CandidatePlan) => {
    const rows = plan.cases.filter((row) => row.state === "ready");
    assert.deepEqual(
      rows.map((row) => row.kind),
      ["type-work", "route-move"],
      "both checked-in source-bound cases must be ready",
    );
    for (const row of rows) {
      const selected = selectCandidatePlan(
        plan,
        row.sources.base.sha,
        row.sources.head.sha,
        "2500",
      );
      assert.equal(selected.route, "candidate");
      assert(selected.route === "candidate");
      assert.equal(selected.selected.kind, row.kind);
      assert.deepEqual(selected.selected, row);
      assert.throws(() =>
        selectCandidatePlan(plan, row.sources.head.sha, row.sources.base.sha, "2500"),
      );
    }
  };
  const plan = validateCandidatePlan(literalPlan());
  verify(plan);
  assert.throws(() => verify(validateCandidatePlan(pending())), /both checked-in/u);
  for (const index of [0, 1]) {
    const missing = structuredClone(plan);
    missing.cases[index] = pending().cases[index];
    assert.throws(() => verify(validateCandidatePlan(missing)), /both checked-in/u);
  }
});

test("one frozen ready case selects only its exact pair; both cases remain independent", () => {
  const plan = ready();
  const selected = selectCandidatePlan(plan, id("c"), id("e"), "2500");
  assert.equal(selected.route, "candidate");
  assert.throws(() => selectCandidatePlan(plan, id("e"), id("c"), "2500"));
  assert.throws(() => selectCandidatePlan(plan, id("c"), id("a"), "2500"));
  for (const mutate of [
    (p: CandidatePlan) => {
      p.optionsFix = null;
    },
    (p: CandidatePlan) => {
      p.optionsFix!.sha = id("b");
    },
    (p: CandidatePlan) => {
      p.cases.reverse();
    },
    (p: CandidatePlan) => {
      p.cases.push(p.cases[0]);
    },
    (p: CandidatePlan) => {
      (p.cases[0] as ReadyCandidate).sourceCi.requiredChecks = "pending" as "success";
    },
    (p: CandidatePlan) => {
      (p.cases[0] as ReadyCandidate).sourceCi.headSha = id("a");
    },
    (p: CandidatePlan) => {
      (p.cases[0] as ReadyCandidate).allocation.base.cli.bytes = 0;
    },
    (p: CandidatePlan) => {
      Object.assign(p.cases[1], { sources: (p.cases[0] as ReadyCandidate).sources });
    },
    (p: CandidatePlan) => {
      Object.assign(p, { guessedMerge: id("a") });
    },
  ]) {
    const changed = structuredClone(plan);
    mutate(changed);
    assert.throws(() => validateCandidatePlan(changed));
  }
});

test("pending CLI retains the exact literal plan and explicit zero-work refusal", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "pending-candidate-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const script = new URL(
    "../../tools/benchmarks/scripts/cold-typechecker-scale-candidate-plan.ts",
    import.meta.url,
  );
  const plan = path.join(root, "pending.json");
  fs.writeFileSync(plan, JSON.stringify(pending()) + "\n");
  const out = path.join(root, "out");
  const result = spawnSync(
    process.execPath,
    [
      script.pathname,
      "--plan",
      plan,
      "--out",
      out,
      "--base",
      id("c"),
      "--head",
      id("e"),
      "--sfcs",
      "2500",
    ],
    { encoding: "utf8" },
  );
  assert.notEqual(result.status, 0);
  assert.equal(result.stdout, "");
  assert(fs.readFileSync(path.join(out, "plan.raw.json")).equals(fs.readFileSync(plan)));
  assert(!fs.existsSync(path.join(out, "source-case.json")));
  const refusal = JSON.parse(fs.readFileSync(path.join(out, "refusal.json"), "utf8"));
  assert.equal(refusal.targetCommands, 0);
  assert.equal(refusal.productBuilds, 0);
  assert.match(refusal.error, /pending/u);
});

test("candidate route preserves source-first ordering, separate normal images and raw-only profile retention", () => {
  const workflow = fs.readFileSync(
    new URL("../../.github/workflows/cold-typechecker-scale.yml", import.meta.url),
    "utf8",
  );
  const check = (body: string) => {
    const order = [
      "cold-typechecker-scale-candidate-plan.ts --plan",
      "git worktree add",
      "--stage verify",
      '--role "normal-$role"',
      "--stage allocation",
      "--observer-source-sha",
      "cold-typechecker-scale-candidate-paired.ts",
    ];
    for (let index = 1; index < order.length; index++)
      assert(
        body.indexOf(order[index - 1]) >= 0 &&
          body.indexOf(order[index]) > body.indexOf(order[index - 1]),
      );
    assert(body.includes("if: env.PAIRED_ROUTE == 'legacy'"));
    assert(body.includes("--exclude='*/native-profile-data'"));
    assert(body.includes("--exclude='*/input-blobs'"));
    assert(body.includes("observations/native/*/native-profile-data"));
    assert(body.includes('go-version: "1.27.2"'));
  };
  check(workflow);
  for (const marker of [
    "--stage verify",
    "--stage allocation",
    "--observer-source-sha",
    "--exclude='*/native-profile-data'",
    "observations/native/*/native-profile-data",
  ])
    assert.throws(() => check(workflow.replace(marker, "missing")));
});
