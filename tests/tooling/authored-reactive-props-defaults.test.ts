import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { compileFunction } from "node:vm";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const fromDevtools = createRequire(path.join(root, "npm/devtools/package.json"));
const vue = fromDevtools("vue");
const compiler = fromDevtools("@vue/compiler-sfc");
const ts = fromDevtools("typescript");
const gallery = "npm/builder/vite-musea/gallery/components/";
type Values = Record<string, any>;

// Execute the authored setup only: child UI, icons and the two non-Vue helpers
// are isolated here. This test makes no Vize/native compiler or template claim.
function setupComponent(relative: string, previewInputs: Values[] = []) {
  const filename = path.join(root, relative);
  const parsed = compiler.parse(readFileSync(filename, "utf8"), { filename });
  assert.deepEqual(parsed.errors, []);
  const script = compiler.compileScript(parsed.descriptor, {
    id: relative,
    genDefaultAs: "__authored_component",
  });
  const source = ts.createSourceFile(
    filename + ".ts",
    script.content,
    ts.ScriptTarget.Latest,
    true,
  );
  const names: string[] = [];
  const values: unknown[] = [];
  const stubs: Record<string, Values> = {
    "@mdi/js": { mdiChevronUp: "up", mdiChevronDown: "down" },
    "../composables/useActions": {
      useActions: () => ({ events: vue.ref([]), clear: () => undefined }),
    },
    "../../../src/tokens/preview.js": {
      resolveTokenPreview: (input: Values) => {
        previewInputs.push(input);
        return { kind: "color", value: input.token.value };
      },
    },
  };
  const children = new Set([
    "./HighlightedCode.vue",
    "./MdiIcon.vue",
    "./SpacingPreview.vue",
    "./TypographyPreview.vue",
    "./TokenPreview.vue",
    "./TokenCard.vue",
  ]);
  for (const statement of source.statements) {
    if (!ts.isImportDeclaration(statement)) continue;
    assert(ts.isStringLiteral(statement.moduleSpecifier));
    const module = statement.moduleSpecifier.text;
    const clause = statement.importClause;
    assert(clause);
    if (clause.isTypeOnly) continue;
    const bind = (local: string, imported: string) => {
      const dependency = module === "vue" ? vue : stubs[module];
      assert(
        dependency && Object.hasOwn(dependency, imported),
        `${relative}: ${module}/${imported}`,
      );
      names.push(local);
      values.push(dependency[imported]);
    };
    if (clause.name) {
      assert(children.has(module), `unexpected authored child import: ${module}`);
      names.push(clause.name.text);
      values.push({ render: () => null });
    }
    if (clause.namedBindings) {
      assert(ts.isNamedImports(clause.namedBindings));
      for (const specifier of clause.namedBindings.elements) {
        if (!specifier.isTypeOnly)
          bind(specifier.name.text, (specifier.propertyName ?? specifier.name).text);
      }
    }
  }
  const body = ts.factory.updateSourceFile(
    source,
    source.statements.filter((statement: any) => !ts.isImportDeclaration(statement)),
  );
  const output = ts.transpileModule(ts.createPrinter().printFile(body), {
    fileName: filename + ".ts",
    reportDiagnostics: true,
    compilerOptions: { target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext },
  });
  assert.deepEqual(output.diagnostics, []);
  return compileFunction(output.outputText + "\nreturn __authored_component;", names)(...values);
}

