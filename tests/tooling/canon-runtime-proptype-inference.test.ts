import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import ts from "typescript";

const root = new URL("../../", import.meta.url);
const assets = [
  "crates/vize_canon/src/virtual_ts/helpers/declarations/core-types.d.ts.txt",
  "npm/cli/virtual-ts-globals.d.ts",
  "npm/cli/generated/content-mapper-types.d.ts",
];

function runtimeHelpers(asset: string): string {
  const source = ts.createSourceFile(
    asset,
    readFileSync(new URL(asset, root), "utf8"),
    ts.ScriptTarget.Latest,
    true,
  );
  const declarations: string[] = [];
  const visit = (node: ts.Node) => {
    if (ts.isTypeAliasDeclaration(node) && node.name.text.startsWith("__RuntimeProp"))
      declarations.push(node.getText(source));
    ts.forEachChild(node, visit);
  };
  visit(source);
  assert.equal(declarations.length, 6);
  return declarations.join("\n");
}

// Vue 3.5.43+ permits PropMethod<T> for every T, with a constructor result
// independent of its call result. Keep that upstream contract as a fixture.
const fixture = `
type PropConstructor<T> = (new (...args: any[]) => T & {}) | (() => T) | {
  new (): any;
  (): T;
  readonly prototype: any;
};
type PropType<T> = PropConstructor<T> | (PropConstructor<T> | null)[];
interface Editor { groups?: { id: string }[] }
declare const props: __RuntimePropShape<{
  editor: { type: PropType<Editor>; required: true };
  editors: { type: readonly [PropType<Editor>, null]; required: true };
  callback: { type: PropType<(value: string) => number>; required: true };
  count: { type: NumberConstructor; required: true };
  date: { type: DateConstructor; required: true };
}>;
const group = props.editor.groups?.find(item => item.id === "x");
const nested = props.editors.groups?.find(item => item.id === "x");
const output: number = props.callback("ok");
const count: number = props.count;
const date: Date = props.date;
const wrong: number = props.editor;
const wrongNested: number = props.editors;
const wrongCall = props.callback(123);
declare class Item { id: string }
declare const instance: __RuntimePropCtor<typeof Item>;
const id: string = instance.id;
const wrongClass: number = instance;
`;

for (const asset of assets) {
  test(`${asset}: runtime PropType retains call results and constructor-only values`, () => {
    const file = fileURLToPath(new URL("tests/_fixtures/runtime-proptype-inference.ts", root));
    const text = `${runtimeHelpers(asset)}\n${fixture}`;
    const options: ts.CompilerOptions = {
      noEmit: true,
      strict: true,
      skipLibCheck: true,
      types: [],
      target: ts.ScriptTarget.ES2022,
    };
    const host = ts.createCompilerHost(options);
    const original = host.getSourceFile.bind(host);
    host.getSourceFile = (name, language, error, fresh) =>
      name === file
        ? ts.createSourceFile(name, text, language, true)
        : original(name, language, error, fresh);
    const program = ts.createProgram([file], options, host);
    assert.deepEqual(
      ts.getPreEmitDiagnostics(program).map((diagnostic) => ({
        code: diagnostic.code,
        message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
      })),
      [
        { code: 2322, message: "Type 'Editor' is not assignable to type 'number'." },
        { code: 2322, message: "Type 'Editor' is not assignable to type 'number'." },
        {
          code: 2345,
          message: "Argument of type 'number' is not assignable to parameter of type 'string'.",
        },
        { code: 2322, message: "Type 'Item' is not assignable to type 'number'." },
      ],
    );
  });
}
