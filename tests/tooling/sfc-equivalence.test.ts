import assert from "node:assert/strict";
import { test } from "node:test";

import { compareSfcEquivalence } from "./support/sfc-equivalence.ts";

const original =
  '<template>\n  <div title="x" :class="foo" v-bind="rest">a {{n+1}}</div>\n</template>\n';

test("glyph SFC comparator accepts layout-only formatting", () => {
  assert.deepEqual(
    compareSfcEquivalence(
      original,
      '<template>\n  <div\n    title="x"\n    :class="foo"\n    v-bind="rest"\n  >a {{ n + 1 }}</div>\n</template>\n',
      "App.vue",
    ),
    [],
  );
});

test("glyph SFC comparator preserves attribute and spread order", () => {
  assert.deepEqual(
    compareSfcEquivalence(
      '<template>\n  <Widget\n    v-model:pos="\n      buttonPosition\n    "\n  />\n</template>\n',
      '<template>\n  <Widget v-model:pos="buttonPosition" />\n</template>\n',
      "App.vue",
    ),
    [],
  );
  assert.match(
    compareSfcEquivalence(
      original,
      '<template>\n  <div :class="foo" v-bind="rest">a {{ n + 1 }}</div>\n</template>\n',
      "App.vue",
    ).join("\n"),
    /<div>\[0\]/,
  );
  assert.match(
    compareSfcEquivalence(
      original,
      '<template>\n  <div v-bind="rest" title="x" :class="foo">a {{ n + 1 }}</div>\n</template>\n',
      "App.vue",
    ).join("\n"),
    /<div>\[0\]/,
  );
  for (const [before, after] of [
    [
      '<template><div title="before" :="rest" id="after" /></template>\n',
      '<template><div :="rest" title="before" id="after" /></template>\n',
    ],
    [
      '<template><button @click="before" @="listeners" @focus="after" /></template>\n',
      '<template><button @="listeners" @click="before" @focus="after" /></template>\n',
    ],
  ]) {
    assert.match(compareSfcEquivalence(before, after, "App.vue").join("\n"), /<(div|button)>\[0\]/);
  }
});

test("glyph SFC comparator preserves text, interpolation, and effect order", () => {
  assert.match(
    compareSfcEquivalence(
      original,
      '<template>\n  <div title="x" :class="foo" v-bind="rest">a {{ n + 2 }}</div>\n</template>\n',
      "App.vue",
    ).join("\n"),
    /#interpolation/,
  );
  assert.match(
    compareSfcEquivalence(
      "<template><p>A{{ name }}B</p></template>\n",
      "<template><p>A {{ name }} B</p></template>\n",
      "App.vue",
    ).join("\n"),
    /#text/,
  );
  assert.match(
    compareSfcEquivalence(
      '<template><Widget :z="first()" :a="second()" /></template>\n',
      '<template><Widget :a="second()" :z="first()" /></template>\n',
      "App.vue",
    ).join("\n"),
    /<Widget>/,
  );
});

test("glyph SFC comparator preserves native raw text and v-pre", () => {
  assert.match(
    compareSfcEquivalence(
      "<template><pre>  a  b</pre></template>\n",
      "<template><pre>  a b</pre></template>\n",
      "App.vue",
    ).join("\n"),
    /<pre>/,
  );
  for (const tag of ["pre", "textarea", "listing"]) {
    assert.match(
      compareSfcEquivalence(
        `<template><${tag}><span>  keep   me  </span></${tag}></template>\n`,
        `<template><${tag}><span> keep me </span></${tag}></template>\n`,
        "App.vue",
      ).join("\n"),
      new RegExp(`<${tag}>`),
    );
  }
  assert.match(
    compareSfcEquivalence(
      "<template><div v-pre>{{  raw   text  }}</div></template>\n",
      "<template><div v-pre>{{ raw text }}</div></template>\n",
      "App.vue",
    ).join("\n"),
    /#text/,
  );
  assert.match(
    compareSfcEquivalence(
      '<template><div :title="a > b" v-pre>{{  raw   text  }}</div></template>\n',
      '<template><div :title="a > b" v-pre>{{ raw text }}</div></template>\n',
      "App.vue",
    ).join("\n"),
    /#text/,
  );
  // PascalCase component names are not native whitespace-preserving elements.
  assert.deepEqual(
    compareSfcEquivalence(
      "<template><Pre>  layout   text  </Pre></template>\n",
      "<template><Pre> layout text </Pre></template>\n",
      "App.vue",
    ),
    [],
  );
});

