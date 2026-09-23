/** Main .vue SFC loader. Compiles SFC → JS; must be used with `oneOf` (mutual exclusion with style-loader). */

import { sources, type LoaderContext } from "@rspack/core";
import fs from "node:fs";
import path from "node:path";
import { compileFile, generateOutputWithMap } from "../shared/compiler.ts";
import { matchesPattern, extractSrcInfo, extractCustomBlocks } from "../shared/utils.ts";
import { resolveNativeCss as resolveNativeCssMode } from "../shared/nativeCss.ts";
import { inlineSrcBlock, MappedModule } from "../shared/source-map.ts";
import type { VizeSfcLoaderOptions as VizeLoaderOptions } from "../types/index.ts";

/** .ce.vue → custom element */
const DEFAULT_CE_PATTERN = /\.ce\.vue$/;

export default function vizeLoader(this: LoaderContext<VizeLoaderOptions>, source: string): void {
  const callback = this.async();
  const options = this.getOptions();
  const resourcePath = this.resourcePath;
  const resourceQuery = this.resourceQuery;
  const requestPath = normalizeRequestPath(this, resourcePath);

  const isProduction =
    options.isProduction ?? (this.mode === "production" || process.env.NODE_ENV === "production");
  const isSsr = options.ssr ?? options.compilerOptions?.ssr ?? false;
  const needsHotReload = !isSsr && !isProduction && options.hotReload !== false;
  const nativeCss = resolveNativeCss(this, options);
  const sourceMap =
    options.sourceMap ?? options.compilerOptions?.sourceMap ?? this.sourceMap ?? !isProduction;

  this.addDependency(resourcePath);

  if (resourceQuery?.includes("type=style")) {
    callback(
      new Error(
        `[vize] Main loader received style sub-request: ${resourcePath}${resourceQuery}. ` +
          `Use module.rules[].oneOf with resourceQuery branches so style requests are handled by @vizejs/rspack-plugin/style-loader.`,
      ),
    );
    return;
  }

  // Custom block sub-requests (e.g. ?vue&type=i18n&index=0)
  if (
    resourceQuery &&
    resourceQuery.includes("vue") &&
    resourceQuery.includes("type=") &&
    !resourceQuery.includes("type=style")
  ) {
    const params = new URLSearchParams(resourceQuery.slice(1));
    const blockType = params.get("type");
    if (blockType && blockType !== "style") {
      const blockIndex = parseInt(params.get("index") || "0", 10);
      const customBlocks = extractCustomBlocks(source);
      const block = customBlocks[blockIndex];
      if (block) {
        if (block.src) {
          const blockPath = path.resolve(path.dirname(resourcePath), block.src);
          this.addDependency(blockPath);
          try {
            const blockContent = fs.readFileSync(blockPath, "utf-8");
            callback(null, blockContent);
          } catch {
            callback(
              new Error(
                `[vize] Custom block <${blockType} src="${block.src}"> not found (resolved: ${blockPath}) in ${resourcePath}`,
              ),
            );
          }
          return;
        }
        callback(null, block.content);
      } else {
        callback(null, "");
      }
      return;
    }
  }

  if (!shouldCompileFile(resourcePath, options)) {
    this.emitWarning(
      new Error(
        `[vize] File is filtered out by loader options include/exclude: ${resourcePath}. ` +
          `Passing through source unchanged.`,
      ),
    );
    callback(null, source);
    return;
  }

  try {
    const isCustomElement = resolveCustomElement(resourcePath, options.customElement);

    // Resolve external src references
    const srcInfo = extractSrcInfo(source);
    const resolvedSource = new MappedModule(
      source,
      sourceMap && (srcInfo.scriptSrc || srcInfo.templateSrc)
        ? new sources.OriginalSource(source, resourcePath).map({ columns: true })
        : null,
    );

    if (srcInfo.scriptSrc) {
      const scriptPath = path.resolve(path.dirname(resourcePath), srcInfo.scriptSrc);
      this.addDependency(scriptPath);
      try {
        const scriptContent = fs.readFileSync(scriptPath, "utf-8");
        inlineSrcBlock(resolvedSource, "script", scriptContent, scriptPath);
      } catch {
        callback(
          new Error(
            `[vize] <script src="${srcInfo.scriptSrc}"> not found (resolved: ${scriptPath}) in ${resourcePath}`,
          ),
        );
        return;
      }
    }

    if (srcInfo.templateSrc) {
      const templatePath = path.resolve(path.dirname(resourcePath), srcInfo.templateSrc);
      this.addDependency(templatePath);
      try {
        const templateContent = fs.readFileSync(templatePath, "utf-8");
        inlineSrcBlock(resolvedSource, "template", templateContent, templatePath);
      } catch {
        callback(
          new Error(
            `[vize] <template src="${srcInfo.templateSrc}"> not found (resolved: ${templatePath}) in ${resourcePath}`,
          ),
        );
        return;
      }
    }

    const rootContext = options.root
      ? path.resolve(this.rootContext, options.root)
      : this.rootContext;
    const compiled = compileFile(resourcePath, resolvedSource.code, {
      sourceMap,
      ssr: isSsr,
      vapor: options.vapor ?? options.compilerOptions?.vapor ?? false,
      compilerOptions: options.compilerOptions,
      isCustomElement,
      rootContext,
      isProduction,
      transformAssetUrls: options.transformAssetUrls,
    });

    for (const warning of compiled.warnings) {
      this.emitWarning(new Error(`[vize] ${warning}`));
    }

    if (compiled.errors.length > 0) {
      for (const error of compiled.errors) {
        this.emitError(new Error(`[vize] ${error}`));
      }
      const errorSummary = compiled.errors.join("\\n");
      callback(new Error(`[vize] Compilation failed for ${resourcePath}:\n${errorSummary}`));
      return;
    }

    const output = generateOutputWithMap(compiled, {
      requestPath,
      hmr: needsHotReload,
      filePath: resourcePath,
      isProduction,
      rootContext,
      nativeCss,
    });

    const map =
      output.map && resolvedSource.map
        ? new sources.SourceMapSource(
            output.code,
            resourcePath,
            JSON.stringify(output.map),
            resolvedSource.code,
            JSON.stringify(resolvedSource.map),
            true,
          ).map({ columns: true })
        : output.map;
    callback(null, output.code, map ? JSON.stringify(map) : undefined);
  } catch (error) {
    callback(error as Error);
  }
}

