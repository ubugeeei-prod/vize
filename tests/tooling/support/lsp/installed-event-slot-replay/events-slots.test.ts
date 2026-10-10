import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { applyWorkspaceEdit } from "../installed-alias-replay/apply.ts";
import {
  auditEventSlotRecord,
  eventSlotSourceShas,
  eventSlotTsconfig,
  eventSlotVizeConfig,
  loadEventSlotCases,
  prepareEventSlotExpectation,
  type EventSlotCase,
  type EventSlotRecord,
} from "./events-slots.ts";

const repository = fileURLToPath(new URL("../../../../../", import.meta.url));
const cases = loadEventSlotCases(repository);
const projectRoot = path.join(os.tmpdir(), "authored-event-slot-law-root");
const uri = (file: string) => pathToFileURL(path.join(projectRoot, file)).href;
type ObjectValue = Record<string, any>;
function clone<T>(value: T): T {
  return structuredClone(value);
}
/** Pre-query audit-law inputs are expectations, never a simulated provider. */
function lawRecord(testCase: EventSlotCase): EventSlotRecord {
  const expected = prepareEventSlotExpectation(testCase, projectRoot);
  const publications = [1, 2, 3].flatMap((version) =>
    testCase.files.map(({ file }) => ({
      jsonrpc: "2.0",
      method: "textDocument/publishDiagnostics",
      params: { uri: uri(file), version, diagnostics: [] },
    })),
  );
  const count = testCase.files.length;
  return {
    expected,
    actual: clone(expected),
    initialPublications: publications.slice(0, count),
    changedPublications: publications.slice(count, count * 2),
    repairedPublications: publications.slice(count * 2),
    referencesReply: { jsonrpc: "2.0", id: 2, result: expected.references },
    renameReply: { jsonrpc: "2.0", id: 3, result: expected.rename },
    notificationStart: 0,
    allNotifications: [],
    allowedWholePublications: publications,
    supplementalAllowedWholePublications: [],
    failures: [],
  };
}

