import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import type * as TS from "typescript";
import successors from "../../playground/e2e/fixtures/generated-code-snapshot-successors-20261105.json" with { type: "json" };
import { normalizeGeneratedCodeForSnapshot } from "../../playground/e2e/snapshot-utils.ts";

const ts: typeof TS = createRequire(new URL("../../npm/cli/package.json", import.meta.url))(
  "typescript",
);
const root = new URL("../../", import.meta.url);
const hash = (value: string | Buffer) => createHash("sha256").update(value).digest("hex");
type Value = string | Record<string, string>;
type Shape = { kind: number; fields: Record<string, string | boolean | number>; children: Shape[] };
const presentation = new Set([
  "pos",
  "end",
  "id",
  "kind",
  "flags",
  "transformFlags",
  "modifierFlagsCache",
  "singleQuote",
  "multiLine",
]);

// Compare the complete independently parsed module. Redundant parentheses and
// shorthand property spelling do not change the expression or binding tree.
function shape(node: TS.Node): Shape {
  if (ts.isParenthesizedExpression(node)) return shape(node.expression);
  if (
    ts.isShorthandPropertyAssignment(node) &&
    !node.objectAssignmentInitializer &&
    node.name.text !== "__proto__"
  )
    return {
      kind: ts.SyntaxKind.PropertyAssignment,
      fields: {},
      children: [shape(node.name), shape(node.name)],
    };
  if (ts.isSourceFile(node))
    return { kind: node.kind, fields: {}, children: node.statements.map(shape) };
  const fields: Shape["fields"] = {};
  for (const [key, value] of Object.entries(node))
    if (
      !presentation.has(key) &&
      (typeof value === "string" || typeof value === "boolean" || typeof value === "number")
    )
      fields[key] = value;
  const children: Shape[] = [];
  ts.forEachChild(node, (child) => {
    children.push(shape(child));
  });
  return { kind: node.kind, fields, children };
}

function moduleShape(source: string): Shape {
  const parsed = ts.createSourceFile(
    "module.js",
    source,
    ts.ScriptTarget.Latest,
    false,
    ts.ScriptKind.JS,
  );
  const diagnostics = ts.transpileModule(source, {
    fileName: "module.js",
    reportDiagnostics: true,
    compilerOptions: {
      allowJs: true,
      target: ts.ScriptTarget.Latest,
      module: ts.ModuleKind.ESNext,
    },
  }).diagnostics;
  assert.deepEqual(diagnostics, [], "the complete observed module must parse independently");
  return shape(parsed);
}

function serialized(value: Value): string {
  return typeof value === "string"
    ? `"${value}"`
    : `{\n${Object.entries(value)
        .map(([key, code]) => `  "${key}": "${code}",`)
        .join("\n")}\n}`;
}

