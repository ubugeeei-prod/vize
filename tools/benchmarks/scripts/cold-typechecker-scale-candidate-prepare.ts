/** Hosted preparation only. No target invocation is performed by this source/build entry. */
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { parseArgs } from "node:util";
import { pathToFileURL } from "node:url";
import { execFileSync } from "node:child_process";
import { verifyCandidateSources } from "./cold-typechecker-scale-candidate-source.ts";
import {
  selectCandidatePlan,
  type CandidatePlan,
} from "./cold-typechecker-scale-candidate-plan.ts";
import { prepareCandidateAllocation } from "./cold-typechecker-scale-candidate-allocation.ts";
import { verifyColdScaleBuildPair } from "./cold-typechecker-scale-build.ts";

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  const { values } = parseArgs({
    options: {
      stage: { type: "string" },
      plan: { type: "string" },
      base: { type: "string" },
      head: { type: "string" },
      provider: { type: "string" },
      out: { type: "string" },
      role: { type: "string" },
    },
  });
  for (const key of ["stage", "plan", "base", "head", "provider", "out"] as const)
    assert(typeof values[key] === "string");
  assert(values.stage === "verify" || values.stage === "allocation");
  const plan = JSON.parse(fs.readFileSync(values.plan!, "utf8")) as CandidatePlan;
  const selected = selectCandidatePlan(plan, process.env.BASE_SHA!, process.env.HEAD_SHA!, "2500");
  assert.equal(selected.route, "candidate");
  assert(selected.route === "candidate");
  const roots = { base: path.resolve(values.base!), head: path.resolve(values.head!) };
  const out = path.resolve(values.out!);
  assert(plan.optionsFix !== null);
  for (const root of Object.values(roots)) {
    const actualTree = execFileSync("git", ["rev-parse", plan.optionsFix.sha + "^{tree}"], {
      cwd: root,
      encoding: "utf8",
      maxBuffer: 1024 * 1024,
    }).trim();
    assert.equal(
      actualTree,
      plan.optionsFix.tree,
      "whole actual Options merge tree must match its frozen receipt",
    );
  }
  // Both original source roots must still be whole, clean and authenticated before any overlay.
  const source = verifyCandidateSources(
    roots,
    path.resolve(values.provider!),
    selected.selected.sources,
  );
  if (values.stage === "verify") {
    fs.writeFileSync(
      path.join(out, "source-before-build.json"),
      JSON.stringify(
        { ...source, optionsFix: plan.optionsFix, sourceCi: selected.selected.sourceCi },
        null,
        2,
      ) + "\n",
      { flag: "wx" },
    );
  } else {
    assert(values.role === "base" || values.role === "head");
    const role = values.role;
    verifyColdScaleBuildPair(
      path.join(out, "builds/base.json"),
      path.join(out, "builds/head.json"),
      process.cwd(),
    );
    const normal = JSON.parse(fs.readFileSync(path.join(out, `builds/${role}.json`), "utf8"));
    assert.equal(normal.sourceSha, selected.selected.sources[role].sha);
    assert.equal(normal.treeSha, selected.selected.sources[role].tree);
    assert.equal(normal.allocation, false);
    prepareCandidateAllocation({
      sourceRoot: roots[role],
      sourceSha: selected.selected.sources[role].sha,
      destination: path.join(path.dirname(out), "cold-candidate-allocation-source-" + role),
      target: path.join(path.dirname(out), "cold-candidate-allocation-target-" + role),
      output: path.join(out, "allocation", role),
      role,
      kind: selected.selected.kind,
      pins: selected.selected.allocation[role],
    });
  }
}
