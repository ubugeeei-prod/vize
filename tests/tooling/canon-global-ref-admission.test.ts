import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { globalRefAdmission } from "./support/type-admission-global-fixtures.ts";
import { typeProofs, mutatedAdmission } from "./support/type-admission-fixtures.ts";
import { check, diagnosticIdentity, workspace } from "./support/upstream/vue-language-tools.ts";
import { vueTscDiagnostics } from "./support/vue-tsc-oracle.ts";
import {
  requireTypecheckDependency,
  resolveTypecheckRuntime,
} from "./support/typecheck-dependency.ts";
import { isDiagnosticsForUri } from "./support/lsp/assertions.ts";
import type { PublishDiagnosticsParams } from "./support/lsp/protocol.ts";
import { root } from "./support/lsp/paths.ts";
import { LspSession } from "./support/lsp/session.ts";
import { writeVueConfigFixture } from "./support/vue-config-fixture.ts";

// Each project has its own real ambient declaration. A shared global Vue shim
// would conceal both custom-contract false positives and missing-import errors.
for (const row of globalRefAdmission) {
  test(`actual ambient ref contract: ${row.name}`, async (t) => {
    const checker = requireTypecheckDependency(
      t,
      resolveTypecheckRuntime(root),
      "global ref admission backend",
      "TypeScript native backend unavailable",
    );
    if (!checker) return;
    const directory = workspace(`global-ref-${row.name}-`);
    const capture = path.join(root, "target/differential/template-ref-global-admission", row.name);
    fs.mkdirSync(capture, { recursive: true });
    const record = (name: string, value: unknown) =>
      fs.writeFileSync(path.join(capture, name + ".json"), JSON.stringify(value, null, 2) + "\n");
    fs.writeFileSync(
      path.join(directory, "proofs.ts"),
      typeProofs.replace(
        'declare global { const useTemplateRef: typeof import("vue").useTemplateRef; }',
        row.ambient ? `declare global { ${row.ambient} }` : "",
      ),
    );
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
        // These custom globals are not Vue composables. Disable the oracle's
        // name-only inference; real Vue imports remain explicitly typed here.
        vueCompilerOptions: {
          composables: { useTemplateRef: row.vueComposable ? ["useTemplateRef"] : [] },
        },
        include: ["*.vue", "*.ts"],
      }),
    );
    writeVueConfigFixture(directory, {
      lsp: { hover: true, lint: false, typecheck: true },
      typeChecker: { corsaPath: checker },
    });
    const mutation = mutatedAdmission(row.source, row.mutation);
    const expected = [
      { file: "Probe.vue", line: mutation.line, column: mutation.column, code: mutation.code },
    ];
    record("authored-inputs", {
      row,
      mutation,
      expected,
      source: process.env.GITHUB_SHA,
      backend: checker,
    });
    let session: LspSession | undefined;
    let initialized = false;
    const file = path.join(directory, "Probe.vue");
    const uri = pathToFileURL(file).href;
    try {
      for (const [stage, source, diagnostics] of [
        ["valid", row.source, []],
        ["invalid", mutation.source, expected],
        ["repaired", row.source, []],
      ] as const) {
        fs.writeFileSync(file, source);
        const captureName = `template-ref-admission-global-${row.name.toLowerCase()}-${stage}`;
        const actual = await check(directory, ["--corsa-path", checker], {
          name: captureName,
          sources: ["tsconfig.json", "proofs.ts", "Probe.vue"],
        });
        const oracleCapture = `${captureName}-vue`;
        fs.mkdirSync(
          path.join(root, "target/differential/fallthrough-listener-inputs", oracleCapture),
          { recursive: true },
        );
        const oracle = vueTscDiagnostics(directory, oracleCapture);
        record(`${stage}-compiler-packets`, { actual, oracle });
        assert.deepEqual(actual.map(diagnosticIdentity), diagnostics, `${row.name} ${stage}: CLI`);
        assert.deepEqual(oracle, diagnostics, `${row.name} ${stage}: independent Vue oracle`);
      }
      session = new LspSession();
      await session.initialize(directory, {
        editor: true,
        hover: true,
        lint: false,
        typecheck: true,
      });
      initialized = true;
      for (const [index, source] of [row.source, mutation.source, row.source].entries()) {
        fs.writeFileSync(file, source);
        const version = index + 1;
        if (index === 0)
          session.notify("textDocument/didOpen", {
            textDocument: { uri, languageId: "vue", version, text: source },
          });
        else
          session.notify("textDocument/didChange", {
            textDocument: { uri, version },
            contentChanges: [{ text: source }],
          });
        const packet = (await session.waitForNotification(
          "textDocument/publishDiagnostics",
          (value) => isDiagnosticsForUri(value, uri) && value.version === version,
          120_000,
        )) as PublishDiagnosticsParams;
        record(`lsp-version-${version}-whole-packet`, packet);
        const actual = packet.diagnostics.map((diagnostic) => {
          const position = diagnostic.range?.start;
          assert.ok(
            position && typeof position.line === "number" && typeof position.character === "number",
          );
          return {
            file: "Probe.vue",
            line: position.line + 1,
            column: position.character + 1,
            code: Number(diagnostic.code),
          };
        });
        assert.deepEqual(
          actual,
          version === 2 ? expected : [],
          `${row.name} LSP version ${version}`,
        );
      }
    } finally {
      if (session) {
        if (initialized) await session.shutdown();
        else await session.kill();
      }
      fs.rmSync(directory, { recursive: true, force: true });
    }
  });
}
