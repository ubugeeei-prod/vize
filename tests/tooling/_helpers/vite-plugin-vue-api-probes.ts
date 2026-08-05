/**
 * `vite:vue` plugin-api parity probes for the `@vitejs/plugin-vue` gate (#3227).
 *
 * Host frameworks reach Vize through the `vite:vue` shim's `api` object rather
 * than through its options, so the shim is a compatibility surface in its own
 * right. These probes run the real upstream plugin as the oracle.
 */

import assert from "node:assert/strict";
import type { Plugin } from "vite";

import { createUpstreamPlugin } from "./vite-plugin-vue-parity.ts";
import { vize } from "../../../npm/builder/vite/src/plugin/index.ts";

type FilterApi = {
  exclude?: unknown;
  include?: unknown;
};

function shimApi(options: Record<string, unknown> = {}): FilterApi {
  const plugin = (vize({ configMode: false, scanPatterns: [], ...options }) as Plugin[]).find(
    (candidate) => candidate.name === "vite:vue",
  );
  assert.ok(plugin, "Vize must contribute a `vite:vue` compatibility shim");
  return (plugin as { api?: FilterApi }).api as FilterApi;
}

function upstreamApi(options: Record<string, unknown> = {}): FilterApi {
  const plugin = createUpstreamPlugin(options) as Plugin;
  return (plugin as { api?: FilterApi }).api as FilterApi;
}

/**
 * `api.include` and `api.exclude` report the patterns the filter actually uses.
 *
 * A configured value must round-trip in the shape it was given, because a host
 * that reads one to extend the filter feeds it straight back in. The `include`
 * default is compared against upstream; `exclude` cannot be, and the assertion
 * below pins why.
 */
export function probeFilterApi(): void {
  for (const key of ["include", "exclude"] as const) {
    for (const configured of [[/\.custom\.vue$/], /\.custom\.vue$/, "**/*.custom.vue"]) {
      assert.deepEqual(
        shimApi({ [key]: configured })[key],
        configured,
        `api.${key} must report a configured value unchanged`,
      );
    }
  }

  assert.deepEqual(
    shimApi().include,
    upstreamApi().include,
    "the include default must match plugin-vue's",
  );

  // plugin-vue leaves `exclude` undefined and lets Vite's own pipeline keep
  // node_modules out; Vize filters on the pattern itself. Reporting upstream's
  // `undefined` here would describe a filter Vize does not have, so the shim
  // reports the real one and this assertion keeps the difference deliberate.
  assert.equal(upstreamApi().exclude, undefined);
  assert.deepEqual(shimApi().exclude, /node_modules/);
}

/** Assigning `api.include` / `api.exclude` rewrites the filter the plugin builds. */
export async function probeFilterApiAssignment(): Promise<void> {
  const plugins = vize({ configMode: false, scanPatterns: [] }) as Plugin[];
  const shim = plugins.find((candidate) => candidate.name === "vite:vue");
  const main = plugins.find((candidate) => candidate.name === "vite-plugin-vize");
  assert.ok(shim);
  assert.ok(main);

  const api = (shim as { api?: FilterApi }).api as FilterApi;
  api.include = [/\.probe\.vue$/];
  assert.deepEqual(
    api.include,
    [/\.probe\.vue$/],
    "the assignment must be visible through the api",
  );

  // Upstream refuses the same write once its filter is resolved; silently
  // dropping it would leave the host believing it changed the filter.
  const configResolved = (main as { configResolved?: unknown }).configResolved;
  const handler =
    typeof configResolved === "function"
      ? configResolved
      : (configResolved as { handler?: (config: unknown) => unknown } | undefined)?.handler;
  assert.equal(typeof handler, "function");
  await (handler as (config: unknown) => unknown).call(
    {},
    {
      root: process.cwd(),
      base: "/",
      build: { assetsDir: "assets", ssr: false },
      command: "serve",
      define: {},
      isProduction: false,
      mode: "development",
      plugins: [],
      resolve: { alias: [] },
    },
  );

  assert.throws(
    () => {
      api.exclude = [/late\.vue$/];
    },
    /cannot be updated/,
    "a write after the filter is resolved must fail loudly",
  );
}
