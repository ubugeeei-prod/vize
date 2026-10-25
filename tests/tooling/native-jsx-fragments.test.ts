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
  "../../davinci/vize_l2/tests/fixtures/jsx-static-fragments/",
  import.meta.url,
);
const input = readFileSync(new URL("cases.json", fixture), "utf8");
const cases = JSON.parse(input).cases as Array<{
  id: string;
  source: string;
  admit: boolean;
  fileIncomplete?: boolean;
  refusal?: { kind: string; type: string; name: string | null };
}>;
const rootRequire = createRequire(new URL("../../package.json", import.meta.url));
const testsRequire = createRequire(import.meta.url);
const domVersion = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.resolve("happy-dom")), "utf8"),
).version;
const oracleNames = new Map([
  ["20.11.2", "oracle.json"],
  ["21.0.4", "oracle-dom-21.0.4.json"],
]);
const oracleName = oracleNames.get(domVersion);
assert.ok(oracleName, `No complete fragment oracle for DOM harness ${domVersion}`);
const outputPath = new URL(oracleName, fixture);
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
])
  (globalThis as Record<string, unknown>)[name] =
    name === "window" ? window : (window as unknown as Record<string, unknown>)[name];
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
      namespace: string;
      attributes: string[][];
      children: Dom[];
    };
