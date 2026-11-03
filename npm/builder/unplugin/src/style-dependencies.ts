import { parseCssAst } from "@vizejs/native";
import { isPreprocessorLang } from "./style-scope.ts";
import type { StyleBlockInfo } from "./types.ts";

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

/**
 * CSS injected at run time is a string in the page: nothing compiles a
 * preprocessor block or resolves an `@import` there.
 */
export function assertInjectableCss(styles: readonly StyleBlockInfo[], filename: string): void {
  for (const style of styles) {
    // An external `<style src>` is imported, not injected.
    if (style.src) continue;
    if (isPreprocessorLang(style.lang)) {
      throw new Error(
        `[vize] ${filename}: <style lang="${style.lang}"> needs a bundler CSS pipeline. @vizejs/unplugin injects CSS at run time under Babel and esbuild without \`bundle\`, where nothing compiles ${style.lang}. Bundle the build, write the block as CSS, or use @vizejs/vite-plugin.`,
      );
    }
    if (!style.content.includes("@")) {
      continue;
    }
    const parsed: { ast?: unknown } = parseCssAst(style.content, { filename });
    if (hasImportRule(parsed.ast)) {
      throw new Error(
        `[vize] ${filename}: CSS @import in <style> needs a bundler CSS pipeline. @vizejs/unplugin injects CSS at run time under Babel and esbuild without \`bundle\`, where nothing resolves the import. Inline the imported CSS, bundle the build, or use @vizejs/vite-plugin.`,
      );
    }
  }
}
