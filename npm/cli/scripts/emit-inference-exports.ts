/** Keep the public emit resolver on the same canonical policy as the producer. */
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import ts from "typescript";
import { format } from "oxfmt";

const names = [
  "$EmitRequiredKeys",
  "$EmitRequiredFields",
  "$EmitFiniteFields",
  "$EmitFallthrough",
  "$EmitInput",
  "$MemberConstructorEmitPropsFactory",
  "$LooseEmitProps",
  "$DeferredEmitProps",
  "$LooseEmitPropsFactory",
  "$EmitIsWide",
  "$UnionEmit",
  "$NativeEmitPropsFactory",
  "$EmitPropsRoutes",
  "$EmitPropsFactory",
];
// Input helpers the public entry keeps privately beside its own constructor factory.
const inputHelpers = names.slice(0, 5);
// The published five-alias family that predates the private module.
const legacyFamily = [
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
const leaf = "npm/cli/virtual-ts-emits.d.ts";
const previousLeafSha256 = "88aa94a140aff34fa904615d1ca84b3cc70d186ca3209eec3bde3dc34b67c59e";
const previousNative =
  "type $NativeEmitPropsFactory<C> = C extends new (...args: any[]) => { $emit: any } ? C extends { readonly __vizeComponentMarker: true } ? $ConstructorEmitPropsFactory<C extends { readonly __vizeFallthroughProps?: infer F } ? F : {}, $EmitOwnedKeys<C>> : $EmitIsWide<C> extends true ? $LooseEmitPropsFactory<C> : typeof __vizeConstructorEmitProps : $LooseEmitPropsFactory<C>;";
const previousFactory =
  "type $EmitPropsFactory<C> = $IsAny<C> extends true ? (component: C) => (props: {}) => $LooseEmitProps : C extends { __vizeResolveEmitProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : $NativeEmitPropsFactory<C> : C extends { __vizeResolveProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : $NativeEmitPropsFactory<C> : $NativeEmitPropsFactory<C>;";
const namespaceImport =
  'import type * as __vizeEmitTypes from "./virtual-ts-emits.js" with { "resolution-mode": "import" };';
const constructorFactory = "$ConstructorEmitPropsFactory";
const forwards = `
type $NativeEmitPropsFactory<C> = __vizeEmitTypes.$NativeEmitPropsFactory<C>;
export type $EmitPropsFactory<C> = __vizeEmitTypes.$EmitPropsFactory<C>;
`;
const leafHeader = `// Generated from the canonical emit-inference helpers. Do not edit.
import type {
  IsAny as $IsAny,
  $ConstructorEmitPropsFactory,
  $EmitOwnedKeys,
  __EmitProps,
} from "./virtual-ts.js" with { "resolution-mode": "import" };
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
      [shared, name === "$EmitPropsFactory"],
    ] as const) {
      assert.equal(
        current.nodes[index]!.modifiers?.some(
          (node) => node.kind === ts.SyntaxKind.ExportKeyword,
        ) ?? false,
        exported,
      );
    }
  }
  // The public entry deliberately retains its legacy implicit exports. Keep
  // the new private names in a separate module with an explicit export boundary.
  const canonical = await format(
    leaf,
    leafHeader +
      shared.nodes
        .map((node) => node.getText(shared.source).replace(/^export\s+/u, ""))
        .join("\n") +
      "\nexport type { $NativeEmitPropsFactory, $EmitPropsFactory };\n",
    { printWidth: 100 },
  );
  assert.deepEqual(canonical.errors, []);
  const old = family(previous, "known previous public emit factory");
  const previousShared = family(
    shared.nodes
      .filter((node) => legacyFamily.includes(node.name.text))
      .map((node) =>
        node.name.text === "$NativeEmitPropsFactory"
          ? previousNative
          : node.name.text === "$EmitPropsFactory"
            ? previousFactory
            : node.getText(shared.source),
      )
      .join("\n"),
    "known previous shared emit family",
  );
  const forwarded = family(forwards, "known public namespace forwarders");
  const outputs = new Map<string, string>();
  if (pending.has(leaf) || existsSync(path.join(root, leaf))) {
    const existing = read(leaf);
    if (existing !== canonical.code)
      assert.equal(
        createHash("sha256").update(existing).digest("hex"),
        previousLeafSha256,
        `${leaf} refuses noncanonical helper content`,
      );
  }
  outputs.set(leaf, canonical.code);
  for (const extension of ["ts", "mts", "cts"]) {
    const file = `npm/cli/virtual-ts.d.${extension}`;
    const text = read(file);
    const published = family(text, file);
    const actual = {
      source: published.source,
      nodes: published.nodes.filter((node) => !inputHelpers.includes(node.name.text)),
    };
    const constructorFactories = actual.source.statements.filter(
      (node): node is ts.TypeAliasDeclaration =>
        ts.isTypeAliasDeclaration(node) && node.name.text === constructorFactory,
    );
    assert.equal(constructorFactories.length, 1, `${file} one constructor emit factory`);
    assert.equal(
      constructorFactories[0]!.modifiers?.some(
        (modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword,
      ) ?? false,
      true,
      `${file} explicitly exports the cross-file constructor emit factory`,
    );
    const first = actual.nodes[0]!;
    const last = actual.nodes.at(-1)!;
    const isForwarded =
      actual.nodes.length === 2 &&
      identity(actual.nodes[0]!, actual.source) === identity(forwarded.nodes[0]!, forwarded.source);
    const expected =
      actual.nodes.length === 2
        ? isForwarded
          ? forwarded
          : old
        : actual.nodes.length === 5
          ? previousShared
          : shared;
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
    const imports = actual.source.statements.filter(
      (node): node is ts.ImportDeclaration =>
        ts.isImportDeclaration(node) &&
        node.importClause?.namedBindings !== undefined &&
        ts.isNamespaceImport(node.importClause.namedBindings) &&
        node.importClause.namedBindings.name.text === "__vizeEmitTypes",
    );
    assert.equal(imports.length, isForwarded ? 1 : 0, `${file} keeps the private import boundary`);
    const start = isForwarded ? imports[0]!.getStart(actual.source) : first.getStart(actual.source);
    if (isForwarded) {
      assert.equal(imports[0]!.getText(actual.source), namespaceImport);
      assert.ok(text.slice(imports[0]!.end, first.getStart(actual.source)).trim() === "");
    }
    const replacement = await format(file, namespaceImport + forwards, { printWidth: 100 });
    assert.deepEqual(replacement.errors, []);
    outputs.set(file, text.slice(0, start) + replacement.code.trimEnd() + text.slice(last.end));
  }
  return outputs;
}
