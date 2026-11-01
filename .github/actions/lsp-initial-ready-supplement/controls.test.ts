import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import {
  HUGE_NODES,
  DOC,
  hugeSource,
  supportSource,
  expectedDiagnostics,
  tokenRange,
  hoverPacket,
  patchVbenLeaf,
  memoryWorkspace,
  auditPublications,
  diagnosticReceipts,
  inventory,
  type Obj,
} from "./fixtures.ts";
import {
  completionPacket,
  firstPair,
  phaseEvents,
  classifyInterval,
  resolveIssued,
  writeAll,
  type Session,
} from "./native-queries.ts";
import { parseArgs } from "./workload.ts";
import {
  mayIgnoreStatRead,
  parsePss,
  summarizeRows,
  selectedOwnerFacts,
  qualifyMemoryTarget,
} from "./resources.ts";
const source = "a".repeat(40);
const uri = "file:///fixture/App.vue";
function issued(): Obj {
  return {
    label: "invoiceTotal",
    kind: 5,
    sortText: "11",
    opaqueExtension: { values: [false, null, "retained"] },
    data: {
      vizeCompletion: {
        uri,
        revision: 73,
        requestUri: "opaque-canonical-request",
        item: {
          label: "invoiceTotal",
          kind: 5,
          sortText: "11",
          data: {
            fileName: "opaque-native-source",
            position: 987,
            name: "invoiceTotal",
            unknown: { kept: true },
          },
        },
      },
    },
  };
}
const refuse = (run: () => void): void => assert.throws(run, assert.AssertionError);
test("closed runner arguments cannot select a foreign mode/source or repeated flag", () => {
  const args = [
    "--mode",
    "timing",
    "--repo",
    "/repo",
    "--output",
    "/output",
    "--source",
    source,
    "--attempt",
    "1",
  ];
  assert.deepEqual(parseArgs(args), {
    mode: "timing",
    repo: "/repo",
    output: "/output",
    source,
    attempt: 1,
  });
  for (const [index, value] of [
    [1, "profiling"],
    [1, "observe"],
    [1, "memory"],
    [3, "relative"],
    [7, "invalid"],
    [9, "5"],
  ] as const) {
    const changed = [...args];
    changed[index] = value;
    refuse(() => parseArgs(changed));
  }
  refuse(() => parseArgs([...args, "--retry", "true"]));
  const duplicate = [...args];
  duplicate[8] = "--mode";
  refuse(() => parseArgs(duplicate));
});
test("huge fixture has4096 distinct typechecked member nodes and first/final independent errors", () => {
  const invalid = hugeSource(false),
    repaired = hugeSource(true),
    support = supportSource();
  assert.equal((invalid.match(/<div>/g) ?? []).length, HUGE_NODES);
  assert.equal((support.match(/field\d{4}: number/g) ?? []).length, HUGE_NODES);
  assert.ok(invalid.includes("ledger.missingFirst"));
  assert.ok(invalid.includes("ledger.missingFinal"));
  for (let i = 0; i < HUGE_NODES; i++)
    assert.equal(
      (repaired.match(new RegExp(`ledger.field${String(i).padStart(4, "0")} \\}`, "g")) ?? [])
        .length,
      1,
    );
  assert.deepEqual(
    expectedDiagnostics(invalid, "huge4096", false),
    ["missingFirst", "missingFinal"].map((token) => ({
      range: tokenRange(invalid, token),
      severity: 1,
      code: 2339,
      source: "vize/types",
      message: `Property '${token}' does not exist on type 'Ledger'.`,
    })),
  );
  assert.deepEqual(expectedDiagnostics(repaired, "huge4096", true), []);
  assert.ok(repaired.includes("const totals = makeStringTotals();"));
});
test("nonlocal probe is an imported call return, not the local literal shortcut", () => {
  const text = hugeSource(false);
  assert.ok(text.includes("import { makeTotals, makeLedger }"));
  assert.ok(text.includes("const totals = makeTotals();"));
  assert.ok(!text.includes("const totals = {"));
  assert.ok(supportSource().includes(`{\n  /** ${DOC} */\n  invoiceTotal: number;\n}`));
  assert.deepEqual(hoverPacket(text, false), {
    contents: {
      kind: "markdown",
      value:
        "```typescript\n(property) invoiceTotal: number\n```\n**Invoice total** in minor currency units.",
    },
    range: tokenRange(text, "invoiceTotal"),
  });
  assert.ok(
    String((hoverPacket(hugeSource(true), true).contents as Obj).value).includes(
      "invoiceTotal: string",
    ),
  );
});
test("Vben exact patch preserves every surrounding input and invalid declaration range", () => {
  const original = `prefix\nimport { Fallback } from '@vben/common-ui';\n\ndefineOptions({ name: 'FallbackOfflineDemo' });\nsuffix\n<Fallback status="offline" />\n`;
  const changed = patchVbenLeaf(original, false),
    repair = patchVbenLeaf(original, true);
  assert.ok(changed.startsWith("prefix\n"));
  assert.ok(changed.includes("\nsuffix\n"));
  assert.ok(repair.includes("const totals = makeStringTotals();"));
  const diagnostics = expectedDiagnostics(changed, "vben2", false);
  assert.equal(diagnostics.length, 1);
  assert.deepEqual((diagnostics[0]!.range as Obj).start, { line: 3, character: 6 });
  refuse(() => patchVbenLeaf(original + original, false));
  refuse(() => patchVbenLeaf("foreign", false));
});
test("whole native issued item preserves opaque fields and never fabricates projection authority", () => {
  const packet = [issued()];
  const result = completionPacket(packet, uri);
  assert.deepEqual(result.packet, packet);
  assert.equal(result.initialProjectionAuthority, "UNAVAILABLE");
  assert.equal(result.revision, 73);
  for (const changed of [
    [],
    [...packet, ...packet],
    [{ ...issued(), kind: 10 }],
    [{ ...issued(), label: "local" }],
  ])
    refuse(() => completionPacket(changed, uri));
  refuse(() => completionPacket(packet, "file:///foreign.vue"));
  const missing = issued();
  delete (missing.data as Obj).vizeCompletion;
  refuse(() => completionPacket([missing], uri));
});
test("one untimed resolve conserves entire issued data and unknown fields", async () => {
  const original = issued();
  const expected: Obj = {
    ...original,
    detail: "(property) invoiceTotal: number",
    documentation: {
      kind: "markdown",
      value:
        "```typescript\n(property) invoiceTotal: number\n```\n\n**Invoice total** in minor currency units.",
    },
  };
  const calls: unknown[] = [];
  const session = {
    request: async (method: string, params: unknown) => {
      calls.push({ method, params });
      return structuredClone(expected);
    },
  } as unknown as Session;
  await resolveIssued(session, { completion: { issued: original } }, () => {});
  assert.deepEqual(calls, [{ method: "completionItem/resolve", params: original }]);
  const mutated = structuredClone(expected);
  mutated.opaqueExtension = { values: [] };
  const bad = { request: async () => mutated } as unknown as Session;
  await assert.rejects(
    () => resolveIssued(bad, { completion: { issued: original } }, () => {}),
    assert.AssertionError,
  );
});
test("worker selection is not a native API interval and foreign URI cannot bind", () => {
  const worker = `starting initial type diagnostics for ${uri} version 1`;
  assert.deepEqual(
    phaseEvents(worker, uri).map((row) => row.event),
    ["collectorSelected"],
  );
  assert.equal(
    phaseEvents(
      `phase="initial_api_read" route="compiler_kinds" event="entered" logical_uri="file:///foreign.vue"`,
      uri,
    ).length,
    0,
  );
  assert.equal(
    phaseEvents(`phase="initial_api_read" route="file" event="entered" logical_uri="${uri}"`, uri)
      .length,
    0,
  );
});
test("observer coverage follows request arrival while late responses stay separate QoS evidence", () => {
  const events = [
    { event: "apiEntered", sequence: 0, ns: "100" },
    { event: "apiEnded", sequence: 1, ns: "200" },
  ];
  assert.equal(classifyInterval(events, "120", "180").state, "matchedCallerApiWait");
  assert.equal(classifyInterval(events, "120", "180").nativeCpuEntered, "UNKNOWN");
  assert.equal(classifyInterval(events, "120", "250").state, "matchedCallerApiWait");
  assert.equal(classifyInterval(events, "120", "250").responseRelation, "after");
  assert.equal(classifyInterval(events, "120", "180").responseRelation, "before");
  assert.equal(classifyInterval(events, "120").responseRelation, "unknown");
  assert.equal(classifyInterval([], "120").state, "UNMATCHED");
  assert.equal(
    classifyInterval([{ event: "apiEntered", sequence: 0, ns: "100" }], "120").state,
    "UNMATCHED",
  );
  for (const [entry, send, response, end] of [
    ["100", "100", "180", "200"],
    ["100", "120", "120", "100"],
    ["100", "250", "260", "200"],
  ])
    assert.equal(
      classifyInterval(
        [
          { event: "apiEntered", sequence: 0, ns: entry },
          { event: "apiEnded", sequence: 1, ns: end },
        ],
        send!,
        response!,
      ).state,
      "UNMATCHED",
    );
});
test("eight context roots/configs are distinct inputs and missing runtime owners refuse", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "ready-memory-controls-"));
  try {
    const workspace = memoryWorkspace(directory);
    assert.equal(new Set(workspace.roots).size, 8);
    const records = workspace.targets.map(
      (target, i) =>
        `INFO vize_editor_preparation: phase="initial_api_read" event="host_request_binding" host_uri=${target.uri} request_uri=file:///native/target-${i}.ts revision=1\nINFO vize_editor_preparation: phase="initial_api_read" route="compiler_kinds" event="complete" request_uri=file:///native/target-${i}.ts config="/native/config-${i}.json" project="p${i}" snapshot="s${i}"`,
    );
    const facts = selectedOwnerFacts(records.join("\n"), workspace.targets);
    assert.equal(new Set(facts.map((row) => row.selectedNativeConfig)).size, 8);
    refuse(() => selectedOwnerFacts("", workspace.targets));
    for (const root of workspace.roots)
      assert.deepEqual(
        JSON.parse(fs.readFileSync(path.join(root, "tsconfig.json"), "utf8")).include,
        ["*.vue", "*.ts"],
      );
  } finally {
    fs.rmSync(directory, { recursive: true });
  }
});
test("PSS is a parsed untimed observation, permission/missing data cannot become zero", () => {
  assert.equal(parsePss("Rss: 200 kB\nPss: 123 kB\n"), 123);
  refuse(() => parsePss("denied"));
  refuse(() => parsePss("Pss: unknown kB"));
});
test("RSS includes all members without executable whitelist and requires real root", () => {
  const rows = [
    { pid: 1, parentPid: 0, startTicks: "1", state: "S", rssKiB: 100 },
    { pid: 2, parentPid: 1, startTicks: "2", state: "S", rssKiB: 200 },
  ];
  assert.deepEqual(summarizeRows(rows, 1), {
    rootKiB: 100,
    treeKiB: 300,
    processes: 2,
    members: rows,
  });
  refuse(() => summarizeRows(rows, 9));
});
test("final native vectors cannot be replaced by empty, stale, extra-field or unknown publications", () => {
  const text = hugeSource(false),
    repair = hugeSource(true),
    target = { file: "/App.vue", uri, invalid: text, repair };
  const rows = [
    { uri, version: 1, diagnostics: expectedDiagnostics(text, "huge4096", false) },
    { uri, version: 2, diagnostics: [] },
  ];
  auditPublications(rows, [target], "huge4096");
  for (const changed of [
    [{ ...rows[0], diagnostics: [] }, rows[1]!],
    [rows[0]!, { ...rows[1], diagnostics: rows[0]!.diagnostics }],
    [{ ...rows[0], unknown: true }, rows[1]!],
    [{ ...rows[0], uri: "file:///foreign" }, rows[1]!],
  ])
    refuse(() => auditPublications(changed, [target], "huge4096"));
});

