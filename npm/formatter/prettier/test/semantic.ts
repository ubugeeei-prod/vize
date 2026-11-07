import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { compileFunction } from "node:vm";
import { parsers as typescript } from "prettier/plugins/typescript";
import { parsers as postcss } from "prettier/plugins/postcss";
import type { ParserOptions } from "prettier";

const require = createRequire(import.meta.url);
const Vue = require("vue");
const { compile } = createRequire(require.resolve("vue"))("@vue/compiler-dom");
const { parse } = require("vue/compiler-sfc");
const { renderToString } = require("vue/server-renderer");

function normalized(value: unknown, goal: "script" | "style"): unknown {
  if (Array.isArray(value)) return value.map((child) => normalized(child, goal));
  if (!value || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value)
      .filter(
        ([key]) =>
          // PostCSS source geometry is trivia; an ESTree import's source
          // Literal is semantic and must remain part of the comparison.
          !(key === "source" && goal === "style") &&
          ![
            "start",
            "end",
            "range",
            "loc",
            "raw",
            "raws",
            "parent",
            "comments",
            "tokens",
            "leadingComments",
            "trailingComments",
            "extra",
          ].includes(key),
      )
      .map(([key, child]) => [key, normalized(child, goal)]),
  );
}

export async function assertBlocksEquivalent(before: string, after: string): Promise<void> {
  const a = parse(before);
  const b = parse(after);
  assert.deepEqual(a.errors, []);
  assert.deepEqual(b.errors, []);
  for (const key of ["script", "scriptSetup"] as const) {
    assert.equal(Boolean(a.descriptor[key]), Boolean(b.descriptor[key]));
    if (a.descriptor[key]) {
      const options = { parser: "typescript" } as ParserOptions;
      const oldAst = await typescript.typescript.parse(a.descriptor[key].content, options);
      const newAst = await typescript.typescript.parse(b.descriptor[key].content, options);
      assert.deepEqual(normalized(newAst, "script"), normalized(oldAst, "script"));
    }
  }
  assert.equal(a.descriptor.styles.length, b.descriptor.styles.length);
  for (let index = 0; index < a.descriptor.styles.length; index++) {
    const options = { parser: "css" } as ParserOptions;
    const oldAst = await postcss.css.parse(a.descriptor.styles[index].content, options);
    const newAst = await postcss.css.parse(b.descriptor.styles[index].content, options);
    assert.deepEqual(normalized(newAst, "style"), normalized(oldAst, "style"));
  }
}

export async function renderTemplate(
  template: string,
  state: Record<string, unknown>,
): Promise<string> {
  const diagnostics: unknown[] = [];
  const compiled = compile(template, {
    mode: "function",
    onError: (error: unknown) => diagnostics.push(error),
  });
  assert.deepEqual(diagnostics, []);
  const render = compileFunction(compiled.code, ["Vue"])(Vue);
  render._rc = true;
  const stub = {
    render(this: { $slots: { default?: () => unknown } }) {
      return Vue.h("span", this.$slots.default?.());
    },
  };
  const app = Vue.createSSRApp({
    data: () => state,
    render,
    components: { MyChip: stub, MyButton: stub, MyIcon: stub },
  });
  const warnings: string[] = [];
  app.config.warnHandler = (warning: string) => warnings.push(warning);
  const html = await renderToString(app);
  assert.deepEqual(warnings, []);
  return html;
}

export async function assertRenderedEquivalent(before: string, after: string): Promise<void> {
  const a = parse(before);
  const b = parse(after);
  assert.deepEqual(a.errors, []);
  assert.deepEqual(b.errors, []);
  for (const unread of [0, 2]) {
    for (const visible of [false, true]) {
      const state = {
        unread,
        count: unread,
        label: "雪 😀 & <value>",
        openNav: () => {},
        visible,
        isSelected: visible,
        isEnabled: visible,
        hasPermissionForTheSelectedItemInTheCurrentView: true,
        props: {
          selectedIdentifier: visible ? "selected" : "other",
          totalPoint: 103,
          label: "雪 😀 & <value>",
        },
        SELECTED_IDENTIFIER_CONSTANT: "selected",
      };
      assert.equal(
        await renderTemplate(b.descriptor.template.content, state),
        await renderTemplate(a.descriptor.template.content, state),
      );
    }
  }
}
