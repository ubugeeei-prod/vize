import assert from "node:assert/strict";
import fs from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";
import { gunzipSync } from "node:zlib";
import { utf16Position } from "../installed-alias-replay/apply.ts";
import {
  applyLibraryTransaction,
  auditLibraryRecord,
  libraryGuardSource,
  libraryGuardRepair,
  libraryTsconfig,
  libraryVizeConfig,
  loadFrozenLibraryCases,
  prepareLibraryExpectation,
  remapLibraryExpected,
  selectLibraryOracle,
  type LibraryCase,
  type LibraryRecord,
} from "./library.ts";

const fixtureRoot = fileURLToPath(
  new URL(
    "../../../../_fixtures/differential/lsp/installed-event-slot-replay/library/",
    import.meta.url,
  ),
);
const repositoryRoot = path.resolve(fixtureRoot, "../../../../../..");
const projectRoot = path.join(tmpdir(), "library22 pure authored 😀 project");
const cases = loadFrozenLibraryCases(fixtureRoot);
const toggleUri = pathToFileURL(path.join(projectRoot, "src/Toggle.vue")).href;
const appUri = pathToFileURL(path.join(projectRoot, "src/App.vue")).href;
const manifest = JSON.parse(fs.readFileSync(path.join(fixtureRoot, "manifest.json.txt"), "utf8"));

test("authentic22 fixes every context, original config, immutable source and independent golden before queries", () => {
  const contexts = ["\n", "\r\n"].flatMap((newline) => {
    const suffix = `newline=${JSON.stringify(newline)}`;
    return [
      ...[false, true].flatMap((parent) =>
        ["Original", "UnsavedEmoji", "Restored"].map(
          (snapshot) =>
            `original strict event safe update ${snapshot}, parent_open=${parent}, ${suffix}`,
        ),
      ),
      `cold event/value collision, unopened parent, ${suffix}`,
      ...[false, true].map(
        (parent) => `ambiguous mutable event whole refusal, parent_open=${parent}, ${suffix}`,
      ),
      ...["MissingCall", "CallOnly"].map(
        (kind) => `incomplete event application ${kind}, ${suffix}`,
      ),
    ];
  });
  assert.deepEqual(cases.map((item) => item.context).toSorted(), contexts.toSorted());
  assert.deepEqual(libraryVizeConfig, {
    typeChecker: {},
    lsp: { lint: true, typecheck: true, hover: true, crossFile: true },
  });
  assert.deepEqual(JSON.parse(libraryTsconfig).include, ["src/**/*.vue"]);
  assert.equal(libraryGuardRepair, libraryGuardSource.replace("'wrong'", "1"));
  assert.equal(libraryGuardSource.includes("\r"), false);
  assert.ok(Object.isFrozen(cases) && Object.isFrozen(cases[0].inputs));
  assert.throws(() => Object.assign(cases[0].inputs, { toggle: "changed" }), TypeError);
  assert.equal(
    cases.reduce(
      (total, oracle) => total + prepareLibraryExpectation(oracle, projectRoot).responses.length,
      0,
    ),
    48,
  );
});

test("all22 expected-only projections never read historical actual or batch witness fields", () => {
  for (const entry of manifest.cases) {
    const packet = JSON.parse(
      gunzipSync(fs.readFileSync(path.join(fixtureRoot, entry.file))).toString("utf8"),
    );
    for (const key of ["actual", "diagnosticWitness"])
      Object.defineProperty(packet, key, {
        get() {
          throw new Error(`historical ${key} is not an oracle`);
        },
      });
    for (const sequence of packet.publicationSequences)
      Object.defineProperty(sequence, "actual", {
        get() {
          throw new Error("historical actual publication is not an oracle");
        },
      });
    const oracle = selectLibraryOracle(packet);
    assert.equal(oracle.context, entry.context);
    assert.equal("actual" in oracle, false);
    assert.equal("diagnosticWitness" in oracle, false);
    assert.deepEqual(
      oracle,
      cases.find((item) => item.context === entry.context),
    );
  }
});

