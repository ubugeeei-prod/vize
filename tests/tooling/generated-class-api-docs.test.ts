import assert from "node:assert/strict";
import { copyFileSync, mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";

import { renderReferenceDocs } from "../../npm/ui/scripts/generate-reference-docs.ts";
import { moduleExports } from "../../npm/ui/scripts/reference-docs/extract.ts";

const ts = createRequire(new URL("../package.json", import.meta.url))(
  "typescript",
) as typeof import("typescript");
const root = path.resolve(import.meta.dirname, "../..");

const parse = (source: string) =>
  ts.createSourceFile(
    "reference.ts",
    source,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS,
  ) as import("typescript").SourceFile & { parseDiagnostics: import("typescript").Diagnostic[] };
const shape = (source: string) =>
  ts.createPrinter({ removeComments: true }).printFile(parse(source)).replace(/\s+/g, " ").trim();

test("the published disposal class documents its inherited shape and constructor", () => {
  const item = moduleExports(path.join(root, "npm/compose/core/src/disposal-scope.ts")).find(
    (item) => item.name === "DisposalError",
  );
  assert(item);
  assert.equal(
    shape(item.signature),
    'declare class DisposalError extends AggregateError { readonly code: "VIZE_COMPOSE_DISPOSAL_FAILED"; constructor(errors: readonly unknown[]); }',
  );
});

test("class declarations retain inferred fields, generic methods and nominal private members", (t) => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-reference-class-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const file = path.join(directory, "entry.ts");
  writeFileSync(
    file,
    `const LABEL = "public-label";
export class Example<T extends string> extends Error {
  readonly code = LABEL;
  readonly value: T;
  static limit = 5;
  #token = 0;
  private hidden() { return this.#token; }
  constructor(value: T) { super(value); this.value = value; }
  withValue<U extends string>(value: U): Example<U> { return new Example(value); }
  get label(): T { return this.value; }
}
`,
  );
  const [item] = moduleExports(file);
  assert(item);
  assert.equal(
    shape(item.signature),
    'declare class Example<T extends string> extends Error { #private; readonly code = "public-label"; readonly value: T; static limit: number; private hidden; constructor(value: T); withValue<U extends string>(value: U): Example<U>; get label(): T; }',
  );
  assert.deepEqual(parse(item.signature).parseDiagnostics, []);
});

test("an unresolved public class field refuses generation instead of inventing its type", (t) => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-reference-class-invalid-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const file = path.join(directory, "entry.ts");
  writeFileSync(file, "export class Broken { readonly value = missingInput; }\n");
  assert.throws(() => moduleExports(file), /Cannot find name 'missingInput'/);
});

test("a named default abstract class retains its ambient public declaration", (t) => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-reference-class-default-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const file = path.join(directory, "entry.ts");
  writeFileSync(
    file,
    `export default abstract class Example<T> {
  abstract current: T;
  constructor(public readonly name: string) {}
  abstract read(): T;
}\n`,
  );
  const [item] = moduleExports(file);
  assert(item);
  assert.equal(
    shape(item.signature),
    "declare abstract class Example<T> { readonly name: string; abstract current: T; constructor(name: string); abstract read(): T; }",
  );
  assert.deepEqual(parse(item.signature).parseDiagnostics, []);
});

test("every complete generated TypeScript reference fence parses without rewriting its source", () => {
  const failures: string[] = [];
  let checked = 0;
  for (const [file, page] of renderReferenceDocs()) {
    for (const [, metadata, source] of page.matchAll(/^```ts([^\n]*)\n([\s\S]*?)^```/gm)) {
      if (metadata.includes("annotate=")) continue;
      checked += 1;
      for (const diagnostic of parse(source).parseDiagnostics) {
        failures.push(`${file}: ${ts.flattenDiagnosticMessageText(diagnostic.messageText, " ")}`);
      }
    }
  }
  assert(checked > 500, `whole generated reference corpus: ${checked} TypeScript fences`);
  assert.deepEqual(failures, []);
});

test("the actual generated reference corpus passes the existing documentation example parser", (t) => {
  const directory = mkdtempSync(path.join(tmpdir(), "vize-generated-reference-parser-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  for (const [file, page] of renderReferenceDocs()) {
    const output = path.join(directory, "docs/content", file);
    mkdirSync(path.dirname(output), { recursive: true });
    writeFileSync(output, page);
  }
  const parser = path.join(directory, "tests/tooling/docs-structure.test.ts");
  mkdirSync(path.dirname(parser), { recursive: true });
  copyFileSync(path.join(root, "tests/tooling/docs-structure.test.ts"), parser);
  writeFileSync(path.join(directory, "package.json"), '{"type":"module"}\n');
  const modules = path.join(directory, "tests/node_modules");
  mkdirSync(modules);
  const dependency = createRequire(new URL("../package.json", import.meta.url)).resolve(
    "typescript/package.json",
  );
  symlinkSync(path.dirname(dependency), path.join(modules, "typescript"), "junction");
  const env = { ...process.env };
  delete env.NODE_TEST_CONTEXT;
  const result = spawnSync(
    process.execPath,
    ["--test", "--test-name-pattern=JSON, JavaScript and TypeScript examples parse", parser],
    { encoding: "utf8", env },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.signal, null);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /\bpass 1\b/);
  t.diagnostic(result.stdout.trim());
});
