import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs";
import { createHash } from "node:crypto";
import { loadBinding } from "./native.ts";
import type { SfcBlock } from "./model.ts";

import { extractSfcBlocks, formatBlockLabel, getDiagnosticBlock } from "./sfc-blocks.ts";

void test("SFC block extraction classifies common Vue block types", () => {
  const blocks = extractSfcBlocks(
    `<script setup lang="ts">\nconst count = 1\n</script>\n<template>\n  <div>{{ count }}</div>\n</template>\n<style scoped>\n.foo {}\n</style>\n<i18n>\n{}\n</i18n>\n`,
  );

  assert.deepEqual(
    blocks.map((block) => formatBlockLabel(block)),
    ["<script setup>", "<template>", "<style>", "<i18n>"],
  );
});

void test("SFC block extraction keeps quoted angle brackets inside attributes", () => {
  const blocks = extractSfcBlocks(
    `<script setup lang="ts" generic="T extends Record<string, unknown>">\nconst count = 1\n</script>\n`,
  );

  assert.equal(blocks[0]?.kind, "script-setup");
  assert.equal(blocks[0]?.content.trim(), "const count = 1");
});

void test("SFC block labels identify the diagnostic owner", () => {
  const blocks = extractSfcBlocks(
    `<script setup lang="ts">\nconst count = 1\n</script>\n<template>\n  <div>{{ count }}</div>\n</template>\n`,
  );

  assert.equal(
    formatBlockLabel(
      getDiagnosticBlock(
        {
          rule: "vize/vue/mock",
          severity: "error",
          message: "Mock error. Detail: extra context",
          location: {
            start: { line: 5, column: 3, offset: 0 },
            end: { line: 5, column: 8, offset: 0 },
          },
          help: null,
        },
        blocks,
      ),
    ),
    "<template>",
  );
});

// Independent authored boundaries: expected bodies are supplied as fixture data,
// rather than discovered by the production extractor or its native parser.
function positionAfter(text: string) {
  const lines = text.split("\n");
  return { line: lines.length, column: lines[lines.length - 1]!.length + 1 };
}

const structuralCases = [
  {
    name: "nested v-if and later input",
    prefix: "",
    body: '<template v-if="ok"><b>yes</b></template><input />',
  },
  {
    name: "nested sibling and self-closing templates",
    prefix: "",
    body: '<template v-if="ok"/><template v-else><template><b/></template></template><input />',
  },
  {
    name: "outer comment with fake blocks",
    prefix: "<!-- <template></template><script>fake</script> -->\n",
    body: "<div/>",
  },
  {
    name: "inner comment and quoted attribute",
    prefix: "",
    body: '<!-- </template><template> --><div title="</template>"><input /></div>',
  },
  {
    name: "interpolation string and comment",
    prefix: "",
    body: '{{ "</template>" /* <template> */ }}<input />',
  },
  {
    name: "raw text inside template",
    prefix: "",
    body: '<script>const text = "</template>";</script><style>.x::after { content:"</template>"; }</style><input />',
  },
  {
    name: "CRLF and non-BMP anchors",
    prefix: "<!--😀-->\r\n",
    body: '\r\n<template v-if="ok">😀</template>\r\n<input />\r\n',
  },
  {
    name: "mixed endings and bare CR",
    prefix: "<!--é😀-->\r",
    body: "\r<input />\n<template/>\r\n",
  },
  {
    name: "multiline opening attribute with non-BMP text",
    prefix: "",
    open: '<template\r\n data-note="😀">',
    body: "\r\n<input />\r\n",
  },
  {
    name: "BOM and combining text",
    prefix: "\uFEFF<!--e\u0301😀-->",
    body: '<div title="é😀">e\u0301😀</div>',
  },
];
for (const { name, prefix, body, open = "<template>" } of structuralCases) {
  void test(`native SFC descriptor preserves ${name}`, () => {
    const before = `${prefix}${open}`;
    const source = `${before}${body}</template>`;
    const expected: SfcBlock[] = [
      {
        kind: "template",
        name: "template",
        content: body,
        contentStart: positionAfter(before),
        contentEnd: positionAfter(before + body),
      },
    ];
    assert.deepEqual(extractSfcBlocks(source), expected);
    const descriptor = loadBinding().parseSfc(source);
    const bytePosition = (text: string) => {
      const lines = text.split("\n");
      return {
        line: lines.length,
        column: Buffer.byteLength(lines[lines.length - 1]!, "utf8") + 1,
      };
    };
    assert.deepEqual(descriptor.template?.loc, {
      start: Buffer.byteLength(before),
      end: Buffer.byteLength(before + body),
      tagStart: Buffer.byteLength(prefix),
      tagEnd: Buffer.byteLength(source),
      startLine: bytePosition(before).line,
      startColumn: bytePosition(before).column,
      endLine: bytePosition(before + body).line,
      endColumn: bytePosition(before + body).column,
    });
  });
}