test("whole artifact, compressed packet and original source tampering refuse before a provider exists", () => {
  const temporary = fs.mkdtempSync(path.join(tmpdir(), "library22-custody-"));
  const root = path.join(
    temporary,
    "tests/_fixtures/differential/lsp/installed-event-slot-replay/library",
  );
  try {
    fs.cpSync(fixtureRoot, root, { recursive: true });
    for (const file of Object.keys(manifest.sourceFiles)) {
      const target = path.join(temporary, file);
      fs.mkdirSync(path.dirname(target), { recursive: true });
      fs.copyFileSync(path.join(repositoryRoot, file), target);
    }
    assert.equal(loadFrozenLibraryCases(root).length, 22);
    const files = [
      "manifest.json.txt",
      manifest.artifactReceipt.file,
      manifest.archives[0].metadataFile,
      manifest.cases[0].file,
    ];
    for (const file of files) {
      const target = path.join(root, file),
        bytes = fs.readFileSync(target);
      fs.writeFileSync(target, Buffer.concat([bytes, Buffer.from(" ")]));
      assert.throws(() => loadFrozenLibraryCases(root));
      fs.writeFileSync(target, bytes);
    }
    const source = path.join(
      temporary,
      "tests/_fixtures/differential/lsp/rename-library-refusal/8010/Toggle.vue.txt",
    );
    fs.appendFileSync(source, " ");
    assert.throws(() => loadFrozenLibraryCases(root));
  } finally {
    fs.rmSync(temporary, { recursive: true, force: true });
  }
});

test("complete expectations preserve rename-before-references, fixed IDs and both whole prompt/native publications", () => {
  for (const oracle of cases) {
    const expected = prepareLibraryExpectation(oracle, projectRoot);
    assert.ok(Object.isFrozen(expected) && Object.isFrozen(expected.responses));
    assert.deepEqual(
      expected.responses.map((item: Record<string, any>) => item.method),
      oracle.kind === "ambiguous"
        ? Array(4).fill("textDocument/rename")
        : oracle.kind === "partial"
          ? ["textDocument/rename"]
          : ["textDocument/rename", "textDocument/references"],
    );
    assert.deepEqual(
      expected.responses.map((item: Record<string, any>) => item.packet.id),
      Array.from({ length: expected.responses.length }, (_, index) => index + 2),
    );
    assert.equal(expected.nativeGuard.disk, libraryGuardSource);
    assert.equal(expected.nativeGuard.invalid.params.diagnostics[0].code, 2322);
    assert.equal(expected.nativeGuard.repaired.length, 2);
    assert.deepEqual(
      expected.nativeGuard.repaired.map((packet: Record<string, any>) => packet.params.diagnostics),
      [[], []],
    );
    const source = oracle.inputs.toggleBuffer ?? oracle.inputs.toggle;
    if (oracle.kind !== "ambiguous") {
      const position = utf16Position(
        source,
        source.indexOf(oracle.kind === "cold" ? 'change", change' : 'hange", true'),
      );
      assert.deepEqual(position, {
        line: oracle.kind === "cold" || oracle.context.includes("UnsavedEmoji") ? 7 : 6,
        character: oracle.kind === "cold" ? 8 : 9,
      });
      assert.deepEqual(
        Object.keys(expected.transaction.rename.changes).toSorted(),
        [appUri, toggleUri].toSorted(),
      );
      assert.equal(expected.transaction.rename.changes[toggleUri].length, 2);
      assert.equal(expected.transaction.rename.changes[appUri].length, 1);
    }
    for (const group of expected.publicationSequences)
      assert.equal(group.packets.length, group.opening ? 1 : 2);
    if (oracle.kind === "safe") {
      assert.equal(oracle.inputs.toggleDisk.includes("<!-- 😀 -->"), false);
      assert.equal(source.includes("<!-- 😀 -->"), oracle.context.includes("UnsavedEmoji"));
      assert.equal(expected.initialPublications.length, oracle.parentOpen ? 2 : 1);
    }
  }
});

test("pure UTF16 application uses complete owned edits and reproduces every separate LF/CRLF full or deliberately partial golden", () => {
  for (const oracle of cases.filter((item) => item.kind !== "ambiguous")) {
    const expected = prepareLibraryExpectation(oracle, projectRoot);
    const before = JSON.stringify({ oracle, expected });
    const applied = applyLibraryTransaction(oracle, expected.transaction.rename, projectRoot);
    const rows =
      oracle.kind === "partial" ? expected.transaction.partialFiles : expected.transaction.files;
    assert.deepEqual(
      applied,
      rows.map((row: Record<string, any>) => row.text),
    );
    assert.equal(JSON.stringify({ oracle, expected }), before);
    if (oracle.kind === "partial") {
      assert.equal(expected.transaction.rename.changes[toggleUri].length, 2);
      assert.equal(expected.transaction.partialFiles[0].diagnostics[0].code, 2345);
      assert.deepEqual(
        expected.transaction.independentRepair.map((row: Record<string, any>) => row.diagnostics),
        [[], []],
      );
    }
    const goldenToggle = fs
      .readFileSync(
        path.join(
          repositoryRoot,
          oracle.kind === "cold"
            ? "tests/_fixtures/differential/lsp/rename-library-refusal/8010/supplemental/cold-value-collision/UpdatedToggle.vue.txt"
            : "tests/_fixtures/differential/lsp/event-rename/8010/UpdatedToggle.vue.txt",
        ),
        "utf8",
      )
      .replaceAll("\n", oracle.newline);
    assert.equal(
      expected.transaction.independentRepair[0].text,
      (oracle.context.includes("UnsavedEmoji") ? `<!-- 😀 -->${oracle.newline}` : "") +
        goldenToggle,
    );
  }
});

