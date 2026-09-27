import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { root } from "./support/lsp/paths.ts";
import { expectedBuildIdentity, validateBuildReceipt } from "../differential/build-receipt.mjs";
import { workspace } from "./support/upstream/vue-language-tools.ts";

const require = createRequire(import.meta.url);
const ts = createRequire(require.resolve("vue-tsc/package.json"))("typescript");
const corpus = path.join(root, "tests/fixtures/typechecker/slot-outlet-union");
const expected: Array<[string, number, number]> = [
  ["src/ManyWrong.ts", 5, 2322], ["src/ManyWrong.ts", 6, 2322],
  ["src/ManyWrong.ts", 7, 2322], ["src/ManyWrong.ts", 8, 2322],
  ["src/ManyWrong.ts", 9, 2322], ["src/ManyWrong.ts", 10, 2322],
  ["src/OptionalControl.ts", 6, 2322], ["src/Parent.vue", 4, 2322],
  ["src/Parent.vue", 7, 2322],
  ["src/SingleControl.ts", 4, 2322], ["src/Wrong.vue", 4, 2322],
];
const order = (left: unknown[], right: unknown[]) =>
  JSON.stringify(left).localeCompare(JSON.stringify(right));

test("generated slot payloads keep seventeen and eighteen outlets without losing type errors", () => {
  const directory = workspace("slot-outlet-generated-");
  try {
    fs.mkdirSync(path.join(directory, "src"));
    const manifest = JSON.parse(fs.readFileSync(path.join(corpus, "manifest.json"), "utf8"));
    assert.equal(manifest.sources.length, 14);
    for (const file of manifest.sources) {
      fs.copyFileSync(path.join(corpus, file), path.join(directory, "src", file));
    }
    fs.writeFileSync(path.join(directory, "tsconfig.json"), JSON.stringify({
      compilerOptions: { strict: true, target: "ES2022", module: "ESNext", moduleResolution: "bundler", noEmit: true },
      include: ["src/**/*"],
    }));
    assert.ok(fs.existsSync(path.join(directory, "node_modules/vue/package.json")), "required Vue runtime missing");
    const build = expectedBuildIdentity(root);
    validateBuildReceipt(JSON.parse(fs.readFileSync(path.join(root, build.binaryPath + ".differential-build.json"), "utf8")), build);
    const result = spawnSync(path.join(root, build.binaryPath), [
      "check", "--tsconfig", "tsconfig.json", "--format", "json",
      "--show-virtual-ts", "--save-virtual-ts-for", "__vize_helpers.d.ts",
    ], { cwd: directory, env: process.env, encoding: "utf8", timeout: 60_000 });
    const artifact = path.join(root, "target/differential/slot-outlet-union.json");
    fs.mkdirSync(path.dirname(artifact), { recursive: true });
    const raw = { build, status: result.status, stdoutBase64: Buffer.from(result.stdout ?? "").toString("base64"),
      stderrBase64: Buffer.from(result.stderr ?? "").toString("base64") };
    fs.writeFileSync(artifact, JSON.stringify(raw, null, 2) + "\n");
    assert.equal(result.error, undefined);
    assert.equal(result.status, 1, result.stdout + result.stderr);
    const report = JSON.parse(result.stdout);
    assert.equal(report.errorCount, expected.length, result.stdout);
    const actual = report.files.flatMap((file: { file: string; diagnostics: string[] }) =>
      file.diagnostics.map((text) => {
        const match = /^error:(\d+):(\d+) \[TS(\d+)\] /.exec(text);
        assert.ok(match, text);
        return [file.file.replaceAll("\\", "/"), Number(match[1]), Number(match[3])];
      }),
    );
    assert.deepEqual(actual.sort(order), [...expected].sort(order));
    const oracle = JSON.parse(fs.readFileSync(path.join(corpus, "typescript-oracle.json"), "utf8"));
    const observed = report.files.flatMap((file: { file: string; diagnostics: string[] }) =>
      file.diagnostics.map((text) => {
        const match = /^error:(\d+):(\d+) \[TS(\d+)\] ([\s\S]+)$/.exec(text);
        assert.ok(match, text);
        return { file: file.file.replaceAll("\\", "/"), line: Number(match[1]), column: Number(match[2]),
          code: Number(match[3]), message: match[4] };
      }),
    );
    fs.writeFileSync(artifact, JSON.stringify({ ...raw, diagnostics: observed }, null, 2) + "\n");
    const diagnosticOrder = (left: object, right: object) => JSON.stringify(left).localeCompare(JSON.stringify(right));
    assert.deepEqual(observed.filter((d: { file: string }) => d.file.endsWith(".ts")).sort(diagnosticOrder),
      [...oracle.diagnostics].sort(diagnosticOrder));

    // Compile the actual source-built CLI's virtual documents with TypeScript,
    // rather than reconstructing a carrier or matching generated substrings.
    const virtual = new Map<string, string>();
    for (const file of report.files) {
      if (file.virtualTs !== undefined) {
        virtual.set(path.resolve(directory, file.file) + ".ts", file.virtualTs);
      }
    }
    assert.ok(virtual.has(path.join(directory, "src/Many17.vue.ts")));
    assert.ok(virtual.has(path.join(directory, "src/Many18.vue.ts")));
    const names = ["ManyControl.ts", "ManyWrong.ts", "OptionalControl.ts", "SingleControl.ts", "StaticRepeatControl.ts"];
    const options = { strict: true, noEmit: true, target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.ESNext, moduleResolution: ts.ModuleResolutionKind.Bundler,
      skipLibCheck: true, types: [] };
    const host = ts.createCompilerHost(options);
    const readFile = host.readFile.bind(host);
    const fileExists = host.fileExists.bind(host);
    const getSourceFile = host.getSourceFile.bind(host);
    host.getSourceFile = (file: string, languageVersion: unknown, onError: unknown, fresh: boolean) => {
      const content = virtual.get(path.resolve(file));
      return content === undefined ? getSourceFile(file, languageVersion, onError, fresh)
        : ts.createSourceFile(file, content, languageVersion, true);
    };
    host.readFile = (file: string) => virtual.get(path.resolve(file)) ?? readFile(file);
    host.fileExists = (file: string) => virtual.has(path.resolve(file)) || fileExists(file);
    const program = ts.createProgram(names.map((name) => path.join(directory, "src", name)), options, host);
    assert.deepEqual(program.getSyntacticDiagnostics(), []);
    const owned = names.flatMap((name) => {
      const file = program.getSourceFile(path.join(directory, "src", name));
      assert.ok(file, name);
      return program.getSemanticDiagnostics(file).map((diagnostic: { start: number; code: number; messageText: unknown }) => {
        const location = file.getLineAndCharacterOfPosition(diagnostic.start);
        return { file: "src/" + name, line: location.line + 1, column: location.character + 1,
          code: diagnostic.code, message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n") };
      });
    });
    assert.deepEqual(owned.sort(diagnosticOrder), [...oracle.diagnostics].sort(diagnosticOrder));
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});
