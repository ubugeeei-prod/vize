import path from "node:path";
import {
  createNuxtComponentResolver,
  injectNuxtComponentImports,
} from "../../../../npm/framework/nuxt/src/components.ts";
import { createNuxtImportOracle, nuxtTransform } from "./component-import-oracle.ts";
import type { FixturePlugin, ModuleTransformOptions } from "./oracle-support.ts";

export interface BridgeModule {
  readonly id: string;
  readonly ssr: boolean;
  readonly input: string;
  readonly output: string;
}

export function createComponentBridge(
  producer: string,
  { entry, heading, modules }: { entry: string; heading: string; modules: BridgeModule[] },
): FixturePlugin {
  const components = [{ pascalName: "Heading", filePath: heading }];
  const official = producer === "official" ? createNuxtImportOracle(components) : null;
  const resolver = createNuxtComponentResolver({
    rootDir: path.dirname(entry),
    buildDir: path.dirname(entry),
  });
  resolver.register(components);
  return {
    name: "fixture:actual-nuxt-component-imports",
    enforce: "post",
    async transform(input: string, id: string, options?: ModuleTransformOptions) {
      if (id.split("?")[0] !== entry) return undefined;
      const output = official
        ? await nuxtTransform(official, input, id)
        : injectNuxtComponentImports(input, (name) => resolver.resolve(name));
      modules.push({ id, ssr: Boolean(options?.ssr), input, output });
      return { code: output, map: null };
    },
  };
}
