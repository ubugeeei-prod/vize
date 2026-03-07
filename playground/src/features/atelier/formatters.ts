import * as prettier from "prettier/standalone";
import * as parserBabel from "prettier/plugins/babel";
import * as parserEstree from "prettier/plugins/estree";
import * as parserTypescript from "prettier/plugins/typescript";
import * as parserCss from "prettier/plugins/postcss";
import ts from "typescript";
import { formatSsrTemplates } from "./ssrFormatter";
import { formatVaporTemplates } from "./vaporFormatter";

export async function formatCode(code: string, parser: "babel" | "typescript"): Promise<string> {
  try {
    const ssrFormatted = await formatSsrTemplates(code);
    const source = await formatVaporTemplates(ssrFormatted);
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
    return await prettier.format(code, {
      parser: "css",
      plugins: [parserCss],
      printWidth: 80,
    });
  } catch {
    return code;
  }
}

export function transpileToJs(code: string): string {
  try {
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
