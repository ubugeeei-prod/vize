import { parseCssAst } from "@vizejs/native";
import type { StyleBlockNapi } from "./types.ts";

function hasImportRule(ast: unknown): boolean {
  if (typeof ast !== "object" || ast === null || !("rules" in ast)) {
    return false;
  }
  return (
    Array.isArray(ast.rules) &&
    ast.rules.some(
      (rule: unknown) =>
        typeof rule === "object" && rule !== null && "type" in rule && rule.type === "import",
    )
  );
}

/** The inline runtime style path cannot resolve external CSS dependencies. */
export function assertSupportedStyleDependencies(
  styles: readonly StyleBlockNapi[],
  filename: string,
): void {
  for (const style of styles) {
    // `src` blocks are inlined before compilation; one left here was unreadable.
    if (style.src !== undefined) {
      throw new Error(
        `[vize] ${filename}: <style src=${JSON.stringify(style.src)}> could not be inlined by @vizejs/unplugin. Write it as <style src="./file.css"> with a quoted relative path, or use @vizejs/vite-plugin.`,
      );
    }

    // Modules and preprocessors retain their existing host CSS pipeline. Only
    // plain inline CSS is injected directly into JavaScript by this plugin.
    if (style.module || (style.lang && style.lang !== "css") || !style.content.includes("@")) {
      continue;
    }
    const parsed: { ast?: unknown } = parseCssAst(style.content, { filename });
    if (hasImportRule(parsed.ast)) {
      throw new Error(
        `[vize] ${filename}: CSS @import in <style> is not supported by @vizejs/unplugin's inline style output. Inline the imported CSS, or use @vizejs/vite-plugin for stylesheet dependency resolution.`,
      );
    }
  }
}
