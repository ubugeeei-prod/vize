import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";
import ts from "typescript";

const declarations = fs
  .readFileSync(
    new URL(
      "../../crates/vize_canon/src/virtual_ts/helpers/declarations/listener-args.d.ts.txt",
      import.meta.url,
    ),
    "utf8",
  )
  .replaceAll("$ListenerArgs", "ListenerArgs")
  .replaceAll("$ListenerMissing", "ListenerMissing");

const oracle = `
type Equal<A, B> = (<T>() => T extends A ? 1 : 2) extends (<T>() => T extends B ? 1 : 2) ? true : false;
type Expect<T extends true> = T;
type OriginalMissing<P> = P extends { 'onSelect-value'?: (...args: infer A) => any } ? A : unknown[];
type Original<P> = P extends { onSelect?: (...args: infer A) => any } ? A : P extends { onSelect?: infer H } ? [Extract<NonNullable<H>, (...args: any) => any>] extends [never] ? unknown[] : Extract<NonNullable<H>, (...args: any) => any> extends (...args: infer A) => any ? A : unknown[] : unknown[];
type OriginalHyphen<P> = P extends { onSelectValue?: (...args: infer A) => any } ? A : P extends { onSelectValue?: infer H } ? [Extract<NonNullable<H>, (...args: any) => any>] extends [never] ? OriginalMissing<P> : Extract<NonNullable<H>, (...args: any) => any> extends (...args: infer A) => any ? A : OriginalMissing<P> : OriginalMissing<P>;
`;

const shapes = [
  "{}",
  "any",
  "unknown",
  "never",
  "undefined",
  "{ onSelect?: never }",
  "{ onSelect?: undefined }",
  "{ onSelect: null }",
  "{ onSelect?: (x: number) => string }",
  "{ onSelect: (x: number) => string }",
  "{ onSelect?: ((x: 1) => 2) | ((x: 3) => 4) }",
  "{ onSelect?: ((x: number) => string)[] }",
  "{ onSelect?: readonly ((x: number) => string)[] }",
  "{ onSelect?: ((x: number) => string) | ((x: number) => string)[] }",
  "{ onSelect?: { (x: 1): 2; (x: 3): 4 } }",
  "{ onSelect?: any }",
  "{ onSelect?: unknown }",
  "Record<string, unknown>",
  "Record<string, any>",
  "{ onSelect?: (x: 1) => 2 } | { onSelect?: (x: 3) => 4 }",
  "{ onSelect?: (x: 1) => 2 } | {}",
];

function check(source: string) {
  const file = "/listener-relations.ts";
  const options = {
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    target: ts.ScriptTarget.ESNext,
  };
  const host = ts.createCompilerHost(options);
  const original = host.getSourceFile.bind(host);
  host.getSourceFile = (path, languageVersion, ...rest) =>
    path === file
      ? ts.createSourceFile(file, source, languageVersion, true)
      : original(path, languageVersion, ...rest);
  return ts.getPreEmitDiagnostics(ts.createProgram([file], options, host)).map((diagnostic) => ({
    code: diagnostic.code,
    message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
    line: diagnostic.file!.getLineAndCharacterOfPosition(diagnostic.start!).line,
  }));
}

test("named listener relations preserve the original optional-property inference", () => {
  const laws = shapes.map(
    (shape, index) => `
type P${index} = ${shape};
type Law${index} = Expect<Equal<Original<P${index}>, ListenerArgs<P${index}, 'onSelect'>>>;
`,
  );
  const hyphens = [
    "{ 'onSelect-value'?: (x: 1) => 2 }",
    "{ 'onSelect-value'?: (x: 1) => 2; onSelectValue?: (x: 3) => 4 }",
    "{ 'onSelect-value'?: (x: 1) => 2 } | { 'onSelect-value'?: (x: 3) => 4 }",
  ].map(
    (shape, index) =>
      `type Hyphen${index} = Expect<Equal<OriginalHyphen<${shape}>, ListenerArgs<${shape}, 'onSelectValue', 'onSelect-value'>>>;`,
  );
  const controls = (name: string, relation: string) => `
namespace ${name} {
  declare const value: (...args: ${relation}<{ onSelect?: (x: number) => void }>) => void;
  value(1);
  value('bad');
  declare const pair: (...args: ${relation}<{ onSelect?: (x: number, y: string) => void }>) => void;
  pair(1, 'ok');
  pair(1);
}
`;
  const source = [
    oracle,
    declarations,
    ...laws,
    ...hyphens,
    "type Named<P> = ListenerArgs<P, 'onSelect'>;",
    controls("Before", "Original"),
    controls("After", "Named"),
  ].join("\n");
  const diagnostics = check(source);
  assert.deepEqual(
    diagnostics.map(({ code }) => code),
    [2345, 2554, 2345, 2554],
  );
  assert.deepEqual(
    diagnostics.slice(0, 2).map(({ code, message }) => ({ code, message })),
    diagnostics.slice(2).map(({ code, message }) => ({ code, message })),
  );
});

test("local listener relations preserve authored type utility bindings", () => {
  const scopes = [
    "type Extract<T, U> = never;",
    "type NonNullable<T> = never;",
    "type Extract<T, U> = T; type NonNullable<T> = T;",
  ].map(
    (shadow, scope) => `
namespace Shadow${scope} {
  ${shadow}
  ${oracle}
  ${declarations}
  ${shapes
    .map(
      (shape, index) => `
  type P${index} = ${shape};
  type Law${index} = Expect<Equal<Original<P${index}>, ListenerArgs<P${index}, 'onSelect'>>>;
`,
    )
    .join("\n")}
  type Props = { onSelect?: ((value: number) => void) | readonly ((value: number) => void)[] };
  type LocalScope = Expect<Equal<Original<Props>, ListenerArgs<Props, 'onSelect'>>>;
  type GlobalScopeDiffers = Expect<Equal<Equal<Original<Props>, Shared.ListenerArgs<Props, 'onSelect'>>, false>>;
  declare const before: (...args: Original<Props>) => void;
  declare const local: (...args: ListenerArgs<Props, 'onSelect'>) => void;
  declare const shared: (...args: Shared.ListenerArgs<Props, 'onSelect'>) => void;
  before('bad');
  local('bad');
  shared('bad');
}
`,
  );
  const source = [
    "export {};",
    `namespace Shared { ${declarations.replace("type ListenerArgs", "export type ListenerArgs")} }`,
    ...scopes,
  ].join("\n");
  const diagnostics = check(source);
  assert.deepEqual(
    diagnostics.map(({ code, message }) => ({ code, message })),
    Array.from({ length: scopes.length }, () => ({
      code: 2345,
      message: "Argument of type 'string' is not assignable to parameter of type 'number'.",
    })),
  );
  for (const diagnostic of diagnostics) {
    assert.equal(source.split("\n")[diagnostic.line].trim(), "shared('bad');");
  }
});
