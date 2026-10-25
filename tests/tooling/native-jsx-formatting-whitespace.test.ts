import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import test from "node:test";
import { compileFunction } from "node:vm";
import { pathToFileURL } from "node:url";
import babel, { type PluginObj } from "@babel/core";
import jsx from "@vue/babel-plugin-jsx";
import { Window } from "happy-dom";

const fixture = new URL(
  "../../davinci/vize_l2/tests/fixtures/jsx-formatting-whitespace/",
  import.meta.url,
);
const inputPath = new URL("cases.json", fixture);
const outputPath = new URL("oracle.json", fixture);
const input = readFileSync(inputPath, "utf8");
const cases = JSON.parse(input).cases as Array<{
  id: string;
  source: string;
  admit: boolean;
  keepText: boolean;
}>;
const rootRequire = createRequire(new URL("../../package.json", import.meta.url));
const testsRequire = createRequire(import.meta.url);
const window = new Window();
for (const name of [
  "window",
  "document",
  "Node",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Text",
  "Comment",
]) {
  (globalThis as Record<string, unknown>)[name] =
    name === "window" ? window : (window as unknown as Record<string, unknown>)[name];
}
const Vue = await import(
  pathToFileURL(testsRequire.resolve("@vue/runtime-dom/dist/runtime-dom.esm-browser.prod.js")).href
);

type Dom =
  | { kind: "text"; value: string }
  | {
      kind: "element";
      tag: string;
      namespace: string | null;
      attributes: string[][];
      children: Dom[];
    };
function observe(node: Node): Dom {
  if (node.nodeType === 3) return { kind: "text", value: node.textContent ?? "" };
  assert.equal(node.nodeType, 1, "the whole static DOM contains no hidden extra nodes");
  const element = node as Element;
  return {
    kind: "element",
    tag: element.localName,
    namespace: element.namespaceURI,
    attributes: [...element.attributes].map((attribute) => [attribute.name, attribute.value]),
    children: [...element.childNodes].map(observe),
  };
}
function count(nodes: Dom[]): number {
  return nodes.reduce(
    (total, node) => total + 1 + (node.kind === "element" ? count(node.children) : 0),
    0,
  );
}
function run(entry: (typeof cases)[number]) {
  const transformed = babel.transformSync(entry.source, {
    plugins: [jsx],
    filename: "input.jsx",
    configFile: false,
    babelrc: false,
  });
  assert.ok(transformed?.code);
  const executable = babel.transformSync(transformed.code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }: { types: typeof babel.types }): PluginObj => ({
        visitor: {
          ImportDeclaration(path) {
            assert.equal(path.node.source.value, "vue");
            path.replaceWithMultiple(
              path.node.specifiers.map((specifier) => {
                assert.ok(t.isImportSpecifier(specifier));
                return t.variableDeclaration("const", [
                  t.variableDeclarator(
                    specifier.local,
                    t.memberExpression(t.identifier("__Vue"), specifier.imported),
                  ),
                ]);
              }),
            );
          },
          ExportNamedDeclaration(path) {
            assert.ok(path.node.declaration);
            path.replaceWith(path.node.declaration);
          },
        },
      }),
    ],
  });
  assert.ok(executable?.code);
  const vnode = compileFunction(`${executable.code}\nreturn view;`, ["__Vue"])(Vue);
  const container = window.document.createElement("div");
  const app = Vue.createApp({ render: () => vnode });
  try {
    app.mount(container);
    const dom = [...container.childNodes].map(observe);
    const sourceOps: Array<{ kind: string; start: number; end: number }> = [];
    let refusedText: { start: number; end: number } | null = null;
    {
      const ast = babel.parseSync(entry.source, {
        sourceType: "module",
        parserOpts: { plugins: ["jsx"] },
        configFile: false,
        babelrc: false,
      });
      assert.ok(ast);
      const record = (node: { start?: number | null; end?: number | null }, kind: string) => {
        if (node.start == null || node.end == null)
          throw new Error("original syntax range missing");
        sourceOps.push({
          kind,
          start: Buffer.byteLength(entry.source.slice(0, node.start)),
          end: Buffer.byteLength(entry.source.slice(0, node.end)),
        });
      };
      babel.traverse(ast, {
        JSXElement(path) {
          if (entry.admit) record(path.node, "element");
        },
        JSXText(path) {
          if (entry.admit && entry.keepText) record(path.node, "text");
          if (!entry.admit && refusedText === null) {
            const { start, end } = path.node;
            if (start == null || end == null) throw new Error("original text range missing");
            refusedText = {
              start: Buffer.byteLength(entry.source.slice(0, start)),
              end: Buffer.byteLength(entry.source.slice(0, end)),
            };
          }
        },
      });
      if (entry.admit)
        assert.equal(
          sourceOps.length,
          count(dom),
          `${entry.id}: explicit retained-text witness must match every mounted node`,
        );
    }
    if (!entry.admit) assert.ok(refusedText);
    return {
      id: entry.id,
      sourceSha256: createHash("sha256").update(entry.source).digest("hex"),
      admit: entry.admit,
      code: transformed.code,
      html: container.innerHTML,
      dom,
      sourceOps,
      refusal: entry.admit ? [] : [{ kind: "value-normalization", ...refusedText }],
    };
  } finally {
    app.unmount();
  }
}

const actual = {
  schemaVersion: 1,
  inputSha256: createHash("sha256").update(input).digest("hex"),
  versions: {
    jsx: rootRequire("@vue/babel-plugin-jsx/package.json").version,
    babel: rootRequire("@babel/core/package.json").version,
    runtimeDom: testsRequire("@vue/runtime-dom/package.json").version,
    dom: testsRequire("happy-dom/package.json").version,
  },
  cases: cases.map(run),
};
if (process.env.VIZE_WRITE_JSX_FORMATTING_ORACLE === "1")
  writeFileSync(outputPath, `${JSON.stringify(actual, null, 2)}\n`);

test("native JSX formatting admission preserves the pinned transform and complete mounted DOM oracle", () => {
  assert.deepEqual(actual, JSON.parse(readFileSync(outputPath, "utf8")));
});