test("unknown URI keys/values, mixed historical roots and malformed whole edit targets fail without modifying inputs", () => {
  const oracle = cases.find((item) => item.kind === "safe")!;
  for (const value of [
    "file:///foreign/Toggle.vue",
    "https://foreign.example/src/Toggle.vue",
    "untitled:Toggle.vue",
    { changes: { "file:///foreign/App.vue": [] } },
    [{ uri: "file://foreign.example/src/Toggle.vue" }],
  ])
    assert.throws(() => remapLibraryExpected(value, projectRoot));
  const historical =
    "file:///home/runner/_work/vize/vize/target/vize-tests/tests/lsp-rename-authority-X/src/Toggle.vue";
  assert.throws(() =>
    remapLibraryExpected(
      [historical, historical.replace("authority-X", "authority-Y")],
      projectRoot,
    ),
  );
  const rename = prepareLibraryExpectation(oracle, projectRoot).transaction.rename;
  for (const edit of [
    null,
    { changes: { "file:///foreign/Toggle.vue": [] } },
    { ...rename, documentChanges: [] },
  ])
    assert.throws(() => applyLibraryTransaction(oracle, edit, projectRoot));
  const overlap = structuredClone(rename);
  overlap.changes[toggleUri].push(overlap.changes[toggleUri][0]);
  const before = JSON.stringify(overlap);
  assert.throws(() => applyLibraryTransaction(oracle, overlap, projectRoot));
  assert.equal(JSON.stringify(overlap), before);
  const emptyRange = structuredClone(rename);
  emptyRange.changes[toggleUri][0].range.end = emptyRange.changes[toggleUri][0].range.start;
  assert.throws(() => applyLibraryTransaction(oracle, emptyRange, projectRoot), /incomplete/);
  const partial = cases.find((item) => item.kind === "partial")!;
  const foreign = structuredClone(
    prepareLibraryExpectation(partial, projectRoot).transaction.rename,
  );
  foreign.changes["file:///foreign/Unknown.ts"] = [];
  assert.throws(() => applyLibraryTransaction(partial, foreign, projectRoot), /unowned/);
});

// Pure audit values come from preauthored contracts. No transport/process/provider is constructed.
function auditInput(oracle: LibraryCase): LibraryRecord {
  const expected = prepareLibraryExpectation(oracle, projectRoot);
  return {
    expected,
    actual: {
      transaction: structuredClone(expected.transaction),
      nativeGuard: expected.nativeGuard,
      initialDisk: expected.initialDisk,
      initialPublications: expected.initialPublications,
      responses: structuredClone(expected.responses),
      publicationSequences: expected.publicationSequences,
      libraryBytesUnchanged: true,
      errors: [],
    },
    failures: [],
  };
}

test("late publication audit preserves duplicates and refuses missing native completion, stale prompt, foreign URI/version and whole field changes", () => {
  for (const oracle of cases) {
    const record = auditInput(oracle),
      packets = record.expected.publicationGroups.flat();
    const stream = [...packets, packets.at(-1)],
      before = JSON.stringify(stream);
    assert.deepEqual(auditLibraryRecord(record, stream), []);
    assert.equal(JSON.stringify(stream), before);
    assert.equal(record.actual.allDiagnosticPublications.length, stream.length);
    const changed = structuredClone(stream);
    changed[0].params.diagnostics[0].message = "changed";
    assert.ok(
      auditLibraryRecord(record, changed).some(
        (failure) => failure.context === "whole ordered/late publication",
      ),
    );
    for (const params of [
      { ...stream[0].params, uri: "file:///foreign/NativeGuard.vue" },
      { ...stream[0].params, version: 99 },
    ])
      assert.ok(
        auditLibraryRecord(record, [...stream, { ...stream[0], params }]).some(
          (failure) => failure.context === "unowned late publication",
        ),
      );
    const incomplete = packets.filter((packet: Record<string, any>, index: number) => index !== 2);
    assert.ok(
      auditLibraryRecord(record, incomplete).some(
        (failure) => failure.context === "missing complete publication sequence",
      ),
    );
    const malformed = auditInput(oracle);
    malformed.actual.responses[0].packet.extra = true;
    assert.ok(
      auditLibraryRecord(malformed, stream).some((failure) => failure.context === "responses"),
    );
    if (oracle.kind === "partial") {
      const prompt = record.expected.publicationSequences[0].packets[0];
      assert.ok(
        auditLibraryRecord(record, [...stream, prompt]).some(
          (failure) => failure.context === "whole ordered/late publication",
        ),
      );
    }
  }
});
