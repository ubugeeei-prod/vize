/** Pure AST admission of complete real native IR emitter packets; no DOM/runtime credit. */
import assert from "node:assert/strict";
import fs from "node:fs";
import { parseSync, traverse, types as t } from "@babel/core";
type Packet = {
  source: string;
  expression: string;
  expressionSpan: [number, number];
  code: string;
  map: { sourcesContent: string[] };
  links: { authored: [number, number]; generated: [number, number] }[];
  scope: string;
};
const [beforePath, afterPath, outputPath] = process.argv.slice(2);
assert.ok(beforePath && afterPath && outputPath);
const before = JSON.parse(fs.readFileSync(beforePath, "utf8")) as Packet;
const after = JSON.parse(fs.readFileSync(afterPath, "utf8")) as Packet;
const receipt = {
  before,
  after,
  parsed: [] as unknown[],
  nativeRuntimeCredit: false,
  wholeCorpusQualified: false,
};
try {
  for (const key of ["source", "expression", "expressionSpan", "scope"] as const)
    assert.deepEqual(before[key], after[key]);
  for (const [role, packet] of [
    ["before", before],
    ["after", after],
  ] as const) {
    assert.deepEqual(packet.map.sourcesContent, [packet.source]);
    const expressionBytes = Buffer.from(packet.source)
      .subarray(...packet.expressionSpan)
      .toString();
    assert.equal(expressionBytes, packet.expression);
    const ast = parseSync(packet.code, {
      configFile: false,
      babelrc: false,
      filename: "typed-child.js",
    });
    assert.ok(ast);
    const helpers = new Set<string>();
    const arrays: t.ArrayExpression[] = [];
    traverse(ast, {
      ImportDeclaration(path) {
        if (path.node.source.value !== "vue") return;
        for (const specifier of path.node.specifiers)
          if (
            t.isImportSpecifier(specifier) &&
            t.isIdentifier(specifier.imported, { name: "createElementVNode" })
          )
            helpers.add(specifier.local.name);
      },
    });
    assert.equal(helpers.size, 1);
    traverse(ast, {
      CallExpression(path) {
        const n = path.node;
        if (
          t.isIdentifier(n.callee) &&
          helpers.has(n.callee.name) &&
          t.isStringLiteral(n.arguments[0], { value: "main" })
        ) {
          assert.ok(t.isArrayExpression(n.arguments[2]));
          arrays.push(n.arguments[2]);
        }
      },
    });
    assert.equal(arrays.length, 1);
    const children = arrays[0]!.elements;
    assert.equal(
      children.length,
      role === "before" ? 3 : 1,
      "array arity of the actual emitted module",
    );
    const parts =
      role === "before"
        ? children
        : (() => {
            assert.ok(t.isSequenceExpression(children[0]));
            return children[0].expressions;
          })();
    assert.equal(parts.length, 3);
    for (const [index, label] of ["left", "right"].entries()) {
      const part = parts[index];
      assert.ok(t.isCallExpression(part));
      assert.ok(t.isMemberExpression(part.callee));
      assert.ok(t.isIdentifier(part.callee.property, { name: "record" }));
      assert.equal(part.arguments.length, 1);
      assert.ok(t.isStringLiteral(part.arguments[0], { value: label }));
    }
    assert.ok(t.isMemberExpression(parts[2]));
    assert.ok(t.isIdentifier(parts[2].property, { name: "value" }));
    receipt.parsed.push({ role, ast, array: arrays[0] });
  }
} finally {
  fs.writeFileSync(outputPath, JSON.stringify(receipt, null, 2) + "\n");
}
