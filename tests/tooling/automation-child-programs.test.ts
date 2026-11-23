import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import ts from "typescript";

import { isProgramCommand } from "../../tools/support/ci/automation-commands.ts";

const root = fileURLToPath(new URL("../../", import.meta.url));
const interpreters =
  /^(?:node(?:js)?|bun|deno|python[\d.]*|perl|ruby|(?:ba|da|z|k)?sh|powershell|pwsh)(?:\.exe)?$/iu;

function interpreter(node: ts.Expression | undefined, source: ts.SourceFile): string | undefined {
  if (!node) return undefined;
  if (node.getText(source) === "process.execPath") return "node";
  if (!ts.isStringLiteralLike(node)) return undefined;
  const executable = path.posix.basename(node.text.replaceAll("\\", "/"));
  return interpreters.test(executable) ? executable : undefined;
}

function inlineDispatches(file: string, text: string): number[] {
  const lines: number[] = [];
  if (!/["'](?:-[\w-]*(?:[cep]|command)|eval|data:)/iu.test(text)) return lines;
  const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true);
  function visit(node: ts.Node): void {
    if (ts.isArrayLiteralExpression(node)) {
      let args: readonly ts.Expression[] = node.elements;
      let executable = interpreter(args[0], source);
      if (executable) {
        args = args.slice(1);
      } else {
        const call = node.parent;
        if (ts.isCallExpression(call) && call.arguments[1] === node) {
          executable = interpreter(call.arguments[0], source);
        }
      }
      // A dynamic filename ends interpreter options, just as a literal filename does.
      const words = args.map((arg) => (ts.isStringLiteralLike(arg) ? arg.text : "argument"));
      if (executable && !isProgramCommand([executable, ...words])) {
        lines.push(source.getLineAndCharacterOfPosition(node.getStart(source)).line + 1);
      }
    }
    ts.forEachChild(node, visit);
  }
  visit(source);
  return lines;
}

const fixtureWriters = new Map([
  ["writeFakeCommand", 2],
  ["writeFakeVize", 1],
  ["writeManager", 3],
  ["writeVueTsc", 1],
  ["createFakeBlacksmith", 0],
  ["runWarningBudget", 0],
]);

function inlineFixtureBodies(file: string, text: string): number[] {
  if (![...fixtureWriters.keys()].some((name) => text.includes(name))) return [];
  const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true);
  const lines: number[] = [];
  function visit(node: ts.Node): void {
    if (ts.isCallExpression(node) && ts.isIdentifier(node.expression)) {
      const index = fixtureWriters.get(node.expression.text);
      const argument = index === undefined ? undefined : node.arguments[index];
      if (
        argument &&
        (ts.isStringLiteralLike(argument) ||
          ts.isTemplateExpression(argument) ||
          ts.isTaggedTemplateExpression(argument) ||
          (ts.isBinaryExpression(argument) &&
            argument.operatorToken.kind === ts.SyntaxKind.PlusToken) ||
          (ts.isCallExpression(argument) &&
            ts.isPropertyAccessExpression(argument.expression) &&
            argument.expression.name.text === "join" &&
            ts.isArrayLiteralExpression(argument.expression.expression)))
      )
        lines.push(source.getLineAndCharacterOfPosition(argument.getStart(source)).line + 1);
    }
    ts.forEachChild(node, visit);
  }
  visit(source);
  return lines;
}

test("automation clients and executable fixtures load named programs", () => {
  const files = execFileSync(
    "git",
    [
      "ls-files",
      "--cached",
      "--others",
      "--exclude-standard",
      "-z",
      "--",
      ".github",
      "tools",
      "npm",
      "tests/_helpers",
      "tests/tooling",
    ],
    { cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  );
  const failures: string[] = [];
  for (const file of files.split("\0")) {
    if (
      !/\.[cm]?[jt]s(?:\.in)?$/u.test(file) ||
      /(?:^|\/)(?:_?fixtures|__fixtures__|testdata)\//u.test(file)
    )
      continue;
    const text = readFileSync(path.join(root, file), "utf8");
    if (file.startsWith("tests/tooling/")) {
      for (const line of inlineFixtureBodies(file, text)) failures.push(file + ":" + line);
    }
    if (/(?:\.d|\.test|\.spec)\.[cm]?[jt]s$/u.test(file)) continue;
    const program = file.endsWith(".in") ? text.replace(/@@[A-Z][A-Z0-9_]*@@/gu, "argument") : text;
    for (const line of inlineDispatches(file, program)) failures.push(file + ":" + line);
  }
  assert.deepEqual(failures, [], "Move interpreted program bodies to named files");
});

test("child-process checks distinguish interpreter options from program arguments", () => {
  const file = "probe.ts";
  assert.deepEqual(inlineDispatches(file, 'spawn("bash", ["-e", "-c", body])'), [1]);
  assert.deepEqual(
    inlineDispatches(file, 'run([process.execPath, "--input-type=module", "-e", source])'),
    [1],
  );
  assert.deepEqual(inlineDispatches(file, 'spawn(process.execPath, [compiler, "-p", config])'), []);
  assert.deepEqual(inlineDispatches(file, 'run(["git", "-c", option])'), []);
});

test("fixture writers take their command bodies from named sources", () => {
  assert.deepEqual(
    inlineFixtureBodies(
      "probe.ts",
      'writeFakeCommand(bin, "tool", ["process.exit(1)"].join("\\n"))',
    ),
    [1],
  );
  assert.deepEqual(
    inlineFixtureBodies("probe.ts", "writeVueTsc(path, `process.exit(${code})`)"),
    [1],
  );
  assert.deepEqual(
    inlineFixtureBodies(
      "probe.ts",
      'writeFakeCommand(bin, "tool", readToolingProgram("commands/tool.cjs"))',
    ),
    [],
  );
});
