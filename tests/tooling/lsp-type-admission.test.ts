import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { admissionFiles } from "./support/type-admission-project.ts";
import { templateRefAdmission } from "./support/type-admission-fixtures.ts";
import { templateRefEdgeAdmission } from "./support/type-admission-ref-edges.ts";
import { hoverToText, isDiagnosticsForUri, offsetToPosition } from "./support/lsp/assertions.ts";
import { root } from "./support/lsp/paths.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { LspSession } from "./support/lsp/session.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { workspace } from "./support/upstream/vue-language-tools.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

test("LSP type evidence and unique ref mutations survive open, change, and repair", async (t) => {
  const checker = requireTypecheckDependency(
    t,
    resolveTypecheckRuntime(root),
    "type admission backend",
    "TypeScript native backend unavailable",
  );
  if (!checker) return;
  const directory = workspace("lsp-type-admission-");
  const valid = admissionFiles("valid");
  const invalid = admissionFiles("invalid");
  const capture = path.join(root, "target/differential/template-ref-type-admission-lsp");
  fs.mkdirSync(capture, { recursive: true });
  const record = (name: string, value: unknown) =>
    fs.writeFileSync(path.join(capture, name + ".json"), JSON.stringify(value, null, 2) + "\n");
  record("authored-inputs", { valid, invalid, source: process.env.GITHUB_SHA, backend: checker });
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        strict: true,
        noEmit: true,
        skipLibCheck: true,
        module: "ESNext",
        moduleResolution: "Bundler",
        target: "ES2024",
        lib: ["ES2024", "DOM"],
        types: [],
      },
      include: ["*.vue", "*.ts"],
    }),
  );
  writeVueConfigFixture(directory, {
    lsp: { hover: true, lint: false, typecheck: true },
    typeChecker: { corsaPath: checker },
  });
  for (const [file, source] of Object.entries(valid.files))
    fs.writeFileSync(path.join(directory, file), source);
  const session = new LspSession();
  const uri = (file: string) => pathToFileURL(path.join(directory, file)).href;
  const rows = invalid.expected;
  const diagnostics = async (file: string, version: number) => {
    return (await session.waitForNotification(
      "textDocument/publishDiagnostics",
      (packet) => isDiagnosticsForUri(packet, uri(file)) && packet.version === version,
      120_000,
    )) as PublishDiagnosticsParams;
  };
  try {
    await session.initialize(directory, {
      editor: true,
      hover: true,
      lint: false,
      typecheck: true,
    });
    for (const row of rows)
      session.notify("textDocument/didOpen", {
        textDocument: {
          uri: uri(row.file),
          languageId: "vue",
          version: 1,
          text: valid.files[row.file],
        },
      });
    const clean = await Promise.all(rows.map((row) => diagnostics(row.file, 1)));
    record("valid-whole-packets", clean);
    for (const packet of clean) assert.deepEqual(packet.diagnostics, [], packet.uri);

    // Hover is an additional typed query, independent of the diagnostic stream.
    // Authored Exact/NotAny/NotUnknown/NotNever constraints cover deeper shapes.
    const hoverPackets: unknown[] = [];
    for (const [file, anchor, token, expected] of [
      ["TypedDom-1.vue", "button.value?.click", "button", /HTMLButtonElement/],
      ["InferredDom-1.vue", "button.value?.click", "button", /HTMLButtonElement/],
      ["InferredComponent-1.vue", "child.value?.focus", "focus", /focused/],
      ["GenericComponent-1.vue", "generic.value?.value.extra", "extra", /boolean/],
    ] as const) {
      const source = valid.files[file];
      const anchorOffset = source.indexOf(anchor);
      assert.ok(anchorOffset >= 0 && anchor.indexOf(token) >= 0);
      const offset = anchorOffset + anchor.indexOf(token);
      const hover = await session.request("textDocument/hover", {
        textDocument: { uri: uri(file) },
        position: offsetToPosition(source, offset),
      });
      hoverPackets.push({ file, anchor, token, hover });
      record("typed-hover-whole-packets", hoverPackets);
      const text = hoverToText(hover as { contents?: unknown } | null);
      assert.match(text, expected, file);
      assert.doesNotMatch(text, /\b(?:any|unknown|never)\b|__Vize|__VLS/, file);
    }

    for (const row of rows) {
      fs.writeFileSync(path.join(directory, row.file), invalid.files[row.file]);
      session.notify("textDocument/didChange", {
        textDocument: { uri: uri(row.file), version: 2 },
        contentChanges: [{ text: invalid.files[row.file] }],
      });
    }
    const broken = await Promise.all(rows.map((row) => diagnostics(row.file, 2)));
    record("invalid-whole-packets", broken);
    for (const [index, row] of rows.entries()) {
      const actual = broken[index].diagnostics;
      assert.deepEqual(
        actual.map((diagnostic) => {
          const start = diagnostic.range?.start;
          assert.ok(start && typeof start.line === "number" && typeof start.character === "number");
          return {
            code: Number(diagnostic.code),
            line: start.line + 1,
            column: start.character + 1,
          };
        }),
        [{ code: row.code, line: row.line, column: row.column }],
        row.file,
      );
      assert.ok(actual[0]?.message && actual[0].message.length > 0);
    }
    for (const row of rows) {
      fs.writeFileSync(path.join(directory, row.file), valid.files[row.file]);
      session.notify("textDocument/didChange", {
        textDocument: { uri: uri(row.file), version: 3 },
        contentChanges: [{ text: valid.files[row.file] }],
      });
    }
    const repaired = await Promise.all(rows.map((row) => diagnostics(row.file, 3)));
    record("repaired-whole-packets", repaired);
    for (const packet of repaired) assert.deepEqual(packet.diagnostics, [], packet.uri);
    assert.equal(
      rows.length,
      [...templateRefAdmission, ...templateRefEdgeAdmission].reduce(
        (sum, row) => sum + row.mutations.length,
        0,
      ),
    );
  } finally {
    await session.shutdown();
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
