import { isPreprocessorLang } from "./style-scope.ts";
import type { ParsedVueRequest, ParsedVueRequestQuery, StyleBlockInfo } from "./types.ts";

export function isVueFile(id: string): boolean {
  return id.endsWith(".vue");
}

export function isJsxFile(id: string): boolean {
  return id.endsWith(".jsx") || id.endsWith(".tsx");
}

export function isVueStyleRequest(id: string): boolean {
  // Style requests always carry the `?vue&type=style` query (the `vue` param is
  // serialized first, so the id always contains `?vue`). Skip URLSearchParams
  // parsing for the overwhelmingly common ids that cannot be a style request.
  if (!id.includes("?vue")) {
    return false;
  }
  const { query } = parseVueRequest(id);
  return query.vue && query.type === "style";
}

/**
 * The module id of a delegated style block, in Vue's request shape:
 * `App.vue?vue&type=style&index=0[&scoped=…][&module[=name]]&lang.<ext>`.
 *
 * CSS pipelines classify a module by the extension that ends the id, and treat
 * a query on a stylesheet path as another import kind (`?raw`, `?inline`), so
 * the path stays the SFC and `<ext>` names what `load` returns: a scoped
 * preprocessor block is compiled before scoping, so it is CSS.
 */
export function styleRequestId(
  filePath: string,
  block: StyleBlockInfo,
  scoped: string | null,
): string {
  const lang = block.lang ?? "css";
  const delivered = scoped && isPreprocessorLang(lang) ? "css" : lang;
  let query = `vue&type=style&index=${block.index}`;
  if (scoped) {
    query += `&scoped=${encodeURIComponent(scoped)}`;
  }
  if (block.module !== false) {
    query +=
      typeof block.module === "string" ? `&module=${encodeURIComponent(block.module)}` : "&module";
  }
  const ext = block.module === false ? delivered : `module.${delivered}`;
  return `${filePath}?${query}&lang.${ext}`;
}

export function parseVueRequest(id: string): ParsedVueRequest {
  const [filename, rawQuery = ""] = id.split("?", 2);
  const params = new URLSearchParams(rawQuery);
  const moduleValue = params.has("module") ? params.get("module") || true : false;
  const indexValue = params.get("index");
  const query: ParsedVueRequestQuery = {
    vue: params.has("vue"),
    type: params.get("type"),
    index: indexValue === null ? null : Number.parseInt(indexValue, 10),
    module: moduleValue,
    scoped: params.get("scoped"),
  };

  return {
    filename,
    query,
  };
}
