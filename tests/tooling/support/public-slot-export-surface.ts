import assert from "node:assert/strict";
import ts from "../../../npm/cli/node_modules/typescript/lib/typescript.js";

export function publicSlotExportSurface(checker: ts.TypeChecker, source: ts.SourceFile) {
  const imported = source.statements.find(ts.isImportDeclaration);
  assert.ok(imported);
  const module = checker.getSymbolAtLocation(imported.moduleSpecifier);
  assert.ok(module);
  const symbols = checker.getExportsOfModule(module);
  assert.ok(symbols.length > 0);
  const exports = symbols.map((symbol) => {
    const target = symbol.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
    const declarations = target.declarations ?? [];
    return {
      name: symbol.name,
      type: (target.flags & ts.SymbolFlags.Type) !== 0,
      value: (target.flags & ts.SymbolFlags.Value) !== 0,
      namespace: (target.flags & ts.SymbolFlags.Namespace) !== 0,
      declarations: declarations.map((declaration) => {
        const parameters =
          ts.isTypeAliasDeclaration(declaration) ||
          ts.isInterfaceDeclaration(declaration) ||
          ts.isClassDeclaration(declaration) ||
          ts.isFunctionDeclaration(declaration)
            ? declaration.typeParameters
            : undefined;
        return {
          kind: ts.SyntaxKind[declaration.kind],
          parameters:
            parameters?.map((parameter) => ({
              name: parameter.name.text,
              constraint: parameter.constraint?.getText() ?? null,
              default: parameter.default?.getText() ?? null,
            })) ?? [],
          signatures: ts.isFunctionDeclaration(declaration)
            ? checker
                .getTypeOfSymbolAtLocation(target, declaration)
                .getCallSignatures()
                .map((signature) =>
                  checker.signatureToString(
                    signature,
                    declaration,
                    ts.TypeFormatFlags.NoTruncation,
                  ),
                )
            : [],
        };
      }),
    };
  });
  assert.equal(new Set(exports.map((entry) => entry.name)).size, exports.length);
  return { module: module.name, exports };
}

export function assertPublicSlotExportSurface(
  actual: ReturnType<typeof publicSlotExportSurface>,
  expected: ReturnType<typeof publicSlotExportSurface>,
) {
  // Keep the complete original order in receipts. Module member lookup is by
  // name; compare every member's type/value/namespace role and full arity.
  assert.deepEqual(
    Object.fromEntries(actual.exports.map((entry) => [entry.name, entry])),
    Object.fromEntries(expected.exports.map((entry) => [entry.name, entry])),
    "all former implicit and explicit entry exports retain their kinds and parameters",
  );
}
