import assert from "node:assert/strict";
import { test } from "node:test";

import { readPlan, registerRecutChecks, sectionBetween } from "./support/davinci-recut.ts";

registerRecutChecks({
  phase: 4,
  taskFiles: ["phase-4-tasks.md", "phase-4-tasks-later.md", "phase-4-tasks-last.md"],
  predecessorFiles: ["phase-3.md"],
  provisionalGateLines: [
    "- [ ] TS-40 check parity; TS-39 lint agreement; TS-5 + TS-41 glyph gates",
    "- [ ] Consumption matrix: every computed group ≥1 consumer or gated (TS-12)",
    "- [ ] TS-36 witnesses verify; TS-37 100% recall per class; TS-38 zero untriaged candidates",
    "- [ ] canon/maestro projection duplicates + glyph byte scanner + musea hand parser: deleted",
  ],
});

test("Phase 4: TS-53 is mandatory at the phase-4 exit", () => {
  const row = /^\| P4 +\| (?<suites>[^|]+)\|$/mu.exec(readPlan("test-suites.md"));
  assert.ok(row?.groups?.suites.includes("TS-53"), "P4 mandatory suites must include TS-53");
});

test("Phase 4: the open questions it depends on carry a written recommendation", () => {
  const questions = readPlan("../open-questions.md");
  for (const heading of ["Complexity metric definition", "App-level fact provider contract"]) {
    const section = sectionBetween(
      questions,
      new RegExp(`^## ${heading}$`, "mu"),
      /^## |$(?![\s\S])/mu,
      heading,
    );
    assert.match(section, /\*\*Recommendation \(phase-4 re-cut, 2026-09-21\)/u, heading);
    assert.match(section, /plan\/phase-4-tasks(?:-later|-last)?\.md#p4-/u, `${heading} owner link`);
  }
});