type Vnode = {
  type: string | symbol;
  props: Record<string, string> | null;
  children: Vnode[] | string | null;
};
function observe(node: Node): Dom {
  if (node.nodeType === 3) return { kind: "text", value: node.textContent ?? "" };
  assert.equal(node.nodeType, 1, "every mounted node must be explicitly represented");
  const element = node as Element;
  assert.ok(element.namespaceURI);
  return {
    kind: "element",
    tag: element.localName,
    namespace: element.namespaceURI,
    attributes: [...element.attributes].map((a) => [a.name, a.value]),
    children: [...element.childNodes].map(observe),
  };
}
function count(nodes: Dom[]): number {
  return nodes.reduce((n, node) => n + 1 + (node.kind === "element" ? count(node.children) : 0), 0);
}
// Both views derive from the real transform's VNodes before mount. Semantic
// fragments have no wrapper; backend realization explicitly accounts for both
// empty anchors. No mounted node is filtered or silently discarded.
function view(node: Vnode | string, anchors: boolean): Dom[] {
  if (typeof node === "string") return [{ kind: "text", value: node }];
  const children = () =>
    typeof node.children === "string"
      ? node.children === ""
        ? []
        : [{ kind: "text" as const, value: node.children }]
      : (node.children ?? []).flatMap((child) => view(child, anchors));
  if (node.type === Vue.Fragment)
    return anchors
      ? [{ kind: "text", value: "" }, ...children(), { kind: "text", value: "" }]
      : children();
  if (node.type === Vue.Text) {
    assert.equal(typeof node.children, "string");
    return [{ kind: "text", value: node.children as string }];
  }
  assert.equal(typeof node.type, "string");
  return [
    {
      kind: "element",
      tag: node.type as string,
      namespace: "http://www.w3.org/1999/xhtml",
      attributes: Object.entries(node.props ?? {}),
      children: children(),
    },
  ];
}
function transform(source: string, execute: boolean) {
  const transformed = babel.transformSync(source, {
    plugins: [jsx],
    filename: "input.jsx",
    configFile: false,
    babelrc: false,
  });
  assert.ok(transformed?.code);
  if (!execute)
    return {
      code: transformed.code,
      execute: (): never => {
        throw new Error("unsupported input must not execute");
      },
    };
  const executable = babel.transformSync(transformed.code, {
    configFile: false,
    babelrc: false,
    plugins: [
      ({ types: t }: { types: typeof babel.types }): PluginObj => ({
        visitor: {
          ImportDeclaration(path) {
            assert.equal(path.node.source.value, "vue");
            path.replaceWithMultiple(
              path.node.specifiers.map((s) => {
                assert.ok(t.isImportSpecifier(s));
                return t.variableDeclaration("const", [
                  t.variableDeclarator(
                    s.local,
                    t.memberExpression(t.identifier("__Vue"), s.imported),
                  ),
                ]);
              }),
            );
          },
        },
      }),
    ],
  });
  assert.ok(executable?.code);
  return {
    code: transformed.code,
    execute: () => compileFunction(`${executable.code}\nreturn view;`, ["__Vue"])(Vue) as Vnode,
  };
}
function witnesses(entry: (typeof cases)[number]) {
  const ast = babel.parseSync(entry.source, {
    sourceType: "module",
    parserOpts: { plugins: ["jsx"] },
    configFile: false,
    babelrc: false,
  });
  assert.ok(ast);
  const range = (node: { start?: number | null; end?: number | null }) => {
    assert.ok(node.start != null && node.end != null);
    return {
      start: Buffer.byteLength(entry.source.slice(0, node.start)),
      end: Buffer.byteLength(entry.source.slice(0, node.end)),
    };
  };
  const sourceOps: Array<object> = [],
    boundaries: Array<object> = [],
    refusal: Array<object> = [];
  babel.traverse(ast, {
    enter(path) {
      if (entry.refusal && refusal.length === 0 && path.node.type === entry.refusal.type) {
        const name = babel.types.isJSXIdentifier(path.node)
          ? path.node.name
          : babel.types.isJSXAttribute(path.node) && babel.types.isJSXIdentifier(path.node.name)
            ? path.node.name.name
            : null;
        if (entry.refusal.name === null || entry.refusal.name === name)
          refusal.push({ kind: entry.refusal.kind, ...range(path.node) });
      }
    },
    JSXElement(path) {
      if (entry.admit) sourceOps.push({ kind: "element", ...range(path.node) });
    },
    JSXText(path) {
      if (entry.admit && !(/^[ \t\r\n]*$/.test(path.node.value) && /[\r\n]/.test(path.node.value)))
        sourceOps.push({ kind: "text", ...range(path.node) });
    },
    JSXFragment: {
      enter(path) {
        const parent = path.findParent((p) => p.isJSXFragment());
        boundaries.push({
          kind: "opening",
          ...range(path.node.openingFragment),
          fragment: range(path.node),
          parentFragment: parent ? range(parent.node) : null,
        });
      },
      exit(path) {
        const parent = path.findParent((p) => p.isJSXFragment());
        boundaries.push({
          kind: "closing",
          ...range(path.node.closingFragment),
          fragment: range(path.node),
          parentFragment: parent ? range(parent.node) : null,
        });
      },
    },
  });
  assert.equal(refusal.length, entry.admit ? 0 : 1);
  if (entry.fileIncomplete)
    refusal.push({ kind: "file-incomplete", start: 0, end: Buffer.byteLength(entry.source) });
  return { sourceOps, boundaries, refusal };
}
function run(entry: (typeof cases)[number]) {
  const transformed = transform(entry.source, entry.admit),
    original = witnesses(entry);
  const common = {
    id: entry.id,
    sourceSha256: createHash("sha256").update(entry.source).digest("hex"),
    admit: entry.admit,
    code: transformed.code,
    ...original,
  };
  if (!entry.admit) return { ...common, semantic: null, dom: null, html: null };
  const vnode = transformed.execute();
  const semantic = view(vnode, false),
    realized = view(vnode, true);
  assert.equal(count(semantic), original.sourceOps.length);
  assert.equal(count(realized), count(semantic) + original.boundaries.length);
  const container = window.document.createElement("div"),
    app = Vue.createApp({ render: () => vnode });
  try {
    app.mount(container);
    const dom = [...container.childNodes].map(observe);
    assert.deepEqual(
      dom,
      realized,
      `${entry.id}: full mounted DOM must retain every fragment anchor`,
    );
    return { ...common, semantic, dom, html: container.innerHTML };
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
if (process.env.VIZE_WRITE_JSX_FRAGMENT_ORACLE === "1")
  writeFileSync(outputPath, `${JSON.stringify(actual, null, 2)}\n`);
test("native fragments retain independent transform semantics, all mounted anchors and original ranges", () => {
  assert.deepEqual(actual, JSON.parse(readFileSync(outputPath, "utf8")));
});
test("both executed DOM harnesses retain identical complete fragment packets", () => {
  const records = [...oracleNames.values()].map((name) =>
    JSON.parse(readFileSync(new URL(name, fixture), "utf8")),
  );
  for (const record of records) {
    assert.deepEqual(record.cases, records[0].cases);
    assert.equal(record.inputSha256, records[0].inputSha256);
    for (const name of ["jsx", "babel", "runtimeDom"])
      assert.equal(record.versions[name], records[0].versions[name]);
  }
});
