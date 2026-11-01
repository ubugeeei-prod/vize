import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { captureWholeValue, isRecord } from "./sfc-entry-validation-reference.ts";

export { captureWholeValue, isRecord };
export const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
export const directory = "tests/_fixtures/differential/formatter/sfc-encoded-ampersand-data-6098";
export interface AmpCase {
  id: string;
  file: string;
  head: ":title" | "@click";
  bytes: number;
  sha256: string;
  source: string;
  expected: { expression: string; result: string; calls: unknown[] };
  reference: { kind: "return"; code: string } | { kind: "throw"; error: unknown };
}
export function loadAmpCases(root: string) {
  const bytes = fs.readFileSync(path.join(root, directory, "cases.json"));
  assert.equal(hash(bytes), "32f32df3cf499d761f1bbbea38e737a5d26c45904675c37d10df5051264e51f8");
  const corpus = JSON.parse(bytes.toString("utf8")) as {
    schema: string;
    options: Record<string, unknown>;
    cases: Omit<AmpCase, "source">[];
  };
  assert.equal(corpus.schema, "vize.glyph.encoded-ampersand-data.v1");
  assert.equal(corpus.cases.length, 13);
  const cases = corpus.cases.map((row): AmpCase => {
    assert.match(row.file, /^[a-z-]+\.vue\.txt$/u);
    assert.equal(row.file, `${row.id}.vue.txt`);
    const bytes = fs.readFileSync(path.join(root, directory, row.file));
    assert.equal(bytes.length, row.bytes);
    assert.equal(hash(bytes), row.sha256);
    const source = bytes.toString("utf8");
    assert.deepEqual(Buffer.from(source), bytes);
    assert(row.head === ":title" || row.head === "@click");
    return { ...row, source };
  });
  assert.equal(new Set(cases.map((row) => row.id)).size, 13);
  return { cases, options: corpus.options, catalogSha256: hash(bytes) };
}
export function observe(run: () => unknown) {
  try {
    return { kind: "returned" as const, value: run() };
  } catch (error) {
    return { kind: "threw" as const, error };
  }
}
export function sourceProviders(
  providerRoot = fileURLToPath(new URL("../../../", import.meta.url)),
) {
  const fromUi = createRequire(path.join(providerRoot, "npm/ui/package.json"));
  const fromVue = createRequire(fromUi.resolve("vue/package.json"));
  const fromBench = createRequire(path.join(providerRoot, "tools/benchmarks/scripts/package.json"));
  const prettier = fromBench("prettier") as {
    version: string;
    format(source: string, options: Record<string, unknown>): Promise<string>;
  };
  const compiler = fromVue("@vue/compiler-sfc") as {
    version: string;
    parse(source: string, options: Record<string, unknown>): unknown;
  };
  const dom = fromVue("@vue/compiler-dom") as {
    compile(source: string, options: Record<string, unknown>): unknown;
  };
  assert.equal(prettier.version, "3.8.3");
  assert.equal(compiler.version, "3.5.35");
  assert.equal((fromVue("vue/package.json") as { version: string }).version, "3.5.35");
  assert.equal((fromUi("happy-dom/package.json") as { version: string }).version, "21.0.4");
  const files = [
    fromBench.resolve("prettier"),
    fromVue.resolve("@vue/compiler-sfc"),
    fromVue.resolve("@vue/compiler-dom"),
    fromVue.resolve("vue"),
    fromUi.resolve("happy-dom"),
  ];
  const receipt = {
    node: process.version,
    prettier: prettier.version,
    vue: compiler.version,
    happyDom: "21.0.4",
    files: files.map((file) => {
      const bytes = fs.readFileSync(file);
      return { file, bytes: bytes.length, sha256: hash(bytes) };
    }),
  };
  return { fromUi, fromVue, prettier, compiler, dom, receipt };
}
interface App {
  config: { warnHandler: (message: string) => void; errorHandler: (error: unknown) => void };
  mount(element: Element): { $forceUpdate(): void };
  unmount(): void;
}
interface VueRuntime {
  version: string;
  nextTick(): Promise<void>;
  onUpdated(callback: () => void): void;
  onUnmounted(callback: () => void): void;
  createApp(options: { setup(): Record<string, unknown>; render: unknown }): App;
}
/** Real Vue runtime-dom on happy-dom. Compilation, lifecycle and DOM observations
 * are retained separately so byte-different render functions cannot mask a
 * changed semantic outcome. This is a DOM consumer, not browser acceptance.
 */