type Host = { parent: Host | null; children: Host[]; text: string };
const host = (text = ""): Host => ({ parent: null, children: [], text });
function mounted(component: Values, initial: Values) {
  const container = host();
  const renderer = vue.createRenderer({
    createElement: () => host(),
    createText: host,
    createComment: host,
    setText: (node: Host, text: string) => {
      node.text = text;
    },
    setElementText: (node: Host, text: string) => {
      node.text = text;
    },
    parentNode: (node: Host) => node.parent,
    nextSibling: (node: Host) => {
      const siblings = node.parent?.children ?? [];
      return siblings[siblings.indexOf(node) + 1] ?? null;
    },
    patchProp: () => undefined,
    insert: (node: Host, parent: Host, anchor: Host | null) => {
      if (node.parent) node.parent.children.splice(node.parent.children.indexOf(node), 1);
      node.parent = parent;
      const index = anchor ? parent.children.indexOf(anchor) : -1;
      parent.children.splice(index < 0 ? parent.children.length : index, 0, node);
    },
    remove: (node: Host) => {
      assert(node.parent);
      node.parent.children.splice(node.parent.children.indexOf(node), 1);
      node.parent = null;
    },
  });
  let props: Values | undefined;
  let state: Values | undefined;
  const probe = {
    ...component,
    setup(authoredProps: Values, context: Values) {
      props = authoredProps;
      state = component.setup(authoredProps, context);
      return state;
    },
    render: () => vue.h("setup-probe"),
  };
  renderer.render(vue.h(probe, initial), container);
  assert(props && state);
  return {
    props,
    state,
    async replace(next: Values) {
      renderer.render(vue.h(probe, next), container);
      await vue.nextTick();
    },
    unmount: () => renderer.render(null, container),
  };
}

function distinctDefault<T>(first: Values, second: Values, key: string, empty: T): T {
  assert.deepEqual(first[key], empty);
  assert.deepEqual(second[key], empty);
  assert.notEqual(first[key], second[key], `${key} default must belong to one instance`);
  return first[key];
}

test("authored ActionsPanel retains fresh array defaults and reactive capture settings", async (t) => {
  t.diagnostic(`Vue ${vue.version}; compiler-sfc ${compiler.version}; TypeScript ${ts.version}`);
  assert.equal(compiler.version, vue.version);
  const component = setupComponent(gallery + "ActionsPanel.vue");
  const first = mounted(component, {});
  const second = mounted(component, {});
  try {
    const defaults = distinctDefault<string[]>(first.props, second.props, "captureEvents", []);
    defaults.push("click");
    assert.deepEqual(second.props.captureEvents, []);
    assert.equal(first.state.captureLabel.value, "standard capture");
    await first.replace({ captureEvents: ["mousemove"] });
    assert.equal(first.state.tracksMousemove.value, true);
    assert.equal(first.state.captureLabel.value, "mousemove enabled");
    await first.replace({});
    assert.equal(first.props.captureEvents, defaults);
    assert.equal(first.state.tracksMousemove.value, false);
  } finally {
    first.unmount();
    second.unmount();
  }
});

test("authored TokenPreview reads replacement token, path and map from reactive props", async () => {
  const calls: Values[] = [];
  const component = setupComponent(gallery + "tokens/TokenPreview.vue", calls);
  const original = { tokenPath: "color.base", token: { value: "red" } };
  const first = mounted(component, original);
  const second = mounted(component, original);
  try {
    const defaults = distinctDefault<Values>(first.props, second.props, "tokenMap", {});
    defaults.private = { value: "only-first" };
    assert.deepEqual(second.props.tokenMap, {});
    assert.equal(first.state.value.value, "red");
    const next = {
      tokenPath: "color.accent",
      token: { value: "blue" },
      tokenMap: { accent: { value: "green" } },
    };
    await first.replace(next);
    assert.equal(first.state.value.value, "blue");
    assert.equal(calls.at(-1)?.tokenPath, next.tokenPath);
    assert.equal(calls.at(-1)?.token, next.token);
    assert.equal(calls.at(-1)?.tokenMap, next.tokenMap);
    const map = { other: { value: "gold" } };
    await first.replace({ ...next, tokenMap: map });
    const count = calls.length;
    void first.state.preview.value;
    assert.equal(calls.length, count + 1, "map replacement invalidates the authored computed");
    assert.equal(calls.at(-1)?.tokenMap, map);
    await first.replace(original);
    assert.equal(first.props.tokenMap, defaults);
    assert.equal(first.state.value.value, "red");
  } finally {
    first.unmount();
    second.unmount();
  }
});