void test("native SFC geometry retains all five block families in physical order", () => {
  const parts = [
    {
      open: '<style scoped module="classes">',
      body: '.x::after { content:"</template>"; }',
      close: "</style>",
      kind: "style",
      name: "style",
    },
    {
      open: '<script setup lang="ts" generic="T extends Record<string, unknown>">',
      body: 'const close = "</script>"; const text = "</template>";\n// <template>\n',
      close: "</script>",
      kind: "script-setup",
      name: "script",
    },
    {
      open: "<template>",
      body: '<template v-if="ok"><input/></template><input/>',
      close: "</template>",
      kind: "template",
      name: "template",
    },
    {
      open: "<docs>",
      body: "<template>literal</template>",
      close: "</docs>",
      kind: "custom",
      name: "docs",
    },
    { open: "<script>", body: "", close: "</script>", kind: "script", name: "script" },
  ] as const;
  let source = "";
  const expected = parts.map(({ open, body, close, kind, name }) => {
    const before = source + open;
    const block = {
      kind,
      name,
      content: body,
      contentStart: positionAfter(before),
      contentEnd: positionAfter(before + body),
    };
    source = before + body + close + "\r\n";
    return block;
  });
  assert.deepEqual(extractSfcBlocks(source), expected);
});

void test("self-closing and empty scripts keep exact zero-length native content anchors", () => {
  assert.deepEqual(extractSfcBlocks("<script setup/><script></script><template/>"), [
    {
      kind: "script-setup",
      name: "script",
      content: "",
      contentStart: { line: 1, column: 16 },
      contentEnd: { line: 1, column: 16 },
    },
    {
      kind: "script",
      name: "script",
      content: "",
      contentStart: { line: 1, column: 24 },
      contentEnd: { line: 1, column: 24 },
    },
    {
      kind: "template",
      name: "template",
      content: "",
      contentStart: { line: 1, column: 44 },
      contentEnd: { line: 1, column: 44 },
    },
  ]);
});

void test("an unclosed outer comment cannot manufacture a top-level SFC block", () => {
  assert.deepEqual(extractSfcBlocks("<!-- <template><input/></template>"), []);
});

const malformed = [
  ["<template><input/>", "Malformed <template> block: the closing tag is missing."],
  [
    '<template><template v-if="ok"><b/></template>',
    "Malformed <template> block: the closing tag is missing.",
  ],
  ["<template/><template/>", "SFC can only contain one <template> block"],
  ['<template title="unfinished', "Malformed <template> block: the opening tag is incomplete."],
] as const;
for (const [source, message] of malformed) {
  void test(`invalid native SFC refuses guessed ownership: ${source}`, () => {
    assert.throws(
      () => loadBinding().parseSfc(source),
      (error: unknown) => {
        assert.ok(error instanceof Error);
        assert.equal((error as Error & { code: string }).code, "InvalidSfc");
        assert.equal(error.message, message);
        return true;
      },
    );
    assert.deepEqual(extractSfcBlocks(source), []);
  });
}

