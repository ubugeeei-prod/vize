import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  CONFIG_COHORT,
  configCohortRow,
} from "../../tools/commands/fixtures/linter-config-history-case.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const read = (file: string) => fs.readFileSync(path.join(root, file), "utf8");

// #6881: every title-fix commit of the pinned Patina inventory is either pinned
// by complete current oracles or reviewed as having no lint behavior.
void test("every linter fix-history ledger row resolves to registered oracles", () => {
  const [header, ...rows] = read("docs/davinci/plan/linter-fix-history.tsv")
    .split("\n")
    .filter(Boolean)
    .map((line) => line.split("\t"));
  assert.deepEqual(header, ["commit", "subject", "state", "cases", "note"]);
  assert.equal(rows.length, 258);
  const owners = new Map<string, string>();
  for (const wire of JSON.parse(read("crates/vize_patina/tests/fixtures/lint-history/cases.json")))
    owners.set(`current-api/${wire.id}`, wire.history);
  const configCases = JSON.parse(
    read(`tests/_fixtures/differential/linter/${CONFIG_COHORT}/cases.json`),
  );
  for (const wire of configCases) owners.set(`${CONFIG_COHORT}/${wire.id}`, wire.history);
  const states: Record<string, number> = {};
  for (const [commit, subject, state, cases, note] of rows) {
    assert.match(commit, /^[0-9a-f]{40}$/);
    assert.match(subject, /^fix(?:\([^)]*\))?!?: /);
    states[state] = (states[state] ?? 0) + 1;
    if (state === "oracle-pinned") {
      assert.notEqual(cases, "-", commit);
      for (const id of cases.split(",")) assert.equal(owners.get(id), commit, `${commit}: ${id}`);
    } else {
      assert.equal(state, "no-lint-behavior", `${commit} is not admitted`);
      assert.equal(cases, "-");
      assert(note.length > 40, `${commit}: a no-lint-behavior row names its coverage`);
    }
  }
  assert.deepEqual(states, { "oracle-pinned": 233, "no-lint-behavior": 25 });
  // Every configured case belongs to a pinned row, and its cohort files are registered.
  const pinned = new Set(rows.flatMap(([, , state, cases]) => (state === "oracle-pinned" ? cases.split(",") : [])));
  for (const wire of configCases) assert(pinned.has(`${CONFIG_COHORT}/${wire.id}`), wire.id);
  const manifest = JSON.parse(read("tests/_fixtures/differential/linter/manifest.json"));
  assert.deepEqual(
    manifest.cases.find((fixture: any) => fixture.id === `linter/${CONFIG_COHORT}`),
    configCohortRow(),
  );
});
