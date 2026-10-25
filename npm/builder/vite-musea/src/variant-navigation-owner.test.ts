import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import test from "node:test";
import { compileFunction } from "node:vm";
import ts from "typescript";
import { MUSEA_PROPS_QUERY } from "../gallery/utils/propsPermalink.ts";

const corpus = new URL(
  "../../../../tests/_fixtures/differential/musea/variant-navigation/",
  import.meta.url,
);
const manifest = JSON.parse(await readFile(new URL("manifest.json", corpus), "utf8"));
const before = await readFile(new URL("before-script.txt", corpus), "utf8");
const current = await readFile(
  new URL("../gallery/views/ComponentView.vue", import.meta.url),
  "utf8",
);
const helper = await readFile(
  new URL("../gallery/utils/variantNavigation.ts", import.meta.url),
  "utf8",
);

// Execute the complete production navigation block, including its actual cleanup watchers.
// Controlled nextTick resolution makes superseded continuations deterministic without timers.
function harness(script: string) {
  const extracted = script === current;
  const start = script.indexOf(
    extracted ? "const {\n  disconnectVariantObserver," : "function disconnectVariantObserver() {",
  );
  assert.ok(start >= 0);
  const block = script.slice(start).split("</script>")[0];
  const callbacks: Array<() => void> = [];
  const unmounts: Array<() => void> = [];
  const watches: Array<{ source: unknown; callback: () => void }> = [];
  const scrolls: Array<{ name: string; options: ScrollIntoViewOptions }> = [];
  class Section {
    dataset: { variantName: string };
    constructor(readonly name: string) {
      this.dataset = { variantName: name };
    }
    scrollIntoView(options: ScrollIntoViewOptions) {
      scrolls.push({ name: this.name, options });
    }
  }
  const sections = new Map(["first", "last"].map((name) => [name, new Section(name)]));
  const observers: Array<{ callback: (entries: unknown[]) => void; disconnected: boolean }> = [];
  class Observer {
    disconnected = false;
    constructor(readonly callback: (entries: unknown[]) => void) {
      observers.push(this);
    }
    observe() {}
    disconnect() {
      this.disconnected = true;
    }
  }
  const scope = {
    activeTab: { value: "variants" },
    route: { hash: "#last", query: {} as Record<string, unknown> },
    MUSEA_PROPS_QUERY,
    artPath: { value: "Keys" },
    art: { value: { path: "Keys" } },
    selectedVariantName: { value: "first" },
    variantSectionIds: { value: { first: "first", last: "last" } },
    variantSectionElements: new Map(sections),
    nextTick: () => new Promise<void>((resolve) => callbacks.push(resolve)),
    watch: (source: unknown, callback: () => void) => watches.push({ source, callback }),
    onMounted: () => {},
    onUnmounted: (callback: () => void) => unmounts.push(callback),
    clearActions: () => {},
    load: () => {},
    initActions: () => {},
    window: { IntersectionObserver: Observer },
    IntersectionObserver: Observer,
    Element: Section,
    HTMLElement: Section,
    document: { getElementById: (id: string) => sections.get(id) },
  };
  const javascript = ts.transpileModule(block, {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
  }).outputText;
  const helperJavascript = ts.transpileModule(helper, {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext },
  }).outputText;
  assert.equal(helperJavascript.split("export function createVariantNavigation").length, 2);
  const transport = helperJavascript.replace(
    "export function createVariantNavigation",
    "function createVariantNavigation",
  );
  assert.equal(
    transport.replace(
      "function createVariantNavigation",
      "export function createVariantNavigation",
    ),
    helperJavascript,
  );
  const initialization = extracted
    ? transport
    : "let variantObserver = null; let variantObserverGeneration = 0;";
  const execute = compileFunction(
    `
    const { ${Object.keys(scope).join(", ")} } = scope;
    ${initialization}
    ${javascript}
    return { setupVariantObserver, syncSelectionFromHash, handleVariantSelect, disconnectVariantObserver };
    `,
    ["scope"],
    { filename: manifest.producer },
  );
  const api = execute(scope) as {
    setupVariantObserver(): Promise<void>;
    syncSelectionFromHash(): Promise<void>;
    handleVariantSelect(name: string): void;
    disconnectVariantObserver(): void;
  };
  return {
    ...api,
    scope,
    observers,
    sections,
    scrolls,
    tick: () => callbacks.splice(0).forEach((resolve) => resolve()),
    unmount: () => unmounts.forEach((callback) => callback()),
    refreshNavigation: () => {
      // Select the production art/tab watcher by what it watches. Additional
      // independent query watchers must not change this navigation contract.
      const watcher = watches.find(
        ({ source }) => typeof source === "function" && Array.isArray(source()),
      );
      assert.ok(watcher);
      watcher.callback();
    },
    replaceRoute: () => {
      scope.artPath.value = "Controls";
      const watcher = watches.find(({ source }) => source === scope.artPath);
      assert.ok(watcher);
      watcher.callback();
    },
    entry: (name: string) => ({
      target: sections.get(name),
      isIntersecting: true,
      intersectionRatio: 1,
      boundingClientRect: { top: 0 },
    }),
  };
}

