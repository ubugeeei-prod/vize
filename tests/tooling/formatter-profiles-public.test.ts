import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { test } from "node:test";
import {
  installedInitializeParams,
  installedInitializationOptions,
  type InstalledAliasInitializationOptions,
} from "./support/lsp/installed-alias-replay/protocol-types.ts";
import {
  originalBytes,
  originalContract,
  validateOriginal,
} from "../../tools/support/release/formatter_profiles_public/authority.ts";
import {
  assertAuthoredUnchanged,
  assertVersionUnchanged,
  packagePayload,
  preservePayload,
  type Context,
} from "../../tools/support/release/formatter_profiles_public/install.ts";
import {
  assertWholeFilesystem,
  assertWholeReply,
  changedTextEvidence,
  diagnosticTextLimits,
  formattingInitialization,
} from "../../tools/support/release/formatter_profiles_public/session.ts";
import {
  invalidSemiFlags,
  semiCases,
  semiCells,
} from "../../tools/support/release/formatter_profiles_public/semi.ts";
import {
  applyTextEdits,
  defaultFormattingOptions,
  sortOriginal,
} from "../../tools/support/release/formatter_profiles_public/sort.ts";
import {
  artOriginal,
  artProfileBodies,
} from "../../tools/support/release/formatter_profiles_public/art.ts";
import {
  assertEnvironment,
  forbiddenOverrides,
} from "../../tools/support/release/formatter_profiles_public/run.ts";
import { Recorder } from "../../tools/support/config/windows_consumer/common.ts";
import { inventory, type Entry } from "../../tools/support/release/editor_vite_stub/filesystem.ts";

const original = originalContract();
test("all immutable original inputs and complete source oracles retain their actual Git bytes", () => {
  assert.equal(original.references.length, 19);
  assert.equal(new Set(original.references.map((row) => row.path)).size, 19);
  for (const mutation of ["byte", "sha", "source", "missing", "duplicate", "order"]) {
    const value = structuredClone(original.raw);
    const rows = value.references as Record<string, unknown>[];
    if (mutation === "byte") rows[0]!.rawBase64 = Buffer.from("changed").toString("base64");
    else if (mutation === "sha") rows[0]!.gitBlob = "0".repeat(40);
    else if (mutation === "source") value.source = "1".repeat(40);
    else if (mutation === "missing") rows.pop();
    else if (mutation === "duplicate") rows.push(rows[0]!);
    else rows.reverse();
    assert.throws(() => validateOriginal(value), mutation);
  }
});

test("the actual complete initializer preserves defaults and the original formatter's explicit false flags", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-formatter-initialize-"));
  try {
    const projectRoot = realpathSync(directory),
      rootUri = pathToFileURL(projectRoot).href;
    const defaults = installedInitializeParams({ projectRoot, crossFile: true });
    assert.equal(
      JSON.stringify(defaults),
      JSON.stringify({
        processId: null,
        rootUri,
        capabilities: {},
        initializationOptions: { lint: false, typecheck: true, hover: true, crossFile: true },
      }),
    );
    assert.deepEqual(installedInitializeParams({ projectRoot, crossFile: false, lint: true }), {
      processId: null,
      rootUri,
      capabilities: {},
      initializationOptions: { lint: true, typecheck: true, hover: true, crossFile: false },
    });
    const custom = { ...formattingInitialization };
    const result = installedInitializeParams({
      projectRoot,
      crossFile: false,
      initializationOptions: custom,
    });
    assert.deepEqual(result, {
      processId: null,
      rootUri,
      capabilities: {},
      initializationOptions: { editor: true, lint: false, typecheck: false, formatting: true },
    });
    assert.notEqual(result.initializationOptions, custom);
    result.initializationOptions.typecheck = true;
    assert.equal(custom.typecheck, false);
    for (const invalid of [
      null,
      [],
      true,
      { typecheck: null },
      { formatting: "true" },
      { unknown: true },
    ]) {
      assert.throws(() =>
        installedInitializationOptions({
          crossFile: false,
          initializationOptions: invalid as InstalledAliasInitializationOptions,
        }),
      );
    }
  } finally {
    rmSync(directory, { recursive: true });
  }
});

