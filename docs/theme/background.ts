import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const SCRIPT_BASENAMES = ["vein", "syntax-highlight"];
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
  return `<canvas id="${docsBackgroundCanvasId}" style="position:fixed;top:0;left:0;width:100%;height:100%;z-index:-1;pointer-events:none;"></canvas>`;
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
