/**
 * `@vizejs/vite-plugin` vs `@vitejs/plugin-vue` option parity gate (#3227).
 *
 * The drop-in claim is only worth as much as the option surface behind it, so
 * this test enumerates the installed `@vitejs/plugin-vue` surface and requires
 * every option, `Api` member, and plugin hook to be either proven honored by a
 * behavioral probe below, or recorded as an explicit gap in
 * `tests/_fixtures/vite-plugin-vue-option-parity.json`.
 */

import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import type { Plugin, ResolvedConfig } from "vite";

import {
  honoredEvidence,
  readLedger,
  upstreamSurface,
  validateLedger,
} from "./_helpers/vite-plugin-vue-parity.ts";
import { vize } from "../../npm/builder/vite/src/plugin/index.ts";

type AnyHook = (...args: never[]) => unknown;

const templateOnly = `<template><div class="probe">probe</div></template>\n`;
const withStyle = `${templateOnly}<style>.probe{color:red}</style>\n`;
const restyled = `${templateOnly}<style>.probe{color:blue}</style>\n`;

function hook<T extends AnyHook>(candidate: unknown): T {
  const handler =
    typeof candidate === "function" ? candidate : (candidate as { handler?: T })?.handler;
  assert.equal(typeof handler, "function", "expected an implemented plugin hook");
  return handler as T;
}

function resolvedConfig(root: string, overrides: Record<string, unknown> = {}): ResolvedConfig {
  return {
    root,
    base: "/",
    build: { assetsDir: "assets", ssr: false },
    command: "serve",
    define: {},
    isProduction: false,
    mode: "development",
    plugins: [],
    resolve: { alias: [] },
    ...overrides,
  } as unknown as ResolvedConfig;
}

function createFixture(files: Record<string, string>): string {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-vue-parity-"));
  for (const [name, source] of Object.entries(files)) {
    fs.writeFileSync(path.join(root, name), source);
  }
  return root;
}

/** Boot the plugin set against a fixture root and return the main plugin. */
async function bootPlugin(
  root: string,
  options: Record<string, unknown> = {},
  configOverrides: Record<string, unknown> = {},
): Promise<Plugin> {
  // `configMode: false` skips `vize.config.*` discovery and an empty
  // `scanPatterns` skips startup pre-compilation, leaving the hook under test
  // as the only thing being measured.
  const plugins = vize({ configMode: false, scanPatterns: [], ...options }) as Plugin[];
  const plugin = plugins.find((candidate) => candidate.name === "vite-plugin-vize");
  assert.ok(plugin, "the Vize plugin set must expose its main plugin");
  await hook<(config: ResolvedConfig) => Promise<void>>(plugin.configResolved).call(
    {},
    resolvedConfig(root, configOverrides),
  );
  return plugin;
}

async function resolveVueId(plugin: Plugin, id: string): Promise<string | null> {
  const resolved = await hook<AnyHook>(plugin.resolveId).call(
    { resolve: () => null },
    id as never,
    undefined as never,
    {} as never,
  );
  return typeof resolved === "string" ? resolved : null;
}

async function loadVueModule(plugin: Plugin, id: string): Promise<string> {
  const loaded = await hook<AnyHook>(plugin.load).call(
    { addWatchFile() {} },
    id as never,
    {} as never,
  );
  const code = typeof loaded === "string" ? loaded : (loaded as { code?: string } | null)?.code;
  assert.equal(typeof code, "string", `loading ${id} must produce module code`);
  return code as string;
}

/** `include` narrows the set of files the plugin claims. */
async function probeIncludeFilter(): Promise<void> {
  const root = createFixture({ "Kept.vue": withStyle, "Other.vue": templateOnly });
  const plugin = await bootPlugin(root, { include: [/Kept\.vue$/] });

  assert.ok(await resolveVueId(plugin, path.join(root, "Kept.vue")), "included SFCs stay claimed");
  assert.equal(
    await resolveVueId(plugin, path.join(root, "Other.vue")),
    null,
    "an SFC outside `include` must be left to the rest of the pipeline",
  );
}

/** `exclude` releases files the default include would otherwise claim. */
async function probeExcludeFilter(): Promise<void> {
  const root = createFixture({ "Kept.vue": withStyle, "Skipped.vue": templateOnly });
  const plugin = await bootPlugin(root, { exclude: [/Skipped\.vue$/] });

  assert.ok(
    await resolveVueId(plugin, path.join(root, "Kept.vue")),
    "unexcluded SFCs stay claimed",
  );
  assert.equal(
    await resolveVueId(plugin, path.join(root, "Skipped.vue")),
    null,
    "an excluded SFC must be left to the rest of the pipeline",
  );
}

