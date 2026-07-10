import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);

export interface VueRuntimeCompilerAlias {
  find: RegExp;
  replacement: string;
}

export interface VueRuntimeCompilerAliasOptions {
  root?: string;
  runtimeCompiler?: string;
}

export function createVueRuntimeCompilerAlias(
  options: VueRuntimeCompilerAliasOptions = {},
): VueRuntimeCompilerAlias {
  return { find: /^vue$/, replacement: resolveVueRuntimeCompiler(options) };
}

function resolveVueRuntimeCompiler(options: VueRuntimeCompilerAliasOptions): string {
  if (options.runtimeCompiler) {
    return normalizeRuntimeCompilerOverride(options.runtimeCompiler, options.root);
  }

  const root = options.root ?? process.cwd();
  const projectRequire = createRequire(path.join(root, "package.json"));
  try {
    return projectRequire.resolve("vue/dist/vue.esm-bundler.js");
  } catch {
    // Fall through to package-local/bare resolution below.
  }

  try {
    return require.resolve("vue/dist/vue.esm-bundler.js");
  } catch {
    return "vue/dist/vue.esm-bundler.js";
  }
}

function normalizeRuntimeCompilerOverride(value: string, root: string | undefined): string {
  if (path.isAbsolute(value) || !value.startsWith(".")) {
    return value;
  }
  return path.resolve(root ?? process.cwd(), value);
}
