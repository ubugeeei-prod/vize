import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

export interface NuxtNitroPluginVariants {
  nitro2: string;
  nitro3: string;
}

/** Use the host's server API selection while retaining older Nuxt/Bridge Kit support. */
export async function registerNuxtServerPlugin(
  rootDir: string,
  variants: NuxtNitroPluginVariants,
  registerLegacy: (plugin: string) => void,
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
      registerLegacy(variants.nitro2);
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
  registerLegacy(variants.nitro2);
}
