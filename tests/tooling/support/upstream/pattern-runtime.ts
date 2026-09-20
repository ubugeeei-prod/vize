// Compile patterned-template SFCs with the real CLI and run them against the
// installed Vue runtime: client render, server render and hydration.
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { resolveVizeLaunchCommand } from "../lsp/launch.ts";
import { root } from "../lsp/paths.ts";
import { workspace } from "./vue-language-tools.ts";

export type PatternBackend = "dom" | "ssr" | "vapor";
export type PatternBuild = {
  directory: string;
  /** Import a compiled component, or a support module, from one backend's output. */
  load<T = Record<string, unknown>>(backend: PatternBackend, file: string): Promise<T>;
  dispose(): void;
};

const backendFlags = { dom: [], ssr: ["--ssr"], vapor: ["--vapor"] } as const;

/**
 * The one Vue build that carries both renderers. Node resolves `vue` to a
 * build without Vapor, so a parity run points every module at this file and
 * the two renderers share a single reactivity instance.
 */
export const vaporRuntime = pathToFileURL(
  path.join(root, "tests/node_modules/vue/dist/vue.runtime-with-vapor.esm-browser.js"),
).href;

/**
 * Build `components` (`Name.vue` -> source) for the client and the server.
 * `modules` are plain ES modules the components import, such as shared state;
 * both backends load the same copy so a test drives one reactive source.
 */
export function buildPatternComponents(
  components: Record<string, string>,
  modules: Record<string, string> = {},
  options: { backends?: PatternBackend[]; runtime?: string } = {},
): PatternBuild {
  const backends = options.backends ?? ["dom", "ssr"];
  const withRuntime = (code: string) =>
    options.runtime ? code.replaceAll(/(from\s*)(["'])vue\2/g, `$1"${options.runtime}"`) : code;
  const directory = workspace("pattern-runtime-");
  fs.mkdirSync(path.join(directory, "src"));
  fs.writeFileSync(
    path.join(directory, "vize.config.json"),
    JSON.stringify({ experimentals: { patternedTemplate: true } }),
  );
  for (const [file, source] of Object.entries(components)) {
    fs.writeFileSync(path.join(directory, "src", file), source);
  }
  const [binary] = resolveVizeLaunchCommand();
  for (const backend of backends) {
    const flags = backendFlags[backend];
    const output = path.join(directory, backend);
    const built = spawnSync(binary, ["build", "src", "-o", output, ...flags], {
      cwd: directory,
      encoding: "utf8",
    });
    assert.equal(built.status, 0, `${backend} build failed:\n${built.stdout}\n${built.stderr}`);
    for (const file of fs.readdirSync(output)) {
      // The installed workspace exposes the server renderer through `vue`.
      // Sibling components are built next to this one, as `.js`.
      const compiled = fs
        .readFileSync(path.join(output, file), "utf8")
        .replaceAll('"@vue/server-renderer"', '"vue/server-renderer"')
        .replaceAll(/(from\s*["']\.\/[\w-]+)\.vue(["'])/g, "$1.js$2");
      fs.writeFileSync(path.join(output, file), withRuntime(compiled));
    }
    for (const [file, source] of Object.entries(modules)) {
      // One shared instance: both backends re-export the module in `src`.
      fs.writeFileSync(path.join(directory, "src", file), withRuntime(source));
      fs.writeFileSync(
        path.join(output, file),
        `export * from ${JSON.stringify(`../src/${file}`)};\n`,
      );
    }
  }
  return {
    directory,
    load: (backend, file) => import(pathToFileURL(path.join(directory, backend, file)).href),
    dispose: () => fs.rmSync(directory, { recursive: true, force: true }),
  };
}

/** Install a DOM before `vue` is first imported; Vue captures `document` at load. */
export async function installDom(): Promise<void> {
  if (globalThis.document) return;
  const { Window } = await import("happy-dom");
  const window = new Window();
  for (const key of [
    "document",
    "Node",
    "Element",
    "HTMLElement",
    "SVGElement",
    "MathMLElement",
    "Text",
    "Comment",
    "DocumentFragment",
    "Event",
    "MouseEvent",
    "navigator",
  ]) {
    Object.defineProperty(globalThis, key, { value: window[key], configurable: true });
  }
  Object.defineProperty(globalThis, "window", { value: window, configurable: true });
}