test("exact frozen eighteen originals preserve family/origin/newline and owned controls", () => {
  assert.equal(cases.length, 18);
  assert.deepEqual(
    ["event", "slot", "quoted-slot"].map(
      (family) => cases.filter((item) => item.family === family).length,
    ),
    [6, 6, 6],
  );
  assert.equal(new Set(cases.map(({ context }) => context)).size, 18);
  assert.equal(new Set(cases.map(({ id }) => id)).size, 18);
  assert.equal(Object.keys(eventSlotSourceShas).length, 13);
  for (const item of cases) {
    assert.ok(
      Object.isFrozen(item) && Object.isFrozen(item.files) && Object.isFrozen(item.files[0]),
    );
    assert.ok(Object.isFrozen(item.expected) && Object.isFrozen(item.expected.rename));
    assert.ok(!("nativeGuard" in item.expected), "original18 never retrospectively gains a guard");
    for (const file of item.files) {
      assert.equal(file.text.includes("\r"), item.newline === "\r\n");
      assert.equal(file.golden.includes("\r"), item.newline === "\r\n");
    }
    if (item.family === "quoted-slot") {
      assert.equal(
        item.files[2].text,
        item.files[2].golden,
        "Other.vue foreign owner stays byte-exact",
      );
      assert.match(item.files[1].golden, /#item-row="\{ title \}"/);
      assert.match(item.files[1].golden, /#\[slotName\]/);
      assert.match(item.files[0].golden, /typeof payload\.title/);
    }
  }
});

test("all complete authored edit packets produce only independent full goldens", () => {
  for (const item of cases) {
    const expected = prepareEventSlotExpectation(item, projectRoot);
    const applied = applyWorkspaceEdit(
      expected.rename,
      item.files.map(({ file, text }) => ({ uri: uri(file), text, version: 1 })),
    );
    assert.deepEqual(
      applied.map(({ text }) => text),
      item.files.map(({ golden }) => golden),
    );
    assert.deepEqual(
      expected.files,
      item.files.map(({ file, golden }) => ({ file, text: golden, disk: golden, diagnostics: [] })),
    );
    assert.deepEqual(
      expected.independentRepair,
      item.files.map(({ file, golden }) => ({
        file,
        text: golden,
        disk: golden,
        diagnostics: [],
        version: 3,
      })),
    );
    assert.equal((expected.references as unknown[]).length, 3);
  }
});

test("quoted first-occurrence positions exclude foreign owners and preserve complete ordering", () => {
  const item = cases.find(
    ({ family, origin, newline }) =>
      family === "quoted-slot" && origin === "consumer" && newline === "\n",
  )!;
  const expected = prepareEventSlotExpectation(item, projectRoot) as ObjectValue;
  assert.ok(
    item.files[1].text.indexOf(item.query.needle) <
      item.files[1].text.lastIndexOf(item.query.needle),
  );
  assert.deepEqual(expected.references, [
    {
      uri: uri("Card.vue"),
      range: { start: { line: 4, character: 3 }, end: { line: 4, character: 11 } },
    },
    {
      uri: uri("Card.vue"),
      range: { start: { line: 11, character: 16 }, end: { line: 11, character: 24 } },
    },
    {
      uri: uri("CardUser.vue"),
      range: { start: { line: 9, character: 15 }, end: { line: 9, character: 23 } },
    },
  ]);
  assert.deepEqual(Object.keys(expected.rename.changes), [uri("Card.vue"), uri("CardUser.vue")]);
  assert.ok(!Object.keys(expected.rename.changes).some((key) => key.endsWith("/Other.vue")));
});

test("original source, independent golden, and Rust-law mutations fail byte custody", () => {
  const copy = fs.mkdtempSync(path.join(os.tmpdir(), "event-slot-custody-"));
  const owners = [
    ...Object.keys(eventSlotSourceShas).map((file) => `tests/_fixtures/differential/lsp/${file}`),
    "crates/vize/tests/lsp_original_event_rename_cli.rs",
    "crates/vize/tests/lsp_original_slot_rename_cli.rs",
    "crates/vize/tests/support/lsp_authored_rename.rs",
    "crates/vize/tests/support/lsp_vue_project.rs",
  ];
  try {
    for (const owner of owners) {
      const target = path.join(copy, owner);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.copyFileSync(path.join(repository, owner), target);
    }
    assert.equal(loadEventSlotCases(copy).length, 18);
    for (const owner of [
      owners[0],
      "tests/_fixtures/differential/lsp/event-rename/8010/UpdatedToggle.vue.txt",
      owners.at(-1)!,
    ]) {
      const target = path.join(copy, owner),
        original = fs.readFileSync(target);
      fs.appendFileSync(target, "\n");
      assert.throws(() => loadEventSlotCases(copy), /SHA/);
      fs.writeFileSync(target, original);
    }
  } finally {
    fs.rmSync(copy, { recursive: true, force: true });
  }
});

test("URI rebinding requires explicit owned identities and rejects foreign values/keys", () => {
  const item = cases[0];
  assert.throws(() => prepareEventSlotExpectation(item, "relative-root"), /absolute/);
  for (const value of [
    { references: [{ uri: "file:///foreign/Toggle.vue" }] },
    { rename: { changes: { "file:///foreign/Toggle.vue": [] } } },
  ])
    assert.throws(
      () => prepareEventSlotExpectation({ ...item, expected: value }, projectRoot),
      /explicitly owned/,
    );
  const expected = prepareEventSlotExpectation(item, projectRoot);
  assert.ok(Object.isFrozen(expected));
  assert.deepEqual(Object.keys(expected).toSorted(), [
    "files",
    "independentRepair",
    "references",
    "rename",
  ]);
});

test("whole original contracts reject reordered references, partial edits, and hidden diagnostics", () => {
  const item = cases[0];
  for (const mutate of [
    (record: EventSlotRecord) => (record.actual.references as unknown[]).reverse(),
    (record: EventSlotRecord) =>
      delete (record.actual.rename as ObjectValue).changes[uri("App.vue")],
    (record: EventSlotRecord) =>
      ((record.actual.files as ObjectValue[])[0].diagnostics = [{ code: 2322 }]),
    (record: EventSlotRecord) =>
      (record.initialPublications[0] = { ...record.initialPublications[0], extra: true }),
    (record: EventSlotRecord) => (record.renameReply = { ...record.renameReply, id: 4 }),
  ]) {
    const record = lawRecord(item);
    mutate(record);
    assert.ok(auditEventSlotRecord(record, record.allowedWholePublications).length > 0);
  }
});

test("late full publications remain visible; repeated allowed values never become a count oracle", () => {
  const record = lawRecord(cases[0]);
  const beforeController = {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri: uri("NativeGuard.vue"), version: 1, diagnostics: [{ code: 2322 }] },
  };
  record.notificationStart = 1;
  const repeated = [
    beforeController,
    ...record.allowedWholePublications,
    record.allowedWholePublications[1],
    record.allowedWholePublications[1],
  ];
  assert.deepEqual(auditEventSlotRecord(record, repeated), []);
  assert.equal(record.allNotifications.length, record.allowedWholePublications.length + 2);
  const late = {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri: uri("App.vue"), version: 3, diagnostics: [{ code: 2322 }] },
  };
  assert.ok(
    auditEventSlotRecord(record, [...repeated, late]).some(
      ({ context }) => context === "complete original publication stream",
    ),
  );
  assert.deepEqual(record.allNotifications.at(-1), late);
});

test("original root tsconfig and public runtime adaptation preserve all other options", () => {
  assert.deepEqual(JSON.parse(eventSlotTsconfig), {
    compilerOptions: {
      strict: true,
      target: "ES2022",
      module: "ESNext",
      moduleResolution: "bundler",
      noEmit: true,
    },
    include: ["*.vue"],
  });
  assert.deepEqual(eventSlotVizeConfig, {
    experimentals: { patternedTemplate: false },
    typeChecker: { checkFallthroughAttrs: false, optionsApi: false },
    lsp: { lint: false, typecheck: true, hover: true, crossFile: true },
  });
});

test("supplemental whole guard values allow late duplicates without entering the original oracle", () => {
  const record = lawRecord(cases[0]);
  const repaired = {
    jsonrpc: "2.0",
    method: "textDocument/publishDiagnostics",
    params: { uri: uri("NativeGuard.vue"), version: 2, diagnostics: [] },
  };
  record.supplementalAllowedWholePublications = [repaired];
  const original = clone(record.expected);
  const stream = [...record.allowedWholePublications, repaired, repaired];
  assert.deepEqual(auditEventSlotRecord(record, stream), []);
  assert.deepEqual(record.expected, original);
  assert.equal(record.allNotifications.length, record.allowedWholePublications.length + 2);
  for (const packet of [
    { ...repaired, extra: true },
    { ...repaired, params: { ...repaired.params, uri: uri("Foreign.vue") } },
    { ...repaired, params: { ...repaired.params, diagnostics: [{ code: 2322 }] } },
  ]) {
    record.supplementalAllowedWholePublications = [packet];
    assert.ok(
      auditEventSlotRecord(record, [...stream, packet]).some(
        ({ context }) => context === "supplemental pre-query guard contract",
      ),
    );
  }
});
