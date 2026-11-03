import { isPreprocessorLang } from "./style-scope.ts";
import type { HostCss, ParsedVueRequest, ParsedVueRequestQuery, StyleBlockInfo } from "./types.ts";

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
 * the path stays the SFC and `<ext>` names what `load` returns. A preprocessor
 * block arrives compiled when it is scoped (scoping follows preprocessing) or
 * when the host compiles no preprocessor language.
 *
 * A `<style src>` block's module sits beside its src file, as
 * `styles/a.css.__vize_style_0?vue&type=style&index=0&owner=<SFC>&lang.css`,
 * so every host resolves the file's relative `@import`, `@use` and `url()`
 * from its own directory. The path names no real file and no SFC; `owner`
 * names the SFC that holds the block.
 */
export function styleRequestId(
  filePath: string,
  block: StyleBlockInfo,
  scoped: string | null,
  hostCss: HostCss,
): string {
  const lang = block.lang ?? "css";
  const compiled = scoped !== null || hostCss !== "preprocessors";
  const delivered = compiled && isPreprocessorLang(lang) ? "css" : lang;
  let query = `vue&type=style&index=${block.index}`;
  if (block.srcFile) {
    query += `&owner=${encodeURIComponent(filePath)}`;
  }
  if (scoped) {
    query += `&scoped=${encodeURIComponent(scoped)}`;
  }
  if (block.module !== false) {
    query +=
      typeof block.module === "string" ? `&module=${encodeURIComponent(block.module)}` : "&module";
  }
  const ext = block.module === false ? delivered : `module.${delivered}`;
  const path = block.srcFile ? styleSrcModulePath(block.srcFile, block.index) : filePath;
  return `${path}?${query}&lang.${ext}`;
}

/** Where a `<style src>` block's module sits: beside the src file, under no real name. */
export function styleSrcModulePath(src: string, index: number): string {
  return `${src}.__vize_style_${index}`;
}

export function parseVueRequest(id: string): ParsedVueRequest {
  const [path, rawQuery = ""] = id.split("?", 2);
  const params = new URLSearchParams(rawQuery);
  // A `<style src>` module's path is beside its src file (see styleRequestId).
  const filename = params.get("owner") ?? path;
  const moduleValue = params.has("module") ? params.get("module") || true : false;
  const indexValue = params.get("index");
  const langKey = [...params.keys()].find((key) => key.startsWith("lang."));
  const query: ParsedVueRequestQuery = {
    vue: params.has("vue"),
    type: params.get("type"),
    index: indexValue === null ? null : Number.parseInt(indexValue, 10),
    lang: langKey?.replace(/^lang\.(?:module\.)?/, "") ?? null,
    module: moduleValue,
    scoped: params.get("scoped"),
  };

  return {
    filename,
    query,
  };
}
