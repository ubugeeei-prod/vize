/** Compile the actual Markdown source packets against the public package contracts. */
import { readFileSync } from "node:fs";
import path from "node:path";

import ts from "typescript";

const root = path.resolve(import.meta.dirname, "../../../..");

/** One complete module displayed in an authored documentation page. */
export interface MarkdownSourceExample {
  readonly document: string;
  readonly line: number;
  readonly language: "ts" | "js";
  readonly source: string;
  readonly filename: string;
}

function importTargets(): ReadonlyMap<string, string> {
  const sdk = "npm/plugin-sdk";
  const manifest = JSON.parse(readFileSync(path.join(root, sdk, "package.json"), "utf8")) as {
    name: string;
    exports: Record<string, { types: string }>;
  };
  return new Map([
    ...Object.entries(manifest.exports).map(([subpath, entry]): [string, string] => [
      `${manifest.name}${subpath.slice(1)}`,
      path.join(root, sdk, entry.types),
    ]),
    ["@vizejs/native", path.join(root, "npm/native/index.d.ts")],
    ["@vizejs/devtools", path.join(root, "npm/devtools/src/index.ts")],
  ]);
}

/** Read every TypeScript or JavaScript fence, preserving its full displayed source. */
export function markdownSourceExamples(document: string): readonly MarkdownSourceExample[] {
  const markdown = readFileSync(path.join(root, document), "utf8");
  return [...markdown.matchAll(/^```(ts|js)\s*\n([\s\S]*?)^```\s*$/gmu)].map((match, index) => ({
    document,
    line: markdown.slice(0, match.index).split("\n").length + 1,
    language: match[1] as "ts" | "js",
    source: match[2]!,
    filename: path.join(
      root,
      path.dirname(document),
      `${path.basename(document)}.example-${index + 1}.${match[1] === "ts" ? "mts" : "mjs"}`,
    ),
  }));
}

/** Strict diagnostics for the displayed modules and the real source/declarations they import. */
export function typecheckMarkdownExamples(examples: readonly MarkdownSourceExample[]): string[] {
  const targets = importTargets();
  const options: ts.CompilerOptions = {
    noEmit: true,
    strict: true,
    skipLibCheck: true,
    allowJs: true,
    checkJs: true,
    allowImportingTsExtensions: true,
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
    types: ["node"],
    paths: Object.fromEntries([...targets].map(([name, target]) => [name, [target]])),
  };
  const packets = new Map(examples.map((example) => [example.filename, example]));
  const host = ts.createCompilerHost(options);
  const read = host.readFile.bind(host);
  const exists = host.fileExists.bind(host);
  host.readFile = (file) => packets.get(file)?.source ?? read(file);
  host.fileExists = (file) => packets.has(file) || exists(file);
  const program = ts.createProgram([...packets.keys()], options, host);
  return ts.getPreEmitDiagnostics(program).map((diagnostic) => {
    const file = diagnostic.file;
    const packet = file && packets.get(file.fileName);
    const position = file?.getLineAndCharacterOfPosition(diagnostic.start ?? 0);
    const location = packet
      ? `${packet.document}:${packet.line + (position?.line ?? 0)}`
      : (file?.fileName ?? "compiler options");
    return `${location} TS${diagnostic.code}: ${ts.flattenDiagnosticMessageText(diagnostic.messageText, " ")}`;
  });
}

/** Resolve public imports to their real workspace implementation for native CI execution. */
export function resolveMarkdownExampleImports(
  example: MarkdownSourceExample,
  resolve: (specifier: string) => string,
): string {
  const source = ts.createSourceFile(
    example.filename,
    example.source,
    ts.ScriptTarget.Latest,
    true,
  );
  const replacements: { start: number; end: number; text: string }[] = [];
  for (const statement of source.statements) {
    if (!ts.isImportDeclaration(statement) || !ts.isStringLiteral(statement.moduleSpecifier))
      continue;
    const specifier = statement.moduleSpecifier;
    if (!specifier.text.startsWith("@vizejs/")) continue;
    replacements.push({
      start: specifier.getStart(source),
      end: specifier.end,
      text: JSON.stringify(resolve(specifier.text)),
    });
  }
  return replacements
    .reverse()
    .reduce(
      (text, replacement) =>
        text.slice(0, replacement.start) + replacement.text + text.slice(replacement.end),
      example.source,
    );
}
