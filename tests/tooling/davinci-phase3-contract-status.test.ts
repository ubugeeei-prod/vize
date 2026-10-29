import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const phasePath = path.join(repoRoot, "docs/davinci", "plan", "phase-3.md");
const recordsRoot = path.join(repoRoot, "docs/davinci", "plan", "phase-3-records");
const phaseExitRecordPath = path.join(recordsRoot, "p3-16.md");
const budgetsPath = path.join(repoRoot, "docs/davinci", "plan", "budgets.toml");
const benchmarkBaselineRoot = path.join(
  repoRoot,
  "tools",
  "benchmarks",
  "results",
  "davinci",
  "baseline",
);

const phase = fs.readFileSync(phasePath, "utf8");

type PhaseExitGate = {
  id: string;
  status: "open" | "pass";
  witnesses: string[];
  sourceSha: string;
  actionsRun: string;
  artifact: string;
};

const phaseExitGateDefinitions = [
  {
    id: "p3-16-parity",
    text: "Vapor: TS-33 behavioral parity green; SSR **and VDOM**: TS-11 byte-empty (P3-7 changes patch-flag derivation, so DOM parity re-gates here)",
    witnesses: [".github/workflows/davinci-lean.yml", "tools/commands/davinci/corpus-diff.rs"],
  },
  {
    id: "p3-16-source-map",
    text: "TS-31 source-map coverage ≥ budget on all three backends",
    witnesses: [
      "crates/vize_atelier_sfc/tests/davinci_ts31_sourcemap.rs",
      "tests/tooling/davinci-sourcemap-coverage.test.ts",
    ],
  },
  {
    id: "p3-16-production-reach",
    text: "Production reach: `[reach]` floors held and raised, production-path DOM parity oracle empty, no DOM shape refused for its Croquis summary (P3-17)",
    witnesses: [
      "crates/vize_atelier_sfc/tests/davinci_production_reach.rs",
      ".github/workflows/davinci-canonical-corpus.yml",
    ],
  },
  {
    id: "p3-16-vapor-performance",
    text: "Vapor compile bench beats the pinned double-transform floor",
    witnesses: [
      ".github/workflows/criterion-bench.yml",
      "docs/davinci/plan/phase-3-records/p3-6-production-perf.md",
    ],
  },
  {
    id: "p3-16-legacy-retirement",
    text: "TS-32 remarks-diff clean; old vapor/ssr lanes + flags deleted",
    witnesses: [
      "davinci/vize_l1_to_l2/tests/davinci_remarks_corpus.rs",
      "tests/tooling/davinci-remarks-diff-lane.test.ts",
    ],
  },
  {
    id: "p3-16-mandatory-suites",
    text: "TS-27/TS-28/TS-29/TS-30 all mandatory-green; TS-20 totality fuzz extended to L2→L3 green",
    witnesses: [".github/workflows/davinci-lean.yml", ".github/workflows/fuzz.yml"],
  },
] as const;

const retirementSentinels = [
  {
    label: "Vapor legacy selector",
    path: "crates/vize_atelier_vapor/src/l3.rs",
    pattern: /enum VaporL3BridgeStatus[\s\S]*?Legacy\(LegacyReason\)/u,
  },
  {
    label: "SSR legacy selector",
    path: "crates/vize_atelier_ssr/src/l4.rs",
    pattern: /enum SsrL4Selection[\s\S]*?Legacy\(LegacyReason\)/u,
  },
  {
    label: "Vapor legacy-differential feature",
    path: "crates/vize_atelier_vapor/Cargo.toml",
    pattern: /^legacy-differential\s*=/mu,
  },
  {
    label: "SSR legacy-differential feature",
    path: "crates/vize_atelier_ssr/Cargo.toml",
    pattern: /^legacy-differential\s*=/mu,
  },
] as const;

