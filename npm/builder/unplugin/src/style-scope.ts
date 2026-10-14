// Scoped styles follow Vue's order: preprocess first, then scope the CSS.
// Webpack keeps the host's preprocessor loaders (see webpack-style-loader.ts);
// Rollup, Rolldown and esbuild have no shared preprocessor pipeline, so the
// plugin runs the project's own sass, less or stylus before scoping.
import { createRequire } from "node:module";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { scopeViteCssForPipeline, transformViteCssVarsForPipeline } from "@vizejs/native";

const PREPROCESSOR_LANGS = new Set(["scss", "sass", "less", "stylus", "styl"]);

export function isPreprocessorLang(lang: string | null | undefined): boolean {
  return lang != null && PREPROCESSOR_LANGS.has(lang);
}

/** Bundlers whose virtual style module must already be plain CSS. */
export function preprocessesInPlugin(framework: string): boolean {
  return framework !== "webpack" && framework !== "rspack";
}

export function scopeCss(css: string, scoped: string): string {
  return scopeViteCssForPipeline(css, scoped);
}

/**
 * `v-bind()` in a delegated block, named like the compiled `useCssVars` call.
 * The plugin compiles without `isProd`, so both sides use the dev names.
 */
export function transformStyleVars(css: string, scopeId: string): string {
  return transformViteCssVarsForPipeline(css, scopeId);
}

interface SassModule {
  compileString(
    source: string,
    options: { syntax: "scss" | "indented"; url: URL; loadPaths: string[] },
  ): { css: string };
}

interface LessModule {
  render(source: string, options: { filename: string; paths: string[] }): Promise<{ css: string }>;
}

type StylusModule = (source: string) => {
  set(key: string, value: string): ReturnType<StylusModule>;
  include(dir: string): ReturnType<StylusModule>;
  render(): string;
};

function loadPreprocessor<T>(root: string, names: string[], lang: string): T {
  const requireFromRoot = createRequire(path.join(root, "package.json"));
  for (const name of names) {
    try {
      const loaded = requireFromRoot(name) as T & { default?: T };
      return loaded.default ?? loaded;
    } catch (error) {
      if ((error as { code?: string }).code !== "MODULE_NOT_FOUND") {
        throw error;
      }
    }
  }
  throw new Error(
    `[vize] <style scoped lang="${lang}"> needs ${names.join(" or ")} installed in ${root}.`,
  );
}

export async function preprocessStyle(
  source: string,
  lang: string,
  filename: string,
  root: string,
): Promise<string> {
  const dir = path.dirname(filename);
  if (lang === "scss" || lang === "sass") {
    const sass = loadPreprocessor<SassModule>(root, ["sass", "sass-embedded"], lang);
    return sass.compileString(source, {
      syntax: lang === "sass" ? "indented" : "scss",
      url: pathToFileURL(filename),
      loadPaths: [dir, root],
    }).css;
  }
  if (lang === "less") {
    const less = loadPreprocessor<LessModule>(root, ["less"], lang);
    return (await less.render(source, { filename, paths: [dir, root] })).css;
  }
  const stylus = loadPreprocessor<StylusModule>(root, ["stylus"], lang);
  return stylus(source).set("filename", filename).include(dir).render();
}
