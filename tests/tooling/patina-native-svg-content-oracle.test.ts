import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import { runInNewContext } from "node:vm";

const fixtureRoot = new URL("../_fixtures/differential/lint/native-svg-content/", import.meta.url);
const read = (name: string) => readFileSync(new URL(name, fixtureRoot), "utf8");
const cases = JSON.parse(read("cases.json")) as {
  id: string;
  template: string;
  component: boolean;
  eslintDefault: boolean;
}[];
const provenance = JSON.parse(read("provenance.json")) as {
  licenseSha256: string;
  files: { file: string; sha256: string; eslintDefaultLocation: [number, number] }[];
};
const fromBench = createRequire(
  new URL("../../tools/benchmarks/scripts/package.json", import.meta.url),
);
const fromUi = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const fromVue = createRequire(fromUi.resolve("vue/package.json"));
const vue = fromVue("vue");
const compiler = fromVue("@vue/compiler-dom");
const sfc = fromVue("@vue/compiler-sfc");
const { ESLint } = fromBench("eslint");
const rule = "vue/no-v-text-v-html-on-component";
const hash = (source: string) => createHash("sha256").update(source).digest("hex");

function reference(ignoreElementNamespaces: boolean) {
  return new ESLint({
    overrideConfigFile: true,
    ignore: false,
    overrideConfig: [
      {
        files: ["**/*.vue"],
        languageOptions: {
          parser: fromBench("vue-eslint-parser"),
          parserOptions: {
            parser: fromBench("@typescript-eslint/parser"),
            ecmaVersion: "latest",
            sourceType: "module",
          },
        },
        plugins: { vue: fromBench("eslint-plugin-vue") },
        rules: { [rule]: ["error", { ignoreElementNamespaces }] },
      },
    ],
  });
}

type Element = {
  type: number;
  tag?: string;
  tagType?: number;
  props?: { type: number; name?: string }[];
  children?: Element[];
  loc: { source: string };
};

function contentElements(template: string): Element[] {
  const nodes: Element[] = [];
  const visit = (node: Element) => {
    if (
      node.type === 1 &&
      node.props?.some((prop) => prop.type === 7 && /^(html|text)$/.test(prop.name ?? ""))
    ) {
      nodes.push(node);
    }
    for (const child of node.children ?? []) visit(child);
  };
  visit(compiler.parse(template));
  return nodes;
}

void test("native content classification follows pinned Vue, with the ESLint namespace distinction retained", async () => {
  for (const name of ["vue", "@vue/compiler-dom", "@vue/compiler-sfc"]) {
    assert.equal(fromVue(`${name}/package.json`).version, "3.5.35");
  }
  assert.equal(fromBench("eslint-plugin-vue/package.json").version, "10.9.2");
  assert.equal(fromBench("vue-eslint-parser/package.json").version, "10.4.1");
  const defaultLint = reference(false);
  const nativeLint = reference(true);
  for (const fixture of cases) {
    const nodes = contentElements(fixture.template);
    assert.equal(nodes.length, 1, fixture.id);
    assert.equal(nodes[0].tagType, fixture.component ? 1 : 0, fixture.id);
    for (const [lint, expected] of [
      [defaultLint, fixture.eslintDefault],
      [nativeLint, fixture.component],
    ] as const) {
      const [result] = await lint.lintText(`<template>${fixture.template}</template>`, {
        filePath: `${fixture.id}.vue`,
      });
      assert.equal(result.messages.length, Number(expected), fixture.id);
      for (const message of result.messages) assert.equal(message.ruleId, rule, fixture.id);
    }
  }
});

void test("the complete pinned npmx sources compile native g elements and retain the original upstream diagnostics", async () => {
  assert.equal(hash(read("LICENSE")), provenance.licenseSha256);
  for (const fixture of provenance.files) {
    const source = read(fixture.file);
    assert.equal(hash(source), fixture.sha256, fixture.file);
    const parsed = sfc.parse(source);
    assert.deepEqual(parsed.errors, [], fixture.file);
    const template = parsed.descriptor.template.content;
    const nodes = contentElements(template);
    assert.deepEqual(
      nodes.map((node) => [node.tag, node.tagType]),
      [["g", 0]],
      fixture.file,
    );
    const errors: unknown[] = [];
    const compiled = compiler.compile(template, {
      mode: "module",
      expressionPlugins: ["typescript"],
      onError: (error: unknown) => errors.push(error),
    });
    assert.deepEqual(errors, [], fixture.file);
    assert.equal(compiled.code.includes('resolveComponent("g")'), false, fixture.file);
    const [defaultResult] = await reference(false).lintText(source, { filePath: fixture.file });
    assert.equal(defaultResult.messages.length, 1, fixture.file);
    assert.equal(defaultResult.messages[0].ruleId, rule, fixture.file);
    assert.deepEqual(
      [defaultResult.messages[0].line, defaultResult.messages[0].column],
      fixture.eslintDefaultLocation,
      fixture.file,
    );
    const [nativeResult] = await reference(true).lintText(source, { filePath: fixture.file });
    assert.deepEqual(nativeResult.messages, [], fixture.file);
  }
});

