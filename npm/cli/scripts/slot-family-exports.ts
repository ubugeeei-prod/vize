import assert from "node:assert/strict";
import ts from "typescript";

export function isExported(node: ts.Node) {
  return (
    ts.canHaveModifiers(node) &&
    (ts.getModifiers(node)?.some((modifier) => modifier.kind === ts.SyntaxKind.ExportKeyword) ??
      false)
  );
}

export function deriveForwardedSlotAliases(family: readonly ts.Statement[]) {
  const aliases = new Set(family.filter(ts.isTypeAliasDeclaration).map((node) => node.name.text));
  assert.deepEqual(
    family
      .filter(ts.isTypeAliasDeclaration)
      .filter((node) => !isExported(node))
      .map((node) => node.name.text),
    [
      "$NativeSlotsFactory",
      "$HasSlotResolver",
      "$DirectSlotConstructor",
      "$SlotConstructorArgumentLists",
      "$SlotConstructorHasArguments",
    ],
    "complete private slot aliases forwarded across the SDK module boundary",
  );
  return aliases;
}

export function renderSlotFamily(
  family: readonly ts.Statement[],
  source: ts.SourceFile,
  forwardedAliases: ReadonlySet<string>,
) {
  return family
    .map((node) => {
      const declaration = node.getText(source);
      if (
        ts.isTypeAliasDeclaration(node) &&
        forwardedAliases.has(node.name.text) &&
        !isExported(node)
      )
        return `export ${declaration}`;
      return (ts.isFunctionDeclaration(node) ? "declare " : "") + declaration;
    })
    .join("\n");
}

export function assertForwardedSlotExports(
  relativePath: string,
  source: string,
  forwardedAliases: ReadonlySet<string>,
) {
  const parsed = ts.createSourceFile(relativePath, source, ts.ScriptTarget.Latest, true);
  const aliases = new Map<string, ts.TypeAliasDeclaration>(
    parsed.statements.filter(ts.isTypeAliasDeclaration).map((node) => [node.name.text, node]),
  );
  for (const name of forwardedAliases) {
    const declaration = aliases.get(name);
    assert.ok(declaration && isExported(declaration), `${relativePath} must export ${name}`);
  }
}
