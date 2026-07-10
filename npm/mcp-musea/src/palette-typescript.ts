import ts from "typescript";

export interface PaletteTypeControl {
  name: string;
  control: string;
  required: boolean;
  options: Array<{ value: unknown }>;
}

export function buildPaletteTypescript(title: string, controls: PaletteTypeControl[]): string {
  const members = controls.map((control) =>
    ts.factory.createPropertySignature(
      undefined,
      propertyNameNode(control.name),
      control.required ? undefined : ts.factory.createToken(ts.SyntaxKind.QuestionToken),
      controlTypeNode(control),
    ),
  );
  const declaration = ts.factory.createInterfaceDeclaration(
    [ts.factory.createModifier(ts.SyntaxKind.ExportKeyword)],
    interfaceNameFromTitle(title),
    undefined,
    undefined,
    members,
  );
  const sourceFile = ts.createSourceFile(
    "palette.ts",
    "",
    ts.ScriptTarget.Latest,
    false,
    ts.ScriptKind.TS,
  );
  const printer = ts.createPrinter({ newLine: ts.NewLineKind.LineFeed });
  return `${printer.printNode(ts.EmitHint.Unspecified, declaration, sourceFile)}\n`;
}

function interfaceNameFromTitle(title: string): string {
  const compact = title.replace(/\s+/g, "");
  const candidate = `${compact || "Component"}Props`.replace(/[^A-Za-z0-9_$]/g, "_");
  return /^[A-Za-z_$]/.test(candidate) ? candidate : `_${candidate}`;
}

function propertyNameNode(name: string): ts.PropertyName {
  return /^[$A-Z_a-z][$\w]*$/.test(name)
    ? ts.factory.createIdentifier(name)
    : ts.factory.createStringLiteral(name);
}

function controlTypeNode(control: PaletteTypeControl): ts.TypeNode {
  if (control.control === "boolean") {
    return ts.factory.createKeywordTypeNode(ts.SyntaxKind.BooleanKeyword);
  }
  if (control.control === "number") {
    return ts.factory.createKeywordTypeNode(ts.SyntaxKind.NumberKeyword);
  }
  if (control.control === "select" && control.options.length > 0) {
    return ts.factory.createUnionTypeNode(
      control.options.map((option) =>
        ts.factory.createLiteralTypeNode(ts.factory.createStringLiteral(String(option.value))),
      ),
    );
  }
  return ts.factory.createKeywordTypeNode(ts.SyntaxKind.StringKeyword);
}