type Host = { tag: string; parent: Host | null; children: Host[]; props: Record<string, unknown> };
const host = (tag: string): Host => ({ tag, parent: null, children: [], props: {} });
const render = (template: string) =>
  runInNewContext(
    `(function () { ${compiler.compile(template, { mode: "function", prefixIdentifiers: true }).code} })()`,
    { Vue: vue },
  );

void test("Vue renders the original chart slot and root fragment in the receiving SVG namespace", () => {
  const creations: { tag: string; namespace?: string }[] = [];
  const writes: { tag: string; key: string; value: unknown; namespace?: string }[] = [];
  const renderer = vue.createRenderer({
    insert(child: Host, parent: Host, anchor?: Host) {
      child.parent = parent;
      const index = anchor ? parent.children.indexOf(anchor) : -1;
      if (index < 0) parent.children.push(child);
      else parent.children.splice(index, 0, child);
    },
    remove(child: Host) {
      if (child.parent) child.parent.children.splice(child.parent.children.indexOf(child), 1);
    },
    createElement(tag: string, namespace?: string) {
      creations.push({ tag, namespace });
      return host(tag);
    },
    createText: () => host("#text"),
    createComment: () => host("#comment"),
    setText(node: Host, text: string) {
      node.props.text = text;
    },
    setElementText(node: Host, text: string) {
      node.props.text = text;
    },
    parentNode: (node: Host) => node.parent,
    nextSibling(node: Host) {
      const children = node.parent?.children ?? [];
      return children[children.indexOf(node) + 1] ?? null;
    },
    patchProp(node: Host, key: string, _previous: unknown, value: unknown, namespace?: string) {
      node.props[key] = value;
      writes.push({ tag: node.tag, key, value, namespace });
    },
  });
  const facetTemplate = sfc.parse(read("FacetBarChart.vue")).descriptor.template.content;
  const originalGroup = contentElements(facetTemplate)[0].loc.source;
  const svgTemplate = sfc.parse(read("TimelineChartDepSizeSvgSlot.vue")).descriptor.template
    .content;
  const watermark = '<path d="M0 0" />';
  const SvgSlot = {
    props: ["svg", "watermark", "activeVersionPlot", "colors", "pauseAnimations", "isCopyingPng"],
    render: render(svgTemplate),
  };
  const Chart = {
    setup:
      (_: unknown, { slots }: { slots: { svg(props: unknown): unknown } }) =>
      () =>
        vue.h("svg", slots.svg({ svg: { isPrintingSvg: true } })),
  };
  for (const template of [
    `<Chart><template #svg="{ svg }">${originalGroup}</template></Chart>`,
    '<svg><SvgSlot :svg="svg" :watermark="watermark" :active-version-plot="null" :colors="{}" :pause-animations="false" /></svg>',
  ]) {
    creations.length = writes.length = 0;
    let registeredGroupCalls = 0;
    const app = renderer.createApp({
      data: () => ({
        svg: { isPrintingSvg: true },
        watermark,
        isCopyingPng: false,
        watermarkColors: {},
        drawSmallNpmxLogoAndTaglineWatermark: () => watermark,
      }),
      render: render(template),
    });
    app.config.globalProperties.$t = (value: string) => value;
    app.component("Chart", Chart);
    app.component("SvgSlot", SvgSlot);
    app.component("g", {
      render() {
        registeredGroupCalls++;
        return vue.h("div");
      },
    });
    const root = host("root");
    app.mount(root);
    assert.equal(registeredGroupCalls, 0);
    assert.ok(creations.some(({ tag }) => tag === "g"));
    assert.ok(creations.every(({ namespace }) => namespace === "svg"));
    assert.deepEqual(
      writes.filter(({ key }) => key === "innerHTML"),
      [{ tag: "g", key: "innerHTML", value: watermark, namespace: "svg" }],
    );
    app.unmount();
    assert.deepEqual(root.children, []);
  }
  let componentCalls = 0;
  const app = renderer.createApp({
    data: () => ({ watermark }),
    render: render('<widget v-html="watermark" />'),
  });
  app.component("widget", {
    render() {
      componentCalls++;
      return vue.h("div");
    },
  });
  const root = host("root");
  app.mount(root);
  assert.equal(componentCalls, 1, "unknown lowercase tags still resolve registered components");
  app.unmount();
  assert.deepEqual(root.children, []);
});
