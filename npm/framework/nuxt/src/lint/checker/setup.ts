import path from "node:path";

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
  options: {
    buildDir: string;
    builder?: unknown;
    dev?: boolean;
    rootDir: string;
    srcDir?: string;
  };
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

function builderKind(builder: unknown): "unsupported" | "vite" | "webpack" {
  if (typeof builder !== "string") return "unsupported";
  if (builder === "vite" || builder.includes("vite-builder")) return "vite";
  if (builder === "webpack" || builder.includes("webpack-builder")) return "webpack";
  return "unsupported";
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
  const builder = builderKind(nuxt.options.builder);

  if (builder === "vite") {
    const register = dependencies.addVitePlugin ?? addVitePlugin;
    await register(createNuxtLintCheckerVitePlugin(config));
  } else if (builder === "webpack") {
    const register = dependencies.addWebpackPlugin ?? addWebpackPlugin;
    await register(createNuxtLintCheckerWebpackPlugin(config));
  } else {
    const label = typeof nuxt.options.builder === "string" ? nuxt.options.builder : "unknown";
    (dependencies.warn ?? console.warn)(
      `Unsupported Nuxt builder ${label}; Vize lint checker is disabled.`,
    );
  }

  return { builder, configFile: generation.configFile, options };
}