test("authored TokenCard keeps scalar defaults and updates its tier after token replacement", async () => {
  const component = setupComponent(gallery + "tokens/TokenCard.vue");
  const original = {
    name: "Base",
    tokenPath: "color.base",
    token: { value: "red", $tier: "primitive" },
  };
  const first = mounted(component, original);
  const second = mounted(component, original);
  try {
    distinctDefault(first.props, second.props, "tokenMap", {});
    assert.equal(first.props.usageCount, 0);
    assert.equal(first.state.tierLabel.value, "Primitive");
    const next = {
      name: "Accent",
      tokenPath: "color.accent",
      token: { value: "blue", $tier: "semantic" },
      tokenMap: {},
      usageCount: 3,
    };
    await first.replace(next);
    assert.equal(first.state.tierLabel.value, "Semantic");
    for (const key of Object.keys(next))
      assert.equal(first.props[key], next[key as keyof typeof next]);
    await first.replace(original);
    assert.equal(first.props.usageCount, 0);
    assert.equal(first.state.tierLabel.value, "Primitive");
  } finally {
    first.unmount();
    second.unmount();
  }
});

test("authored TokenCategorySection keeps fresh maps and reactive category, level and usage", async () => {
  const component = setupComponent(gallery + "tokens/TokenCategorySection.vue");
  const original = { category: { name: "Color", tokens: { base: { value: "red" } } } };
  const first = mounted(component, original);
  const second = mounted(component, original);
  try {
    for (const key of ["usageMap", "tokenMap"]) distinctDefault(first.props, second.props, key, {});
    assert.equal(first.state.headingLevel.value, 2);
    const next = {
      category: { name: "Theme", tokens: { accent: { value: "blue" } } },
      level: 5,
      parentPath: "root",
      usageMap: { "root.theme.accent": [{ matches: [1, 2] }] },
      tokenMap: { accent: { value: "blue" } },
    };
    await first.replace(next);
    assert.equal(first.state.headingLevel.value, 5);
    assert.deepEqual(first.state.tokenEntries.value, [
      { name: "accent", token: next.category.tokens.accent },
    ]);
    assert.equal(first.state.getTokenPath("accent"), "root.theme.accent");
    assert.equal(first.state.getUsageCount("accent"), 2);
    assert.equal(first.props.tokenMap, next.tokenMap);
    await first.replace({ ...next, level: 8, parentPath: undefined, usageMap: {} });
    assert.equal(first.state.headingLevel.value, 6);
    assert.equal(first.state.getTokenPath("accent"), "theme.accent");
    assert.equal(first.state.getUsageCount("accent"), 0);
    await first.replace(original);
    assert.equal(first.state.headingLevel.value, 2);
  } finally {
    first.unmount();
    second.unmount();
  }
});

test("authored DevtoolsTracePanel retains its empty label and reacts to snapshot replacement", async () => {
  const component = setupComponent("npm/devtools/src/panel/DevtoolsTracePanel.vue");
  const snapshot = {
    events: [],
    renderTree: [],
    provideTree: [],
    suspenseTree: [],
    reactiveGraph: { nodes: [], edges: [] },
  };
  const first = mounted(component, { snapshot });
  try {
    assert.equal(first.props.emptyLabel, "No trace events");
    assert.equal(first.state.eventCount.value, 0);
    const next = {
      events: [1, 2],
      renderTree: ["render"],
      provideTree: ["provide"],
      suspenseTree: ["suspense"],
      reactiveGraph: { nodes: [{ id: "count" }], edges: ["edge"] },
    };
    await first.replace({ snapshot: next, emptyLabel: "Waiting for activity" });
    assert.equal(first.props.emptyLabel, "Waiting for activity");
    assert.equal(first.state.eventCount.value, 2);
    assert.deepEqual(first.state.renderNodes.value, next.renderTree);
    assert.deepEqual(first.state.provideNodes.value, next.provideTree);
    assert.deepEqual(first.state.suspenseNodes.value, next.suspenseTree);
    assert.deepEqual(first.state.graphNodes.value, next.reactiveGraph.nodes);
    assert.deepEqual(first.state.graphEdges.value, next.reactiveGraph.edges);
    await first.replace({ snapshot });
    assert.equal(first.props.emptyLabel, "No trace events");
    assert.equal(first.state.eventCount.value, 0);
  } finally {
    first.unmount();
  }
});
