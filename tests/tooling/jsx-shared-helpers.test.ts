// The JSX/TSX component and context contracts are program-wide ambient
// helpers. L4 documents name them but never redeclare them per file.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";
import { beforeJsxSharedHelpersSource } from "../../tools/support/compat/content-mapper/jsx-shared-helpers-transition.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const globalsPath = path.join(root, "npm/cli/virtual-ts-globals.d.ts");
const declarations = path.join(root, "crates/vize_canon/src/virtual_ts/helpers/declarations");

const helperNames = [
  "Ctx",
  "__vize_jsx_component_spread__",
  "__vize_jsx_component__",
  "__vize_jsx_component_slot__",
];

function ambientNames(): string[] {
  const text = fs.readFileSync(globalsPath, "utf8");
  const file = ts.createSourceFile(globalsPath, text, ts.ScriptTarget.Latest, true);
  const names: string[] = [];
  for (const statement of file.statements) {
    if (!ts.isModuleDeclaration(statement) || statement.name.getText(file) !== "global") continue;
    if (!statement.body || !ts.isModuleBlock(statement.body)) continue;
    for (const member of statement.body.statements) {
      if ((ts.isFunctionDeclaration(member) || ts.isTypeAliasDeclaration(member)) && member.name) {
        names.push(member.name.text);
      }
    }
  }
  return names;
}

test("the published ambient set declares every JSX shared helper once", () => {
  const names = ambientNames();
  for (const helper of helperNames) {
    assert.equal(names.filter((name) => name === helper).length, 1, helper);
  }

  const context = fs.readFileSync(path.join(declarations, "jsx-context.d.ts.txt"), "utf8");
  const components = fs.readFileSync(path.join(declarations, "jsx-components.d.ts.txt"), "utf8");
  assert.match(context, /type Ctx</);
  for (const helper of helperNames.slice(1)) assert.ok(components.includes(helper), helper);
});

test("older source custody sees the exact pre-publication sources", () => {
  const relative = "crates/vize_canon/src/virtual_ts/helpers/preamble.rs";
  const current = fs.readFileSync(path.join(root, relative), "utf8");
  const previous = beforeJsxSharedHelpersSource(relative, current);
  assert.notEqual(previous, current);
  assert.ok(!previous.includes("declarations/jsx-context.d.ts.txt"));
  assert.ok(!previous.includes("declarations/jsx-components.d.ts.txt"));
  assert.equal(beforeJsxSharedHelpersSource("unrelated.ts", current), current);
  const globals = fs.readFileSync(globalsPath, "utf8");
  const previousGlobals = beforeJsxSharedHelpersSource("npm/cli/virtual-ts-globals.d.ts", globals);
  assert.equal(Buffer.byteLength(previousGlobals), 18_023);
  assert.equal(previousGlobals.includes("__vize_jsx_component__"), false);
});

test("the ambient helpers preserve component, emit, and slot diagnostics", () => {
  const source = `
declare const Widget: (props: { label: string; count?: number }) => unknown;
__vize_jsx_component__(Widget)({ label: "ok" });
__vize_jsx_component__(Widget)({ label: 1 });
__vize_jsx_component__(Widget)({});
const View = (_props: {}, { emit, slots }: Ctx<
  { change: [value: number] },
  { default(props: { n: number }): unknown }
>) => {
  emit("change", 1);
  emit("change", "bad");
  slots.default({ n: 1 });
  slots.default({ n: "bad" });
};
void View;
`;
  const sourcePath = path.join(root, "npm/ui/l4-jsx-shared-helper-law.ts");
  const options: ts.CompilerOptions = {
    noEmit: true,
    strict: true,
    types: [],
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.ESNext,
    moduleResolution: ts.ModuleResolutionKind.Bundler,
    moduleDetection: ts.ModuleDetectionKind.Force,
    skipLibCheck: true,
  };
  const host = ts.createCompilerHost(options);
  const read = host.readFile.bind(host);
  const exists = host.fileExists.bind(host);
  host.readFile = (name) => (path.resolve(name) === sourcePath ? source : read(name));
  host.fileExists = (name) => path.resolve(name) === sourcePath || exists(name);
  const program = ts.createProgram([sourcePath, globalsPath], options, host);
  const file = program.getSourceFile(sourcePath);
  assert.ok(file);
  const diagnostics = [
    ...program.getSyntacticDiagnostics(file),
    ...program.getSemanticDiagnostics(file),
  ].map((diagnostic) => ({
    code: diagnostic.code,
    text: source.slice(diagnostic.start, (diagnostic.start ?? 0) + (diagnostic.length ?? 0)),
    message: ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
  }));

  assert.deepEqual(
    diagnostics.map(({ code, text }) => ({ code, text })),
    [
      { code: 2322, text: "label" },
      { code: 2345, text: "{}" },
      { code: 2345, text: '"bad"' },
      { code: 2322, text: "n" },
    ],
  );
  assert.ok(diagnostics.every(({ message }) => message.includes("not assignable")));
});
