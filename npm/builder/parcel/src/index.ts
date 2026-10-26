import path from "node:path";
import { Transformer } from "@parcel/plugin";
import type { TransformerResult } from "@parcel/types";
import { extractSfcStyleBlocks, parseSfc } from "@vizejs/native";
import { fail, loadConfig, type VizeParcelOptions } from "./config.ts";
import { compilationContext } from "./context.ts";
import { compileComponent } from "./component.ts";
import { blockMap } from "./maps.ts";
import { styleRequest } from "./style-request.ts";
import { nativeOperation } from "./trace.ts";

export type { VizeParcelOptions } from "./config.ts";
interface ParsedSfc {
  styles: Array<{ loc: { start: number; end: number } }>;
}

/** Keep authored component execution separate from host stylesheet invalidation. */
export default new Transformer<VizeParcelOptions>({
  loadConfig,
  async transform(args) {
    const { asset, options, tracer } = args;
    if (asset.pipeline === "vize-component") return compileComponent(args);
    const source = await asset.getCode();
    const { filename, scope, isProd, ssr, needsHmr } = compilationContext(
      asset,
      options,
      source,
      tracer,
    );
    const styles = nativeOperation(tracer, "extractSfcStyleBlocks", asset.filePath, () =>
      extractSfcStyleBlocks(source, asset.filePath),
    );
    const descriptor =
      asset.env.sourceMap && !ssr
        ? (nativeOperation(tracer, "parseSfc", asset.filePath, () =>
            parseSfc(source, { filename: asset.filePath }),
          ) as ParsedSfc)
        : undefined;
    const children: TransformerResult[] = [];
    const styleImports: string[] = [];
    for (const style of styles) {
      if (ssr) continue;
      if (style.module)
        fail(
          "SFC CSS modules are not supported by the Parcel adapter yet. Use a separate .module.css import.",
          asset.filePath,
        );
      const language = style.lang ?? "css";
      if (!["css", "scss", "sass", "less", "styl", "stylus"].includes(language))
        fail(`Unsupported style language: ${language}.`, asset.filePath);
      if (style.src) {
        styleImports.push(
          `import ${JSON.stringify(styleRequest(style.src, scope, style.scoped, isProd ? filename : undefined))};`,
        );
      } else {
        const key = `${asset.id}-vize-style-${style.index}`;
        const loc = descriptor?.styles[style.index]?.loc;
        children.push({
          type: language === "styl" ? "stylus" : language,
          content: style.content,
          uniqueKey: key,
          sideEffects: true,
          meta: {
            vizeStyle: {
              scope: style.scoped ? scope : undefined,
              variables: scope,
              productionFilename: isProd ? filename : undefined,
            },
          },
          ...(loc
            ? { map: blockMap(source, loc.start, loc.end, asset.filePath, options.projectRoot) }
            : {}),
        });
        styleImports.push(`import ${JSON.stringify(key)};`);
      }
    }
    const component = `vize-component:./${encodeURIComponent(path.basename(asset.filePath))}`;
    asset.type = "js";
    asset.setMap(null);
    asset.setCode(`${styleImports.join("\n")}
export { default } from ${JSON.stringify(component)};
export * from ${JSON.stringify(component)};
${needsHmr ? "if (module.hot) module.hot.accept();" : ""}`);
    return [asset, ...children];
  },
});
