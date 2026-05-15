import { formatSsrTemplates } from "./ssrFormatter";
import { formatVaporTemplates } from "./vaporFormatter";

let prettierRuntimePromise: Promise<{
  prettier: typeof import("prettier/standalone");
  parserBabel: typeof import("prettier/plugins/babel");
  parserEstree: typeof import("prettier/plugins/estree");
  parserTypescript: typeof import("prettier/plugins/typescript");
  parserCss: typeof import("prettier/plugins/postcss");
}> | null = null;

let typeScriptRuntimePromise: Promise<typeof import("typescript")> | null = null;

async function loadPrettierRuntime() {
  if (!prettierRuntimePromise) {
    prettierRuntimePromise = Promise.all([
      import("prettier/standalone"),
      import("prettier/plugins/babel"),
      import("prettier/plugins/estree"),
      import("prettier/plugins/typescript"),
      import("prettier/plugins/postcss"),
    ]).then(([prettier, parserBabel, parserEstree, parserTypescript, parserCss]) => ({
      prettier,
      parserBabel,
      parserEstree,
      parserTypescript,
      parserCss,
    }));
  }

  return prettierRuntimePromise;
}

async function loadTypeScriptRuntime() {
  if (!typeScriptRuntimePromise) {
    typeScriptRuntimePromise = import("typescript");
  }

  return typeScriptRuntimePromise;
}

export async function formatCode(code: string, parser: "babel" | "typescript"): Promise<string> {
  try {
    const ssrFormatted = await formatSsrTemplates(code);
    const source = await formatVaporTemplates(ssrFormatted);
    const { prettier, parserBabel, parserEstree, parserTypescript } = await loadPrettierRuntime();
    return await prettier.format(source, {
      parser,
      plugins: [parserBabel, parserEstree, parserTypescript],
      semi: false,
      singleQuote: true,
      printWidth: 80,
    });
  } catch {
    return code;
  }
}

export async function formatCss(code: string): Promise<string> {
  try {
    const { prettier, parserCss } = await loadPrettierRuntime();
    return await prettier.format(code, {
      parser: "css",
      plugins: [parserCss],
      printWidth: 80,
    });
  } catch {
    return code;
  }
}

export async function transpileToJs(code: string): Promise<string> {
  try {
    const ts = await loadTypeScriptRuntime();
    const result = ts.transpileModule(code, {
      compilerOptions: {
        module: ts.ModuleKind.ESNext,
        target: ts.ScriptTarget.ESNext,
        jsx: ts.JsxEmit.Preserve,
        removeComments: false,
      },
    });
    return result.outputText;
  } catch {
    return code;
  }
}