const originalPacket = JSON.parse(
  fs.readFileSync(
    new URL("./test-support/original-sfc-host-7004-7005.json", import.meta.url),
    "utf8",
  ),
) as {
  cases: {
    filename: string;
    source: string;
    sha256: string;
    rule: string;
    anchors: { line: number; column: number; length: number }[];
  }[];
};
for (const entry of originalPacket.cases) {
  void test(`whole original ${entry.filename} findings retain their root template owner`, () => {
    assert.equal(createHash("sha256").update(entry.source).digest("hex"), entry.sha256);
    const binding = loadBinding();
    const actual = binding.lintPatinaSfc(entry.source, {
      filename: entry.filename,
      preset: "incremental",
      enabledRules: [entry.rule],
    }).diagnostics;
    assert.equal(actual.length, entry.anchors.length);
    const blocks = extractSfcBlocks(entry.source);
    assert.deepEqual(
      actual.map((diagnostic) => ({
        rule: diagnostic.rule,
        line: diagnostic.location.start.line,
        column: diagnostic.location.start.column,
        length: diagnostic.location.end.offset - diagnostic.location.start.offset,
        owner: formatBlockLabel(getDiagnosticBlock(diagnostic, blocks)),
      })),
      entry.anchors.map((anchor) => ({ rule: entry.rule, ...anchor, owner: "<template>" })),
    );
  });
}

void test("parseSfc retains the complete descriptor and all content/tag byte geometry", () => {
  const source =
    '<script setup></script>\r\n<template><template v-if="ok"/><input/></template><style scoped module="classes">.x{}</style><docs>note</docs><script>const a=1;</script>';
  const expected = {
    filename: "anonymous.vue",
    source:
      '<script setup></script>\r\n<template><template v-if="ok"/><input/></template><style scoped module="classes">.x{}</style><docs>note</docs><script>const a=1;</script>',
    scriptSetup: {
      content: "",
      loc: {
        start: 14,
        end: 14,
        tagStart: 0,
        tagEnd: 23,
        startLine: 1,
        startColumn: 15,
        endLine: 1,
        endColumn: 15,
      },
      lang: null,
      setup: true,
    },
    template: {
      content: '<template v-if="ok"/><input/>',
      loc: {
        start: 35,
        end: 64,
        tagStart: 25,
        tagEnd: 75,
        startLine: 2,
        startColumn: 11,
        endLine: 2,
        endColumn: 40,
      },
      lang: null,
    },
    styles: [
      {
        content: ".x{}",
        loc: {
          start: 106,
          end: 110,
          tagStart: 75,
          tagEnd: 118,
          startLine: 2,
          startColumn: 82,
          endLine: 2,
          endColumn: 86,
        },
        lang: null,
        scoped: true,
        module: "classes",
      },
    ],
    customBlocks: [
      {
        content: "note",
        loc: {
          start: 124,
          end: 128,
          tagStart: 118,
          tagEnd: 135,
          startLine: 2,
          startColumn: 100,
          endLine: 2,
          endColumn: 104,
        },
        type: "docs",
      },
    ],
    script: {
      content: "const a=1;",
      loc: {
        start: 143,
        end: 153,
        tagStart: 135,
        tagEnd: 162,
        startLine: 2,
        startColumn: 119,
        endLine: 2,
        endColumn: 129,
      },
      lang: null,
      setup: false,
    },
  };
  assert.deepEqual(loadBinding().parseSfc(source), expected);
});

void test("parseSfc preserves the dedicated InvalidSfc code and complete parser reason", () => {
  assert.throws(() => loadBinding().parseSfc("<template><input/>"), {
    code: "InvalidSfc",
    message: "Malformed <template> block: the closing tag is missing.",
  });
});
