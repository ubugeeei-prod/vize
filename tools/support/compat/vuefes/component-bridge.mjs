import path from "node:path";
import {
  createNuxtComponentResolver,
  injectNuxtComponentImports,
} from "../../../../npm/framework/nuxt/src/components.ts";
import { createNuxtImportOracle } from "./component-import-oracle.mjs";

export function createComponentBridge(producer, { entry, heading, modules }) {
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
    async transform(input, id, options) {
      if (id.split("?")[0] !== entry) return;
      const output = official
        ? (await official.transform(input, id)).code
        : injectNuxtComponentImports(input, (name) => resolver.resolve(name));
      modules.push({ id, ssr: Boolean(options?.ssr), input, output });
      return { code: output, map: null };
    },
  };
}
