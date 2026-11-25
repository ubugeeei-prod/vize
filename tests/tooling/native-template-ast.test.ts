import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import {
  parseTemplate,
  parseTemplateAst,
  type TemplateRootNode,
  type TemplateSourceLocation,
} from "../../npm/native/index.js";
import { parseTemplateAst as parseTemplateAstFromVize } from "../../npm/cli/src/template.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const require = createRequire(import.meta.url);

type AnyNode = { type: string; loc?: TemplateSourceLocation; [key: string]: unknown };

/** Every node of the tree (props, arguments, modifiers and expressions included). */
function nodes(node: unknown): AnyNode[] {
  if (Array.isArray(node)) return node.flatMap(nodes);
  if (node === null || typeof node !== "object") return [];
  const found = "type" in node ? [node as AnyNode] : [];
  return [...found, ...Object.values(node).flatMap(nodes)];
}

const text = (source: string, loc: TemplateSourceLocation) =>
  source.slice(loc.start.offset, loc.end.offset);

/** Assert each location's line and column agree with its UTF-16 offset. */
function assertPositions(source: string, ast: TemplateRootNode) {
  for (const node of nodes(ast)) {
    for (const loc of [node.loc, node.nameLoc as TemplateSourceLocation | undefined]) {
      if (!loc) continue;
      for (const point of [loc.start, loc.end]) {
        const before = source.slice(0, point.offset);
        assert.equal(point.line, before.split("\n").length, JSON.stringify(node));
        assert.equal(point.column, before.length - before.lastIndexOf("\n"), JSON.stringify(node));
      }
    }
  }
}

test("the reported template keeps its comment in place as a node", () => {
  const source = "<Transition><!-- note --><div></div></Transition>";
  const { ast, errors, warnings } = parseTemplateAst(source);
  assert.deepEqual([errors, warnings], [[], []]);
  const [transition] = ast.children;
  assert.equal(transition.type, "ELEMENT");
  if (transition.type !== "ELEMENT") return;
  assert.equal(transition.tagType, "COMPONENT");
  assert.deepEqual(
    transition.children.map((child) => [child.type, text(source, child.loc)]),
    [
      ["COMMENT", "<!-- note -->"],
      ["ELEMENT", "<div></div>"],
    ],
  );
  assert.equal(
    transition.children[0].type === "COMMENT" && transition.children[0].content,
    " note ",
  );
  assert.equal(text(source, ast.loc), source);
});

test("a self-closing non-void element parses as one element without a diagnostic", () => {
  const source = "<Transition><div /></Transition>";
  const { ast, errors, warnings } = parseTemplateAst(source);
  assert.deepEqual([errors, warnings], [[], []]);
  const div = (ast.children[0] as { children: AnyNode[] }).children[0];
  assert.deepEqual(
    { type: div.type, tag: div.tag, isSelfClosing: div.isSelfClosing, children: div.children },
    { type: "ELEMENT", tag: "div", isSelfClosing: true, children: [] },
  );
  assert.equal(text(source, div.loc!), "<div />");
  // The summary API treats the self-closing rewrite as the recovery note it is.
  assert.equal(parseTemplate(source).type, "ROOT");
  assert.throws(() => parseTemplate("<div>"), /Parse errors/);
});

test("nested components and directives carry their arguments, modifiers and expressions", () => {
  const source = [
    '<List v-for="(item, index) in items" :key="item.id">',
    '  <template #row="{ row }"><Cell v-if="row.ok" @click.stop="pick(row)">{{ row.name }}</Cell></template>',
    "</List>",
  ].join("\n");
  const { ast, errors } = parseTemplateAst(source);
  assert.deepEqual(errors, []);
  assertPositions(source, ast);
  const directives = nodes(ast)
    .filter((node) => node.type === "DIRECTIVE")
    .map((node) => {
      const exp = node.exp as AnyNode | null;
      const arg = node.arg as AnyNode | null;
      return [
        node.name,
        arg && arg.content,
        (node.modifiers as AnyNode[]).map((modifier) => modifier.content),
        exp && exp.content,
        exp && text(source, exp.loc!),
      ];
    });
  assert.deepEqual(directives, [
    ["for", null, [], "(item, index) in items", "(item, index) in items"],
    ["bind", "key", [], "item.id", "item.id"],
    ["slot", "row", [], "{ row }", "{ row }"],
    ["if", null, [], "row.ok", "row.ok"],
    ["on", "click", ["stop"], "pick(row)", "pick(row)"],
  ]);
  const elements = nodes(ast).filter((node) => node.type === "ELEMENT");
  assert.deepEqual(
    elements.map((node) => [node.tag, node.tagType]),
    [
      ["List", "COMPONENT"],
      ["template", "TEMPLATE"],
      ["Cell", "COMPONENT"],
    ],
  );
  const interpolation = nodes(ast).find((node) => node.type === "INTERPOLATION")!;
  assert.equal(text(source, interpolation.loc!), "{{ row.name }}");
});

