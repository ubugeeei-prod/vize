/** Recognize compiled modules without assigning raw Vue/style queries to Vize. */
export function isVizeGeneratedVueModuleId(id: string): boolean {
  const normalized = id.replace(/^\/@id\/__x00__|^__x00__/, "");
  if (/\.vue\.tsx?(?:\?|$)/.test(normalized)) return true;
  const queryIndex = normalized.indexOf("?");
  if (queryIndex === -1 || !normalized.slice(0, queryIndex).endsWith(".vue")) return false;
  const params = new URLSearchParams(normalized.slice(queryIndex + 1));
  return params.has("vue") && (params.has("vize") || params.has("vize-ssr")) && !params.has("type");
}
