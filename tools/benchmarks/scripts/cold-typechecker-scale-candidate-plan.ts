/** Literal controller cases. Pending source/merge facts never select a candidate build. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { pathToFileURL } from "node:url";
import type { BodyPin, CandidateSources } from "./cold-typechecker-scale-candidate-source.ts";

type Kind = CandidateSources["kind"];
type Role = "base" | "head";
export interface ReadyCandidate {
  kind: Kind;
  state: "ready";
  sources: CandidateSources;
  allocation: Record<Role, { cli: BodyPin; execution: BodyPin }>;
  sourceCi: {
    headSha: string;
    runId: string;
    executedSourceSha: string;
    executedTreeSha: string;
    requiredChecks: "success";
  };
}
type PendingCandidate = {
  kind: Kind;
  state: "pending";
  sources: null;
  allocation: null;
  sourceCi: null;
};
export interface CandidatePlan {
  schemaVersion: 1;
  optionsFix: { sha: string; tree: string } | null;
  historicalSourceFix: "0eb2f8db16f6541cd422ee610ab3e554bf041e0d";
  cases: (ReadyCandidate | PendingCandidate)[];
}
const oid = (value: unknown) => assert(typeof value === "string" && /^[a-f0-9]{40}$/u.test(value));
const keys = (value: object, expected: string[]) =>
  assert.deepEqual(Object.keys(value).sort(), expected.toSorted(), "closed manifest fields");
const pin = (value: BodyPin) => {
  keys(value, ["bytes", "sha256", "gitBlob"]);
  assert(Number.isSafeInteger(value.bytes) && value.bytes > 0);
  assert.match(value.sha256, /^[a-f0-9]{64}$/u);
  oid(value.gitBlob);
};
export function validateCandidatePlan(plan: CandidatePlan) {
  keys(plan, ["schemaVersion", "optionsFix", "historicalSourceFix", "cases"]);
  assert.equal(plan.schemaVersion, 1);
  assert.equal(plan.historicalSourceFix, "0eb2f8db16f6541cd422ee610ab3e554bf041e0d");
  if (plan.optionsFix !== null) {
    keys(plan.optionsFix, ["sha", "tree"]);
    oid(plan.optionsFix.sha);
    oid(plan.optionsFix.tree);
    assert.notEqual(plan.optionsFix.sha, plan.historicalSourceFix);
  }
  assert.deepEqual(
    plan.cases.map((row) => row.kind),
    ["type-work", "route-move"],
  );
  for (const row of plan.cases) {
    keys(row, ["kind", "state", "sources", "allocation", "sourceCi"]);
    if (row.state === "pending") {
      assert.equal(row.sources, null);
      assert.equal(row.allocation, null);
      assert.equal(row.sourceCi, null);
      continue;
    }
    assert.equal(row.state, "ready");
    assert(plan.optionsFix !== null, "actual merged Options fix is required before any ready case");
    assert.equal(row.sources.kind, row.kind);
    assert.deepEqual(row.sources.containing, {
      optionsFix: plan.optionsFix.sha,
      historicalSourceFix: plan.historicalSourceFix,
    });
    for (const role of ["base", "head"] as const) {
      oid(row.sources[role].sha);
      oid(row.sources[role].tree);
      keys(row.allocation[role], ["cli", "execution"]);
      pin(row.allocation[role].cli);
      pin(row.allocation[role].execution);
    }
    assert.notEqual(row.sources.base.sha, row.sources.head.sha);
    keys(row.allocation, ["base", "head"]);
    keys(row.sourceCi, [
      "headSha",
      "runId",
      "executedSourceSha",
      "executedTreeSha",
      "requiredChecks",
    ]);
    assert.equal(row.sourceCi.headSha, row.sources.head.sha);
    assert.match(row.sourceCi.runId, /^[1-9][0-9]*$/u);
    oid(row.sourceCi.executedSourceSha);
    oid(row.sourceCi.executedTreeSha);
    assert.equal(row.sourceCi.requiredChecks, "success");
  }
  return plan;
}

export function selectCandidatePlan(plan: CandidatePlan, base: string, head: string, sfcs: string) {
  validateCandidatePlan(plan);
  oid(base);
  oid(head);
  assert.equal(sfcs, "2500");
  if (
    base === "e57fd021bf08b197956807eb122941c9cd13b056" &&
    head === "d440bf04becabbb71d916915c640d39c8d8c6450"
  )
    return { route: "legacy" as const };
  const ready = plan.cases.filter((row): row is ReadyCandidate => row.state === "ready");
  const matches = ready.filter(
    (row) => row.sources.base.sha === base && row.sources.head.sha === head,
  );
  assert.equal(
    matches.length,
    1,
    "no single qualified frozen candidate: actual merge/source pins or source CI remain pending",
  );
  return { route: "candidate" as const, selected: matches[0] };
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      plan: { type: "string" },
      out: { type: "string" },
      base: { type: "string" },
      head: { type: "string" },
      sfcs: { type: "string" },
    },
  });
  for (const key of ["plan", "out", "base", "head", "sfcs"] as const)
    assert(typeof values[key] === "string");
  const out = path.resolve(values.out!);
  fs.mkdirSync(out);
  const raw = fs.readFileSync(values.plan!);
  fs.writeFileSync(path.join(out, "plan.raw.json"), raw, { flag: "wx" });
  try {
    const selected = selectCandidatePlan(
      JSON.parse(raw.toString("utf8")),
      values.base!,
      values.head!,
      values.sfcs!,
    );
    fs.writeFileSync(path.join(out, "selection.json"), JSON.stringify(selected, null, 2) + "\n", {
      flag: "wx",
    });
    if (selected.route === "candidate") {
      fs.writeFileSync(
        path.join(out, "source-case.json"),
        JSON.stringify(selected.selected.sources, null, 2) + "\n",
        { flag: "wx" },
      );
    }
    console.log("PAIRED_ROUTE=" + selected.route);
  } catch (error) {
    fs.writeFileSync(
      path.join(out, "refusal.json"),
      JSON.stringify({ error: String(error), targetCommands: 0, productBuilds: 0 }) + "\n",
      { flag: "wx" },
    );
    throw error;
  }
}
