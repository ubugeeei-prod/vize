/** Keep the public emit resolver on the same canonical policy as the producer. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import ts from "typescript";
import { format } from "oxfmt";

const names = [
  "$LooseEmitProps",
  "$LooseEmitPropsFactory",
  "$EmitIsWide",
  "$NativeEmitPropsFactory",
  "$EmitPropsFactory",
];
const previous = `
type $NativeEmitPropsFactory<C> = C extends new (...args: any[]) => { $emit: any } ? C extends { readonly __vizeComponentMarker: true } ? $ConstructorEmitPropsFactory<C extends { readonly __vizeFallthroughProps?: infer F } ? F : {}, $EmitOwnedKeys<C>> : typeof __vizeConstructorEmitProps : (component: C) => (props: any) => {};
export type $EmitPropsFactory<C> = $IsAny<C> extends true ? (component: C) => (props: any) => {} : C extends { __vizeResolveEmitProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : $NativeEmitPropsFactory<C> : C extends { __vizeResolveProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : $NativeEmitPropsFactory<C> : $NativeEmitPropsFactory<C>;
`;
const printer = ts.createPrinter({ removeComments: true });
function family(text: string, label: string) {
  const source = ts.createSourceFile(label, text, ts.ScriptTarget.Latest, true) as ts.SourceFile & {
    parseDiagnostics: readonly ts.Diagnostic[];
  };
  assert.deepEqual(source.parseDiagnostics, [], `${label} syntax`);
  const nodes: ts.TypeAliasDeclaration[] = [];
  function visit(node: ts.Node) {
    if (ts.isTypeAliasDeclaration(node) && names.includes(node.name.text)) nodes.push(node);
    ts.forEachChild(node, visit);
  }
  visit(source);
  return { source, nodes };
}
function identity(node: ts.TypeAliasDeclaration, source: ts.SourceFile) {
  const normalized = ts.transform(node, [
    (context) => {
      const visit: ts.Visitor = (current) =>
        ts.isParenthesizedTypeNode(current)
          ? ts.visitNode(current.type, visit)
          : ts.visitEachChild(current, visit, context);
      return (current) => ts.visitNode(current, visit) as ts.TypeAliasDeclaration;
    },
  ]);
  try {
    return printer
      .printNode(ts.EmitHint.Unspecified, normalized.transformed[0]!, source)
      .replace(/^export\s+/u, "");
  } finally {
    normalized.dispose();
  }
}
export async function emitInferenceExports(
  root: string,
  pending: ReadonlyMap<string, string>,
): Promise<Map<string, string>> {
  const read = (file: string) => pending.get(file) ?? readFileSync(path.join(root, file), "utf8");
  const base = "crates/vize_canon/src/virtual_ts/helpers/declarations/";
  const embedded = family(read(base + "emit-inference.d.ts.txt"), "embedded emit inference");
  const shared = family(read(base + "shared-emit-inference.d.ts.txt"), "shared emit inference");
  for (const current of [embedded, shared])
    assert.deepEqual(
      current.nodes.map((node) => node.name.text),
      names,
    );
  for (const [index, name] of names.entries()) {
    assert.equal(
      identity(embedded.nodes[index]!, embedded.source),
      identity(shared.nodes[index]!, shared.source),
      `${name} canonical policy`,
    );
    for (const [current, exported] of [
      [embedded, false],
      [shared, index === 4],
    ] as const) {
      assert.equal(
        current.nodes[index]!.modifiers?.some(
          (node) => node.kind === ts.SyntaxKind.ExportKeyword,
        ) ?? false,
        exported,
      );
    }
  }
  const canonical = await format(
    "virtual-ts.d.ts",
    shared.nodes.map((node) => node.getText(shared.source)).join("\n") + "\n",
    { printWidth: 100 },
  );
  assert.deepEqual(canonical.errors, []);
  const old = family(previous, "known previous public emit factory");
  const outputs = new Map<string, string>();
  for (const extension of ["ts", "mts", "cts"]) {
    const file = `npm/cli/virtual-ts.d.${extension}`;
    const text = read(file);
    const actual = family(text, file);
    const first = actual.nodes[0]!;
    const last = actual.nodes.at(-1)!;
    const expected = actual.nodes.length === 2 ? old : shared;
    assert.deepEqual(
      actual.nodes.map((node) => node.name.text),
      expected.nodes.map((node) => node.name.text),
      `${file} complete known family`,
    );
    for (const [index, node] of actual.nodes.entries()) {
      assert.equal(
        node.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword) ?? false,
        expected.nodes[index]!.modifiers?.some(
          (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword,
        ) ?? false,
        `${file} keeps factory export visibility`,
      );
      assert.equal(
        identity(node, actual.source),
        identity(expected.nodes[index]!, expected.source),
        `${file} refuses unknown ${node.name.text}`,
      );
    }
    const inside = actual.source.statements.filter(
      (node) =>
        node.getStart(actual.source) >= first.getStart(actual.source) && node.end <= last.end,
    );
    const spans = (nodes: readonly ts.Node[]) =>
      nodes.map((node) => [node.kind, node.getStart(actual.source), node.end]);
    assert.deepEqual(
      spans(inside),
      spans(actual.nodes),
      `${file} refuses foreign statements in the factory family`,
    );
    outputs.set(
      file,
      text.slice(0, first.getStart(actual.source)) +
        canonical.code.trimEnd() +
        text.slice(last.end),
    );
  }
  return outputs;
}
