import assert from "node:assert/strict";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
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
