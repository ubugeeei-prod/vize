// Typed views of the consumer-installed hosts these campaigns drive: Playwright,
// a Vite/Vue runtime host module and the browser realm used inside `evaluate`.
// The packages resolve from the consumer graph at run time, so each loader
// checks the members it relies on before exposing them through these types.
import assert from "node:assert/strict";
import net from "node:net";
import { createRequire } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { at, isRecord } from "../../json-value.ts";

export interface ConsoleMessage {
  type(): string;
  text(): string;
}
export interface PlaywrightResponse {
  url(): string;
  status(): number;
}
export interface Locator {
  click(): Promise<void>;
  waitFor(): Promise<void>;
  count(): Promise<number>;
  evaluate<R>(callback: (element: BrowserElement) => R): Promise<R>;
}
export interface Route {
  abort(): Promise<void>;
}
export interface Page {
  on(event: "pageerror", listener: (error: Error) => void): void;
  on(event: "console", listener: (message: ConsoleMessage) => void): void;
  on(event: "response", listener: (response: PlaywrightResponse) => void): void;
  goto(
    url: string,
    options?: { waitUntil?: "load" | "domcontentloaded" | "networkidle" },
  ): Promise<PlaywrightResponse | null>;
  locator(selector: string): Locator;
  getByRole(role: string, options?: { name?: string }): Locator;
  content(): Promise<string>;
  waitForFunction(callback: () => unknown): Promise<unknown>;
  route(url: string, handler: (route: Route) => unknown): Promise<void>;
  evaluate<R>(callback: () => R): Promise<R>;
  close(): Promise<void>;
}
export interface BrowserContext {
  newPage(): Promise<Page>;
  close(): Promise<void>;
}
export interface Browser {
  newPage(): Promise<Page>;
  newContext(options?: { javaScriptEnabled?: boolean }): Promise<BrowserContext>;
  version(): string;
  close(): Promise<void>;
}
export interface Chromium {
  launch(options?: Record<string, unknown>): Promise<Browser>;
}

/** The DOM members read by callbacks that Playwright runs in the page realm. */
export interface BrowserElement {
  outerHTML: string;
  textContent: string | null;
  href?: string;
  getAttributeNames(): string[];
  hasAttribute(name: string): boolean;
}
export interface BrowserGlobals {
  document: {
    querySelector(selector: string): BrowserElement | null;
    querySelectorAll(selector: string): Iterable<BrowserElement>;
    styleSheets: Iterable<{ cssRules: Iterable<{ cssText: string }> }>;
  };
  getComputedStyle: (element: BrowserElement) => { display: string; gap: string; color: string };
}

export function loadChromium(repoRoot: string): Chromium {
  const playwright: unknown = createRequire(path.join(repoRoot, "tests/package.json"))(
    "@playwright/test",
  );
  const chromium = at(playwright, "chromium");
  assert.equal(typeof at(chromium, "launch"), "function", "@playwright/test chromium.launch");
  return chromium as Chromium;
}

export interface PreviewServer {
  httpServer: { address(): string | { port: number } | null };
  close(): Promise<void>;
}
export interface ViteRuntimeHost {
  build(config: Record<string, unknown>): Promise<unknown>;
  preview(config: Record<string, unknown>): Promise<PreviewServer>;
  createSSRApp(component: unknown): unknown;
  renderToString(application: unknown, context?: { modules?: Iterable<string> }): Promise<string>;
}

/** Import a module and require the named function exports before typing it as `T`. */
export async function importChecked<T>(file: string, functions: readonly string[]): Promise<T> {
  const module: unknown = await import(pathToFileURL(file).href);
  for (const name of functions)
    assert.equal(typeof at(module, name), "function", `${path.basename(file)} exports ${name}`);
  return module as T;
}

export const viteRuntimeFunctions = ["build", "preview", "createSSRApp", "renderToString"];

/** The default export of a generated module, such as an SSR entry or a Nitro chunk. */
export async function importDefault(file: string): Promise<unknown> {
  const module: unknown = await import(pathToFileURL(file).href);
  return at(module, "default");
}

export function previewPort(server: PreviewServer): number {
  const address = server.httpServer.address();
  assert.ok(address && typeof address === "object", "preview server listens on a TCP port");
  return address.port;
}

/** Reserve and release a loopback port for a child server to bind. */
export async function freePort(): Promise<number> {
  const socket = net.createServer();
  await new Promise<void>((resolve, reject) => {
    socket.once("error", reject);
    socket.listen(0, "127.0.0.1", resolve);
  });
  const address = socket.address();
  assert.ok(address && typeof address === "object");
  await new Promise((resolve) => socket.close(resolve));
  return address.port;
}

/** A loaded Nuxt 2 instance; `constructor.version` is read with `at`. */
export interface Nuxt2 {
  options: unknown;
  close(): Promise<void>;
}

export async function loadNuxt2(fixture: string, options: Record<string, unknown>): Promise<Nuxt2> {
  const nuxtModule: unknown = createRequire(path.join(fixture, "package.json"))("nuxt");
  const load = at(nuxtModule, "loadNuxt");
  assert.equal(typeof load, "function", "nuxt exports loadNuxt");
  const nuxt: unknown = await (load as (options: Record<string, unknown>) => Promise<unknown>)(
    options,
  );
  assert.equal(typeof at(nuxt, "close"), "function", "Nuxt 2 instances close");
  return nuxt as Nuxt2;
}

export type LoadHook = (this: unknown, ...args: unknown[]) => unknown;
export interface VitePlugin {
  name?: string;
  load?: LoadHook | { handler: LoadHook };
  [key: string]: unknown;
}
export interface LoadRow {
  args: unknown[];
  result: unknown;
}

function isLoadHook(value: unknown): value is LoadHook {
  return typeof value === "function";
}
function isVitePlugin(value: unknown): value is VitePlugin {
  if (!isRecord(value)) return false;
  const { name, load } = value;
  if (name !== undefined && typeof name !== "string") return false;
  return load === undefined || isLoadHook(load) || isLoadHook(at(load, "handler"));
}
/** Keep the original plugin object; only its name and load hook are inspected. */
export function vitePlugin(value: unknown): VitePlugin {
  assert.ok(isVitePlugin(value), "a Vite plugin with an optional load hook");
  return value;
}

/** Wrap the Vize plugin's load hook to retain loads whose id names an original file. */
export function recordVizeLoads(
  plugins: readonly VitePlugin[],
  rows: LoadRow[],
  originals: readonly string[],
): VitePlugin[] {
  return plugins.map((plugin) => {
    if (plugin.name !== "vite-plugin-vize") return plugin;
    const load = plugin.load;
    assert.ok(load, "the Vize plugin owns a load hook");
    const original = typeof load === "function" ? load : load.handler;
    const observed = async function (this: unknown, ...args: unknown[]) {
      const result = await original.apply(this, args);
      const id = String(args[0]);
      if (result && originals.some((source) => id.includes(source))) rows.push({ args, result });
      return result;
    };
    return {
      ...plugin,
      load: typeof load === "function" ? observed : { ...load, handler: observed },
    };
  });
}

/** Every chunk and asset of a Vite build result, whether one or several outputs. */
export function bundleOutput(result: unknown): unknown[] {
  const output = Array.isArray(result)
    ? result.flatMap((item: unknown) => at(item, "output"))
    : at(result, "output");
  assert.ok(Array.isArray(output), "Vite build returns its bundle output");
  return output;
}
