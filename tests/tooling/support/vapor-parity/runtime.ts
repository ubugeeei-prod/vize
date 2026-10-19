// The Vue release the parity ratchet compares against (`vue-vapor-runtime-supported`,
// 3.6.0-rc.11), its compilers, and one happy-dom document shared by every case.
import { createRequire } from "node:module";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { Window } from "happy-dom";

const fromUi = createRequire(new URL("../../../../npm/ui/package.json", import.meta.url));
const vueEntry = fromUi.resolve("vue-vapor-runtime-supported");
const fromVue = createRequire(vueEntry);
const fromSfc = createRequire(fromVue.resolve("@vue/compiler-sfc"));

export const vueVersion: string = fromUi("vue-vapor-runtime-supported/package.json").version;
type Compile = (source: string, options: Record<string, unknown>) => { code: string };
export const compilerVapor = fromSfc("@vue/compiler-vapor") as { compile: Compile };
export const compilerSsr = fromSfc("@vue/compiler-ssr") as { compile: Compile };
export const serverVue = await import(pathToFileURL(vueEntry).href);
export const serverRenderer = await import(
  pathToFileURL(fromVue.resolve("@vue/server-renderer")).href
);

const window = new Window({ url: "http://localhost/" });
for (const key of [
  "window",
  "document",
  "Node",
  "Element",
  "HTMLElement",
  "SVGElement",
  "Comment",
  "Text",
  "DocumentFragment",
  "Event",
  "MouseEvent",
  "KeyboardEvent",
  "CustomEvent",
  "MutationObserver",
  "HTMLTemplateElement",
  "HTMLInputElement",
  "HTMLSelectElement",
  "HTMLTextAreaElement",
  "navigator",
  "requestAnimationFrame",
  "cancelAnimationFrame",
  "getComputedStyle",
  "MathMLElement",
  "ShadowRoot",
  "customElements",
  "CSSStyleDeclaration",
  "HTMLOptionElement",
] as const) {
  const value = (window as unknown as Record<string, unknown>)[key];
  if (value !== undefined) {
    Object.defineProperty(globalThis, key, { value, configurable: true, writable: true });
  }
}

const runtimeUrl = pathToFileURL(
  fromUi.resolve("vue-vapor-runtime-supported/dist/vue.runtime-with-vapor.esm-browser.js"),
).href;
/** The browser build that carries both renderers. */
export const vue = await import(runtimeUrl);

const modules = mkdtempSync(join(tmpdir(), "vize-vapor-parity-"));
mkdirSync(modules, { recursive: true });
let sequence = 0;

/** Evaluate compiled client code against the browser runtime. */
export function loadClient(code: string): Promise<Record<string, unknown>> {
  const file = join(modules, `c${sequence++}.mts`);
  writeFileSync(file, code.replaceAll(/from ['"]vue['"]/g, `from ${JSON.stringify(runtimeUrl)}`));
  return import(pathToFileURL(file).href);
}

/** Evaluate compiled server code against the node build. */
export function loadServer(code: string): Promise<Record<string, unknown>> {
  const file = join(modules, `s${sequence++}.mts`);
  writeFileSync(
    file,
    code.replaceAll(
      /from ['"](vue|vue\/server-renderer)['"]/g,
      (_, name: string) =>
        `from ${JSON.stringify(pathToFileURL(name === "vue" ? vueEntry : fromVue.resolve("@vue/server-renderer")).href)}`,
    ),
  );
  return import(pathToFileURL(file).href);
}

/** Upstream client output for `source`, or the compile errors. */
export function upstreamClient(source: string): { code: string; errors: string[] } {
  const errors: string[] = [];
  try {
    const { code } = compilerVapor.compile(source, {
      prefixIdentifiers: true,
      mode: "module",
      comments: false,
      expressionPlugins: ["typescript"],
      onError: (error: Error) => errors.push(error.message),
      onWarn: () => {},
    });
    return { code, errors };
  } catch (error) {
    return { code: "", errors: [...errors, `THROW ${(error as Error).message}`] };
  }
}

/** Identifiers a render reads from `_ctx`, so every one can be given a value. */
export function contextReads(code: string): Set<string> {
  return new Set([...code.matchAll(/_ctx\.([A-Za-z_$][\w$]*)/g)].map((match) => match[1]!));
}

export const firstLine = (value: unknown): string =>
  String((value as Error)?.message ?? value).split("\n")[0]!;
