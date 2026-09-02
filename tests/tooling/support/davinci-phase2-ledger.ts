import assert from "node:assert/strict";
import fs from "node:fs";

function p2_11Installment(number: number): URL {
  return new URL(
    `../../../davinci-road/plan/phase-2-records/p2-11/installment-${number}.md`,
    import.meta.url,
  );
}

const P2_11_CURRENT = { number: 83, pr: 5586, sha: "c17902442d" } as const;

const p2_11TableRows = [
  [41, 5359, "5b5ac0924"],
  [42, 5360, "f659b7e4e"],
  [43, 4862, "fdaa8d165"],
  [44, 5363, "1a717a959"],
  [45, 5373, "ee8f222cb"],
  [46, 5376, "eef9f2064"],
  [47, 5379, "481df679f"],
  [48, 5380, "b67020bde"],
  [49, 5381, "b408b67c8"],
  [50, 5386, "2c5465d94"],
  [51, 5387, "41fe266a2"],
  [52, 5390, "16a3fc970"],
  [53, 5391, "e741ff65d"],
  [54, 5396, "f80bbb5d3"],
  [55, 5398, "db06c3aa1"],
  [56, 5399, "4c8a27cae"],
  [57, 5400, "6c21f0a52"],
  [58, 5401, "e7715fbc5"],
  [59, 5404, "86e52e3c7"],
  [60, 5405, "778d7969d"],
  [61, 5467, "86f4b794f"],
  [62, 5515, "85fe7bf151"],
  [63, 5520, "526f400ef"],
  [64, 5531, "589daf801"],
  [65, 5536, "7a98d785b"],
  [66, 5533, "d7040e03d"],
  [67, 5543, "da97fe2d70"],
  [68, 5552, "af800fd399"],
  [69, 5562, "3145454f43"],
  [70, 5563, "b06a3edc65"],
  [71, 5564, "eada2aa7dd"],
  [72, 5565, "07ac91d602"],
  [73, 5566, "6f21c0432e"],
  [74, 5567, "8a420bc402"],
  [75, 5568, "86f40b34b0"],
  [76, 5569, "66fbe814ba"],
  [77, 5572, "d4f6d75936"],
  [78, 5573, "466be4eeac"],
  [79, 5576, "185d49ba9f"],
  [80, 5582, "b6c6948a32"],
  [81, 5583, "e65a078d37"],
  [82, 5585, "7fad0210b5"],
  [83, 5586, "c17902442d"],
] as const;

const p2_11FileExpectations = [
  [20, /14-fixture S2-vs-shipped byte-for-byte battery/],
  [20, /does not tick P2-11/],
  [21, /Vue 2 pipe filters/],
  [22, /Vue 2 filter helper order/],
  [23, /Slot outlet same-name names/],
  [24, /Patch-flag matrix expansion/],
  [25, /Dynamic component patch flags/],
  [26, /Model listener patch order/],
  [27, /Dynamic component model arguments/],
  [28, /SFC style carriers are DOM-inert/],
  [29, /Bare Template Default Slots/],
  [30, /Inert Slot-Template Bindings/],
  [31, /Inline Slot-Template Carriers/],
  [31, /f5aa60553/],
  [32, /V-show Runtime Directives/],
  [32, /2be66b0f0/],
  [33, /V-html Raw HTML Props/],
  [33, /13cff4d99/],
  [34, /V-text Text-Content Props/],
  [34, /11750115a/],
  [35, /V-cloak DOM Cloak Markers/],
  [35, /02c4eb1a7/],
  [36, /Slot Outlet V-on Props/],
  [36, /cf7fc9a22/],
  [37, /Object V-bind Modifiers/],
  [37, /4e577b62/],
  [38, /Object V-on Modifiers/],
  [38, /f3959e7e3/],
  [39, /Recent Patch-Flag Witness/],
  [39, /22674520f/],
  [40, /publish graph firewall/i],
  [40, /be344e787/],
  [41, /Corpus Comparison Count/],
  [41, /5b5ac0924/],
  [42, /S2 DOM Emit Allocations/],
  [42, /f659b7e4e/],
  [43, /Dynamic Directive Argument Prefixing/],
  [43, /fdaa8d165/],
  [44, /Single Nested Slot Wrapper Defaults/],
  [44, /1a717a959/],
  [45, /Event Model Slot Residuals/],
  [45, /ee8f222cb/],
  [46, /If Branches Containing V-for/],
  [46, /eef9f2064/],
  [47, /Handler Body And Slot Hardening/],
  [47, /481df679f/],
  [48, /Static Class Patch Flags/],
  [48, /b67020bde/],
  [49, /Keyed Slot Template Forwarding/],
  [49, /b408b67c8/],
  [50, /Component Once Wrappers/],
  [50, /2c5465d94/],
  [51, /DOM Corpus Lane/],
  [51, /41fe266a2/],
  [52, /Opaque Bind And Empty Text Edges/],
  [52, /16a3fc970/],
  [53, /Trailing Block Comment Expressions/],
  [53, /e741ff65d/],
  [54, /Slot Text Facts/],
  [54, /f80bbb5d3/],
  [55, /V-memo Patch Flag Sites/],
  [55, /db06c3aa1/],
  [56, /Line Comment Expression Edges/],
  [56, /4c8a27cae/],
  [57, /V-once Patch Flag Sites/],
  [57, /6c21f0a52/],
  [58, /CI DOM Corpus Lane/],
  [58, /e7715fbc5/],
  [59, /Slot Outlet Patch Sites/],
  [59, /86e52e3c7/],
  [60, /CreateSlots Patch Sites/],
  [60, /778d7969d/],
  [61, /Nested Interactive Recovery Comparison/],
  [61, /86f4b794f/],
  [62, /Nested Interactive End Tags/],
  [62, /85fe7bf151/],
  [63, /Nested Interactive Close Identity/],
  [63, /526f400ef/],
  [64, /Raw Handler Expressions/],
  [64, /589daf801/],
  [65, /Dynamic Component Directive Patch Flags/],
  [65, /7a98d785b/],
  [66, /Template-Wrapper Component Props/],
  [66, /d7040e03d/],
  [67, /Component Class Binds/],
  [67, /da97fe2d70/],
] as const;

