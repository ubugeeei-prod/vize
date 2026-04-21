import { transform } from "oxc-transform";

function formatErrorMessage(error: {
  message: string;
  codeframe?: string | null;
  helpMessage?: string | null;
}): string {
  const parts = [error.message];
  if (error.helpMessage) {
    parts.push(error.helpMessage);
  }
  if (error.codeframe) {
    parts.push(error.codeframe);
  }
  return parts.join("\n");
}

export async function stripTypeScript(
  filePath: string,
  code: string,
  sourceMap: boolean,
): Promise<{ code: string; map: unknown }> {
  const result = await transform(filePath, code, {
    lang: "ts",
    sourcemap: sourceMap,
    sourceType: "module",
  });

  if (result.errors.length > 0) {
    throw new Error(result.errors.map(formatErrorMessage).join("\n\n"));
  }

  return {
    code: result.code,
    map: result.map ?? null,
  };
}
