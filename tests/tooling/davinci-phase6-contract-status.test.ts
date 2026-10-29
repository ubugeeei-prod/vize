import assert from "node:assert/strict";
import { test } from "node:test";

import { readPlan, registerRecutChecks } from "./support/davinci-recut.ts";

registerRecutChecks({
  phase: 6,
  strictDisjointPaths: true,
  taskFiles: ["phase-6-tasks.md", "phase-6-tasks-later.md"],
  predecessorFiles: ["phase-3.md", "phase-4.md", "phase-5.md"],
  provisionalGateLines: [
    "- [ ] TS-48..51 green; contracts documented with semver policy",
    "- [ ] MoonBit + Volt + JS-rule validations complete; `ExprRef` report closed",
    "- [ ] Completion metrics reconciled; v1 package delivered",
  ],
});

test("Phase 6: retired contracts remain historical rather than completion gates", () => {
  const phase = readPlan("phase-6.md");
  const tasks = `${readPlan("phase-6-tasks.md")}\n${readPlan("phase-6-tasks-later.md")}`;
  const index = readPlan("README.md");

  assert.match(phase, /This entire phase is a frozen historical plan/u);
  assert.match(phase, /unchecked entries are not current\n> backlog, completion blockers, or release gates/u);
  assert.match(phase, /They\nare non-normative after the 2026-10-17 scope decision/u);
  assert.doesNotMatch(phase, /provisional lines are normative/u);
  assert.match(tasks, /Frozen historical contracts/u);
  assert.match(index, /\*\*Archived 2026-10-17\*\*/u);
  assert.match(index, /not current backlog or release criteria/u);
});
