import path from "node:path";

import { getDetectedNuxtBuilderKind, getDetectedNuxtMajor } from "../../builder.ts";
import type { NuxtLintConfigGeneration } from "../generation.ts";
import {
  resolveNuxtLintCheckerOptions,
  type ResolvedVizeNuxtLintCheckerOptions,
  type VizeNuxtLintCheckerOptions,
} from "./options.ts";
import { createNuxtLintCheckerVitePlugin, type NuxtLintCheckerVitePlugin } from "./vite.ts";
import {
  createNuxtLintCheckerWebpackPlugin,
  type NuxtLintCheckerWebpackPlugin,
} from "./webpack.ts";

interface NuxtLintCheckerNuxt {
  hook?(name: string, callback: (...args: unknown[]) => unknown): void;
  options: {
    buildDir: string;
    builder?: unknown;
    bridge?: false | { vite?: boolean };
    dev?: boolean;
    rootDir: string;
    srcDir?: string;
  };
}

function registerNuxt2Checker(
  nuxt: NuxtLintCheckerNuxt,
  builder: "vite" | "webpack",
  plugin: NuxtLintCheckerVitePlugin | NuxtLintCheckerWebpackPlugin,
): void {
  if (!nuxt.hook) throw new Error("Nuxt 2 checker registration requires the host hook API.");
  // A classic Nuxt 2 module receives its instance through `this.nuxt`; it does
  // not establish the implicit Nuxt Kit context used by modern modules.
  if (builder === "webpack") {
    nuxt.hook("webpack:config", (value) => {
      for (const config of value as Array<{ name?: string; plugins?: unknown[] }>) {
        if (config.name === "client") (config.plugins ??= []).push(plugin);
      }
    });
  } else {
    nuxt.hook("vite:extendConfig", (value, context) => {
      if ((context as { isServer?: boolean } | undefined)?.isServer) return;
      const config = value as { plugins?: unknown[] };
      (config.plugins ??= []).push(plugin);
    });
  }
}

type Awaitable<T> = T | Promise<T>;

export interface NuxtLintCheckerSetupDependencies {
  addVitePlugin?: (plugin: NuxtLintCheckerVitePlugin) => Awaitable<void>;
  addWebpackPlugin?: (plugin: NuxtLintCheckerWebpackPlugin) => Awaitable<void>;
  warn?: (message: string) => void;
}

export interface NuxtLintCheckerSetup {
  builder: "unsupported" | "vite" | "webpack";
  configFile: string;
  options: ResolvedVizeNuxtLintCheckerOptions;
}

interface NuxtKitPluginRegistration {
  addVitePlugin(plugin: unknown, options?: { server?: boolean }): void;
  addWebpackPlugin(plugin: unknown, options?: { server?: boolean }): void;
}

async function addVitePlugin(plugin: NuxtLintCheckerVitePlugin): Promise<void> {
  const kit = (await import("@nuxt/kit")) as unknown as NuxtKitPluginRegistration;
  kit.addVitePlugin(plugin, { server: false });
}

async function addWebpackPlugin(plugin: NuxtLintCheckerWebpackPlugin): Promise<void> {
  const kit = (await import("@nuxt/kit")) as unknown as NuxtKitPluginRegistration;
  kit.addWebpackPlugin(plugin, { server: false });
}

/** Register the dev-only adapter over Phase 3's generated config artifact. */
export async function setupNuxtLintChecker(
  checker: boolean | VizeNuxtLintCheckerOptions | undefined,
  nuxt: NuxtLintCheckerNuxt,
  generation: NuxtLintConfigGeneration | undefined,
  dependencies: NuxtLintCheckerSetupDependencies = {},
): Promise<NuxtLintCheckerSetup | undefined> {
  if (checker !== true && (checker === false || checker == null)) return undefined;
  if (nuxt.options.dev !== true) return undefined;
  if (!generation) {
    throw new Error("The Nuxt lint checker requires lint config generation; enable `vize.lint`.");
  }

  const options = resolveNuxtLintCheckerOptions(checker, {
    buildDir: nuxt.options.buildDir,
    srcDir: nuxt.options.srcDir ?? nuxt.options.rootDir,
  });
  if (options === false) return undefined;
  const rootDir = path.resolve(nuxt.options.rootDir);
  const config = { configFile: generation.configFile, options, rootDir };
  const builder = getDetectedNuxtBuilderKind(nuxt);

  if (builder === "vite") {
    const plugin = createNuxtLintCheckerVitePlugin(config);
    if (!dependencies.addVitePlugin && getDetectedNuxtMajor(nuxt) === 2) {
      registerNuxt2Checker(nuxt, builder, plugin);
    } else {
      await (dependencies.addVitePlugin ?? addVitePlugin)(plugin);
    }
  } else if (builder === "webpack") {
    const plugin = createNuxtLintCheckerWebpackPlugin(config);
    if (!dependencies.addWebpackPlugin && getDetectedNuxtMajor(nuxt) === 2) {
      registerNuxt2Checker(nuxt, builder, plugin);
    } else {
      await (dependencies.addWebpackPlugin ?? addWebpackPlugin)(plugin);
    }
  } else {
    const label = typeof nuxt.options.builder === "string" ? nuxt.options.builder : "unknown";
    (dependencies.warn ?? console.warn)(
      `Unsupported Nuxt builder ${label}; Vize lint checker is disabled.`,
    );
  }

  return { builder, configFile: generation.configFile, options };
}
