// Davinci traversal budgets (plan/phase-2.md P2-12a).
//
// Three suites, all reading davinci-road/plan/budgets.toml:
//   1. Traversal reconciliation — every `<backend>_<fixture>` id under
//      [traversal] has a walk-probe recorder that measures it and vice versa.
//      The backend domain is derived from the crates that ship a
//      `tests/davinci_walk_baseline.rs` recorder x the harness LADDER, so a
//      recorder without ceilings and a ceiling without a recorder both fail.
//   2. Cross-copy equality — each ceiling equals the BASELINE table its
//      recorder pins Rust-side, so the two committed copies of the same
//      measurement cannot drift apart.
//   3. The phase target table — [target.phase-2] exists, names a real
//      phase-start rev and carries non-zero values. The numbers themselves are
//      the maintainer's review point, not CI's.
//
// Split from davinci-budgets.test.ts (which keeps the P0-4 [bench] registry)
// because either file alone would breach the 350-line source budget.
//
// The measurement, the exclusion list and the reading of the numbers:
// davinci-road/plan/walk-baseline.md and plan/phase-2-records.md#p2-12a.

import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { parseTomlLite } from "../../legacy-tools/davinci/toml-lite.mjs";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const budgetsPath = path.join(repoRoot, "davinci-road", "plan", "budgets.toml");

const budgetsText = fs.readFileSync(budgetsPath, "utf8");
const budgets = parseTomlLite(budgetsText) as {
  traversal: Record<string, Record<string, unknown>>;
  target: Record<string, Record<string, unknown>>;
};

// The ladder is the fixture domain, read from the harness rather than
// restated, so a fixture added there without a ceiling fails here.
function ladderNames(): string[] {
  const fixturesRs = fs.readFileSync(
    path.join(repoRoot, "benchmarks", "davinci_harness", "src", "fixtures.rs"),
    "utf8",
  );
  const start = fixturesRs.indexOf("pub const LADDER");
  assert.ok(start >= 0, "fixtures.rs must declare the LADDER const");
  const block = fixturesRs.slice(start, fixturesRs.indexOf("];", start));
  const names = [...block.matchAll(/name: "([^"]+)"/g)].map(([, name]) => name);
  assert.ok(names.length > 0, "the LADDER const must name at least one fixture");
  return names;
}

