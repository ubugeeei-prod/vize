/** Fail an exact scale run when a Vize editor stops publishing current diagnostics. */

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { parseArgs } from "node:util";

interface EditSample {
  cycle: number;
  probe: "script" | "template";
  phase: "break" | "repair";
  ok: boolean;
}

interface Session {
  editor: string;
  kind: string;
  round: number;
  cold: { ok: boolean };
  edits: EditSample[];
}

interface Run {
  options: { cycles: number };
  runs: Record<string, { sessions: Session[]; crashes: string[] }>;
}

function expectedEdits(cycles: number): string[] {
  const edits = ["0:script:repair"];
  for (let cycle = 1; cycle <= cycles; cycle += 1) {
    for (const probe of ["script", "template"])
      for (const phase of ["break", "repair"]) edits.push(`${cycle}:${probe}:${phase}`);
  }
  return edits;
}

export function diagnosticConvergenceFailures(
  run: Run,
  fixture: string,
  editorPrefix = "vize head",
): string[] {
  const measured = run.runs[fixture];
  if (!measured) return [`${fixture}: fixture was not measured`];
  const failures = measured.crashes
    .filter((crash) => crash.startsWith(editorPrefix))
    .map((crash) => `${fixture}: ${crash}`);
  const sessions = measured.sessions.filter(
    (session) => session.kind === "vize" && session.editor.startsWith(editorPrefix),
  );
  if (sessions.length === 0) failures.push(`${fixture}: no ${editorPrefix} sessions were recorded`);
  const expected = expectedEdits(run.options.cycles);
  for (const session of sessions) {
    const label = `${fixture}: ${session.editor}, round ${session.round}`;
    if (!session.cold.ok) failures.push(`${label}: cold diagnostics did not converge`);
    const actual = session.edits.map((edit) => `${edit.cycle}:${edit.probe}:${edit.phase}`);
    if (actual.join("\n") !== expected.join("\n")) {
      failures.push(`${label}: recorded ${actual.length}/${expected.length} required edit samples`);
    }
    for (const edit of session.edits) {
      if (!edit.ok)
        failures.push(
          `${label}: cycle ${edit.cycle} ${edit.probe} ${edit.phase} diagnostics did not converge`,
        );
    }
  }
  return failures;
}

function main(): void {
  const { values } = parseArgs({
    options: {
      json: { type: "string" },
      fixture: { type: "string" },
      "editor-prefix": { type: "string", default: "vize head" },
    },
  });
  assert.ok(values.json, "--json is required");
  assert.ok(values.fixture, "--fixture is required");
  const run = JSON.parse(readFileSync(values.json, "utf8")) as Run;
  const failures = diagnosticConvergenceFailures(run, values.fixture, values["editor-prefix"]);
  assert.equal(failures.length, 0, failures.join("\n"));
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
