import { createHash } from "node:crypto";
import path from "node:path";
import type { CompiledModule, StyleBlockInfo } from "./types.js";

const PREPROCESSOR_LANGS = new Set(["scss", "sass", "less", "stylus", "styl"]);

function needsPreprocessor(block: StyleBlockInfo): boolean {
  return block.lang !== null && PREPROCESSOR_LANGS.has(block.lang);
}

function isCssModule(block: StyleBlockInfo): boolean {
  return block.module !== false;
}

export function hasDelegatedStyles(compiled: CompiledModule): boolean {
  return compiled.styles.some((style) => needsPreprocessor(style) || isCssModule(style));
}

export function generateScopeId(
  filename: string,
  root: string,
  isProduction: boolean,
  source: string,
): string {
  const relative = path
    .relative(root, filename)
    .replace(/^(\.\.[/\\])+/, "")
    .replace(/\\/g, "/");
  const input = isProduction ? `${relative}\n${source.replace(/\r\n/g, "\n")}` : relative;
  return createHash("sha256").update(input).digest("hex").slice(0, 8);
}

export function extractStyleBlocks(source: string): StyleBlockInfo[] {
  const blocks: StyleBlockInfo[] = [];
  const styleRegex = /<style([^>]*)>([\s\S]*?)<\/style>/gi;
  let match: RegExpExecArray | null = null;
  let index = 0;

  while ((match = styleRegex.exec(source)) !== null) {
    const attrs = match[1];
    const content = match[2];
    const src = attrs.match(/\bsrc=["']([^"']+)["']/)?.[1] ?? null;
    const lang = attrs.match(/\blang=["']([^"']+)["']/)?.[1] ?? null;
    const scoped = /\bscoped\b/.test(attrs);
    const moduleMatch = attrs.match(/\bmodule(?:=["']([^"']+)["'])?/);
    const moduleValue = moduleMatch ? moduleMatch[1] || true : false;

    blocks.push({
      content,
      src,
      lang,
      scoped,
      module: moduleValue,
      index,
    });

    index++;
  }

  return blocks;
}

function supportsTemplateOnlyHmr(output: string): boolean {
  return /(?:^|\n)(?:_sfc_main|__sfc__)\.render\s*=\s*render\b/m.test(output);
}

export interface GenerateOutputOptions {
  isProduction: boolean;
  isDev: boolean;
  extractCss?: boolean;
  filePath?: string;
}

export function generateOutput(compiled: CompiledModule, options: GenerateOutputOptions): string {
  const { isProduction, isDev, extractCss, filePath } = options;

  let output = compiled.code;
  const exportDefaultRegex = /^export default /m;
  const hasExportDefault = exportDefaultRegex.test(output);
  const hasNamedRenderExport = /^export function render\b/m.test(output);
  const hasSfcMainDefined = /\bconst\s+_sfc_main\s*=/.test(output);

  if (hasExportDefault && !hasSfcMainDefined) {
    output = output.replace(exportDefaultRegex, "const _sfc_main = ");
    if (compiled.hasScoped) {
      output += `\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`;
    }
    output += "\nexport default _sfc_main;";
  } else if (hasExportDefault && hasSfcMainDefined && compiled.hasScoped) {
    output = output.replace(
      /^export default _sfc_main/m,
      `_sfc_main.__scopeId = "data-v-${compiled.scopeId}";\nexport default _sfc_main`,
    );
  } else if (!hasExportDefault && !hasSfcMainDefined && hasNamedRenderExport) {
    output += "\nconst _sfc_main = {};";
    if (compiled.hasScoped) {
      output += `\n_sfc_main.__scopeId = "data-v-${compiled.scopeId}";`;
    }
    output += "\n_sfc_main.render = render;";
    output += "\nexport default _sfc_main;";
  }

  const useDelegatedStyles = hasDelegatedStyles(compiled) && filePath;

  if (useDelegatedStyles) {
    const styleImports: string[] = [];
    const cssModuleImports: string[] = [];

    for (const block of compiled.styles) {
      const lang = block.lang ?? "css";
      const params = new URLSearchParams();
      params.set("vue", "");
      params.set("type", "style");
      params.set("index", String(block.index));
      params.set("lang", lang);

      if (block.scoped) {
        params.set("scoped", `data-v-${compiled.scopeId}`);
      }

      const importUrl = `${filePath}?${params.toString()}`;

      if (isCssModule(block)) {
        const bindingName = typeof block.module === "string" ? block.module : "$style";
        const moduleParams = new URLSearchParams(params);
        moduleParams.set("module", typeof block.module === "string" ? block.module : "");
        cssModuleImports.push(
          `import ${bindingName} from ${JSON.stringify(`${filePath}?${moduleParams.toString()}`)};`,
        );
      } else {
        styleImports.push(`import ${JSON.stringify(importUrl)};`);
      }
    }

    const allImports = [...styleImports, ...cssModuleImports].join("\n");
    if (allImports) {
      output = `${allImports}\n${output}`;
    }

    if (cssModuleImports.length > 0) {
      const cssModuleSetup = compiled.styles
        .filter((block) => isCssModule(block))
        .map((block) => {
          const bindingName = typeof block.module === "string" ? block.module : "$style";
          return `_sfc_main.__cssModules = _sfc_main.__cssModules || {};\n_sfc_main.__cssModules[${JSON.stringify(bindingName)}] = ${bindingName};`;
        })
        .join("\n");

      output = output.replace(
        /^export default _sfc_main;/m,
        `${cssModuleSetup}\nexport default _sfc_main;`,
      );
    }
  } else if (compiled.css && !(isProduction && extractCss)) {
    const cssCode = JSON.stringify(compiled.css);
    const cssId = JSON.stringify(`vize-style-${compiled.scopeId}`);

    output = `
export const __vize_css__ = ${cssCode};
const __vize_css_id__ = ${cssId};
(function() {
  if (typeof document !== "undefined") {
    let style = document.getElementById(__vize_css_id__);
    if (!style) {
      style = document.createElement("style");
      style.id = __vize_css_id__;
      style.textContent = __vize_css__;
      document.head.appendChild(style);
    } else {
      style.textContent = __vize_css__;
    }
  }
})();
${output}`;
  }

  if (!isProduction && isDev && hasExportDefault && supportsTemplateOnlyHmr(output)) {
    output += "";
  }

  return output;
}

export function wrapScopedPreprocessorStyle(
  content: string,
  scoped: string | null,
  lang: string | null,
): string {
  if (!scoped || !lang || lang === "css") {
    return content;
  }

  const lines = content.split("\n");
  const hoisted: string[] = [];
  const body: string[] = [];

  for (const line of lines) {
    const trimmed = line.trimStart();
    if (
      trimmed.startsWith("@use ") ||
      trimmed.startsWith("@forward ") ||
      trimmed.startsWith("@import ")
    ) {
      hoisted.push(line);
      continue;
    }

    body.push(line);
  }

  const hoistedContent = hoisted.length > 0 ? `${hoisted.join("\n")}\n\n` : "";
  return `${hoistedContent}[${scoped}] {\n${body.join("\n")}\n}`;
}