/**
 * `isProduction` overrides Vite's own flag: production output hands styles to
 * Vite as a CSS import instead of injecting them at runtime.
 */
async function probeProductionCssImport(): Promise<void> {
  const root = createFixture({ "Comp.vue": withStyle });
  const id = path.join(root, "Comp.vue");

  const devPlugin = await bootPlugin(root);
  const devId = await resolveVueId(devPlugin, id);
  assert.ok(devId);
  const devCode = await loadVueModule(devPlugin, devId);
  assert.match(devCode, /__vize_css__/, "development output injects component CSS at runtime");

  // Vite still reports `isProduction: false`; only the plugin option changes.
  const prodPlugin = await bootPlugin(root, { isProduction: true });
  const prodId = await resolveVueId(prodPlugin, id);
  assert.ok(prodId);
  const prodCode = await loadVueModule(prodPlugin, prodId);
  assert.doesNotMatch(
    prodCode,
    /__vize_css__/,
    "`isProduction: true` must switch off runtime style injection",
  );
  assert.match(
    prodCode,
    /import ".*Comp\.vue\?vue=&type=style&index=0[^"]*"/,
    "`isProduction: true` must emit an extractable CSS import",
  );
}

/** A style-only edit produces a style-only HMR payload, not a component reload. */
async function probeHotUpdateStyleOnly(): Promise<void> {
  const root = createFixture({ "Comp.vue": withStyle });
  const file = path.join(root, "Comp.vue");
  const plugin = await bootPlugin(root);
  const resolved = await resolveVueId(plugin, file);
  assert.ok(resolved);
  await loadVueModule(plugin, resolved);

  const sent: Array<{ data?: { css?: string; type?: string }; event?: string }> = [];
  const server = {
    moduleGraph: { getModulesByFile: () => undefined, invalidateModule() {} },
    ws: { send: (payload: never) => sent.push(payload) },
  };

  fs.writeFileSync(file, restyled);
  const affected = await hook<AnyHook>(plugin.handleHotUpdate).call({}, {
    file,
    modules: [],
    read: async () => restyled,
    server,
    timestamp: Date.now(),
  } as never);

  assert.deepEqual(affected, [], "a style-only edit must not invalidate the component module");
  assert.equal(sent.length, 1, "a style-only edit must send exactly one HMR payload");
  assert.equal(sent[0].event, "vize:update");
  assert.equal(sent[0].data?.type, "style-only");
  assert.match(sent[0].data?.css ?? "", /color:\s*blue/, "the payload carries the new CSS");
}

/** The named hook is implemented by one of the plugins Vize contributes. */
function probeHookImplemented(name: string): void {
  const plugins = vize({ configMode: false }) as Plugin[];
  const implementations = plugins.filter(
    (plugin) => typeof (plugin as unknown as Record<string, unknown>)[name] !== "undefined",
  );
  assert.notEqual(
    implementations.length,
    0,
    `the Vize plugin set must implement the upstream \`${name}\` hook`,
  );
}

const probes = new Map<string, () => Promise<void> | void>([
  ["include-filter", probeIncludeFilter],
  ["exclude-filter", probeExcludeFilter],
  ["production-css-import", probeProductionCssImport],
  ["hot-update-style-only", probeHotUpdateStyleOnly],
]);

test("the parity ledger stays exhaustive over the pinned @vitejs/plugin-vue surface", () => {
  const surface = upstreamSurface();
  const ledger = readLedger();
  validateLedger(ledger, surface);

  assert.ok(ledger.summary.honored > 0, "the ledger must record the surface Vize does honor");
  assert.ok(
    ledger.summary.unimplemented > 0,
    "the ledger must keep the remaining gaps explicit rather than implying full parity",
  );
});

test("every option the ledger calls honored is backed by a behavioral probe", async () => {
  const evidence = honoredEvidence(readLedger());
  assert.notEqual(evidence.size, 0, "at least one entry must be proven honored");

  const executed = new Set<string>();
  for (const [entry, evidenceId] of evidence) {
    if (evidenceId === "hook-implemented") {
      probeHookImplemented(entry.slice("hooks.".length));
      continue;
    }
    const probe = probes.get(evidenceId);
    assert.ok(probe, `${entry} names unknown evidence ${JSON.stringify(evidenceId)}`);
    if (!executed.has(evidenceId)) {
      await probe();
      executed.add(evidenceId);
    }
  }

  assert.deepEqual(
    [...probes.keys()].filter((id) => !executed.has(id)),
    [],
    "every behavioral probe must back at least one honored ledger entry",
  );
});
