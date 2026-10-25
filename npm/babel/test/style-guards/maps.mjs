import assert from "node:assert/strict";
import { SourceMap } from "node:module";
const point = (text, offset) => {
  const prefix = text.slice(0, offset);
  return {
    line: prefix.split("\n").length - 1,
    column: prefix.length - prefix.lastIndexOf("\n") - 1,
  };
};
export function nativeOrigins(core, packet, input) {
  const native = packet.result;
  assert.equal(typeof native.map, "string");
  const map = JSON.parse(native.map);
  assert.equal(map.version, 3);
  assert.deepEqual(map.sources, ["module.tsx"]);
  assert.deepEqual(map.sourcesContent, [packet.text]);
  const parse = (text) =>
    core.parseSync(text, {
      babelrc: false,
      configFile: false,
      parserOpts: { plugins: ["jsx", "typescript"] },
    });
  const visit = (node, fn) => {
    if (!node || typeof node !== "object") return;
    fn(node);
    for (const key of core.types.VISITOR_KEYS[node.type] ?? []) {
      const child = node[key];
      if (Array.isArray(child)) child.forEach((item) => visit(item, fn));
      else visit(child, fn);
    }
  };
  const generated = parse(native.code),
    styleImports = [];
  visit(generated, (node) => {
    if (node.type === "ImportDeclaration" && node.source.value === "vue")
      for (const specifier of node.specifiers)
        if (specifier.type === "ImportSpecifier" && specifier.imported.name === "setStyle")
          styleImports.push(specifier.local.name);
  });
  const calls = [];
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
    let attribute;
    visit(parse(packet.text), (node) => {
      if (node.type === "JSXAttribute" && node.name.name === "style") attribute = node;
    });
    assert(attribute?.value.type === "JSXExpressionContainer");
    assert.equal(attribute.value.expression.type, "ObjectExpression");
    const name = native.code.indexOf('style=\\"');
    assert(name >= 0);
    const trace = new SourceMap(map);
    for (const [kind, generatedOffset, authoredOffset] of [
      ["style-name", name, attribute.name.start],
      ["aggregate-style-expression", name + 'style=\\"'.length, attribute.value.expression.start],
    ]) {
      const g = point(native.code, generatedOffset),
        a = point(packet.text, authoredOffset),
        actual = trace.findEntry(g.line, g.column);
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
