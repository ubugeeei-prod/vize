import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

export interface NuxtNitroPluginVariants {
  nitro2: string;
  nitro3: string;
}

interface NuxtServerPluginOptions {
  rootDir: string;
  nitro?: { virtual?: Record<string, string> };
}

/** Keep the stylesheet virtual module and its server plugin registration together. */
export async function registerNuxtDevStylesheets(
  options: NuxtServerPluginOptions,
  devAssetBase: string,
  resolver: { resolve(file: string): string },
  registerLegacy: (plugin: string) => void | Promise<void>,
): Promise<void> {
  options.nitro ||= {};
  options.nitro.virtual ||= {};
  options.nitro.virtual["#vizejs/nuxt/dev-stylesheet-links-config"] =
    `export const devAssetBase = ${JSON.stringify(devAssetBase)};`;
  await registerNuxtServerPlugin(
    options.rootDir,
    {
      nitro2: resolver.resolve("./runtime/server/dev-stylesheet-links"),
      nitro3: resolver.resolve("./runtime/server/dev-stylesheet-links-nitro3"),
    },
    registerLegacy,
  );
}

/** Use the host's server API selection while retaining older Nuxt/Bridge Kit support. */
export async function registerNuxtServerPlugin(
  rootDir: string,
  variants: NuxtNitroPluginVariants,
  registerLegacy: (plugin: string) => void | Promise<void>,
): Promise<void> {
  let entry: string;
  try {
    entry = createRequire(`${rootDir}/package.json`).resolve("nuxt/kit");
  } catch (error) {
    if (
      error instanceof Error &&
      "code" in error &&
      (error.code === "MODULE_NOT_FOUND" || error.code === "ERR_PACKAGE_PATH_NOT_EXPORTED")
    ) {
      await registerLegacy(variants.nitro2);
      return;
    }
    throw error;
  }

  const kit: unknown = await import(pathToFileURL(entry).href);
  if (typeof kit === "object" && kit !== null && "addNitroPlugin" in kit) {
    if (typeof kit.addNitroPlugin !== "function") {
      throw new TypeError("The host nuxt/kit addNitroPlugin export must be a function.");
    }
    kit.addNitroPlugin(variants);
    return;
  }
  await registerLegacy(variants.nitro2);
}