test("native diagnostic clocks use earliest full notification receipts, never late backlog qualification", () => {
  const invalid = hugeSource(false);
  const target = { uri, file: "/App.vue", invalid, repair: hugeSource(true) };
  const expected = expectedDiagnostics(invalid, "huge4096", false);
  const receipts = [
    { ns: "100", params: { version: 1, diagnostics: [], uri } },
    { ns: "200", params: { diagnostics: expected, uri, version: 1 } },
    { ns: "999", params: { uri, version: 1, diagnostics: expected } },
  ];
  const result = diagnosticReceipts(receipts, [target], "huge4096");
  assert.equal(result.firstNs, "200");
  assert.equal(result.allNs, "200");
  refuse(() => diagnosticReceipts(receipts.slice(0, 1), [target], "huge4096"));
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "ready-inventory-control-"));
  try {
    memoryWorkspace(directory);
    const result = inventory(directory);
    assert.equal(result.vueFiles, 8);
    assert.equal(result.vueAndTypeScriptFiles, 16);
  } finally {
    fs.rmSync(directory, { recursive: true });
  }
});

test("regular-file evidence writes loop partial UTF8 bytes and refuse zero or invalid progress", () => {
  const expected = Buffer.from("雪🌸 full raw\n");
  const retained: Buffer[] = [];
  writeAll(99, expected, (_fd, bytes, offset, length) => {
    const count = Math.min(2, length);
    retained.push(bytes.subarray(offset, offset + count));
    return count;
  });
  assert.deepEqual(Buffer.concat(retained), expected);
  for (const count of [0, -1, NaN, 0.5, 999]) refuse(() => writeAll(99, "raw", () => count));
});