export function domConsumer(providers: ReturnType<typeof sourceProviders>) {
  const { Window } = providers.fromUi("happy-dom") as {
    Window: new () => Window & { Event: typeof Event; happyDOM: { close(): Promise<void> } };
  };
  const window = new Window();
  const keys = [
    "window",
    "document",
    "Node",
    "Element",
    "HTMLElement",
    "SVGElement",
    "ShadowRoot",
    "Event",
  ] as const;
  const previous = keys.map(
    (key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)] as const,
  );
  for (const key of keys)
    Object.defineProperty(globalThis, key, {
      configurable: true,
      writable: true,
      value: key === "window" ? window : Reflect.get(window, key),
    });
  // Loading runtime-dom before document installation would produce a false host.
  assert.equal(providers.fromVue.cache[providers.fromVue.resolve("vue")], undefined);
  const vue = providers.fromVue("vue") as VueRuntime;
  assert.equal(vue.version, "3.5.35");
  const consume = async (source: string, row: AmpCase) => {
    const parsed = providers.compiler.parse(source, {
      filename: row.file.slice(0, -4),
      sourceMap: true,
    });
    assert(isRecord(parsed) && Array.isArray(parsed.errors));
    assert.deepEqual(parsed.errors, []);
    assert(isRecord(parsed.descriptor) && isRecord(parsed.descriptor.template));
    assert.equal(typeof parsed.descriptor.template.content, "string");
    const compiled = providers.dom.compile(parsed.descriptor.template.content as string, {
      mode: "function",
      prefixIdentifiers: true,
      hoistStatic: false,
      sourceMap: true,
      filename: row.file.slice(0, -4),
    });
    assert(isRecord(compiled) && typeof compiled.code === "string");
    // Authenticated finite authored fixtures only; no imported user program.
    const render: unknown = new Function("Vue", compiled.code)(vue);
    const calls: unknown[] = [],
      warnings: string[] = [],
      errors: unknown[] = [];
    let setups = 0,
      updates = 0,
      unmounts = 0;
    const app = vue.createApp({
      render,
      setup() {
        setups += 1;
        vue.onUpdated(() => {
          updates += 1;
        });
        vue.onUnmounted(() => {
          unmounts += 1;
        });
        return {
          left: "L",
          right: "R",
          prefix: "P",
          read(value: unknown) {
            calls.push(["read", value]);
            return value;
          },
          done() {
            calls.push(["done"]);
            return "done";
          },
        };
      },
    });
    app.config.warnHandler = (message) => {
      warnings.push(message);
    };
    app.config.errorHandler = (error) => {
      errors.push(captureWholeValue(error));
    };
    const container = window.document.createElement("section");
    window.document.body.append(container);
    const view = () => ({
      html: container.innerHTML,
      title: container.firstElementChild?.getAttribute("title") ?? null,
      text: container.textContent,
    });
    let outcome: unknown;
    try {
      const instance = app.mount(container);
      const element = container.firstElementChild;
      assert(element && element.tagName === "DIV");
      const initial = view();
      if (row.head === "@click")
        element.dispatchEvent(new window.Event("click", { bubbles: true }));
      await vue.nextTick();
      const clicked = view();
      instance.$forceUpdate();
      await vue.nextTick();
      const warm = view();
      assert.equal(container.firstElementChild, element);
      if (row.head === "@click")
        element.dispatchEvent(new window.Event("click", { bubbles: true }));
      await vue.nextTick();
      outcome = { initial, clicked, warm, final: view(), calls, setups, updates };
    } finally {
      app.unmount();
    }
    const result = { outcome, unmounts, afterUnmount: view(), warnings, errors };
    container.remove();
    return { parsed: captureWholeValue(parsed), compiled: captureWholeValue(compiled), result };
  };
  return {
    consume,
    async close() {
      await window.happyDOM.close();
      for (const [key, descriptor] of previous) {
        if (descriptor) Object.defineProperty(globalThis, key, descriptor);
        else Reflect.deleteProperty(globalThis, key);
      }
    },
  };
}