test("the delivered twelve Semi cells retain the complete four independent Prettier outputs", () => {
  const cases = semiCases(original);
  assert.deepEqual(semiCells, [
    { configured: false, flags: [], semi: false },
    { configured: false, flags: ["--semi"], semi: true },
    { configured: false, flags: ["--semi=true"], semi: true },
    { configured: true, flags: [], semi: true },
    { configured: true, flags: ["--semi=false"], semi: false },
    { configured: true, flags: ["--no-semi"], semi: false },
  ]);
  assert.deepEqual(invalidSemiFlags, [
    ["--semi=invalid"],
    ["--semi", "--no-semi"],
    ["--no-semi", "--semi=false"],
  ]);
  assert.equal(cases.length * semiCells.length, 12);
  assert.equal(
    cases[0]!.withSemicolons,
    "// 雪🌸 original JS\nconst greeting = 'hello';\nconsole.log(greeting);\n",
  );
  assert.equal(
    cases[0]!.withoutSemicolons,
    "// 雪🌸 original JS\nconst greeting = 'hello'\nconsole.log(greeting)\n",
  );
  assert.equal(
    cases[1]!.withSemicolons,
    "// 雪🌸 original TS\nconst greeting: string = 'hello';\nconsole.log(greeting);\n",
  );
  assert.equal(
    cases[1]!.withoutSemicolons,
    "// 雪🌸 original TS\nconst greeting: string = 'hello'\nconsole.log(greeting)\n",
  );
});

test("whole Sort and Art packets keep their original ranges, complete text and UTF16 boundaries", () => {
  const context = { original },
    sort = sortOriginal(context),
    art = artOriginal(context);
  assert.equal(applyTextEdits(sort.source, sort.edits), sort.sorted);
  assert.deepEqual(defaultFormattingOptions, { tabSize: 0, insertSpaces: false });
  const applied = applyTextEdits(art.source, art.edits);
  assert.equal(
    applied,
    '<template><p>🌸 untouched</p></template>\n<!--🌸--><art title="one">\n  <Button\n    data-z="z"\n    data-a="a"\n  />\n</art>\n<script setup>const   value=1</script>\n',
  );
  assert.deepEqual(
    artProfileBodies.map(([preset]) => preset),
    ["vize", "prettier", "compact", "documentation"],
  );
  const expected = { jsonrpc: "2.0", id: 2, result: art.edits };
  assertWholeReply(expected, 2, art.edits);
  for (const mutated of [
    { ...expected, id: 3 },
    { ...expected, result: [] },
    { ...expected, result: [...(art.edits as unknown[]), ...(art.edits as unknown[])] },
    { ...expected, error: { code: -32603, message: "wrong" } },
  ])
    assert.throws(() => assertWholeReply(mutated, 2, art.edits));
  assert.throws(() =>
    applyTextEdits("🌸", [
      { range: { start: { line: 0, character: 1 }, end: { line: 0, character: 2 } }, newText: "x" },
    ]),
  );
  assert.throws(() =>
    applyTextEdits("abc", [
      { range: { start: { line: 9, character: 0 }, end: { line: 9, character: 1 } }, newText: "x" },
    ]),
  );
  assert.throws(() =>
    applyTextEdits("abc", [
      {
        range: { start: { line: "0", character: 0 }, end: { line: 0, character: 1 } },
        newText: "x",
      },
    ]),
  );
  assert.equal(
    originalBytes(original, "crates/vize_maestro/src/server/handlers/tests/formatting.rs")
      .toString("utf8")
      .includes('ch: "}".to_string()'),
    true,
  );
});

test("whole LSP filesystem guards reject extra overlay/cache rows, payload changes and link retargeting", () => {
  const baseline: Entry[] = [
    { path: "node_modules", kind: "directory", mode: 0o40755 },
    {
      path: "node_modules/vize/bin/vize",
      kind: "file",
      mode: 0o100755,
      bytes: 1,
      sha256: "a".repeat(64),
    },
    {
      path: "retained",
      kind: "link",
      mode: 0o120755,
      target: "node_modules",
      resolved: "/original/node_modules",
    },
  ];
  assertWholeFilesystem(structuredClone(baseline), baseline, "unchanged");
  for (const actual of [
    [
      ...baseline,
      {
        path: "node_modules/.vize/corsa-overlay/extra.ts",
        kind: "file" as const,
        mode: 0o100644,
        bytes: 1,
        sha256: "b".repeat(64),
      },
    ],
    baseline.map((row) => (row.kind === "file" ? { ...row, sha256: "c".repeat(64) } : row)),
    baseline.map((row) =>
      row.kind === "link" ? { ...row, target: "foreign", resolved: "/foreign" } : row,
    ),
  ])
    assert.throws(() => assertWholeFilesystem(actual, baseline, "mutation"));
  const authored: Entry[] = [
    { path: "App.vue", kind: "file", mode: 0o100644, bytes: 1, sha256: "a".repeat(64) },
    { path: "Other.vue", kind: "file", mode: 0o100644, bytes: 1, sha256: "a".repeat(64) },
  ];
  const changed = authored.map((row) => ({ ...row, sha256: "b".repeat(64) }));
  assert.throws(() => assertAuthoredUnchanged(authored, changed, ["App.vue"]));
  assert.throws(() =>
    assertAuthoredUnchanged(
      authored,
      [{ ...authored[0]!, kind: "link", target: "foreign" }, authored[1]!],
      ["App.vue"],
    ),
  );
});

