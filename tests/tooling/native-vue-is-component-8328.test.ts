import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { compile, parseTemplate, type CompilerOptions } from "../../npm/native/index.js";

const fixture = (name: string) =>
  readFileSync(
    new URL(`../_fixtures/differential/compiler/vue-is-component-8328/${name}`, import.meta.url),
    "utf8",
  );
const reported: { original: string; equivalent: string; options: CompilerOptions } = JSON.parse(
  fixture("public-native.json"),
);
const pairs = [
  [reported.original, reported.equivalent],
  [fixture("original.template.txt"), "<div>\n  <my-thing>x</my-thing>\n</div>\n"],
  [
    '<div is="vue:my-thing" title="a" :count="count">x</div>',
    '<my-thing title="a" :count="count">x</my-thing>',
  ],
  ['<div is="v&#117;e:my-thing">x</div>', "<my-thing>x</my-thing>"],
  [fixture("encoded.template.txt"), '<my-thing title="a&amp;b">x</my-thing>'],
  [fixture("ordinary-template.template.txt"), "<my-thing>x</my-thing>"],
] as const;
const output = (source: string, options: CompilerOptions) => {
  const { code, preamble, helpers } = compile(source, options);
  return { code, preamble, helpers };
};

for (const ssr of [false, true]) {
  for (const mode of ["function", "module"]) {
    test(`public native compile resolves #8328 whole ${ssr ? "SSR" : "DOM"} ${mode} output`, () => {
      const options = { ...reported.options, mode, ssr };
      for (const [original, equivalent] of pairs) {
        const actual = output(original, options);
        assert.deepEqual(actual, output(equivalent, options), original);
      }
    });

    test(`public native compile preserves ordinary and verbatim ${ssr ? "SSR" : "DOM"} ${mode} elements`, () => {
      for (const original of [
        fixture("plain.template.txt"),
        '<div :is="view">x</div>',
        '<div is="Vue:my-thing">x</div>',
        '<div v-pre is="vue:my-thing">x</div>',
        '<div v-pre><span is="vue:my-thing">x</span></div>',
      ]) {
        const actual = output(original, { mode, ssr, hoistStatic: false });
        assert.deepEqual(
          actual,
          output(original, { mode, ssr, hoistStatic: false, customElements: ["div", "span"] }),
          original,
        );
      }
      const original = '<div is="vue:my-thing">x</div>';
      const custom = compile(original, { mode, ssr, customElements: ["div"] });
      assert.deepEqual(custom.ast, {
        type: "ROOT",
        children: [
          {
            type: "ELEMENT",
            tag: "div",
            tagType: "Element",
            props: 1,
            children: 1,
            isSelfClosing: false,
          },
        ],
        comments: [],
        helpers: ["createElementVNode", "createTextVNode"],
        components: [],
        directives: [],
      });
    });
  }
}

test("public parseTemplate retains authored tag classification for #8328", () => {
  assert.deepEqual(parseTemplate('<div is="vue:my-thing">x</div>'), {
    type: "ROOT",
    children: [
      {
        type: "ELEMENT",
        tag: "div",
        tagType: "Element",
        props: 1,
        children: 1,
        isSelfClosing: false,
      },
    ],
    comments: [],
    helpers: [],
    components: [],
    directives: [],
  });
});