function phaseTaskIndex(source: string = phase): Map<string, boolean> {
  const todoIndex = sectionBetween(source, /^## TODO index$/mu, /^---$/mu, "Phase 3 TODO index");
  const entries = [...todoIndex.matchAll(/^- \[(?<checked>[ x])\] (?<id>P3-\d+)\b/gmu)].map(
    (match) => [match.groups!.id, match.groups!.checked === "x"] as const,
  );
  assert.ok(entries.length >= 16, "Phase 3 TODO index must cover the task ladder");
  assert.equal(new Set(entries.map(([id]) => id)).size, entries.length, "duplicate P3 task id");
  return new Map(entries);
}

function phaseSection(id: string, source: string = phase): string {
  const start = new RegExp(`^\\*\\*${id} `, "mu").exec(source)?.index;
  assert.notEqual(start, undefined, `missing ${id} section`);
  const tail = source.slice(start);
  const next = /^\*\*P3-\d+ /mu.exec(tail.slice(1))?.index;
  return next == null ? tail : tail.slice(0, next + 1);
}

function recordIds(): string[] {
  return fs
    .readdirSync(recordsRoot)
    .filter((file) => /^p3-\d+\.md$/u.test(file))
    .map((file) => file.replace(/\.md$/u, "").toUpperCase())
    .sort((a, b) => numericId(a) - numericId(b));
}

function recordFiles(): string[] {
  return fs
    .readdirSync(recordsRoot)
    .filter((file) => /^p3-\d+\.md$/u.test(file))
    .sort((a, b) => numericId(recordFileId(a)) - numericId(recordFileId(b)));
}

function recordFileId(file: string): string {
  return file.replace(/\.md$/u, "").toUpperCase();
}

function numericId(id: string): number {
  return Number(/^P3-(?<number>\d+)$/u.exec(id)?.groups?.number);
}

function lineMentionsTask(line: string, id: string): boolean {
  const escaped = id.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  return new RegExp(`(^|[^A-Z0-9-])${escaped}(?!\\d)`, "u").test(line);
}

function recordPath(id: string): string {
  return path.join(recordsRoot, `${id.toLowerCase()}.md`);
}

function recordLink(id: string): RegExp {
  return new RegExp(`phase-3-records/${id.toLowerCase()}\\.md`, "u");
}

function sectionBetween(source: string, start: RegExp, end: RegExp, label: string): string {
  const startMatch = start.exec(source);
  assert.ok(startMatch, `missing ${label}`);
  const tail = source.slice(startMatch.index);
  const endMatch = end.exec(tail.slice(startMatch[0].length));
  assert.ok(endMatch, `missing end marker for ${label}`);
  return tail.slice(0, startMatch[0].length + endMatch.index);
}

function markdownCells(line: string): string[] {
  assert.ok(line.startsWith("|") && line.endsWith("|"), `invalid markdown table row: ${line}`);
  return line
    .slice(1, -1)
    .split("|")
    .map((cell) => cell.trim());
}

function codeCell(cell: string, label: string): string {
  const value = /^`(?<value>[^`]+)`$/u.exec(cell)?.groups?.value;
  assert.ok(value, `${label} must be one backticked value`);
  return value;
}

function phaseExitLedger(): PhaseExitGate[] {
  const record = fs.readFileSync(phaseExitRecordPath, "utf8");
  const section = sectionBetween(
    record,
    /^## Phase-exit gate ledger$/mu,
    /^## Receipt policy$/mu,
    "P3-16 phase-exit gate ledger",
  );
  const rows = section.split("\n").filter((line) => line.startsWith("|"));
  assert.ok(rows.length >= 2, "P3-16 phase-exit gate ledger must be a markdown table");
  assert.deepEqual(markdownCells(rows[0]), [
    "Gate ID",
    "Status",
    "Witness paths",
    "Source SHA",
    "Actions run",
    "Artifact",
  ]);
  assert.ok(
    markdownCells(rows[1]).every((cell) => /^-+$/u.test(cell)),
    "P3-16 phase-exit gate ledger must keep a plain separator row",
  );

  return rows.slice(2).map((row) => {
    const cells = markdownCells(row);
    assert.equal(cells.length, 6, `P3-16 ledger row must have six cells: ${row}`);
    const [idCell, statusCell, witnessCell, sourceSha, actionsRun, artifact] = cells;
    const id = codeCell(idCell, "P3-16 gate ID");
    const status = codeCell(statusCell, `${id} status`);
    assert.ok(status === "open" || status === "pass", `${id} has unknown status ${status}`);
    const witnesses = [...witnessCell.matchAll(/`(?<path>[^`]+)`/gu)].map(
      (match) => match.groups!.path,
    );
    assert.ok(witnesses.length > 0, `${id} must name at least one witness path`);
    for (const witness of witnesses) {
      assert.equal(
        fs.existsSync(path.join(repoRoot, witness)),
        true,
        `${id} witness does not exist: ${witness}`,
      );
    }

    if (status === "pass") {
      assert.match(sourceSha, /^[0-9a-f]{40}$/u, `${id} must carry its exact source SHA`);
      assert.match(
        actionsRun,
        /^https:\/\/github\.com\/ubugeeei-prod\/vize\/actions\/runs\/\d+$/u,
        `${id} must carry its canonical Actions run URL`,
      );
      assert.match(
        artifact,
        /^[A-Za-z0-9][A-Za-z0-9._/-]*@sha256:[0-9a-f]{64}$/u,
        `${id} must carry an immutable artifact name and digest`,
      );
    } else {
      assert.deepEqual(
        [sourceSha, actionsRun, artifact],
        ["-", "-", "-"],
        `${id} is open and must not claim a partial receipt`,
      );
    }

    return { id, status, witnesses, sourceSha, actionsRun, artifact };
  });
}