export function recordsTaskRow(source: string, id: string): string {
  const escaped = id.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
  const match = new RegExp(`^\\| \\[${escaped}\\][^\\n]+$`, "mu").exec(source);
  assert.ok(match, `missing records row for ${id}`);
  return match[0];
}

export function requiredSection(source: string, start: RegExp, end: RegExp, label: string): string {
  const startMatch = start.exec(source);
  assert.ok(startMatch, `missing ${label}`);
  const afterStart = source.slice(startMatch.index);
  const endMatch = end.exec(afterStart.slice(startMatch[0].length));
  if (endMatch == null) return afterStart;
  return afterStart.slice(0, startMatch[0].length + endMatch.index);
}

export function requiredLine(source: string, pattern: RegExp, label: string): string {
  const match = pattern.exec(source);
  assert.ok(match, `missing ${label}`);
  return match[0];
}

export function assertCurrentP2_11Installment(source: string, label: string): void {
  const record = currentP2_11InstallmentRecord(source, label);
  const current = P2_11_CURRENT.number;
  assert.ok(
    [
      new RegExp(`${current} (?:landed\\s+)?installments`, "iu"),
      new RegExp(`through installment ${current}`, "iu"),
      new RegExp(`installment ${current}`, "iu"),
      new RegExp(`^\\| ${current}\\s+\\|`, "imu"),
    ].some((marker) => marker.test(record)),
    `${label} current record must cite installment ${current}`,
  );
  assert.match(
    record,
    new RegExp(`#${P2_11_CURRENT.pr}`, "u"),
    `${label} current record must cite #${P2_11_CURRENT.pr}`,
  );
  assert.doesNotMatch(record, /\bpending\b/iu, `${label} current record must not be pending`);
}

function currentP2_11InstallmentRecord(source: string, label: string): string {
  switch (label) {
    case "roadmap":
      return requiredSection(
        source,
        /^\*\*Current execution ledger/mu,
        /^\*\*Exit gate:/mu,
        "roadmap current execution ledger",
      );
    case "readme":
      return requiredLine(source, /^\| \[phase-2\.md\][^\n]+$/mu, "plan README phase 2 row");
    case "tasks":
      return requiredSection(
        source,
        /^\*\*Current series evidence/mu,
        /^\*\*Steps:\*\*/mu,
        "P2-11 current series evidence",
      );
    case "records":
      return requiredLine(source, /^\| \[P2-11\][^\n]+$/mu, "P2-11 records index row");
    case "p2_11":
      return requiredLine(
        source,
        new RegExp(`^\\| ${P2_11_CURRENT.number}\\s+\\|[^\\n]+$`, "mu"),
        `P2-11 installment ${P2_11_CURRENT.number} row`,
      );
    default:
      throw new Error(`unknown P2-11 current evidence label: ${label}`);
  }
}

export function p2_11CurrentRecordEvidence(source: string): string {
  return [
    requiredSection(
      source,
      /^The `emit\.rs` module docs keep/mu,
      /^## Installment table/mu,
      "P2-11 current record preface",
    ),
    ...p2_11TableRows.map(([number, pr, sha]) =>
      requiredLine(
        source,
        new RegExp(`^\\| ${number}\\s+\\|[^\\n]+#${pr}[^\\n]+${sha}[^\\n]+$`, "mu"),
        `P2-11 installment ${number} row`,
      ),
    ),
    requiredSection(
      source,
      /^## Current named remainder/mu,
      /^## Not series installments/mu,
      "P2-11 current named remainder",
    ),
  ].join("\n");
}