void test("retains the complete original failure and immutable production-before script", async () => {
  assert.equal(createHash("sha256").update(before).digest("hex"), manifest.beforeScriptSha256);
  const raw = await readFile(new URL("original-failure.json", corpus));
  assert.equal(
    createHash("sha256").update(raw).digest("hex"),
    manifest.originalFailure.packetSha256,
  );
  const packet = JSON.parse(raw.toString());
  const snapshots = packet.observations.filter((row: { phase?: string }) =>
    row.phase?.startsWith("literal-nav-"),
  );
  assert.equal(snapshots.length, 2);
  assert.equal(snapshots[0].requestedFirst.ariaCurrent, "true");
  assert.equal(snapshots[1].requestedFirst.ariaCurrent, null);
  assert.equal(snapshots[1].requestedLast.ariaCurrent, "true");
  assert.ok(snapshots[1].requestedFirst.sectionBottom < 0);
  assert.ok(snapshots[1].requestedLast.sectionTop > 0);
});

void test("rejects a formerly owned callback after explicit selection; original producer reverses it", async () => {
  for (const [script, expected] of [
    [before, "last"],
    [current, "first"],
  ]) {
    const h = harness(script);
    const setup = h.setupVariantObserver();
    h.tick();
    await setup;
    const retained = h.observers[0];
    h.handleVariantSelect("first");
    retained.callback([h.entry("last")]);
    assert.equal(h.scope.selectedVariantName.value, expected);
    h.tick();
    await Promise.resolve();
    assert.equal(h.scrolls[0].options.behavior, script === before ? "smooth" : "instant");
  }
});

void test("only the newest pending setup owns an observer; unmount rejects pending setup", async () => {
  for (const [script, count] of [
    [before, 2],
    [current, 1],
  ] as const) {
    const h = harness(script);
    const old = h.setupVariantObserver();
    h.disconnectVariantObserver();
    const latest = h.setupVariantObserver();
    h.tick();
    await Promise.all([old, latest]);
    assert.equal(h.observers.length, count);
  }
  const h = harness(current);
  const pending = h.setupVariantObserver();
  h.unmount();
  h.tick();
  await pending;
  assert.equal(h.observers.length, 0);
});

void test("initial render observes refs created during the existing nextTick", async () => {
  for (const script of [before, current]) {
    const h = harness(script);
    h.scope.variantSectionElements.clear();
    const pending = h.setupVariantObserver();
    h.scope.variantSectionElements.set("first", h.sections.get("first")!);
    h.tick();
    await pending;
    assert.equal(h.observers.length, script === before ? 0 : 1);
  }
});

void test("a no-hash view landing requests immediate intended selection before observing", async () => {
  for (const script of [before, current]) {
    const h = harness(script);
    h.scope.route.hash = "";
    h.refreshNavigation();
    h.tick();
    await Promise.resolve();
    assert.equal(h.scrolls.length, script === before ? 0 : 1);
    if (script === current) {
      assert.deepEqual(h.scrolls[0], {
        name: "first",
        options: { behavior: "instant", block: "start" },
      });
    }
  }
});

for (const edge of ["tab", "route", "unmount", "hash", "click"] as const) {
  void test(`pending hash cannot scroll after ${edge}; original continuation still scrolls`, async () => {
    for (const script of [before, current]) {
      const h = harness(script);
      const pending = h.syncSelectionFromHash();
      if (edge === "tab") h.scope.activeTab.value = "props";
      if (edge === "route") h.replaceRoute();
      if (edge === "unmount") h.unmount();
      if (edge === "hash") h.scope.route.hash = "#first";
      if (edge === "click") h.handleVariantSelect("first");
      const ownScrolls = h.scrolls.length;
      h.tick();
      await pending;
      assert.equal(h.scrolls.length - ownScrolls, script === before ? 1 : 0);
    }
  });
}
