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
const input = readFileSync(inputPath, "utf8");
const cases = JSON.parse(input).cases as Array<{
  id: string;
  source: string;
  admit: boolean;
  keepText: boolean;
}>;
const rootRequire = createRequire(new URL("../../package.json", import.meta.url));
const testsRequire = createRequire(import.meta.url);
// Read metadata beside the exact imported ESM entry, rather than resolving a
// second package through an ambient CJS or runner-provided dependency graph.
const domManifest = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.resolve("happy-dom")), "utf8"),
);
assert.equal(domManifest.name, "happy-dom");
const domVersion: string = domManifest.version;
const oracleNames = new Map([
  ["20.11.2", "oracle.json"],
  ["21.0.4", "oracle-dom-21.0.4.json"],
]);
const oracleName = oracleNames.get(domVersion);
assert.ok(oracleName, `No recorded complete DOM oracle for harness ${domVersion}`);
const outputPath = new URL(oracleName, fixture);
const window = new Window();
Object.assign(globalThis, {
  window,
  document: window.document,
  Node: window.Node,
  Element: window.Element,
  HTMLElement: window.HTMLElement,
  SVGElement: window.SVGElement,
  Text: window.Text,
  Comment: window.Comment,
});
const runtimeEntry = pathToFileURL(
  testsRequire.resolve("@vue/runtime-dom/dist/runtime-dom.esm-browser.prod.js"),
);
const Vue = await import(runtimeEntry.href);
const runtimeDomVersion = JSON.parse(
  readFileSync(new URL("../package.json", runtimeEntry), "utf8"),
).version;

type Dom =
  | { kind: "text"; value: string }
  | {
      kind: "element";
      tag: string;
      namespace: string | null;
      attributes: string[][];
      children: Dom[];
    };
type ObservableNode = { readonly nodeType: number; readonly textContent: string | null };
function observe(node: ObservableNode): Dom {
  if (node.nodeType === 3) return { kind: "text", value: node.textContent ?? "" };
  assert.equal(node.nodeType, 1, "the whole static DOM contains no hidden extra nodes");
  assert.ok(node instanceof window.Element, "mounted element belongs to the original DOM window");
  const element = node;
  return {
    kind: "element",
    tag: element.localName,
    namespace: element.namespaceURI,
    attributes: [...element.attributes].map((attribute) => {
      const { name, value } = attribute;
      assert.ok(typeof name === "string" && typeof value === "string");
      return [name, value];
    }),
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
    const refusedText: { range?: { start: number; end: number } } = {};
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
          if (!entry.admit && refusedText.range === undefined) {
            const { start, end } = path.node;
            if (start == null || end == null) throw new Error("original text range missing");
            refusedText.range = {
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
    const refusal: Array<{ kind: string; start: number; end: number }> = [];
    if (!entry.admit) {
      assert.ok(refusedText.range, "unsupported text has its original source range");
      refusal.push({ kind: "value-normalization", ...refusedText.range });
    }
    return {
      id: entry.id,
      sourceSha256: createHash("sha256").update(entry.source).digest("hex"),
      admit: entry.admit,
      code: transformed.code,
      html: container.innerHTML,
      dom,
      sourceOps,
      refusal,
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
    runtimeDom: runtimeDomVersion,
    dom: domVersion,
  },
  cases: cases.map(run),
};
if (process.env.VIZE_WRITE_JSX_FORMATTING_ORACLE === "1")
  writeFileSync(outputPath, `${JSON.stringify(actual, null, 2)}\n`);

test("native JSX formatting admission preserves the pinned transform and complete mounted DOM oracle", () => {
  assert.deepEqual(actual, JSON.parse(readFileSync(outputPath, "utf8")));
});

// Both explicitly executed DOM harness versions preserve exactly the same
// language/runtime result. Unknown versions still require a recorded oracle.
test("recorded DOM harnesses retain every normative transform and source packet", () => {
  const records = [...oracleNames.values()].map((name) =>
    JSON.parse(readFileSync(new URL(name, fixture), "utf8")),
  );
  for (const record of records) {
    assert.deepEqual(record.cases, records[0].cases);
    assert.equal(record.inputSha256, records[0].inputSha256);
    assert.equal(record.versions.jsx, records[0].versions.jsx);
    assert.equal(record.versions.babel, records[0].versions.babel);
    assert.equal(record.versions.runtimeDom, records[0].versions.runtimeDom);
  }
});
