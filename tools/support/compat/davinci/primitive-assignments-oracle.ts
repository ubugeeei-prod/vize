// Run the reference TypeScript checker on each complete original script, with
// its original coordinates. The local assignability judgment needs no imports.
import assert from "node:assert/strict";
import { createReadStream } from "node:fs";
import { createRequire } from "node:module";
import type { TypeScriptApi } from "../../../../tests/_helpers/template-alias-typescript-11822.ts";
const ts: TypeScriptApi = createRequire(import.meta.url)("typescript");
interface OperandSpans {
  binding: [number, number];
  annotation: [number, number];
  initializer: [number, number];
  expected: string;
  actual: string;
}
type Claim = OperandSpans & ({ message: string | null } | { assignable: boolean });
interface Input {
  name: string;
  source: string;
  tsx: boolean;
  complete?: boolean;
  covered?: number;
  recovery?: "retained" | "first-open" | "upstream-original";
  claims: Claim[];
}

function examine(input: Input) {
  const { source, claims } = input;
  const filename = input.tsx ? "/original.tsx" : "/original.ts";
  const options: Parameters<TypeScriptApi["createCompilerHost"]>[0] = {
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
  const span = (node: Parameters<TypeScriptApi["isIdentifier"]>[0]) => [
    byte(node.getStart(original)),
    byte(node.end),
  ];
  const vars = original.statements.flatMap((st) => {
    if (input.covered !== undefined && byte(st.end) > input.covered) return [];
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
  if (vars.length === 0 && input.covered === undefined) return false;
  const host = ts.createCompilerHost(options);
  host.getSourceFile = (name) => (name === filename ? original : undefined);
  const program = ts.createProgram([filename], options, host);
  if (input.covered !== undefined) {
    assert.ok(program.getSyntacticDiagnostics(original).length > 0, input.name);
  }
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
    const assignable = checker.isTypeAssignableTo(actual, expected);
    if ("message" in row) {
      assert.equal(input.covered, undefined);
      assert.equal(row.message === null, assignable, input.name);
      if (row.message !== null) claimedDiagnostics.push([row.binding, row.message]);
    } else {
      assert.ok(input.covered !== undefined);
      assert.equal(row.assignable, assignable, input.name);
    }
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
const recovery = {
  retained: { scripts: 0, bindings: 0, mismatches: 0 },
  "first-open": { scripts: 0, bindings: 0, mismatches: 0 },
  "upstream-original": { scripts: 0, bindings: 0, mismatches: 0 },
};
// JSON permits literal U+2028 and U+2029 in source strings. Only LF is a
// record delimiter; terminal line readers can also split at those code points.
async function* inputRecords(path: string): AsyncGenerator<string> {
  let pending = "";
  for await (const chunk of createReadStream(path, { encoding: "utf8" })) {
    pending += chunk;
    let start = 0;
    for (let end = pending.indexOf("\n"); end >= 0; end = pending.indexOf("\n", start)) {
      yield pending.slice(start, end);
      start = end + 1;
    }
    pending = pending.slice(start);
  }
  if (pending.length) yield pending;
}
for await (const line of inputRecords(process.argv[2])) {
  const entry: Input = JSON.parse(line);
  assert.equal(typeof entry.covered === "number", process.argv[3] === "--recovered");
  if (examine(entry)) complete++;
  scripts++;
  bindings += entry.claims.length;
  const failed = entry.claims.filter((c) =>
    "message" in c ? c.message !== null : !c.assignable,
  ).length;
  mismatches += failed;
  if (entry.covered !== undefined) {
    assert.ok(
      entry.recovery === "retained" ||
        entry.recovery === "first-open" ||
        entry.recovery === "upstream-original",
    );
    const mode = recovery[entry.recovery];
    mode.scripts++;
    mode.bindings += entry.claims.length;
    mode.mismatches += failed;
  }
}
assert.ok(scripts > 0 && bindings > 0 && mismatches > 0);
if (process.argv[3] !== "--recovered") assert.ok(complete > 0);
else {
  for (const [name, mode] of Object.entries(recovery)) {
    assert.ok(mode.scripts > 0, name);
    if (name !== "upstream-original") assert.ok(mode.bindings > 0 && mode.mismatches > 0, name);
    console.log(
      `TypeScript ${ts.version} ${name} scripts=${mode.scripts} bindings=${mode.bindings} mismatches=${mode.mismatches}`,
    );
  }
}
console.log(
  `TypeScript ${ts.version} original scripts=${scripts} checked bindings=${bindings} mismatches=${mismatches} complete diagnostic comparisons=${complete}`,
);