function shouldCompileFile(file: string, options: VizeLoaderOptions): boolean {
  if (!matchesPattern(file, options.include, true)) {
    return false;
  }

  if (matchesPattern(file, options.exclude, false)) {
    return false;
  }

  return true;
}

function resolveNativeCss(
  loader: LoaderContext<VizeLoaderOptions>,
  options: VizeLoaderOptions,
): boolean {
  const compiler = (
    loader as unknown as {
      _compiler?: { options?: unknown; webpack?: { rspackVersion?: string } };
    }
  )._compiler;
  return resolveNativeCssMode(
    options.css?.native,
    compiler?.options,
    compiler?.webpack?.rspackVersion,
  );
}

/** Resolve custom element mode for a file. */
function resolveCustomElement(
  resourcePath: string,
  customElement: boolean | RegExp | undefined,
): boolean {
  if (customElement === true) return true;
  if (customElement === false || customElement === undefined) {
    return DEFAULT_CE_PATTERN.test(resourcePath);
  }
  return customElement.test(resourcePath);
}

/** Returns `./basename.vue` for style sub-import paths. */
function normalizeRequestPath(
  _context: LoaderContext<VizeLoaderOptions>,
  resourcePath: string,
): string {
  const basename = path.basename(resourcePath);
  return `./${basename}`;
}