test("glyph SFC comparator preserves parse state and SFC block contracts", () => {
  assert.match(
    compareSfcEquivalence(
      original,
      '<template>\n  <div :class="foo">a</span>\n</template>\n',
      "App.vue",
    ).join("\n"),
    /parse errors changed/,
  );
  assert.match(
    compareSfcEquivalence(
      "<template><p/></template>\n<style scoped>.a{}</style>\n",
      "<template><p/></template>\n<style>.a{}</style>\n",
      "App.vue",
    ).join("\n"),
    /styles changed/,
  );
  assert.match(
    compareSfcEquivalence(
      "<script setup>;</script>\n<template><p/></template>\n",
      "<script setup></script>\n<template><p/></template>\n",
      "App.vue",
    ).join("\n"),
    /scriptSetup block disappeared/,
  );
  assert.match(
    compareSfcEquivalence(
      "<template><!--  keep   comment  --></template>\n",
      "<template><!-- keep comment --></template>\n",
      "App.vue",
    ).join("\n"),
    /#comment/,
  );
  assert.match(
    compareSfcEquivalence(
      "<template><p /></template>\n<style scoped>.a{}</style>\n<style module>.b{}</style>\n",
      "<template><p /></template>\n<style module>.b{}</style>\n<style scoped>.a{}</style>\n",
      "App.vue",
    ).join("\n"),
    /styles changed/,
  );
  assert.match(
    compareSfcEquivalence(
      "<template><p /></template>\n<docs>  first   value  </docs>\n<docs>second</docs>\n",
      "<template><p /></template>\n<docs>second</docs>\n<docs> first value </docs>\n",
      "App.vue",
    ).join("\n"),
    /customBlocks changed/,
  );
  assert.deepEqual(
    compareSfcEquivalence(
      "<template><p /></template>\n<markdown>\n  # Heading\n\n  body\n  </markdown>\n",
      "<template><p /></template>\n<markdown>\n# Heading\n\n  body\n</markdown>\n",
      "App.vue",
    ),
    [],
  );
  assert.match(
    compareSfcEquivalence(
      "<template><p /></template>\n<markdown>\n# Heading\n\n  body\n</markdown>\n",
      "<template><p /></template>\n<markdown>\n# Heading\n\nbody\n</markdown>\n",
      "App.vue",
    ).join("\n"),
    /customBlocks changed/,
  );
});

test("glyph SFC comparator normalizes only compiler-defined presence attributes", () => {
  const style = (attr: string): string => `<style ${attr}>.a{}</style>`;
  const script = (attr: string): string => `<script ${attr}>const x=1</script>`;
  const template = (attr: string): string => `<template ${attr}><p/></template>`;
  const presenceCases = [
    ["scoped", style, "<style>.a{}</style>"],
    ["setup", script, "<script>const x=1</script>"],
    ["vapor", template, "<template><p/></template>"],
    ["vapor", script, "<script>const x=1</script>"],
    ["functional", template, "<template><p/></template>"],
  ] as const;
  for (const [attr, render, absent] of presenceCases) {
    const forms = [attr, `${attr}=""`, `${attr}="${attr}"`, `${attr}="false"`];
    for (const left of forms) {
      for (const right of forms) {
        assert.deepEqual(compareSfcEquivalence(render(left), render(right), "App.vue"), []);
      }
    }
    assert.notDeepEqual(compareSfcEquivalence(render(forms[0]), absent, "App.vue"), []);
  }
  for (const [before, after] of [
    ["<style module>.a{}</style>", '<style module="theme">.a{}</style>'],
    ['<style module="first">.a{}</style>', '<style module="second">.a{}</style>'],
    ['<style lang="scss">.a{}</style>', '<style lang="less">.a{}</style>'],
    ['<style custom="first">.a{}</style>', '<style custom="second">.a{}</style>'],
    [
      '<script setup generic="T">const x=1</script>',
      '<script setup generic="U">const x=1</script>',
    ],
    [
      '<template><p/></template><style src="first.css"></style>',
      '<template><p/></template><style src="second.css"></style>',
    ],
    [
      '<template><p/></template><docs scoped="first">x</docs>',
      '<template><p/></template><docs scoped="second">x</docs>',
    ],
  ]) {
    assert.notDeepEqual(compareSfcEquivalence(before, after, "App.vue"), []);
  }
});
