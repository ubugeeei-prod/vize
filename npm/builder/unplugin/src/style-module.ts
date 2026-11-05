// Serves a style module's CSS: the SFC block named by the id, compiled the way
// the id says (see styleRequestId), or a JSX block the compiler already scoped.
import fs from "node:fs";
import { compileJsxModule, compileVueModule } from "./compiler.ts";
import { resolveSrcRequests } from "./host.ts";
import { isJsxFile, parseVueRequest } from "./request.ts";
import {
  isPreprocessorLang,
  preprocessStyle,
  scopeCss,
  transformStyleVars,
} from "./style-scope.ts";
import type {
  CachedCompiledModule,
  CompiledModule,
  NormalizedVizeUnpluginOptions,
  StyleBlockInfo,
} from "./types.ts";

export async function loadStyleBlock(
  context: unknown,
  id: string,
  options: NormalizedVizeUnpluginOptions,
  cache: Map<string, CachedCompiledModule>,
  jsxStyles: Map<string, StyleBlockInfo[]>,
): Promise<string> {
  const request = parseVueRequest(id);
  const index = request.query.index ?? -1;
  if (index < 0) {
    return "";
  }

  if (isJsxFile(request.filename)) {
    // The compiler scoped JSX CSS already.
    const styles =
      jsxStyles.get(request.filename) ??
      (fs.existsSync(request.filename)
        ? compileJsxModule(request.filename, fs.readFileSync(request.filename, "utf8"), options, {
            hostCss: "preprocessors",
          }).styles
        : []);
    return styles[index]?.content ?? "";
  }

  let compiled: CompiledModule | undefined = cache.get(request.filename)?.compiled;

  if (!compiled && fs.existsSync(request.filename)) {
    const source = fs.readFileSync(request.filename, "utf8");
    const resolutions = await resolveSrcRequests(context, request.filename, source);
    compiled = compileVueModule(request.filename, source, options, cache, resolutions).compiled;
  }

  const block = compiled?.styles[index];
  if (!compiled || !block) {
    return "";
  }
  if (block.srcFile) {
    (context as { addWatchFile?: (file: string) => void }).addWatchFile?.(block.srcFile);
  }
  const source = transformStyleVars(
    block.content,
    compiled.scopeId,
    options.isProduction ? request.filename : undefined,
  );

  // Vue order: preprocess, then scope. The id names what this module holds
  // (see styleRequestId). Webpack never loads these ids; its style files go
  // through the host's loaders instead (webpack-styles.ts).
  const css =
    block.lang && isPreprocessorLang(block.lang) && request.query.lang === "css"
      ? await preprocessStyle(source, block.lang, block.srcFile ?? request.filename, options.root)
      : source;
  const scoped = request.query.scoped;
  return scoped ? scopeCss(css, scoped) : css;
}