export function assertP2_11InstallmentFiles(): void {
  const installments = new Map(
    [...new Set(p2_11FileExpectations.map(([number]) => number))].map((number) => [
      number,
      fs.readFileSync(p2_11Installment(number), "utf8"),
    ]),
  );
  for (const [number, pattern] of p2_11FileExpectations) {
    assert.match(installments.get(number)!, pattern);
  }
}

function phaseTaskSection(source: string, id: string): string {
  const start = new RegExp(`^## ${id} —`, "mu").exec(source)?.index;
  if (start == null) throw new Error(`missing ${id} contract`);
  const tail = source.slice(start);
  const next = /^## P2-/mu.exec(tail.slice(1))?.index;
  return next == null ? tail : tail.slice(0, next + 1);
}

function phaseDependencySet(source: string, id: string, taskIds: string[]): string[] {
  const section = phaseTaskSection(source, id);
  const raw = /\*\*Deps:\*\* (?<deps>[\s\S]*?) \*\*Non-goals:\*\*/u.exec(section)?.groups?.deps;
  assert.ok(raw, `missing ${id} dependency clause`);
  if (raw === "all of P2-1..P2-19.") return taskIds.filter((task) => task !== "P2-20");
  return raw.match(/P2-\d+(?:[ab])?/gu) ?? [];
}

function exitGateItems(source: string) {
  const gate = requiredSection(
    source,
    /^## Exit gate \(machine-checkable\)/mu,
    /$a/mu,
    "P2 exit gate",
  );
  return [...gate.matchAll(/^- \[(?<checked>[ x])\] \*\*(?<title>[^*]+)\*\*/gmu)].map((match) => ({
    checked: match.groups!.checked === "x",
    title: match.groups!.title,
  }));
}

export function assertP2_17P2_20ExitBlockers(
  phase: string,
  tasksLater: string,
  taskIds: string[],
  p2_17Checked: boolean | undefined,
  p2_20Checked: boolean | undefined,
): void {
  const phaseLedger = requiredSection(
    phase,
    /^## Current execution ledger/mu,
    /^## Davinci describes/mu,
    "Phase 2 current ledger",
  );
  const p2_17 = phaseTaskSection(tasksLater, "P2-17");
  const p2_20 = phaseTaskSection(tasksLater, "P2-20");
  const gateItems = exitGateItems(phase);

  assert.equal(p2_17Checked, false, "P2-17 must not be ticked before review sign-off");
  assert.equal(p2_20Checked, false, "P2-20 must not be ticked before exit evaluation");
  assert.deepEqual(phaseDependencySet(tasksLater, "P2-17", taskIds), ["P2-11", "P2-12b", "P2-13"]);
  assert.deepEqual(
    phaseDependencySet(tasksLater, "P2-20", taskIds),
    taskIds.filter((id) => id !== "P2-20"),
  );

  assert.match(p2_17, /mechanical half is machine-checked and must land as tests/);
  assert.match(p2_17, /every S2 op's span resolves into its authored SFC/);
  assert.match(p2_17, /`schema_version` is present and negotiated/);
  assert.match(p2_20, /a line is ticked only when it is satisfied/);
  assert.match(p2_20, /an unticked line names its blocker/);
  assert.match(p2_20, /no line's wording is softened to make it tickable/);

  assert.match(phaseLedger, /P2-17\/P2-20 pre-exit blocker map/);
  assert.match(phaseLedger, /P2-11's S2 DOM lane/);
  assert.match(phaseLedger, /P2-12b's traversal-budget swap/);
  assert.match(phaseLedger, /P2-13's failure\s+provenance contract/);
  assert.match(phaseLedger, /span-resolution and `schema_version` negotiation checks/);
  assert.match(phaseLedger, /P2-20 cannot evaluate the exit gate until every P2-1\.\.P2-19/);
  assert.match(phaseLedger, /tick a line only with evidence/);

  assert.equal(gateItems.length, 12, "the P2 exit gate item count changed");
  assert.deepEqual(
    gateItems.filter((item) => item.checked),
    [],
    "P2 exit gate must remain unticked until P2-20 evaluation records evidence",
  );
  assert.ok(
    gateItems.some((item) => item.title === "IR contract review signed off"),
    "P2-17 must remain an exit-gate line",
  );
  assert.ok(
    gateItems.some(
      (item) => item.title === "Differential lanes green and their retirement condition restated",
    ),
    "P2-20 must continue to gate differential-lane retirement",
  );
  assert.ok(
    gateItems.some(
      (item) =>
        item.title === "Corpus waiver ledger empty and the phase-boundary expansion audit done",
    ),
    "P2-20 must continue to gate the C-16 waiver-ledger review",
  );
}
