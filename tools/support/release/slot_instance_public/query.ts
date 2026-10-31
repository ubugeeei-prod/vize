import assert from "node:assert/strict";
import { realpathSync } from "node:fs";
import type ts from "../../../../npm/cli/node_modules/typescript/lib/typescript.js";
import { assertCustody, fileImage, inside } from "../computed_event_custody.ts";

/** The API object comes from this role's installed TypeScript, never the workspace. */
export function query(
  api: typeof ts,
  file: string,
  project: string,
  installation: string,
  preservation: boolean,
) {
  assert.equal(api.version, "6.0.3");
  const program = api.createProgram([file], {
    strict: true,
    noEmit: true,
    skipLibCheck: true,
    target: api.ScriptTarget.ESNext,
    module: api.ModuleKind.Node16,
    moduleResolution: api.ModuleResolutionKind.Node16,
    types: [],
  });
  const source = program.getSourceFile(file);
  assert.ok(source);
  const checker = program.getTypeChecker(),
    rows: unknown[] = [];
  function visit(node: ts.Node) {
    if (
      (api.isVariableDeclaration(node) || api.isTypeAliasDeclaration(node)) &&
      api.isIdentifier(node.name) &&
      (preservation
        ? /^Preserve_/u
        : /^(Result_|WholeSlots_|WholeNoAny_|WholePayload_|WholePayloadAny_)/u
      ).test(node.name.text)
    ) {
      const value = checker.getTypeAtLocation(node.name) as ts.Type & { intrinsicName?: string };
      rows.push({
        name: node.name.text,
        display: checker.typeToString(value, node, api.TypeFormatFlags.NoTruncation),
        flags: value.flags,
        any: (value.flags & api.TypeFlags.Any) !== 0,
        intrinsic: value.intrinsicName ?? null,
      });
    }
    api.forEachChild(node, visit);
  }
  visit(source);
  const imported = source.statements.find(api.isImportDeclaration);
  assert.ok(imported);
  const module = checker.getSymbolAtLocation(imported.moduleSpecifier);
  assert.ok(module);
  const exports = preservation
    ? []
    : checker.getExportsOfModule(module).map((symbol) => {
        const target =
          symbol.flags & api.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
        return {
          name: symbol.name,
          type: (target.flags & api.SymbolFlags.Type) !== 0,
          value: (target.flags & api.SymbolFlags.Value) !== 0,
          namespace: (target.flags & api.SymbolFlags.Namespace) !== 0,
          declarations: (target.declarations ?? []).map((declaration) => {
            const parameters =
              api.isTypeAliasDeclaration(declaration) ||
              api.isInterfaceDeclaration(declaration) ||
              api.isClassDeclaration(declaration) ||
              api.isFunctionDeclaration(declaration)
                ? declaration.typeParameters
                : undefined;
            return {
              kind: api.SyntaxKind[declaration.kind],
              parameters:
                parameters?.map((parameter) => ({
                  name: parameter.name.text,
                  constraint: parameter.constraint?.getText() ?? null,
                  default: parameter.default?.getText() ?? null,
                })) ?? [],
              signatures: api.isFunctionDeclaration(declaration)
                ? checker
                    .getTypeOfSymbolAtLocation(target, declaration)
                    .getCallSignatures()
                    .map((signature) =>
                      checker.signatureToString(
                        signature,
                        declaration,
                        api.TypeFormatFlags.NoTruncation,
                      ),
                    )
                : [],
            };
          }),
        };
      });
  const inputs = program.getSourceFiles().map((item) => {
    const filename = realpathSync(item.fileName);
    assert.ok(
      inside(project, filename) || inside(installation, filename),
      "compiler input escaped the owned project and installed libraries",
    );
    const image = fileImage(filename);
    // The actual program text and the physical bytes must describe the same input.
    assert.equal(api.sys.readFile(filename), item.text);
    return image;
  });
  assertCustody(inputs);
  return { rows, exports, module: module.name, inputs };
}
