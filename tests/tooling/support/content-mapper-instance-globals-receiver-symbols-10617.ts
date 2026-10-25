import assert from "node:assert/strict";
import { createRequire } from "node:module";
import path from "node:path";
import {
  location,
  type GlobalCase,
  type Transform,
} from "./content-mapper-instance-globals-10617.ts";

export type ReceiverProgram = GlobalCase & {
  receiver: "fresh" | "direct";
  candidate?: { name: string; role: "value" | "type-query" };
};
type Node = {
  end: number;
  flags: number;
  text?: string;
  name?: Node;
  expression?: Node;
  initializer?: Node;
  type?: Node;
  typeName?: Node;
  parent?: Node;
  getStart(file: Node): number;
  getText(file: Node): string;
};
type Symbol = { name: string; declarations?: Node[] };
type Host = {
  getSourceFile(
    name: string,
    language: number,
    error?: unknown,
    create?: boolean,
  ): Node | undefined;
};
type Checker = {
  getSymbolAtLocation(node: Node): Symbol | undefined;
  getTypeAtLocation(node: Node): unknown;
  typeToString(type: unknown): string;
};
type Compiler = {
  version: string;
  ScriptTarget: { ESNext: number };
  ModuleKind: { Preserve: number };
  ModuleResolutionKind: { Bundler: number };
  JsxEmit: { Preserve: number };
  ScriptKind: { TSX: number };
  NodeFlags: { Const: number };
  SyntaxKind: Record<number, string>;
  createCompilerHost(options: Record<string, unknown>): Host;
  createSourceFile(
    name: string,
    text: string,
    language: number,
    parents: boolean,
    kind: number,
  ): Node;
  createProgram(
    names: string[],
    options: Record<string, unknown>,
    host: Host,
  ): { getSourceFile(name: string): Node | undefined; getTypeChecker(): Checker };
  isVariableDeclaration(node: Node): boolean;
  isIdentifier(node: Node): boolean;
  isAsExpression(node: Node): boolean;
  isTypeReferenceNode(node: Node): boolean;
  isPropertyAccessExpression(node: Node): boolean;
  isTypeQueryNode(node: Node): boolean;
  forEachChild(node: Node, visit: (node: Node) => void): void;
};

export function assertReceiverSymbols(
  entry: ReceiverProgram,
  result: Transform,
  sourceRoot: string,
  referencePackage: string,
) {
  const ts = createRequire(referencePackage)("typescript") as Compiler;
  assert.equal(ts.version, "5.9.3");
  const name = path.join(sourceRoot, "src/Globals.vue.tsx");
  const options = {
    strict: true,
    noEmit: true,
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.Preserve,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    jsx: ts.JsxEmit.Preserve,
    types: [],
  };
  const host = ts.createCompilerHost(options);
  const original = host.getSourceFile.bind(host);
  host.getSourceFile = (file, language, error, create) =>
    file === name
      ? ts.createSourceFile(name, result.text, language, true, ts.ScriptKind.TSX)
      : original(file, language, error, create);
  const program = ts.createProgram(
    [name, path.join(sourceRoot, "src/globals.d.ts")],
    options,
    host,
  );
  const file = program.getSourceFile(name);
  assert.ok(file);
  const checker = program.getTypeChecker();
  const publicDeclarations: Node[] = [],
    reads: Node[] = [],
    candidates: Node[] = [];
  const visit = (node: Node) => {
    if (
      ts.isVariableDeclaration(node) &&
      node.name &&
      ts.isIdentifier(node.name) &&
      node.initializer &&
      ts.isAsExpression(node.initializer) &&
      node.initializer.type &&
      ts.isTypeReferenceNode(node.initializer.type) &&
      node.initializer.type.typeName?.text === "__Ctx"
    )
      publicDeclarations.push(node);
    if (
      ts.isPropertyAccessExpression(node) &&
      node.name?.text === "$ghost" &&
      node.expression &&
      ts.isIdentifier(node.expression)
    )
      reads.push(node);
    if (
      entry.candidate &&
      ts.isIdentifier(node) &&
      node.text === entry.candidate.name &&
      !(node.parent && ts.isVariableDeclaration(node.parent) && node.parent.name === node)
    )
      candidates.push(node);
    ts.forEachChild(node, visit);
  };
  visit(file);
  assert.equal(publicDeclarations.length, 1, "one real generated public-context declaration");
  const publicSymbol = checker.getSymbolAtLocation(publicDeclarations[0].name!);
  assert.ok(publicSymbol);
  const chain = (
    symbol: Symbol | undefined,
    seen = new Set<Symbol>(),
  ): Array<{ name: string; declaration: string }> => {
    assert.ok(symbol && !seen.has(symbol), "receiver chain cannot be unresolved or cyclic");
    if (symbol === publicSymbol)
      return [{ name: symbol.name, declaration: "public ComponentPublicInstance context" }];
    seen.add(symbol);
    assert.equal(symbol.declarations?.length, 1);
    const declaration = symbol.declarations[0];
    assert.ok(
      ts.isVariableDeclaration(declaration) &&
        declaration.initializer &&
        ts.isIdentifier(declaration.initializer),
    );
    assert.ok(
      declaration.parent && (declaration.parent.flags & ts.NodeFlags.Const) !== 0,
      "receiver aliases are immutable",
    );
    return [
      { name: symbol.name, declaration: declaration.getText(file) },
      ...chain(checker.getSymbolAtLocation(declaration.initializer), seen),
    ];
  };
  assert.equal(reads.length, 1, "one physical getter in these independent interpolation controls");
  const read = reads[0],
    receiver = read.expression!,
    key = read.name!;
  const symbol = checker.getSymbolAtLocation(receiver);
  assert.equal(
    symbol === publicSymbol,
    entry.receiver === "direct",
    "ordinary binders retain the original receiver; actual captures use an alias",
  );
  const aliasChain = chain(symbol);
  const receiverType = checker.typeToString(checker.getTypeAtLocation(receiver));
  assert.notEqual(receiverType, "any");
  const start = Buffer.byteLength(result.text.slice(0, receiver.getStart(file))),
    end = Buffer.byteLength(result.text.slice(0, key.getStart(file)));
  const at = location(entry.source, "$ghost.value", "$ghost");
  assert.deepEqual(
    result.mappings.filter((span) => span[5] === 0),
    [[start, end - start, at.byteStart, at.byteLength, 1, 0]],
    "the actual symbol-owned receiver prefix has only diagnostic identity",
  );
  assert.ok(
    result.mappings.some(
      (span) =>
        span[0] === end &&
        span[1] === at.byteLength &&
        span[2] === at.byteStart &&
        span[3] === at.byteLength &&
        span[4] === 0 &&
        span[5] === 1048575,
    ),
    "the property keeps its exact navigable authored token",
  );
  const unresolved = candidates.map((node) => {
    assert.equal(
      checker.getSymbolAtLocation(node),
      undefined,
      "an authored unresolved candidate must not become the generated alias",
    );
    const role = node.parent && ts.isTypeQueryNode(node.parent) ? "type-query" : "value";
    assert.equal(role, entry.candidate!.role);
    return { name: node.text, role, generatedStart: node.getStart(file) };
  });
  assert.equal(unresolved.length, entry.candidate ? 1 : 0);
  return {
    compiler: ts.version,
    receiver: receiver.text,
    receiverType,
    aliasChain,
    diagnosticPrefix: [start, end - start, at.byteStart, at.byteLength, 1, 0],
    unresolved,
  };
}
