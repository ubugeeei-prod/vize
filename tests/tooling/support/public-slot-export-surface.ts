import assert from "node:assert/strict";
import fs from "node:fs";
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

type ExportSurface = ReturnType<typeof publicSlotExportSurface>;

/**
 * An exported type-parameter default that may differ from the frozen SDK. Each
 * entry names the export and parameter, both texts, and the oracle test that
 * evaluates the old and new defaults over its shape set (both
 * exactOptionalPropertyTypes modes and deferred generics) and requires
 * identical results. Any other textual change still fails the law.
 */
export type ProvenDefaultChange = {
  exportName: string;
  parameter: string;
  before: string;
  after: string;
  oracle: string;
};

export const PROVEN_DEFAULT_CHANGES: readonly ProvenDefaultChange[] = [];

const repositoryRoot = new URL("../../../", import.meta.url);

/** Replace each proven new default with its old text; refuse stale or unproven entries. */
function withProvenDefaultsRestored(
  actual: ExportSurface,
  changes: readonly ProvenDefaultChange[],
): ExportSurface {
  const exports = structuredClone(actual.exports);
  for (const change of changes) {
    const oracle = new URL(change.oracle, repositoryRoot);
    assert.ok(
      fs.existsSync(oracle) && fs.readFileSync(oracle, "utf8").includes(change.exportName),
      `${change.exportName}.${change.parameter}: oracle ${change.oracle} must exist and test it`,
    );
    const parameters = exports
      .find((entry) => entry.name === change.exportName)
      ?.declarations.flatMap((declaration) => declaration.parameters)
      .filter((parameter) => parameter.name === change.parameter);
    assert.equal(
      parameters?.length,
      1,
      `${change.exportName}.${change.parameter}: exactly one listed parameter`,
    );
    assert.equal(
      parameters![0]!.default,
      change.after,
      `${change.exportName}.${change.parameter}: the listed new default is current`,
    );
    parameters![0]!.default = change.before;
  }
  return { ...actual, exports };
}

export function assertPublicSlotExportSurface(
  actual: ExportSurface,
  expected: ExportSurface,
  changes: readonly ProvenDefaultChange[] = PROVEN_DEFAULT_CHANGES,
) {
  // Keep the complete original order in receipts. Module member lookup is by
  // name; compare every member's type/value/namespace role and full arity.
  // Only listed, oracle-proven defaults may differ; their old text must match.
  assert.deepEqual(
    Object.fromEntries(
      withProvenDefaultsRestored(actual, changes).exports.map((entry) => [entry.name, entry]),
    ),
    Object.fromEntries(expected.exports.map((entry) => [entry.name, entry])),
    "all former implicit and explicit entry exports retain their kinds and parameters",
  );
}
