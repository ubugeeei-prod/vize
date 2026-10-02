import assert from "node:assert/strict";
import { createRequire } from "node:module";
import { test } from "node:test";

const require = createRequire(new URL("../package.json", import.meta.url));
type Node = {
  type: number;
  name?: string;
  content?: string;
  children: Node[];
  props: Node[];
  loc: { source: string };
};
const { parse } = require("@vue/compiler-dom") as {
  parse: (source: string, options: { onError: (error: { code: number }) => void }) => Node;
};
const pkg = require("@vue/compiler-dom/package.json") as { version: string };

test("the actual pinned Vue package proves full pre argument/modifier control", () => {
  assert.equal(pkg.version, "3.5.35");
  for (const [heads, raw] of [
    [
      [
        "v-pre",
        "v-pre.foo",
        "v-pre:arg",
        "v-pre:[key]",
        "v-pre:[broken",
        "v-pre:.",
        "v-pre..",
        "v-pre:[",
        "v-pre:",
      ],
      true,
    ],
    [
      [
        "v-pre[broken",
        "v-pre[key].foo",
        "v-pretty",
        "v-prefix",
        "v-PRE",
        ":pre",
        "@pre",
        "#pre",
        ".pre",
        "pre",
        "data-v-pre",
      ],
      false,
    ],
  ] as const) {
    for (const head of heads) {
      const errors: number[] = [];
      const root = parse(`<div ${head}><Child :x="value">{{raw}}</Child></div><p>{{after}}</p>`, {
        onError: (error) => errors.push(error.code),
      });
      const child = root.children[0].children[0];
      assert.equal(child.children[0].type, raw ? 2 : 5, head);
      if (raw) assert.equal(child.children[0].content, "{{raw}}", head);
      assert.equal(child.props[0].type, raw ? 6 : 7, head);
      assert.equal(root.children[1].children[0].type, 5, `${head}: scope exit`);
      assert.deepEqual(errors, head === "v-pre:[broken" || head === "v-pre:[" ? [27] : [], head);
    }
  }
});

test("an unrelated balanced deep head is raw in either same-tag pre order", () => {
  const head = `v-bind:[${"([".repeat(33)}${"])".repeat(33)}]`;
  for (const attributes of [`${head} v-pre`, `v-pre ${head}`]) {
    const errors: number[] = [];
    const root = parse(
      `<div ${attributes}><Child :x="value">{{raw}}</Child></div><p>{{after}}</p>`,
      {
        onError: (error) => errors.push(error.code),
      },
    );
    const owner = root.children[0];
    assert.equal(owner.props.length, 1);
    assert.equal(owner.props[0].type, 6);
    // Vue's converted name can omit ':' in the second order; the authored
    // loc and raw scope are the oracle, not a fidelity-losing name rewrite.
    assert.equal(owner.props[0].loc.source, head);
    assert.equal(owner.children[0].props[0].type, 6);
    assert.equal(owner.children[0].children[0].type, 2);
    assert.equal(owner.children[0].children[0].content, "{{raw}}");
    assert.equal(root.children[1].children[0].type, 5);
    assert.deepEqual(errors, []);
  }
});