function phaseExitChecklist(): { checked: boolean; text: string }[] {
  return [...phaseSection("P3-16").matchAll(/^- \[(?<checked>[ x])\] (?<text>.+)$/gmu)].map(
    (match) => ({ checked: match.groups!.checked === "x", text: match.groups!.text }),
  );
}

function phaseExitGate(id: string): PhaseExitGate {
  const gates = phaseExitLedger();
  const gate = gates.find((candidate) => candidate.id === id);
  assert.ok(gate, `P3-16 ledger is missing ${id}`);
  return gate;
}

test("Phase 3 task index and record slices stay in sync", () => {
  const tasks = phaseTaskIndex();
  const records = new Set(recordIds());

  for (const id of records) {
    assert.ok(tasks.has(id), `${id} has a record file but no Phase 3 TODO entry`);
  }

  for (const [id, checked] of tasks) {
    const section = phaseSection(id);
    const hasRecord = records.has(id);

    if (checked) {
      assert.ok(hasRecord, `${id} is complete but has no record file`);
      assert.match(section, recordLink(id), `${id} must link its completion record`);
      assert.match(
        section,
        /(?:^|[^A-Za-z])(?:Landed|Closed)(?:[^A-Za-z]|$)/u,
        `${id} must state its terminal status`,
      );
      continue;
    }

    if (!hasRecord) {
      assert.doesNotMatch(section, recordLink(id), `${id} must not link a missing record`);
      continue;
    }

    const record = fs.readFileSync(recordPath(id), "utf8");
    assert.match(section, recordLink(id), `${id} slice must be linked from phase-3.md`);
    assert.match(
      `${section}\n${record}`,
      /\b(?:remain|remaining|remains|open)\b/iu,
      `${id} has a slice record but must still name its remaining work`,
    );
  }
});

