import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import type { Plugin, ResolvedConfig } from "vite";

import {
  probeOptionsApiFeature,
  probeProdDevtoolsFeature,
  probeProdHydrationMismatchDetailsFeature,
} from "./vite-plugin-vue-define-probes.ts";
import {
  probeFilterApi,
  probeFilterApiAssignment,
  probeVersionApi,
} from "./vite-plugin-vue-api-probes.ts";
import { vize } from "../../../npm/builder/vite/src/plugin/index.ts";

type AnyHook = (...args: never[]) => unknown;

const templateOnly = `<template><div class="probe">probe</div></template>\n`;
const withStyle = `${templateOnly}<style>.probe{color:red}</style>\n`;
const restyled = `${templateOnly}<style>.probe{color:blue}</style>\n`;
const spacedStyle = `${templateOnly}<style>\n.probe { color: red; }\n</style>\n`;
const withComment = `<template><div><!--kept--><span>probe</span></div></template>\n`;

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

async function bootPlugin(
  root: string,
  options: Record<string, unknown> = {},
  configOverrides: Record<string, unknown> = {},
): Promise<Plugin> {
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

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

async function loadResolvedVueModule(plugin: Plugin, id: string): Promise<string> {
  const resolved = await resolveVueId(plugin, id);
  assert.ok(resolved, `${id} must resolve as a Vue module`);
  return loadVueModule(plugin, resolved);
}

async function loadResolvedStyleModule(plugin: Plugin, id: string): Promise<string> {
  const code = await loadResolvedVueModule(plugin, id);
  const styleId = `${id}?vue=&type=style&index=0&lang=css.css`;
  assert.ok(
    code.includes(`import ${JSON.stringify(styleId)};`),
    `${id} must hand its CSS to Vite through a style import`,
  );
  return loadVueModule(plugin, styleId);
}

async function probeIncludeFilter(): Promise<void> {
  const root = createFixture({ "Kept.vue": withStyle, "Other.vue": templateOnly });
  const plugin = await bootPlugin(root, { include: [/Kept\.vue$/] });

  assert.ok(await resolveVueId(plugin, path.join(root, "Kept.vue")), "included SFCs stay claimed");
  assert.equal(await resolveVueId(plugin, path.join(root, "Other.vue")), null);
}

async function probeExcludeFilter(): Promise<void> {
  const root = createFixture({ "Kept.vue": withStyle, "Skipped.vue": templateOnly });
  const plugin = await bootPlugin(root, { exclude: [/Skipped\.vue$/] });

  assert.ok(await resolveVueId(plugin, path.join(root, "Kept.vue")));
  assert.equal(await resolveVueId(plugin, path.join(root, "Skipped.vue")), null);
}

async function probeProductionCssImport(): Promise<void> {
  const root = createFixture({ "Comp.vue": withStyle });
  const id = path.join(root, "Comp.vue");
  const devPlugin = await bootPlugin(root);
  const devCode = await loadResolvedVueModule(devPlugin, id);
  assert.doesNotMatch(devCode, /__vize_css__/);
  assert.match(await loadResolvedStyleModule(devPlugin, id), /\.probe\{color:red\}/);

  const prodPlugin = await bootPlugin(root, { isProduction: true });
  const prodCode = await loadResolvedVueModule(prodPlugin, id);
  assert.doesNotMatch(prodCode, /__vize_css__/);
  assert.match(await loadResolvedStyleModule(prodPlugin, id), /\.probe\{color:red\}/);
}

async function probeStyleTrim(): Promise<void> {
  const root = createFixture({ "Comp.vue": spacedStyle });
  const id = path.join(root, "Comp.vue");
  const defaultPlugin = await bootPlugin(root);
  const defaultCss = await loadResolvedStyleModule(defaultPlugin, id);
  assert.equal(defaultCss, defaultCss.trim());

  const rawPlugin = await bootPlugin(root, { style: { trim: false } });
  const rawCss = await loadResolvedStyleModule(rawPlugin, id);
  assert.match(rawCss, /^\n/);
  assert.match(rawCss, /\n$/);
}

async function probeTemplateCompilerOptions(): Promise<void> {
  const root = createFixture({ "Comp.vue": withComment });
  const id = path.join(root, "Comp.vue");
  const defaultPlugin = await bootPlugin(root);
  assert.doesNotMatch(await loadResolvedVueModule(defaultPlugin, id), /kept/);

  const commentsPlugin = await bootPlugin(root, {
    template: { compilerOptions: { comments: true } },
  });
  assert.match(await loadResolvedVueModule(commentsPlugin, id), /kept/);
}

function assertCustomElementStyleOutput(code: string, fileName: string): void {
  assert.doesNotMatch(code, /__vize_css__/);
  assert.match(
    code,
    new RegExp(
      `import _style_0 from ".*${escapeRegExp(fileName)}\\.vue\\?vue=&type=style&index=0&lang=css&inline=`,
    ),
  );
  assert.match(code, /_sfc_main\.styles = \[_style_0\];/);
}

async function probeCustomElementOutput(): Promise<void> {
  const root = createFixture({
    "Alias.vue": withStyle,
    "Element.ce.vue": withStyle,
    "Feature.vue": withStyle,
    "Plain.vue": withStyle,
  });

  const defaultPlugin = await bootPlugin(root);
  assertCustomElementStyleOutput(
    await loadResolvedVueModule(defaultPlugin, path.join(root, "Element.ce.vue")),
    "Element.ce",
  );
  const plainId = path.join(root, "Plain.vue");
  assert.doesNotMatch(await loadResolvedVueModule(defaultPlugin, plainId), /__vize_css__/);
  assert.match(await loadResolvedStyleModule(defaultPlugin, plainId), /\.probe\{color:red\}/);

  const featurePlugin = await bootPlugin(root, { features: { customElement: /Feature\.vue$/ } });
  assertCustomElementStyleOutput(
    await loadResolvedVueModule(featurePlugin, path.join(root, "Feature.vue")),
    "Feature",
  );

  const aliasPlugin = await bootPlugin(root, { customElement: /Alias\.vue$/ });
  assertCustomElementStyleOutput(
    await loadResolvedVueModule(aliasPlugin, path.join(root, "Alias.vue")),
    "Alias",
  );
}

async function probeHotUpdateStyleOnly(): Promise<void> {
  const root = createFixture({ "Comp.vue": withStyle });
  const file = path.join(root, "Comp.vue");
  const plugin = await bootPlugin(root);
  await loadResolvedVueModule(plugin, file);

  const sent: Array<{ data?: { css?: string; type?: string }; event?: string }> = [];
  const styleId = `${file}?vue=&type=style&index=0&lang=css.css`;
  const styleModule = { url: styleId };
  const server = {
    moduleGraph: {
      getModulesByFile: (id: string) => (id === styleId ? new Set([styleModule]) : undefined),
      invalidateModule() {},
    },
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

  assert.deepEqual(affected, [styleModule], "a style-only edit must update the Vite CSS module");
  assert.deepEqual(sent, [], "Vite handles the CSS update without a custom HMR event");
  assert.match(await loadVueModule(plugin, styleId), /color:\s*blue/);
}

export function probeHookImplemented(name: string): void {
  const plugins = vize({ configMode: false }) as Plugin[];
  const implementations = plugins.filter(
    (plugin) => typeof (plugin as unknown as Record<string, unknown>)[name] !== "undefined",
  );
  assert.notEqual(implementations.length, 0, `the Vize plugin set must implement \`${name}\``);
}

function probeCachedModuleHook(): void {
  const plugin = (vize({ configMode: false }) as Plugin[]).find(
    (candidate) => candidate.name === "vite-plugin-vize",
  );
  assert.ok(plugin);
  const handler = hook<({ id }: { id: string }) => unknown>(
    (plugin as unknown as Record<string, unknown>).shouldTransformCachedModule,
  );
  assert.equal(handler.call({}, { id: "/project/src/Comp.vue" }), true);
  assert.equal(handler.call({}, { id: "/project/src/main.ts" }), undefined);
}

export const probes = new Map<string, () => Promise<void> | void>([
  ["include-filter", probeIncludeFilter],
  ["exclude-filter", probeExcludeFilter],
  ["custom-element-output", probeCustomElementOutput],
  ["cached-module-hook", probeCachedModuleHook],
  ["production-css-import", probeProductionCssImport],
  ["style-trim", probeStyleTrim],
  ["template-compiler-options", probeTemplateCompilerOptions],
  ["hot-update-style-only", probeHotUpdateStyleOnly],
  ["options-api-feature", probeOptionsApiFeature],
  ["prod-devtools-feature", probeProdDevtoolsFeature],
  ["prod-hydration-mismatch-details-feature", probeProdHydrationMismatchDetailsFeature],
  ["filter-api", probeFilterApi],
  ["filter-api-assignment", probeFilterApiAssignment],
  ["version-api", probeVersionApi],
]);
