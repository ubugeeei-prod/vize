import { readFileSync } from "node:fs";
import { resolve } from "node:path";

/**
 * Theme scripts, concatenated in this order into one inline `<script>`.
 *
 * The docs site has no module loader, so the i18n files talk to each other
 * through globals and the order here is their dependency order: the sitemap
 * and every locale's strings register themselves before `i18n/navigation`
 * reads them. `docs-navigation.test.ts` pins this list against the locales the
 * site actually ships.
 */
export const SCRIPT_BASENAMES = [
  "vein",
  "i18n/sitemap",
  "i18n/locales/en",
  "i18n/locales/ja",
  "i18n/locales/zh-CN",
  "i18n/locales/pt-BR",
  "i18n/locales/fr",
  "i18n/navigation",
  "syntax-highlight",
];
const VERTEX_SHADER_PLACEHOLDER = "__VERT_SRC__";
const FRAGMENT_SHADER_PLACEHOLDER = "__FRAG_SRC__";

export const docsBackgroundCanvasId = "vein-canvas";

function escapeTemplateLiteral(source: string): string {
  return source.replaceAll("\\", "\\\\").replaceAll("`", "\\`");
}

function ensureStatementBoundary(source: string): string {
  return source.startsWith(";") ? source : `;${source}`;
}

export function createDocsBackgroundHtml(): string {
  return `<canvas id="${docsBackgroundCanvasId}" style="position:fixed;top:0;left:0;width:100%;height:100%;z-index:0;pointer-events:none;"></canvas>`;
}

export function buildDocsBackgroundScript(themeDir: string): string {
  const shaderDir = resolve(themeDir, "shaders");
  const vertSrc = readFileSync(resolve(shaderDir, "vein.vert"), "utf-8");
  const fragSrc = readFileSync(resolve(shaderDir, "vein.frag"), "utf-8");
  const scripts = SCRIPT_BASENAMES.map((basename) =>
    readFileSync(resolve(themeDir, `${basename}.js`), "utf-8"),
  );

  return scripts
    .map((script) => ensureStatementBoundary(script))
    .join("\n\n")
    .replace(VERTEX_SHADER_PLACEHOLDER, escapeTemplateLiteral(vertSrc))
    .replace(FRAGMENT_SHADER_PLACEHOLDER, escapeTemplateLiteral(fragSrc));
}