// Inline-table entry lines belonging to one `[section]` header, in file order.
function inlineTableLines(section: string): string[] {
  const lines = budgetsText.split("\n");
  const start = lines.indexOf(`[${section}]`);
  assert.ok(start >= 0, `budgets.toml must declare a [${section}] section`);
  const rest = lines.slice(start + 1);
  const end = rest.findIndex((line) => line.startsWith("["));
  const body = end === -1 ? rest : rest.slice(0, end);
  return body.filter((line) => /^[A-Za-z0-9._-]+ = \{/.test(line));
}

const WALK_RECORDER = "tests/davinci_walk_baseline.rs";

function walkRecorders(): { backend: string; rows: Map<string, [number, number]> }[] {
  const recorders: { backend: string; rows: Map<string, [number, number]> }[] = [];
  for (const pkg of fs.readdirSync(path.join(repoRoot, "crates")).sort()) {
    const file = path.join(repoRoot, "crates", pkg, WALK_RECORDER);
    if (!fs.existsSync(file)) continue;
    const text = fs.readFileSync(file, "utf8");
    const backend = /fn ([a-z0-9_]+)_walk_baseline_holds\(/.exec(text)?.[1];
    assert.ok(
      backend,
      `crates/${pkg}/${WALK_RECORDER} must name its backend via ` +
        "`fn <backend>_walk_baseline_holds()` — the id prefix is derived from it",
    );
    const start = text.indexOf("const BASELINE");
    assert.ok(start >= 0, `crates/${pkg}/${WALK_RECORDER} must declare a BASELINE table`);
    const table = text.slice(start, text.indexOf("];", start));
    const rows = new Map<string, [number, number]>();
    for (const [, fixture, walks, visits] of table.matchAll(/\("([^"]+)",\s*(\d+),\s*(\d+)\)/g)) {
      assert.ok(!rows.has(fixture), `${backend}: BASELINE lists ${fixture} twice`);
      rows.set(fixture, [Number(walks), Number(visits)]);
    }
    assert.ok(rows.size > 0, `crates/${pkg}/${WALK_RECORDER}: BASELINE table is empty`);
    recorders.push({ backend, rows });
  }
  assert.ok(recorders.length > 0, `no walk-probe recorders found (crates/*/${WALK_RECORDER})`);
  return recorders;
}

test("every walk-probe ladder cell has a [traversal] ceiling, and vice versa", () => {
  const fixtures = ladderNames();
  const probeIds = walkRecorders()
    .flatMap(({ backend }) => fixtures.map((fixture) => `${backend}_${fixture}`))
    .sort();
  const budgetIds = Object.keys(budgets.traversal).sort();
  const missingFromBudgets = probeIds.filter((id) => !budgetIds.includes(id));
  const withoutProbe = budgetIds.filter((id) => !probeIds.includes(id));
  assert.deepEqual(
    { missingFromBudgets, withoutProbe },
    { missingFromBudgets: [], withoutProbe: [] },
    "budgets.toml [traversal] and the walk-probe recorders must reconcile exactly",
  );
  assert.deepEqual(budgetIds, probeIds);
});

test("every [traversal] ceiling equals the count its recorder pins", () => {
  for (const { backend, rows } of walkRecorders()) {
    for (const [fixture, [walks, visits]] of rows) {
      const id = `${backend}_${fixture}`;
      assert.deepEqual(
        budgets.traversal[id],
        { walks, visits },
        `[traversal.${id}] must equal the BASELINE row crates/*/${WALK_RECORDER} pins ` +
          "— the ceiling and the Rust-side pin are the same measurement",
      );
    }
  }
});

test("every traversal entry is one single-line inline table", () => {
  const entryLines = inlineTableLines("traversal");
  assert.equal(
    entryLines.length,
    Object.keys(budgets.traversal).length,
    "every [traversal] entry must be exactly one `<id> = { … }` line",
  );
  for (const line of entryLines) {
    assert.match(
      line,
      /^[A-Za-z0-9._-]+ = \{ walks = \d+, visits = \d+ \}$/,
      `traversal entries must keep the canonical field order and spacing:\n${line}`,
    );
  }
});

test("the phase-2 target table is pinned with a real phase-start rev", () => {
  const target = budgets.target?.["phase-2"];
  assert.ok(target, "budgets.toml must carry a [target.phase-2] table (P2-12a)");
  const rev = target.phase_start_rev;
  assert.ok(
    typeof rev === "string" && /^[0-9a-f]{40}$/.test(rev),
    `[target.phase-2] phase_start_rev must be a full 40-hex commit sha, got ${String(rev)}`,
  );
  assert.match(
    String(target.phase_start_date),
    /^\d{4}-\d{2}-\d{2}$/,
    "[target.phase-2] phase_start_date must be an ISO date string",
  );
  // The numbers are the maintainer's review point; what CI owns is that every
  // one of them is present and non-zero, so the exit gate can never be scored
  // against an unset target (the P1-13 miss P2-12a exists to prevent).
  for (const field of [
    "dom_walks_max",
    "dom_visits_ratio_max",
    "dom_compile_allocs_ratio_max",
    "ssr_visits_ratio_max",
    "vapor_visits_ratio_max",
  ]) {
    const value = target[field];
    assert.ok(
      typeof value === "number" && value > 0,
      `[target.phase-2] ${field} must be a positive number, got ${String(value)}`,
    );
  }
  assert.ok(
    ladderNames().includes(String(target.dom_visits_control_fixture)),
    "[target.phase-2] dom_visits_control_fixture must name a ladder fixture",
  );
  assert.equal(
    target.wall_time,
    "report-only",
    "wall time stays report-only until the Blacksmith reference recording lands (P0-4)",
  );
});