test("denied known owned process cannot disappear from RSS while unrelated denial and vanished PID are distinct", () => {
  const known = new Map([
    [1, "birth-root"],
    [2, "birth-native"],
  ]);
  for (const code of ["EACCES", "EPERM"]) {
    assert.equal(mayIgnoreStatRead(code, 2, known), false);
    assert.equal(mayIgnoreStatRead(code, 999, known), true);
  }
  for (const code of ["ENOENT", "ESRCH"]) {
    assert.equal(mayIgnoreStatRead(code, 2, known), true);
    assert.equal(mayIgnoreStatRead(code, 999, known), true);
  }
  assert.equal(mayIgnoreStatRead("EIO", 2, known), false);
});

test("closed support retains multiline member JSDoc and each revision imports only its active factory", () => {
  for (const repair of [false, true]) {
    const text = hugeSource(repair);
    assert.equal(text.includes("makeStringTotals"), repair);
    assert.equal(text.includes("makeTotals"), !repair);
  }
  const multiline = supportSource();
  assert.equal((multiline.match(/\{\n  \/\*\*/g) ?? []).length, 2);
  assert.ok(!multiline.includes("{ /**"));
  const original =
    "import { Fallback } from '@vben/common-ui';\n\ndefineOptions({ name: 'FallbackOfflineDemo' });\n<Fallback status=\"offline\" />";
  for (const repair of [false, true]) {
    const text = patchVbenLeaf(original, repair);
    assert.equal(text.includes("makeStringTotals"), repair);
    assert.equal(text.includes("makeTotals"), !repair);
  }
});
test("canonical API window requires the exact host binding and same-owner route response", () => {
  const prefix = 'INFO vize_editor_preparation: phase="initial_api_read" ';
  const binding =
    prefix +
    `event="host_request_binding" host_uri=${uri} request_uri=file:///private/App.vue.ts revision=73`;
  const route = (event: string, suffix = "") =>
    prefix +
    `route="compiler_kinds" event="${event}" request_uri=file:///private/App.vue.ts project="p" snapshot="s" config="cfg"` +
    suffix;
  assert.deepEqual(phaseEvents(route("entered"), uri), []);
  assert.deepEqual(
    phaseEvents(binding.replace(uri, "file:///foreign.vue") + "\n" + route("entered"), uri),
    [],
  );
  assert.deepEqual(
    phaseEvents(binding + "\n" + route("entered").replace(' snapshot="s"', ""), uri).map(
      (row) => row.event,
    ),
    ["hostRequestBinding"],
  );
  const first = phaseEvents(binding, uri);
  const next = phaseEvents(route("entered") + "\n" + route("complete"), uri, first);
  assert.deepEqual(
    next.map((row) => row.event),
    ["apiEntered", "apiEnded"],
  );
  const events = next.map((row, i) => ({ ...row, ns: String(100 + 100 * i), sequence: i }));
  assert.equal(classifyInterval(events, "120", "250").state, "matchedCallerApiWait");
  assert.equal(
    classifyInterval([events[0]!, { ...events[1], snapshot: "foreign" }], "120").state,
    "UNMATCHED",
  );
  assert.equal(
    classifyInterval([events[0]!, { ...events[1], config: "foreign-config" }], "120").state,
    "UNMATCHED",
  );
  assert.deepEqual(
    phaseEvents(
      `starting initial type diagnostics for ${uri} version 1\nfinished initial type diagnostics for ${uri} version 1`,
      uri,
    ).map((row) => row.event),
    ["collectorSelected", "collectorFinished"],
  );
});
test("a failed whole native query still waits for full memory diagnostics and collector completion", async () => {
  let waited = 0;
  const errors: string[] = [],
    queries: Obj[] = [];
  const t = { uri, file: "/App.vue", invalid: hugeSource(false), repair: hugeSource(true) };
  const session = {
    request: async () => null,
    waitForNotification: async () => {
      waited++;
      return { uri, version: 1, diagnostics: [] };
    },
    stderrText: `finished initial type diagnostics for ${uri} version 1`,
  } as unknown as Session;
  await qualifyMemoryTarget(session, t, 1, () => {}, queries, errors, [
    { uri, version: 1, diagnostics: [] },
  ]);
  assert.equal(waited, 1);
  assert.equal(queries.length, 0);
  assert.equal(errors.length, 1);
  const changed: string[] = [];
  await qualifyMemoryTarget(session, t, 1, () => {}, [], changed, [
    { uri, version: 1, diagnostics: [{ code: 6133 }] },
  ]);
  assert.equal(changed.length, 2, "query failure and whole unexpected diagnostic both survive");
});

test("type-only hover and genuine extra unused-import diagnostic are still whole-packet failures", async () => {
  const t = { uri, file: "/App.vue", invalid: hugeSource(false), repair: hugeSource(true) };
  const session = {
    request: async (method: string) =>
      method === "textDocument/hover"
        ? {
            contents: {
              kind: "markdown",
              value: "```typescript\n(property) invoiceTotal: number\n```",
            },
            range: tokenRange(t.invalid, "invoiceTotal"),
          }
        : [issued()],
  } as unknown as Session;
  await assert.rejects(() => firstPair(session, t, false, () => {}), assert.AssertionError);
  const diagnostics = expectedDiagnostics(t.invalid, "huge4096", false);
  refuse(() =>
    auditPublications(
      [
        {
          uri,
          version: 1,
          diagnostics: [
            ...diagnostics,
            { code: 6133, message: "'makeStringTotals' is declared but its value is never read." },
          ],
        },
        { uri, version: 2, diagnostics: [] },
      ],
      [t],
      "huge4096",
    ),
  );
});
