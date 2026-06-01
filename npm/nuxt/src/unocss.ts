import fs from "node:fs";

export const MAX_UNOCSS_ORIGINAL_SOURCE_BYTES = 2 * 1024 * 1024;

type ReadTextFile = (filePath: string) => string;
type ReadFileSize = (filePath: string) => number;

function readTextFile(filePath: string): string {
  return fs.readFileSync(filePath, "utf-8");
}

function readFileSize(filePath: string): number {
  return fs.statSync(filePath).size;
}

export function appendOriginalVueSourceForUnoCss(
  code: string,
  normalizedId: string,
  options: {
    maxBytes?: number;
    readFile?: ReadTextFile;
    readSize?: ReadFileSize;
  } = {},
): string {
  const filePath = normalizedId.split("?")[0];
  if (!filePath) {
    return code;
  }

  const maxBytes = Math.max(1, Math.floor(options.maxBytes ?? MAX_UNOCSS_ORIGINAL_SOURCE_BYTES));
  const readSize = options.readSize ?? readFileSize;
  const readFile = options.readFile ?? readTextFile;

  try {
    if (readSize(filePath) > maxBytes) {
      return code;
    }
  } catch {
    return code;
  }

  try {
    return `${code}\n${readFile(filePath)}`;
  } catch {
    return code;
  }
}