test("version initialization refuses every unrelated dependency and authored-file mutation", () => {
  const file = (path: string): Entry => ({
    path,
    kind: "file",
    mode: 0o100644,
    bytes: 1,
    sha256: "a".repeat(64),
  });
  const before = {
    project: [file("package-lock.json"), file("vite.config.mjs")],
    modules: [file("unrelated/index.js")],
  };
  assertVersionUnchanged(before, structuredClone(before));
  for (const part of ["project", "modules"] as const) {
    const changed = structuredClone(before);
    changed[part][0]!.sha256 = "b".repeat(64);
    assert.throws(() => assertVersionUnchanged(before, changed));
    const added = structuredClone(before);
    added[part].push(file("unexpected"));
    assert.throws(() => assertVersionUnchanged(before, added));
    const removed = structuredClone(before);
    removed[part].pop();
    assert.throws(() => assertVersionUnchanged(before, removed));
  }
});

test("actual payload guard persists the changed finite fixture before refusing it", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-formatter-payload-"));
  try {
    const root = join(directory, "consumer"),
      modules = join(root, "node_modules");
    mkdirSync(join(modules, "vize"), { recursive: true });
    const target = join(modules, "vize", "entry.js");
    writeFileSync(target, "original");
    const record = new Recorder(join(directory, "proof"));
    const context: Context = {
      root,
      version: "0.626.0",
      cli: "unused-no-execution",
      identity: {},
      original,
      record,
      payloadRoots: ["vize"],
      payload: packagePayload(inventory(modules), ["vize"]),
    };
    preservePayload(context, "before");
    writeFileSync(target, "tampered");
    assert.throws(() => preservePayload(context, "after"));
    const evidence: unknown = JSON.parse(
      readFileSync(join(record.root, "after.modules.json"), "utf8"),
    );
    assert(Array.isArray(evidence) && evidence.length > 0);
  } finally {
    rmSync(directory, { recursive: true });
  }
});

test("many small changed files and aggregate bytes are bounded without hiding any filesystem failure", () => {
  const directory = mkdtempSync(join(tmpdir(), "vize-formatter-diagnostic-cap-"));
  try {
    for (let index = 0; index < 130; index++)
      writeFileSync(join(directory, index + ".js"), "changed");
    const rows = inventory(directory).filter((row) => row.kind === "file");
    const proof = changedTextEvidence(directory, rows);
    assert.deepEqual(proof.limits, {
      perFileBytes: 2 * 1024 ** 2,
      totalBytes: 8 * 1024 ** 2,
      files: 128,
    });
    assert.deepEqual(diagnosticTextLimits, proof.limits);
    assert.equal(proof.capturedFiles, 128);
    assert.equal(proof.omittedFiles, 2);
    assert.equal(proof.omissionReasons["file-count-budget"], 2);
    assert.equal(proof.candidateEntries, 130);
    assert.throws(() => assertWholeFilesystem(rows, [], "all130changedrows-still-refused"));
    const bytes = changedTextEvidence(directory, rows.slice(0, 3), {
      perFileBytes: 16,
      totalBytes: 12,
      files: 128,
    });
    assert.equal(bytes.rawBytes, 7);
    assert.equal(bytes.capturedFiles, 1);
    assert.equal(bytes.omissionReasons["aggregate-byte-budget"], 2);
    assert.equal(rows.length, 130);
  } finally {
    rmSync(directory, { recursive: true });
  }
});

test("source/provider overrides and non-main execution cannot turn the role into another provider", () => {
  const env = { GITHUB_ACTIONS: "true", GITHUB_REF: "refs/heads/main" };
  assertEnvironment(env);
  for (const key of [...forbiddenOverrides, "VIZE_LSP_LOG_FILE"])
    assert.throws(() => assertEnvironment({ ...env, [key]: "unexpected" }), key);
  assert.throws(() => assertEnvironment({ ...env, GITHUB_REF: "refs/heads/feature" }));
  assert.throws(() => assertEnvironment({ ...env, VIZE_LSP_LOG_FILE: "" }));
});
