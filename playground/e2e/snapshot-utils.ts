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

function normalizeGeneratedValue<T>(value: T): T {
  if (typeof value === "string") {
    return normalizeGeneratedCode(value) as T;
  }

  if (Array.isArray(value)) {
    return value.map((item) => normalizeGeneratedValue(item)) as T;
  }

  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([key, item]) => [key, normalizeGeneratedValue(item)]),
    ) as T;
  }

  return value;
}
import successors from "./fixtures/generated-code-snapshot-successors-20261105.json" with { type: "json" };
import currentSuccessors from "./fixtures/generated-code-snapshot-successors-46d8fec5.json" with { type: "json" };

// #11698 stopped printing the whole generated module just to erase authored
// TypeScript. Keep the twelve historical snapshot bodies without applying a
// printer to arbitrary compiler output. Only complete reviewed values match;
// every foreign byte or field still reaches the ordinary whole snapshot check.
const historicalSpelling = new Map(
  [...successors.cases, ...currentSuccessors.cases].map(({ current, previous }) => [
    JSON.stringify(normalizeGeneratedValue(current)),
    previous,
  ]),
);

export function normalizeGeneratedCodeForSnapshot<T>(value: T): T {
  const current = normalizeGeneratedValue(value);
  const previous = historicalSpelling.get(JSON.stringify(current));
  return previous === undefined ? current : (structuredClone(previous) as T);
}
