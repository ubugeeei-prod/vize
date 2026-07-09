import fs from "node:fs";
import path from "node:path";
import * as vite from "vite";
import type { TransformResult } from "vite";

import type { VizePluginState } from "./state.ts";
import { applyDefineReplacements } from "../transform.ts";

type TransformOutput = { code: string; map?: unknown };
type OxcOptions = { lang: "ts"; sourcemap: false; target: "esnext" };
type EsbuildOptions = { loader: "ts"; sourcemap: false; target: "esnext" };
type TransformWithOxc = (
  code: string,
  id: string,
  options: OxcOptions,
) => TransformOutput | Promise<TransformOutput>;
type TransformWithEsbuild = (
  code: string,
  id: string,
  options: EsbuildOptions,
) => TransformOutput | Promise<TransformOutput>;

interface ViteTransformApi {
  transformWithOxc?: TransformWithOxc;
  transformWithEsbuild?: TransformWithEsbuild;
}

export function createVirtualTypeScriptTransformer(viteApi: ViteTransformApi) {
  return async (code: string, id: string): Promise<TransformOutput> => {
    // This pass only strips TypeScript from Vize virtual Vue modules.
    // Syntax lowering belongs to Vite's normal build target transform.
    if (typeof viteApi.transformWithOxc === "function") {
      return viteApi.transformWithOxc(code, id, {
        lang: "ts",
        sourcemap: false,
        target: "esnext",
      });
    }
    if (typeof viteApi.transformWithEsbuild === "function") {
      return viteApi.transformWithEsbuild(code, id, {
        loader: "ts",
        sourcemap: false,
        target: "esnext",
      });
    }
    throw new Error("Installed Vite does not expose transformWithOxc or transformWithEsbuild");
  };
}

const TYPE_DECLARATION_RE = /\b(?:interface|type|enum|namespace|declare)\s+[A-Za-z_$]/;
const TYPE_ASSERTION_RE =
  /\bas\s+(?:const|unknown|never|any|string|number|boolean|readonly\b|[A-Z][A-Za-z0-9_$]*(?:\s*[<[{&|),;=]|$))/;
const TYPED_BINDING_RE = /\b(?:const|let|var)\s+[A-Za-z_$][\w$]*\s*:/;
const GENERIC_FUNCTION_RE = /\bfunction\s+[A-Za-z_$][\w$]*\s*<[^>{}]*>\s*\(/;
const TYPED_PARAMETER_RE = /[(,]\s*(?:\.\.\.)?[A-Za-z_$][\w$]*\??\s*:\s*[^,)=]+/;
const RETURN_TYPE_RE = /\)\s*:\s*[^=<{;]+[{=>]/;
const ACCESS_MODIFIER_RE = /\b(?:public|private|protected|readonly|abstract|implements)\b/;
const SATISFIES_RE = /\bsatisfies\s+[A-Za-z_$]/;

function hasUnbalancedDelimiters(code: string): boolean {
  const stack: string[] = [];
  let quote: "'" | '"' | "`" | null = null;
  let escaped = false;
  let lineComment = false;
  let blockComment = false;

  for (let index = 0; index < code.length; index += 1) {
    const char = code[index]!;
    const next = code[index + 1];

    if (lineComment) {
      if (char === "\n" || char === "\r") {
        lineComment = false;
      }
      continue;
    }
    if (blockComment) {
      if (char === "*" && next === "/") {
        blockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }

    if (char === "/" && next === "/") {
      lineComment = true;
      index += 1;
      continue;
    }
    if (char === "/" && next === "*") {
      blockComment = true;
      index += 1;
      continue;
    }
    if (char === "'" || char === '"' || char === "`") {
      quote = char;
      continue;
    }
    if (char === "{" || char === "(" || char === "[") {
      stack.push(char);
      continue;
    }
    if (char === "}" || char === ")" || char === "]") {
      const open = stack.pop();
      if (
        (char === "}" && open !== "{") ||
        (char === ")" && open !== "(") ||
        (char === "]" && open !== "[")
      ) {
        return true;
      }
    }
  }

  return quote !== null || blockComment || stack.length > 0;
}

export function needsVirtualTypeScriptTransform(code: string): boolean {
  return (
    TYPE_DECLARATION_RE.test(code) ||
    TYPE_ASSERTION_RE.test(code) ||
    TYPED_BINDING_RE.test(code) ||
    GENERIC_FUNCTION_RE.test(code) ||
    TYPED_PARAMETER_RE.test(code) ||
    RETURN_TYPE_RE.test(code) ||
    ACCESS_MODIFIER_RE.test(code) ||
    SATISFIES_RE.test(code) ||
    hasUnbalancedDelimiters(code)
  );
}

export const transformVirtualTypeScript = createVirtualTypeScriptTransformer(vite);

function getOxcDumpPath(root: string, realPath: string): string {
  const dumpDir = path.resolve(root || process.cwd(), "node_modules", ".vize", "oxc-dumps");
  fs.mkdirSync(dumpDir, { recursive: true });
  return path.join(dumpDir, `vize-oxc-error-${path.basename(realPath)}.ts`);
}

function getVirtualModuleDefines(
  state: Pick<VizePluginState, "clientViteDefine" | "isProduction" | "serverViteDefine">,
  ssr: boolean,
): Record<string, string> {
  return {
    "import.meta.client": ssr ? "false" : "true",
    "import.meta.server": ssr ? "true" : "false",
    "import.meta.dev": state.isProduction ? "false" : "true",
    "import.meta.test": "false",
    "import.meta.prerender": "false",
    ...(ssr ? state.serverViteDefine : state.clientViteDefine),
  };
}

function formatUnknownError(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export async function transformVizeVirtualModule(
  state: VizePluginState,
  code: string,
  realPath: string,
  ssr: boolean,
  forceTypeScriptTransform = false,
): Promise<TransformResult | null> {
  const needsTsTransform = forceTypeScriptTransform || needsVirtualTypeScriptTransform(code);
  try {
    const result = needsTsTransform ? await transformVirtualTypeScript(code, realPath) : { code };
    let transformed = result.code;
    if (transformed.includes("import.meta.")) {
      transformed = applyDefineReplacements(transformed, getVirtualModuleDefines(state, ssr));
    }
    return transformed === code ? null : { code: transformed, map: null };
  } catch (e: unknown) {
    state.logger.error(`transformWithOxc failed for ${realPath}:`, e);
    let dumpPath: string | null = null;
    try {
      dumpPath = getOxcDumpPath(state.root, realPath);
      fs.writeFileSync(dumpPath, code, "utf-8");
      state.logger.error(`Dumped failing code to ${dumpPath}`);
    } catch (dumpError: unknown) {
      state.logger.error(`Failed to dump failing virtual module for ${realPath}:`, dumpError);
    }
    const message = [
      `[vize] Virtual module transform failed for ${realPath}: ${formatUnknownError(e)}`,
      dumpPath ? `Dumped failing code to ${dumpPath}` : null,
    ]
      .filter(Boolean)
      .join("\n");
    throw new Error(message);
  }
}
