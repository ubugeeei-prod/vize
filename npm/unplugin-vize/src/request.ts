import type { ParsedVueRequest, ParsedVueRequestQuery } from "./types.ts";

const STYLE_MARKER = ".__vize_style_";

export function isVueFile(id: string): boolean {
  return id.endsWith(".vue");
}

export function isVueStyleRequest(id: string): boolean {
  const { query } = parseVueRequest(id);
  return query.vue && query.type === "style";
}

export function isVirtualStyleId(id: string): boolean {
  return id.includes(STYLE_MARKER);
}

export function parseVueRequest(id: string): ParsedVueRequest {
  const [path, rawQuery = ""] = id.split("?", 2);
  const params = new URLSearchParams(rawQuery);
  const filename = params.get("vize-file") ?? path;
  const moduleValue = params.has("module") ? params.get("module") || true : false;
  const indexValue = params.get("index");
  const query: ParsedVueRequestQuery = {
    vue: params.has("vue"),
    type: params.get("type"),
    index: indexValue === null ? null : Number.parseInt(indexValue, 10),
    lang: params.get("lang"),
    module: moduleValue,
    scoped: params.get("scoped"),
    vizeFile: params.get("vize-file"),
  };

  return {
    filename,
    path,
    query,
  };
}

export function createVirtualStyleId(id: string): string {
  const { filename, query } = parseVueRequest(id);
  const index = query.index ?? 0;
  const lang = query.lang ?? "css";
  const suffix = query.module !== false ? `.module.${lang}` : `.${lang}`;
  const params = new URLSearchParams();

  params.set("vue", "");
  params.set("type", "style");
  params.set("index", String(index));
  params.set("lang", lang);
  params.set("vize-file", filename);

  if (query.scoped) {
    params.set("scoped", query.scoped);
  }

  if (query.module !== false) {
    params.set("module", typeof query.module === "string" ? query.module : "");
  }

  return `${filename}${STYLE_MARKER}${index}${suffix}?${params.toString()}`;
}
