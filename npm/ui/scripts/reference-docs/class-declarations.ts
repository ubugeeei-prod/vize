import path from "node:path";
import ts from "typescript";

const declarations = new WeakMap<ts.SourceFile, ReadonlyMap<string, string>>();
const diagnosticsHost: ts.FormatDiagnosticsHost = {
  getCanonicalFileName: (file) => file,
  getCurrentDirectory: () => process.cwd(),
  getNewLine: () => "\n",
};

function compilerOptions(file: string): ts.CompilerOptions {
  const configPath = ts.findConfigFile(path.dirname(file), (candidate) =>
    ts.sys.fileExists(candidate),
  );
  if (configPath == null)
    return {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.ESNext,
      moduleResolution: ts.ModuleResolutionKind.Bundler,
      strict: true,
    };
  const config = ts.readConfigFile(configPath, (candidate) => ts.sys.readFile(candidate));
  if (config.error != null) throw new Error(ts.formatDiagnostics([config.error], diagnosticsHost));
  const parsed = ts.parseJsonConfigFileContent(config.config, ts.sys, path.dirname(configPath));
  if (parsed.errors.length > 0)
    throw new Error(ts.formatDiagnostics(parsed.errors, diagnosticsHost));
  return parsed.options;
}

/** The compiler retains inferred public types, inheritance and nominal private members. */
export function classDeclarations(source: ts.SourceFile): ReadonlyMap<string, string> {
  const cached = declarations.get(source);
  if (cached != null) return cached;
  const program = ts.createProgram([source.fileName], {
    ...compilerOptions(source.fileName),
    declaration: true,
    declarationMap: false,
    emitDeclarationOnly: true,
    noEmit: false,
  });
  const diagnostics = ts.getPreEmitDiagnostics(program);
  if (diagnostics.length > 0) throw new Error(ts.formatDiagnostics(diagnostics, diagnosticsHost));
  const input = program.getSourceFile(source.fileName);
  if (input == null) throw new Error(`Missing class source: ${source.fileName}`);
  let output: string | undefined;
  const result = program.emit(
    input,
    (file, text) => {
      if (file.endsWith(".d.ts")) output = text;
    },
    undefined,
    true,
  );
  if (result.emitSkipped || result.diagnostics.length > 0 || output == null)
    throw new Error(
      `Cannot emit public class declarations: ${source.fileName}\n` +
        `Declaration emit skipped: ${result.emitSkipped}; output produced: ${output != null}.\n` +
        ts.formatDiagnostics(result.diagnostics, diagnosticsHost),
    );
  const emitted = ts.createSourceFile(
    `${source.fileName}.d.ts`,
    output,
    ts.ScriptTarget.Latest,
    true,
  );
  const classes = new Map<string, string>();
  const printer = ts.createPrinter({ newLine: ts.NewLineKind.LineFeed });
  for (const statement of emitted.statements) {
    if (!ts.isClassDeclaration(statement) || statement.name == null) continue;
    const declaration = ts.factory.updateClassDeclaration(
      statement,
      [
        ts.factory.createModifier(ts.SyntaxKind.DeclareKeyword),
        ...(statement.modifiers ?? []).filter(
          (modifier) =>
            modifier.kind !== ts.SyntaxKind.ExportKeyword &&
            modifier.kind !== ts.SyntaxKind.DefaultKeyword &&
            modifier.kind !== ts.SyntaxKind.DeclareKeyword,
        ),
      ],
      statement.name,
      statement.typeParameters,
      statement.heritageClauses,
      statement.members,
    );
    ts.setEmitFlags(declaration, ts.EmitFlags.NoLeadingComments);
    classes.set(
      statement.name.text,
      printer.printNode(ts.EmitHint.Unspecified, declaration, emitted),
    );
  }
  declarations.set(source, classes);
  return classes;
}
