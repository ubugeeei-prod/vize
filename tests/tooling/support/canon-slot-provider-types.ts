import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import ts from "typescript";

type IntrinsicType = ts.Type & { intrinsicName?: string; debugIntrinsicName?: string };

/** Inspect the actual bound slot value, not an IsAny-only surrogate. */
export function slotProviderTypes(
  file: string,
  text: string,
  control: {
    roots?: string[];
    providerNames?: string[];
    noUnusedLocals?: boolean;
    noUnusedParameters?: boolean;
  } = {},
) {
  assert.equal(ts.version, "6.0.3", "intrinsic error identities require the pinned compiler");
  const options: ts.CompilerOptions = {
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    target: ts.ScriptTarget.ES2024,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    types: [],
    noUnusedLocals: control.noUnusedLocals,
    noUnusedParameters: control.noUnusedParameters,
  };
  const host = ts.createCompilerHost(options),
    original = host.getSourceFile.bind(host);
  host.getSourceFile = (name, language, error, fresh) =>
    name === file
      ? ts.createSourceFile(name, text, language, true)
      : original(name, language, error, fresh);
  const roots = [file, ...(control.roots ?? [])];
  const program = ts.createProgram(roots, options, host);
  const source = program.getSourceFile(file);
  assert.ok(source);
  const checker = program.getTypeChecker();
  const rows: Array<{
    role: string;
    start: number;
    end: number;
    text: string;
    display: string;
    flags: number;
    any: boolean;
    intrinsicName: string | null;
    debugIntrinsicName: string | null;
    symbol: string | null;
  }> = [];
  const record = (role: string, node: ts.Node) => {
    const type = checker.getTypeAtLocation(node) as IntrinsicType;
    rows.push({
      role,
      start: node.getStart(source),
      end: node.end,
      text: node.getText(source),
      display: checker.typeToString(type),
      flags: type.flags,
      any: (type.flags & ts.TypeFlags.Any) !== 0,
      intrinsicName: type.intrinsicName ?? null,
      debugIntrinsicName: type.debugIntrinsicName ?? null,
      symbol: checker.getSymbolAtLocation(node)?.getName() ?? null,
    });
  };
  const visit = (node: ts.Node) => {
    if (
      ts.isVariableDeclaration(node) &&
      ts.isIdentifier(node.name) &&
      control.providerNames?.includes(node.name.text)
    )
      record("actual-provider", node.name);
    if (ts.isIdentifier(node) && node.text === "WitnessChild") record("provider", node);
    if (ts.isBindingElement(node) && ts.isIdentifier(node.name) && node.name.text === "item")
      record("slot-binding", node.name);
    if (
      ts.isCallExpression(node) &&
      ts.isIdentifier(node.expression) &&
      node.expression.text === "slotReport"
    )
      record("slot-report", node);
    ts.forEachChild(node, visit);
  };
  visit(source);
  return {
    compiler: ts.version,
    roots: program.getRootFileNames(),
    sourceSha256: createHash("sha256").update(text).digest("hex"),
    file,
    rows,
    diagnostics: ts.getPreEmitDiagnostics(program).map((diagnostic) => ({
      file: diagnostic.file?.fileName ?? null,
      start: diagnostic.start ?? null,
      length: diagnostic.length ?? null,
      code: diagnostic.code,
      message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
    })),
  };
}
