// Run the reference TypeScript checker on each complete original script, with
// its original coordinates. The local assignability judgment needs no imports.
import assert from "node:assert/strict";
import { createReadStream } from "node:fs";
import { createRequire } from "node:module";
import { createInterface } from "node:readline";
import type * as TS from "typescript";
const ts: typeof TS = createRequire(import.meta.url)("typescript");
interface Claim {
  binding: [number, number];
  annotation: [number, number];
  initializer: [number, number];
  expected: string;
  actual: string;
  message: string | null;
}
interface Input {
  name: string;
  source: string;
  tsx: boolean;
  complete: boolean;
  claims: Claim[];
}

function examine(input: Input) {
  const { source, claims } = input;
  const filename = input.tsx ? "/original.tsx" : "/original.ts";
  const options: TS.CompilerOptions = {
    target: ts.ScriptTarget.Latest,
    module: ts.ModuleKind.ESNext,
    moduleDetection: ts.ModuleDetectionKind.Force,
    jsx: ts.JsxEmit.Preserve,
    strict: true,
    noLib: true,
    noResolve: true,
  };
  const original = ts.createSourceFile(filename, source, options.target!, true);
  const byte = (offset: number) => Buffer.byteLength(source.slice(0, offset));
  const span = (node: TS.Node) => [byte(node.getStart(original)), byte(node.end)];
  const vars = original.statements.flatMap((st) => {
    if (
      !ts.isVariableStatement(st) ||
      st.modifiers?.some((m) => m.kind === ts.SyntaxKind.DeclareKeyword)
    )
      return [];
    if ((st.declarationList.flags & ts.NodeFlags.Using) !== 0) return [];
    return st.declarationList.declarations.filter((d) => {
      if (!ts.isIdentifier(d.name) || !d.type || !d.initializer || d.exclamationToken) return false;
      return (
        ["string", "number", "boolean"].includes(d.type.getText(original)) &&
        [
          ts.SyntaxKind.StringLiteral,
          ts.SyntaxKind.NumericLiteral,
          ts.SyntaxKind.TrueKeyword,
          ts.SyntaxKind.FalseKeyword,
        ].includes(d.initializer.kind)
      );
    });
  });
  assert.equal(claims.length, vars.length, input.name);
  if (vars.length === 0) return false;
  const host = ts.createCompilerHost(options);
  host.getSourceFile = (name) => (name === filename ? original : undefined);
  const program = ts.createProgram([filename], options, host);
  const checker = program.getTypeChecker();
  const claimedDiagnostics = [];
  const names = new Set<string>();
  for (const [index, declaration] of vars.entries()) {
    const row = claims[index];
    const { name, type, initializer } = declaration;
    assert.ok(type && initializer && ts.isIdentifier(name));
    names.add(name.text);
    assert.deepEqual(row.binding, span(name), input.name);
    assert.deepEqual(row.annotation, span(type), input.name);
    assert.deepEqual(row.initializer, span(initializer), input.name);
    const expected = checker.getTypeFromTypeNode(type);
    const actual = checker.getTypeAtLocation(initializer);
    assert.equal(row.expected, checker.typeToString(expected), input.name);
    assert.equal(
      row.actual,
      checker.typeToString(checker.getBaseTypeOfLiteralType(actual)),
      input.name,
    );
    assert.equal(row.message === null, checker.isTypeAssignableTo(actual, expected), input.name);
    if (row.message !== null) claimedDiagnostics.push([row.binding, row.message]);
  }
  if (!input.complete || names.size !== vars.length) return false;
  const diagnostics = program
    .getSemanticDiagnostics(original)
    .filter((d) => d.code === 2322)
    .map((d) => [
      [byte(d.start!), byte(d.start! + d.length!)],
      ts.flattenDiagnosticMessageText(d.messageText, "\n"),
    ]);
  assert.deepEqual(diagnostics, claimedDiagnostics, input.name);
  return true;
}

let scripts = 0,
  bindings = 0,
  mismatches = 0,
  complete = 0;
for await (const line of createInterface({
  input: createReadStream(process.argv[2]),
  crlfDelay: Infinity,
})) {
  const entry: Input = JSON.parse(line);
  if (examine(entry)) complete++;
  scripts++;
  bindings += entry.claims.length;
  mismatches += entry.claims.filter((c) => c.message !== null).length;
}
assert.ok(scripts > 0 && bindings > 0 && mismatches > 0 && complete > 0);
console.log(
  `TypeScript ${ts.version} original scripts=${scripts} checked bindings=${bindings} mismatches=${mismatches} complete diagnostic comparisons=${complete}`,
);
