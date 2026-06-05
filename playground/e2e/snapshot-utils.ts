function normalizeImportSpecifiers(specifiers: string): string {
  return specifiers
    .split(",")
    .map((specifier) => specifier.trim())
    .filter(Boolean)
    .sort((left, right) => {
      const leftKey = left.match(/\bas\s+([\w$]+)$/)?.[1] ?? left;
      const rightKey = right.match(/\bas\s+([\w$]+)$/)?.[1] ?? right;
      return leftKey.localeCompare(rightKey) || left.localeCompare(right);
    })
    .join(", ");
}

function normalizeGeneratedCode(code: string): string {
  return code
    .replace(
      /import \{([^{}]+)\} from (["'])(vue|@vue\/server-renderer)\2/g,
      (_, specifiers: string, quote: string, source: string) =>
        `import { ${normalizeImportSpecifiers(specifiers)} } from ${quote}${source}${quote}`,
    )
    .replace(/\n{3,}/g, "\n\n")
    .replace(/^(type [^\n]+)\n\n(?=(?:const|let|var|function|class|export)\b)/gm, "$1\n")
    .replace(/[ \t]+\n/g, "\n")
    .trimEnd();
}

export function normalizeGeneratedCodeForSnapshot<T>(value: T): T {
  if (typeof value === "string") {
    return normalizeGeneratedCode(value) as T;
  }

  if (Array.isArray(value)) {
    return value.map((item) => normalizeGeneratedCodeForSnapshot(item)) as T;
  }

  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, normalizeGeneratedCodeForSnapshot(item)]),
    ) as T;
  }

  return value;
}
