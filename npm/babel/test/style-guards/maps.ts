import assert from "node:assert/strict";
import { SourceMap, type SourceMapPayload } from "node:module";
import type { types as t } from "@babel/core";
import type { BabelAst, BabelCore } from "../host-types.ts";
import type { NativeCall } from "./types.ts";
import type { StyleCase } from "./cases.ts";
const point = (text: string, offset: number) => {
  const prefix = text.slice(0, offset);
  return {
    line: prefix.split("\n").length - 1,
    column: prefix.length - prefix.lastIndexOf("\n") - 1,
  };
};
export function nativeOrigins(core: BabelCore, packet: NativeCall, input: StyleCase) {
  const native = packet.result;
  assert.equal(typeof native.map, "string");
  const map = JSON.parse(native.map!) as SourceMapPayload;
  assert.equal(map.version, 3);
  assert.deepEqual(map.sources, ["module.tsx"]);
  assert.deepEqual(map.sourcesContent, [packet.text]);
  const parse = (text: string) =>
    core.parseSync(text, {
      babelrc: false,
      configFile: false,
      parserOpts: { plugins: ["jsx", "typescript"] },
    });
  const visit = (node: unknown, fn: (node: BabelAst) => void): void => {
    if (!node || typeof node !== "object") return;
    const ast = node as BabelAst;
    fn(ast);
    for (const key of core.types.VISITOR_KEYS[ast.type] ?? []) {
      const child = (ast as unknown as Record<string, unknown>)[key];
      if (Array.isArray(child)) child.forEach((item) => visit(item, fn));
      else visit(child, fn);
    }
  };
  const generated = parse(native.code),
    styleImports: string[] = [];
  visit(generated, (node) => {
    if (node.type === "ImportDeclaration" && node.source.value === "vue")
      for (const specifier of node.specifiers)
        if (
          specifier.type === "ImportSpecifier" &&
          specifier.imported.type === "Identifier" &&
          specifier.imported.name === "setStyle"
        )
          styleImports.push(specifier.local.name);
  });
  const calls: t.CallExpression[] = [];
  visit(generated, (node) => {
    if (
      node.type === "CallExpression" &&
      node.callee.type === "Identifier" &&
      styleImports.includes(node.callee.name)
    )
      calls.push(node);
  });
  assert.equal(
    calls.length,
    input.fold ? 0 : 1,
    "literal eligibility preserves the actual runtime style operation frontier",
  );
  const anchors = [];
  if (input.fold) {
    let attribute: t.JSXAttribute | undefined;
    visit(parse(packet.text), (node) => {
      if (
        node.type === "JSXAttribute" &&
        node.name.type === "JSXIdentifier" &&
        node.name.name === "style"
      )
        attribute = node;
    });
    assert(attribute?.value?.type === "JSXExpressionContainer");
    assert.equal(attribute.value.expression.type, "ObjectExpression");
    const name = native.code.indexOf('style=\\"');
    assert(name >= 0);
    const trace = new SourceMap(map);
    for (const [kind, generatedOffset, authoredOffset] of [
      ["style-name", name, attribute.name.start!],
      ["aggregate-style-expression", name + 'style=\\"'.length, attribute.value.expression.start!],
    ] as const) {
      const g = point(native.code, generatedOffset),
        a = point(packet.text, authoredOffset),
        actual = trace.findEntry(g.line, g.column);
      assert("originalSource" in actual && "originalLine" in actual && "originalColumn" in actual);
      assert.deepEqual(
        { source: actual.originalSource, line: actual.originalLine, column: actual.originalColumn },
        { source: "module.tsx", ...a },
      );
      anchors.push({ kind, generated: g, authored: a, actual });
    }
  }
  return {
    scope:
      "native per-node original-printed-AST anchors; final Babel map independently retains original authored source",
    map,
    anchors,
    styleCallSites: calls.length,
  };
}
