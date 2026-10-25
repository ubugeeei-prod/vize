const basic = 'export const render = () => <div data-probe="original">hello</div>;\n';
export const coldFixtures = [
  { id: "ordinary-module", source: "export const answer = 42;\n" },
  { id: "jsx-looking-text", source: '// <Fake /> is a comment\nexport const note = "<Fake />";\n' },
  {
    id: "type-only-tsx-without-jsx",
    typescript: true,
    source:
      'import type { ReactNode } from "react";\ntype Text = ReactNode;\nexport const note: Text = "<Fake />";\n',
  },
  {
    id: "hashbang-and-directives-without-jsx",
    source: '#!/usr/bin/env node\n"use client";\nexport const answer = 42;\n',
  },
  { id: "from-ast-jsx-removed", fromAst: "remove", source: basic },
  { id: "pre-hook-jsx-removed", pre: "remove", source: basic },
];
export const positiveFixtures = [
  {
    id: "vize-before-module-transform",
    source: basic,
    order: "first",
    html: '<div data-probe="original">hello</div>',
  },
  {
    id: "vize-after-module-transform",
    source: basic,
    order: "last",
    html: '<div data-probe="original">hello</div>',
  },
  {
    id: "preceding-pre-edits-current-ast",
    source: basic,
    pre: "edit",
    ordinaryEdit: true,
    html: '<div data-probe="pre-edited">hello</div>',
    exported: { preAdded: 19 },
  },
  {
    id: "transform-from-ast-stale-original-code",
    source: basic,
    fromAst: "edit",
    html: '<div data-probe="ast-edited">hello</div>',
  },
  {
    id: "transform-from-ast-without-cloning",
    source: basic,
    fromAst: "edit",
    cloneInputAst: false,
    html: '<div data-probe="ast-edited">hello</div>',
  },
  {
    id: "preceding-pre-introduces-jsx",
    source: "export const existing = 1;\n",
    pre: "create",
    html: "<p>created-before-native</p>",
    exported: { existing: 1 },
  },
  {
    id: "typed-parameter-and-type-only-import",
    typescript: true,
    source:
      'import type { ReactNode } from "react";\ntype Props = { label: string; extra?: ReactNode };\nexport const render = (props: Props) => <p>{props.label}</p>;\n',
    renderProps: { label: "typed" },
    html: "<p>typed</p>",
  },
  {
    id: "type-only-react-specifier",
    typescript: true,
    source:
      'import { type ReactNode } from "react";\ntype Props = { label: string; extra?: ReactNode };\nexport const render = (props: Props) => <p>{props.label}</p>;\n',
    renderProps: { label: "specifier" },
    html: "<p>specifier</p>",
  },
  {
    id: "ordinary-decorator-transform",
    decorators: true,
    source:
      'function mark(value) { value.observed = "decorated"; return value; }\n@mark\nclass Decorated {}\nexport const render = () => <p>{Decorated.observed}</p>;\n',
    html: "<p>decorated</p>",
  },
  {
    id: "ordinary-class-property-transform",
    classProperties: true,
    source:
      "class Holder { field = 42; }\nexport const render = () => <p>{new Holder().field}</p>;\n",
    html: "<p>42</p>",
  },
  {
    id: "unicode-crlf-comments-directive-hashbang",
    source:
      '#!/usr/bin/env node\r\n"use client";\r\n// authored-leading 🧭\r\nexport const render = () => <p>🧭 café</p>;\r\n// authored-trailing\r\n',
    comments: true,
    prologue: true,
    html: "<p>🧭 café</p>",
  },
  {
    id: "comments-disabled",
    source:
      "// authored-leading\nexport const render = () => <p>plain</p>;\n// authored-trailing\n",
    comments: false,
    html: "<p>plain</p>",
  },
  {
    id: "upstream-map-unicode-crlf",
    source:
      'const compass = "🧭"; const authoredToken = "café";\r\nexport const render = () => <p>{compass + " " + authoredToken}</p>;\r\n',
    upstream: true,
    html: "<p>🧭 café</p>",
  },
  {
    id: "foreign-value-runtime-refuses",
    source: 'import React from "react";\nexport const render = () => <p>foreign</p>;\n',
    error: "Vize compiles Vue JSX; scope this plugin to Vue files using Babel overrides",
  },
  {
    id: "late-pre-jsx-refuses",
    source: basic,
    latePre: true,
    error:
      "JSX was introduced after the native phase; run JSX-producing transforms in an earlier Babel pass",
  },
];

const foreignRuntime =
  "Vize compiles Vue JSX; scope this plugin to Vue files using Babel overrides";
coldFixtures.push(
  {
    id: "automatic-runtime-before-vize-refuses-cold",
    source: basic,
    reactTransform: true,
    order: "last",
    error: foreignRuntime,
  },
  {
    id: "automatic-runtime-after-vize-refuses-cold",
    source: basic,
    reactTransform: true,
    order: "first",
    error: foreignRuntime,
  },
  {
    id: "non-vue-file-uses-actual-overridden-runtime",
    source: basic,
    overrideScope: "react",
    filenameSuffix: ".react.jsx",
  },
);
for (const order of ["first", "last"])
  coldFixtures.push({
    id: "late-post-introduces-jsx-" + order,
    source: "export const existing = 1;\n",
    order,
    latePost: true,
    error:
      "JSX was introduced after the native phase; run JSX-producing transforms in an earlier Babel pass",
  });
positiveFixtures.push(
  {
    id: "later-post-jsx-refuses-after-real-native",
    source: basic,
    latePost: true,
    order: "first",
    error:
      "JSX was introduced after the native phase; run JSX-producing transforms in an earlier Babel pass",
  },
  {
    id: "multiple-real-host-generators-refuse",
    source: basic,
    generatorOverrides: 2,
    error: "More than one plugin attempted to override codegen.",
  },
);
for (const order of ["first", "last"])
  for (const cloneInputAst of [false, true]) {
    coldFixtures.push({
      id: `real-host-generator-plain-${order}-clone-${cloneInputAst}`,
      source: 'const compass = "🧭"; export const authoredToken = "雪";\r\n',
      fromAst: "unchanged",
      order,
      cloneInputAst,
      generatorOverrides: 1,
      upstream: true,
    });
    positiveFixtures.push({
      id: `real-host-generator-jsx-${order}-clone-${cloneInputAst}`,
      source:
        'const compass = "🧭"; const authoredToken = "café";\r\nexport const render = () => <div data-probe="original">{authoredToken}</div>;\r\n',
      fromAst: "edit",
      order,
      cloneInputAst,
      upstream: true,
      generatorOverrides: 1,
      html: '<div data-probe="ast-edited">café</div>',
    });
  }
const reserved =
  'const compass = "🧭"; export const authoredToken = "雪";\r\nexport const render = () => <div title={authoredToken}>{authoredToken}</div>;\r\n';
positiveFixtures.push(
  {
    id: "deferred-uids-before-and-after-native",
    source: reserved,
    uidReservation: "split",
    modules: false,
    upstream: true,
    exported: { reservedA: "mineA", reservedB: "mineB" },
    html: '<div title="雪">雪</div>',
  },
  {
    id: "deferred-uids-both-before-native",
    source: reserved,
    uidReservation: "before",
    modules: false,
    upstream: true,
    exported: { reservedA: "mineA", reservedB: "mineB" },
    html: '<div title="雪">雪</div>',
  },
  {
    id: "vue-file-uses-only-overridden-vize",
    source: basic,
    overrideScope: "vue",
    filenameSuffix: ".vue.jsx",
    html: '<div data-probe="original">hello</div>',
  },
);
