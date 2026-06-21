import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

export interface VueRuntimeCompilerAlias {
  find: RegExp;
  replacement: string;
}

export function createVueRuntimeCompilerAlias(): VueRuntimeCompilerAlias {
  return { find: /^vue$/, replacement: resolveVueRuntimeCompiler() };
}

function resolveVueRuntimeCompiler(): string {
  try {
    return require.resolve("vue/dist/vue.esm-bundler.js");
  } catch {
    return "vue/dist/vue.esm-bundler.js";
  }
}