test("completed Phase 3 tasks are not described as unfinished in sibling records", () => {
  const completedTasks = [...phaseTaskIndex()].filter(([, checked]) => checked).map(([id]) => id);
  const unfinishedTerms = [
    "does not close",
    "still needs",
    "still remains open",
    "remains open",
    "remaining work",
  ];

  for (const file of recordFiles()) {
    const fileId = recordFileId(file);
    const source = fs.readFileSync(path.join(recordsRoot, file), "utf8");
    const lines = source.split("\n");

    for (const completedId of completedTasks) {
      if (completedId === fileId) continue;

      for (const [lineIndex, line] of lines.entries()) {
        if (!lineMentionsTask(line, completedId)) continue;

        const saysUnfinished = unfinishedTerms.some((term) => line.toLowerCase().includes(term));
        assert.equal(
          saysUnfinished,
          false,
          `${file}:${lineIndex + 1} describes completed ${completedId} as unfinished`,
        );
      }
    }
  }
});

test("P3-16 exit gates have a bijective fail-closed receipt ledger", () => {
  const tasks = phaseTaskIndex();
  const checklist = phaseExitChecklist();
  const ledger = phaseExitLedger();

  assert.equal(checklist.length, phaseExitGateDefinitions.length, "P3-16 must keep six exit gates");
  assert.equal(ledger.length, phaseExitGateDefinitions.length, "P3-16 must keep six ledger rows");

  for (const [index, definition] of phaseExitGateDefinitions.entries()) {
    const item = checklist[index];
    const gate = ledger[index];
    assert.equal(item.text, definition.text, `${definition.id} checklist text drifted`);
    assert.equal(gate.id, definition.id, `${definition.id} ledger order drifted`);
    assert.deepEqual(gate.witnesses, definition.witnesses, `${definition.id} witnesses drifted`);
    assert.equal(
      item.checked,
      gate.status === "pass",
      `${definition.id} checkbox and receipt status must agree`,
    );
  }

  if (tasks.get("P3-16")) {
    assert.equal(tasks.get("P3-6"), true, "P3-16 cannot close while P3-6 remains open");
    assert.equal(
      ledger.every((gate) => gate.status === "pass"),
      true,
      "P3-16 cannot close until all six exact receipts pass",
    );
  }
});

test("P3-16 Vapor performance fails closed without wall baselines", () => {
  const budgets = fs.readFileSync(budgetsPath, "utf8");
  const rows = [
    ...budgets.matchAll(
      /^atelier_vapor_(?<stage>transform|lower|generate|compile)_[^ =]+ = \{ wall_p50_ns = (?<wall>\d+),/gmu,
    ),
  ];
  assert.deepEqual(
    [...new Set(rows.map((row) => row.groups!.stage))].sort(),
    ["compile", "generate", "lower", "transform"],
    "the performance sentinel must cover every Vapor benchmark stage",
  );
  const zeroBudgets = rows
    .filter((row) => row.groups!.wall === "0")
    .map((row) => row[0].slice(0, row[0].indexOf(" =")));
  const baselineFiles = fs.existsSync(benchmarkBaselineRoot)
    ? fs.readdirSync(benchmarkBaselineRoot).filter((file) => file.endsWith(".json"))
    : [];

  if (zeroBudgets.length > 0 || baselineFiles.length === 0) {
    assert.equal(
      phaseExitGate("p3-16-vapor-performance").status,
      "open",
      `Vapor performance must stay open: zero budgets=${zeroBudgets.length}, baselines=${baselineFiles.length}`,
    );
    assert.equal(
      phaseExitChecklist()[3].checked,
      false,
      "Vapor performance cannot be checked without wall budgets and committed baselines",
    );
  }
});

test("P3-16 retirement fails closed while legacy selectors or features remain", () => {
  const present = retirementSentinels.filter(({ path: relativePath, pattern }) =>
    pattern.test(fs.readFileSync(path.join(repoRoot, relativePath), "utf8")),
  );

  if (present.length > 0) {
    assert.equal(
      phaseExitGate("p3-16-legacy-retirement").status,
      "open",
      `legacy retirement must stay open while present: ${present.map(({ label }) => label).join(", ")}`,
    );
    assert.equal(
      phaseExitChecklist()[4].checked,
      false,
      "legacy retirement cannot be checked while Vapor/SSR selectors or features remain",
    );
  }
});