test("all twelve exact browser successors retain the immutable whole snapshots and original inputs", () => {
  assert.equal(
    hash(
      fs.readFileSync(
        new URL("playground/e2e/fixtures/generated-code-snapshot-successors-20261105.json", root),
      ),
    ),
    "c2855d19c54d9dcbe41a143a77abfcffc5f362ee332a4a5d699b5bda5b35b3a1",
  );
  assert.equal(successors.cases.length, 12);
  assert.equal(new Set(successors.cases.map(({ current }) => JSON.stringify(current))).size, 12);
  assert.equal(successors.qualificationCredit, false);
  assert.equal(successors.source.revision, "8c5ba830e3e9d99605336a33dd32d56aa8ec2a2a");
  assert.equal(successors.source.job, 127152857371);
  for (const binding of successors.historicalSnapshots) {
    const bytes = fs.readFileSync(new URL(binding.path, root));
    assert.equal(bytes.length, binding.bytes);
    assert.equal(hash(bytes), binding.sha256);
  }
  let modules = 0;
  for (const row of successors.cases) {
    const previous = row.previous as Value,
      current = row.current as Value;
    const file = ts.createSourceFile(
      row.snapshotFile,
      fs.readFileSync(new URL(`playground/e2e/__snapshots__/${row.snapshotFile}`, root), "utf8"),
      ts.ScriptTarget.Latest,
      true,
      ts.ScriptKind.JS,
    );
    let archived: string | undefined;
    for (const statement of file.statements) {
      if (!ts.isExpressionStatement(statement) || !ts.isBinaryExpression(statement.expression))
        continue;
      const { left, right } = statement.expression;
      if (
        ts.isElementAccessExpression(left) &&
        ts.isNoSubstitutionTemplateLiteral(left.argumentExpression) &&
        left.argumentExpression.text === row.snapshotName
      ) {
        assert.ok(ts.isNoSubstitutionTemplateLiteral(right));
        archived = right.text;
      }
    }
    assert.equal(archived, `\n${serialized(previous)}\n`, row.snapshotName);
    const input = ts.createSourceFile(
      row.input.path,
      fs.readFileSync(new URL(row.input.path, root), "utf8"),
      ts.ScriptTarget.Latest,
      true,
    );
    const name = row.snapshotName.split(" > ").at(-1)!.replace(/ 1$/u, "");
    let source: string | undefined;
    function scan(node: TS.Node) {
      if (
        ts.isCallExpression(node) &&
        ts.isIdentifier(node.expression) &&
        node.expression.text === "it" &&
        node.arguments[0] &&
        ts.isStringLiteral(node.arguments[0]) &&
        node.arguments[0].text === name
      ) {
        function read(child: TS.Node) {
          if (
            ts.isVariableDeclaration(child) &&
            ts.isIdentifier(child.name) &&
            child.name.text === "sfc" &&
            child.initializer &&
            ts.isNoSubstitutionTemplateLiteral(child.initializer)
          )
            source = child.initializer.text;
          ts.forEachChild(child, read);
        }
        read(node);
      }
      ts.forEachChild(node, scan);
    }
    scan(input);
    assert.equal(typeof source, "string", row.snapshotName);
    assert.equal(Buffer.byteLength(source!), row.input.sourceBytes);
    assert.equal(hash(source!), row.input.sourceSha256);
    assert.deepEqual(normalizeGeneratedCodeForSnapshot(current), previous);
    assert.notDeepEqual(current, previous);
    const pairs =
      typeof current === "string"
        ? [[previous as string, current]]
        : Object.keys(current).map((key) => [
            (previous as Record<string, string>)[key]!,
            current[key]!,
          ]);
    for (const [before, after] of pairs) {
      assert.deepEqual(moduleShape(after!), moduleShape(before!), row.snapshotName);
      modules++;
    }
  }
  assert.equal(modules, 15);
});

test("foreign code, missing modules and opaque fields cannot acquire a reviewed browser snapshot", () => {
  for (const row of successors.cases) {
    const current = row.current as Value,
      previous = row.previous as Value;
    const mutate = (suffix: string): Value =>
      typeof current === "string"
        ? `${current}${suffix}`
        : Object.fromEntries(
            Object.entries(current).map(([key, code]) => [key, `${code}${suffix}`]),
          );
    for (const suffix of [
      "\n// foreign comment",
      "\nthrow new Error('unreviewed');",
      "\nexport const unexpected = true;",
    ])
      assert.notDeepEqual(
        normalizeGeneratedCodeForSnapshot(mutate(suffix)),
        previous,
        row.snapshotName,
      );
    if (typeof current === "object") {
      assert.notDeepEqual(
        normalizeGeneratedCodeForSnapshot({ ...current, foreign: "opaque" }),
        previous,
      );
      assert.notDeepEqual(normalizeGeneratedCodeForSnapshot({ dom: current.dom }), previous);
    }
  }
  assert.deepEqual(
    normalizeGeneratedCodeForSnapshot({ errors: ["unrelated"], map: { mappings: "AAAA" } }),
    { errors: ["unrelated"], map: { mappings: "AAAA" } },
  );
});
