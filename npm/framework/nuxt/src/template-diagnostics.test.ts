import assert from "node:assert/strict";
import { test } from "node:test";

import {
  captureNuxtTemplateDiagnostics,
  createNuxtTemplateDiagnosticsBridge,
  resolveServerComponentTags,
  setupNuxtTemplateDiagnostics,
  toNuxtTemplateRoot,
  type NuxtTemplateFinding,
} from "./template-diagnostics.ts";

const at = { line: 3, column: 7, offset: 41 };
const loc = { start: at, end: at, source: "" };

void test("captures the last Nuxt template diagnostics transform only when the option is on", () => {
  const nodeTransform = () => {};
  const other = () => {};
  const options = {
    experimental: { templateDiagnostics: true },
    vite: { vue: { template: { compilerOptions: { nodeTransforms: [nodeTransform, other] } } } },
  };
  assert.equal(captureNuxtTemplateDiagnostics(options), nodeTransform);
  assert.equal(
    captureNuxtTemplateDiagnostics({ ...options, experimental: { templateDiagnostics: false } }),
    undefined,
  );
  assert.equal(
    captureNuxtTemplateDiagnostics({ experimental: { templateDiagnostics: true } }),
    undefined,
  );
});

void test("server component tags follow Nuxt's naming and skip names a client component shares", () => {
  assert.deepEqual(
    resolveServerComponentTags([
      { mode: "server", pascalName: "Card", kebabName: "card" },
      { mode: "server", pascalName: "Pair", kebabName: "pair" },
      { mode: "client", pascalName: "Pair", kebabName: "pair" },
      { mode: "all", pascalName: "Plain", kebabName: "plain" },
    ]),
    ["Card", "card", "LazyCard", "lazy-card"],
  );
});

void test("each finding becomes the element Nuxt's transform reads it from", () => {
  const findings: NuxtTemplateFinding[] = [
    { kind: "link", tag: "NuxtLink", ...at, attribute: "to", to: "/missing" },
    { kind: "event", tag: "ServerCard", ...at, directive: "@click.stop" },
    { kind: "prop", tag: "ServerCard", ...at, prop: "format", value: "a function" },
    {
      kind: "island",
      tag: "NuxtIsland",
      ...at,
      prop: "store",
      value: "a class instance",
      islandName: "Chart",
      islandProp: "props",
    },
  ];
  const root = toNuxtTemplateRoot("<template />", findings);
  assert.equal(root.type, 0);
  assert.equal(root.source, "<template />");
  assert.deepEqual(
    root.children.map((child) => [child.type, child.tag, child.tagType]),
    [
      [1, "NuxtLink", 1],
      [1, "ServerCard", 1],
      [1, "ServerCard", 1],
      [1, "NuxtIsland", 1],
    ],
  );
  const [link, event, prop, island] = root.children.map((child) => child.props);
  assert.deepEqual(link, [
    { type: 6, name: "to", value: { type: 2, content: "/missing", loc }, loc },
  ]);
  assert.deepEqual(event, [{ type: 7, name: "model", rawName: "@click.stop", modifiers: [], loc }]);
  assert.deepEqual(prop, [
    {
      type: 7,
      name: "bind",
      rawName: ":format",
      arg: { type: 4, content: "format", isStatic: true },
      exp: { type: 4, content: "", ast: { type: "ArrowFunctionExpression" }, loc },
      modifiers: [],
      loc,
    },
  ]);
  assert.deepEqual(island, [
    { type: 6, name: "name", value: { type: 2, content: "Chart", loc }, loc },
    {
      type: 7,
      name: "bind",
      rawName: ":props",
      arg: { type: 4, content: "props", isStatic: true },
      exp: {
        type: 4,
        content: "",
        ast: {
          type: "ObjectExpression",
          properties: [
            {
              type: "ObjectProperty",
              computed: false,
              key: { type: "Identifier", name: "store" },
              value: { type: "NewExpression" },
            },
          ],
        },
      },
      modifiers: [],
      loc,
    },
  ]);
});

function bridgeFor(source: string, serverTags: string[] = []) {
  const calls: Array<{ root: ReturnType<typeof toNuxtTemplateRoot>; context: unknown }> = [];
  const searched: string[] = [];
  let exits = 0;
  const bridge = createNuxtTemplateDiagnosticsBridge({
    getTransform: () => (root, context) => {
      calls.push({ root: root as ReturnType<typeof toNuxtTemplateRoot>, context });
      return () => exits++;
    },
    getServerTags: () => serverTags,
    loadFinder: async () => (text) => {
      searched.push(text);
      return [{ kind: "link", tag: "NuxtLink", ...at, attribute: "to", to: "/a" }];
    },
    readSource: () => source,
  });
  return { bridge, calls, searched, exits: () => exits };
}

void test("each Vize-compiled SFC compile reaches Nuxt's transform once per environment", async () => {
  const { bridge, calls, searched, exits } = bridgeFor('<template><NuxtLink to="/a" /></template>');
  await bridge.transform.handler("", "/app/pages/index.vue?vue&vize");
  await bridge.transform.handler("", "/app/pages/index.vue?vue&vize", { ssr: true });
  assert.equal(searched.length, 2);
  assert.equal(exits(), 2);
  assert.deepEqual(
    calls.map(({ context }) => context),
    [
      { filename: "/app/pages/index.vue", ssr: false, inSSR: false, bindingMetadata: {} },
      { filename: "/app/pages/index.vue", ssr: true, inSSR: true, bindingMetadata: {} },
    ],
  );
  assert.equal(calls[0]!.root.children.length, 1);
});

void test("a template naming no checked tag skips the native read but still retires warnings", async () => {
  const { bridge, calls, searched } = bridgeFor("<template><div /></template>", ["ServerCard"]);
  await bridge.transform.handler("", "/app/pages/index.vue?vue&vize");
  assert.equal(searched.length, 0);
  assert.equal(calls.length, 1);
  assert.deepEqual(calls[0]!.root.children, []);
});

void test("modules outside the app and host-compiled SFCs are left alone", async () => {
  const { bridge, calls } = bridgeFor('<template><NuxtLink to="/a" /></template>');
  await bridge.transform.handler("", "/app/node_modules/ui/Button.vue?vue&vize");
  await bridge.transform.handler("", "/app/pages/index.vue?vue&type=template");
  await bridge.transform.handler("", "/app/pages/index.vue");
  assert.equal(calls.length, 0);
});

void test("setup adds the bridge only when Nuxt runs its template diagnostics", () => {
  const hooks: string[] = [];
  const nuxt = (templateDiagnostics?: boolean) => ({
    hook: (name: string) => void hooks.push(name),
    options: { experimental: { templateDiagnostics } },
  });
  const off: unknown[] = [];
  setupNuxtTemplateDiagnostics(nuxt(undefined), off);
  assert.deepEqual(off, []);
  const on: unknown[] = [];
  setupNuxtTemplateDiagnostics(nuxt(true), on);
  assert.equal((on[0] as { name: string }).name, "vizejs:nuxt-template-diagnostics");
  assert.deepEqual(hooks, ["modules:done"]);
});
