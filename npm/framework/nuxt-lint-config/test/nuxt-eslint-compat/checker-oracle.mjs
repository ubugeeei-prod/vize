/** Record the portable checker contract from the pinned `@nuxt/eslint`. */
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

function replaceExactlyOnce(source, needle, replacement) {
  const first = source.indexOf(needle);
  if (first < 0 || source.indexOf(needle, first + needle.length) >= 0) {
    throw new Error(`Unable to instrument the pinned checker at: ${needle}`);
  }
  return `${source.slice(0, first)}${replacement}${source.slice(first + needle.length)}`;
}

/** Execute the pinned checker while intercepting only its builder adapters. */
async function loadCheckerSetup(moduleEntry) {
  const checkerFile = fileURLToPath(new URL("./chunks/checker.mjs", moduleEntry));
  const temporary = mkdtempSync(join(dirname(checkerFile), ".vize-checker-oracle-"));
  const key = `__vizeNuxtCheckerOracle${process.pid}${Date.now()}`;
  let source = readFileSync(checkerFile, "utf8");
  source = replaceExactlyOnce(
    source,
    "import { addVitePlugin, addWebpackPlugin, useLogger } from '@nuxt/kit';",
    [
      `const __oracle = () => globalThis[${JSON.stringify(key)}];`,
      "const addVitePlugin = (...args) => __oracle().addVitePlugin(...args);",
      "const addWebpackPlugin = (...args) => __oracle().addWebpackPlugin(...args);",
      "const useLogger = (...args) => __oracle().useLogger(...args);",
    ].join("\n"),
  );
  source = replaceExactlyOnce(
    source,
    "await import('vite-plugin-eslint2')",
    `await Promise.resolve({ default: globalThis[${JSON.stringify(key)}].vitePlugin })`,
  );
  writeFileSync(join(temporary, "checker.mjs"), source);
  globalThis[key] = {
    addVitePlugin() {},
    addWebpackPlugin() {},
    useLogger: () => ({ info() {}, warn() {} }),
    vitePlugin: () => ({ name: "checker-oracle" }),
  };
  try {
    const setup = (await import(join(temporary, "checker.mjs"))).setupESLintChecker;
    return { key, setup };
  } catch (error) {
    delete globalThis[key];
    throw error;
  } finally {
    rmSync(temporary, { force: true, recursive: true });
  }
}

/** Record the complete portable checker option object produced upstream. */
export async function recordCheckerOptions(moduleEntry, cases) {
  const { key, setup } = await loadCheckerSetup(moduleEntry);
  const results = {};
  try {
    for (const entry of cases) {
      let captured;
      globalThis[key] = {
        addVitePlugin(factory) {
          factory();
        },
        addWebpackPlugin() {
          throw new Error("checker oracle unexpectedly selected webpack");
        },
        useLogger: () => ({ info() {}, warn() {} }),
        vitePlugin(options) {
          captured = structuredClone(options);
          return { name: "checker-oracle" };
        },
      };
      const nuxt = {
        options: {
          buildDir: "/project/.nuxt",
          builder: "@nuxt/vite-builder",
          rootDir: "/project",
          srcDir: "/project/app",
          watch: [],
        },
        hook() {},
      };
      await setup({ checker: structuredClone(entry.checker) }, nuxt);
      if (!captured) throw new Error(`upstream checker did not register for ${entry.id}`);
      results[entry.id] = {
        cache: captured.cache,
        include: captured.include,
        exclude: captured.exclude,
        formatter: captured.formatter,
        lintOnStart: captured.lintOnStart,
        emitWarning: captured.emitWarning,
        emitError: captured.emitError,
        fix: captured.fix ?? false,
        worker: captured.lintInWorker,
      };
    }
  } finally {
    delete globalThis[key];
  }
  return results;
}
