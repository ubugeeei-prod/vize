import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
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

function writesNodeProgram(node: ts.Expression): boolean {
  if (ts.isStringLiteralLike(node)) return hasNodeProgram(node.text);
  if (ts.isTemplateExpression(node))
    return (
      hasNodeProgram(node.head.text) ||
      node.templateSpans.some((span) => hasNodeProgram(span.literal.text))
    );
  if (ts.isTaggedTemplateExpression(node)) return writesNodeProgram(node.template);
  if (ts.isBinaryExpression(node) && node.operatorToken.kind === ts.SyntaxKind.PlusToken)
    return writesNodeProgram(node.left) || writesNodeProgram(node.right);
  if (
    ts.isCallExpression(node) &&
    ts.isPropertyAccessExpression(node.expression) &&
    node.expression.name.text === "join" &&
    ts.isArrayLiteralExpression(node.expression.expression)
  )
    return node.expression.expression.elements.some(writesNodeProgram);
  return false;
}

function hasNodeProgram(source: string): boolean {
  return (
    /\b(?:process\.(?:argv|stdout|stderr|exit)\b|node:child_process)/u.test(source) &&
    /(?:^|[\n;])\s*(?:import\b|export\b|(?:const|let|var)\s+|process\.|require\()/u.test(source)
  );
}

function inlineFixtureBodies(file: string, text: string): number[] {
  if (
    !text.includes("writeFile") &&
    ![...fixtureWriters.keys()].some((name) => text.includes(name))
  )
    return [];
  const source = ts.createSourceFile(file, text, ts.ScriptTarget.Latest, true);
  const lines: number[] = [];
  function visit(node: ts.Node): void {
    if (ts.isCallExpression(node)) {
      const callee = ts.isPropertyAccessExpression(node.expression)
        ? node.expression.name
        : node.expression;
      const name = ts.isIdentifier(callee) ? callee.text : "";
      if (
        (name === "writeFileSync" || name === "writeFile") &&
        node.arguments[1] &&
        writesNodeProgram(node.arguments[1])
      )
        lines.push(
          source.getLineAndCharacterOfPosition(node.arguments[1].getStart(source)).line + 1,
        );
      const index = fixtureWriters.get(name);
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
      "crates",
      "davinci",
      "tools",
      "npm",
      "tests",
      "examples",
      "editors",
      "docs",
      "playground",
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
    const source = path.join(root, file);
    if (!existsSync(source)) continue;
    const text = readFileSync(source, "utf8");
    for (const line of inlineFixtureBodies(file, text)) failures.push(file + ":" + line);
    if (/\.d\.[cm]?[jt]s$/u.test(file)) continue;
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
    inlineFixtureBodies("probe.ts", 'fs.writeFileSync(file, "process.exit(2)")'),
    [1],
  );
  assert.deepEqual(
    inlineFixtureBodies(
      "probe.ts",
      'fs.writeFileSync(file, JSON.stringify({example:"process.exit(2)"}))',
    ),
    [],
  );

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

function inlineRustProcess(source: string): boolean {
  const launches = /Command::new\(\s*"([^"]+)"\s*\)([^;]{0,1600})/gu;
  for (const [, executable, chain] of source.matchAll(launches)) {
    const name = path.posix.basename(executable);
    if (!interpreters.test(name)) continue;
    const args = chain.match(/\.args?\(\s*(?:\[\s*)?(?:"--input-type=module",\s*)?"([^"]+)"/u);
    if (!args) continue;
    if (/^(?:node(?:js)?|bun|deno|python[\d.]*|perl|ruby)(?:\.exe)?$/iu.test(name)) {
      if (!isProgramCommand([name, args[1], "program"])) return true;
    } else if (
      /\.args?\(\s*(?:\[\s*)?"(?:-[\w]*c|-command|-encodedcommand)"\s*(?:,\s*|\)\s*\.arg\(\s*)(?:b?r?#*")/iu.test(
        chain,
      )
    )
      return true;
  }
  return false;
}

test("Rust subprocess clients launch named interpreter programs", () => {
  assert.ok(inlineRustProcess('Command::new("node").arg("-e").arg(source)'));
  assert.ok(inlineRustProcess('Command::new("node").args(["--input-type=module", "-e", OBSERVE])'));
  assert.ok(inlineRustProcess('Command::new("/bin/sh").args(["-c", "exit 0"])'));
  assert.ok(inlineRustProcess('Command::new("pwsh").arg("-Command").arg("Get-Date")'));
  assert.equal(inlineRustProcess('Command::new("node").arg("probe.cjs").arg("-e")'), false);
  assert.equal(
    inlineRustProcess('Command::new("git").args(["-c", "core.hooksPath=/dev/null"])'),
    false,
  );
  const files = execFileSync(
    "git",
    ["ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", "*.rs"],
    { cwd: root, encoding: "utf8", maxBuffer: 8 * 1024 * 1024 },
  );
  const embedded: string[] = [];
  for (const file of files.split("\0")) {
    if (!file || file.startsWith("upstream/") || !existsSync(path.join(root, file))) continue;
    const text = readFileSync(path.join(root, file), "utf8");
    if (text.includes("Command::new") && inlineRustProcess(text)) embedded.push(file);
  }
  assert.deepEqual(embedded, []);
});
