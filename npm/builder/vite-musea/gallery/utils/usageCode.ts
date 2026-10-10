export function usagePropsAttributes(values: Record<string, unknown>): string {
  return Object.entries(values)
    .filter(([, value]) => value !== undefined)
    .map(([name, value]) => {
      if (typeof value === "boolean") return value ? ` ${name}` : ` :${name}="false"`;
      if (typeof value === "string") return ` ${name}="${escapeAttribute(value)}"`;
      return ` :${name}="${escapeAttribute(usageExpression(value))}"`;
    })
    .join("");
}

function escapeAttribute(value: string): string {
  const entities: Record<string, string> = {
    "&": "&amp;",
    '"': "&quot;",
    "<": "&lt;",
    ">": "&gt;",
    "\r": "&#13;",
    "\n": "&#10;",
    "\t": "&#9;",
  };
  return value.replace(/[&"<>\r\n\t]/g, (character) => entities[character]);
}

function usageExpression(value: unknown): string {
  if (typeof value === "number") return Object.is(value, -0) ? "-0" : String(value);
  if (Array.isArray(value)) return `[${value.map(usageExpression).join(",")}]`;
  if (value !== null && typeof value === "object") {
    return `{${Object.entries(value)
      .map(([name, item]) => {
        // A literal __proto__ key changes an object's prototype in JavaScript.
        const key = name === "__proto__" ? `[${JSON.stringify(name)}]` : JSON.stringify(name);
        return `${key}:${usageExpression(item)}`;
      })
      .join(",")}}`;
  }
  return JSON.stringify(value) ?? "undefined";
}

export function usageScript(content?: string, isolated = true): string {
  if (!content) return "";
  const lines = content.split("\n");
  const componentName = extractDefineArtComponentName(content);
  const kept: string[] = [];
  let defineArtBalance = 0;

  for (const line of lines) {
    const trimmed = line.trim();
    if (defineArtBalance > 0) {
      defineArtBalance += parenBalance(line);
      continue;
    }
    if (/\bdefineArt\s*\(/.test(trimmed)) {
      defineArtBalance = Math.max(0, parenBalance(line));
      continue;
    }
    if (isolated && componentName && importDeclaresName(line, componentName)) {
      continue;
    }
    kept.push(line);
  }

  return kept.join("\n").trim();
}

export function extractDefineArtComponentName(content: string): string | undefined {
  const sourceMatch = content.match(/\bdefineArt\s*\(\s*(['"])([^'"]+)\1/);
  if (sourceMatch) {
    return componentNameFromSource(sourceMatch[2]);
  }
  return content.match(/\bdefineArt\s*\(\s*([A-Za-z_$][\w$]*)/)?.[1];
}

export function componentNameFromSource(source: string): string {
  const withoutQuery = source.split(/[?#]/, 1)[0] || source;
  const filename = withoutQuery.split(/[\\/]/).pop() || "Component";
  const stem = filename.replace(/\.[^.]+$/, "");
  const name = stem
    .split(/[^A-Za-z0-9]+/)
    .filter(Boolean)
    .map((segment) => segment.charAt(0).toUpperCase() + segment.slice(1))
    .join("");
  return name || "Component";
}

export function importDeclaresName(line: string, name: string): boolean {
  const trimmed = line.trim();
  if (!trimmed.startsWith("import ") || trimmed.startsWith("import type ")) return false;
  return new RegExp(`^import\\s+${name}(?:\\s|,|$)`).test(trimmed);
}

export function parenBalance(line: string): number {
  let balance = 0;
  for (const char of line) {
    if (char === "(") balance += 1;
    else if (char === ")") balance -= 1;
  }
  return balance;
}

export function indentUsage(code: string): string {
  return code
    .split("\n")
    .map((line) => (line.trim() ? `  ${line}` : line))
    .join("\n");
}