test("every location slices its node out of non-ASCII source in UTF-16 units", () => {
  const source =
    '<p title="日本語">\r\n  こんにちは 😀 {{ greeting }}<!-- ✓ -->\n  <b :x="\'é\'">ü</b></p>';
  const { ast, errors } = parseTemplateAst(source);
  assert.deepEqual(errors, []);
  assertPositions(source, ast);
  const checked = new Map<string, string[]>();
  for (const node of nodes(ast)) {
    if (!node.loc) continue;
    const slice = text(source, node.loc);
    checked.set(node.type, [...(checked.get(node.type) ?? []), slice]);
    const content = String(node.content);
    if (node.type === "TEXT" && !content.includes("&")) {
      assert.ok(slice === content || slice === `"${content}"`, slice);
    }
    if (node.type === "SIMPLE_EXPRESSION") assert.equal(slice, content);
    if (node.type === "COMMENT") assert.equal(slice, `<!--${content}-->`);
  }
  assert.deepEqual(checked.get("ELEMENT"), [source, "<b :x=\"'é'\">ü</b>"]);
  assert.deepEqual(checked.get("INTERPOLATION"), ["{{ greeting }}"]);
  assert.deepEqual(checked.get("ATTRIBUTE"), ['title="日本語"']);
  assert.deepEqual(checked.get("DIRECTIVE"), [":x=\"'é'\""]);
});

test("syntax errors are returned with the recovered tree instead of thrown", () => {
  const source = "<div><span>text</div></b>";
  const { ast, errors, warnings } = parseTemplateAst(source);
  assert.deepEqual(
    errors.map((error) => [error.code, text(source, error.loc)]),
    [
      ["compiler/missing-end-tag", "<span>"],
      ["compiler/invalid-end-tag", "</b>"],
    ],
  );
  assert.deepEqual(warnings, []);
  assert.equal(ast.children[0].type, "ELEMENT");
});

test("the vize package exports the same native tree", () => {
  const source = '<Comp v-model="value"><!-- c --><i /></Comp>';
  assert.deepEqual(parseTemplateAstFromVize(source), parseTemplateAst(source));
  const index = fs.readFileSync(path.join(root, "npm/cli/src/index.ts"), "utf8");
  assert.match(index, /export \{ parseTemplateAst \} from "\.\/template\.js";/);
});

test("the shipped declarations type the tree as a discriminated union", (t) => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "template-ast-types-"));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  fs.writeFileSync(
    path.join(directory, "consumer.mts"),
    [
      `import { parseTemplateAst } from ${JSON.stringify(path.join(root, "npm/native/index.js"))};`,
      `import type { TemplateChildNode, TemplateNode } from ${JSON.stringify(path.join(root, "npm/cli/src/template.js"))};`,
      "const { ast, errors, warnings } = parseTemplateAst('<div />', { customElements: ['x-*'] });",
      "const codes: string[] = [...errors, ...warnings].map((diagnostic) => diagnostic.code);",
      "function walk(node: TemplateChildNode): number {",
      "  switch (node.type) {",
      "    case 'ELEMENT': {",
      "      const tagType: 'ELEMENT' | 'COMPONENT' | 'SLOT' | 'TEMPLATE' = node.tagType;",
      "      for (const prop of node.props) {",
      "        if (prop.type === 'DIRECTIVE') {",
      "          const expression: string | undefined = prop.exp?.content;",
      "          const modifiers: string[] = prop.modifiers.map((modifier) => modifier.content);",
      "          void [expression, modifiers, prop.arg?.isStatic, prop.rawName];",
      "        } else {",
      "          const value: string | undefined = prop.value?.content;",
      "          void [value, prop.nameLoc.start.offset];",
      "        }",
      "      }",
      "      void tagType;",
      "      return node.children.reduce((count, child) => count + walk(child), 1);",
      "    }",
      "    case 'INTERPOLATION':",
      "      return node.content.content.length;",
      "    case 'TEXT':",
      "    case 'COMMENT':",
      "      return node.content.length + node.loc.end.line;",
      "  }",
      "}",
      "const total: number = ast.children.reduce((count, child) => count + walk(child), 0);",
      "const any: TemplateNode = ast;",
      "// @ts-expect-error a root has no tag.",
      "void ast.tag;",
      "// @ts-expect-error unknown options are rejected.",
      "parseTemplateAst('', { mode: 'module' });",
      "void [codes, total, any];",
      "",
    ].join("\n"),
  );
  fs.writeFileSync(
    path.join(directory, "tsconfig.json"),
    JSON.stringify({
      compilerOptions: {
        module: "NodeNext",
        moduleResolution: "NodeNext",
        noEmit: true,
        skipLibCheck: true,
        strict: true,
        target: "ES2022",
        // The `vize` entry loads the native binding through `node:module`.
        typeRoots: [path.join(root, "node_modules/@types")],
        types: ["node"],
      },
      files: ["consumer.mts"],
    }),
  );
  const result = spawnSync(
    process.execPath,
    [require.resolve("typescript/bin/tsc"), "-p", directory, "--pretty", "false"],
    { encoding: "utf8", timeout: 60_000 },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
});
