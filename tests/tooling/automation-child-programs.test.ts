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

test("automation clients pass named programs to interpreter subprocesses", () => {
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
      "tests/tooling/support",
    ],
    { cwd: root, encoding: "utf8", maxBuffer: 16 * 1024 * 1024 },
  );
  const failures: string[] = [];
  for (const file of files.split("\0")) {
    if (
      !/\.[cm]?[jt]s$/u.test(file) ||
      /(?:^|\/)(?:_?fixtures|__fixtures__|testdata)\//u.test(file) ||
      /(?:\.d|\.test|\.spec)\.[cm]?[jt]s$/u.test(file)
    )
      continue;
    for (const line of inlineDispatches(file, readFileSync(path.join(root, file), "utf8"))) {
      failures.push(file + ":" + line);
    }
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
