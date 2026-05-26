import type {
  InspectorFile,
  InspectorGraphEdge,
  InspectorGraphNode,
  InspectorGraphNodeKind,
} from "./types";

interface ImportRecord {
  specifier: string;
  kind: InspectorGraphEdge["kind"];
  locals: string[];
}

export function buildInspectorGraph(files: InspectorFile[]): {
  nodes: InspectorGraphNode[];
  edges: InspectorGraphEdge[];
} {
  const fileMap = new Map(files.map((file) => [normalizePath(file.path), file]));
  const nodes = files.map((file) => ({
    path: normalizePath(file.path),
    kind: fileKind(file.path),
    isEntry: isEntryPath(file.path),
    sourceLines: lineCount(file.source),
    sourceBytes: new TextEncoder().encode(file.source).length,
    issueCount: 0,
  }));
  const edges: InspectorGraphEdge[] = [];

  for (const file of files) {
    const from = normalizePath(file.path);
    for (const record of extractImports(file.source)) {
      const target = resolveImport(fileMap, from, record.specifier);
      if (!target) continue;

      pushEdge(edges, {
        from,
        to: target.path,
        kind: record.kind,
        specifier: record.specifier,
      });

      if (target.path.endsWith(".vue") && componentIsUsed(file.source, record.locals)) {
        pushEdge(edges, {
          from,
          to: target.path,
          kind: "component",
          specifier: record.specifier,
        });
      }
    }
  }

  return { nodes, edges };
}

function pushEdge(edges: InspectorGraphEdge[], edge: InspectorGraphEdge) {
  if (
    edges.some(
      (existing) =>
        existing.from === edge.from &&
        existing.to === edge.to &&
        existing.kind === edge.kind &&
        existing.specifier === edge.specifier,
    )
  ) {
    return;
  }
  edges.push(edge);
}

function extractImports(source: string): ImportRecord[] {
  const records: ImportRecord[] = [];
  const staticImportPattern = /import\s+([\s\S]*?)\s+from\s+["']([^"']+)["']/g;
  const dynamicImportPattern = /import\s*\(\s*["']([^"']+)["']\s*\)/g;
  let match: RegExpExecArray | null;

  while ((match = staticImportPattern.exec(source)) !== null) {
    records.push({
      specifier: match[2],
      kind: "import",
      locals: extractImportLocals(match[1]),
    });
  }

  while ((match = dynamicImportPattern.exec(source)) !== null) {
    records.push({
      specifier: match[1],
      kind: "dynamic-import",
      locals: [],
    });
  }

  return records;
}

function extractImportLocals(clause: string): string[] {
  const locals: string[] = [];
  const trimmed = clause.trim().replace(/^type\s+/, "");
  const defaultName = trimmed.split(",")[0]?.trim();
  if (defaultName && /^[A-Za-z_$][\w$]*$/.test(defaultName)) {
    locals.push(defaultName);
  }

  const namedMatch = trimmed.match(/\{([^}]+)\}/);
  if (namedMatch) {
    for (const part of namedMatch[1].split(",")) {
      const local = part
        .split(/\s+as\s+/)
        .pop()
        ?.trim();
      if (local && /^[A-Za-z_$][\w$]*$/.test(local)) {
        locals.push(local);
      }
    }
  }

  return locals;
}

function componentIsUsed(source: string, locals: string[]): boolean {
  return locals.some((local) => {
    const pascal = escapeRegExp(local);
    const kebab = escapeRegExp(toKebabCase(local));
    return new RegExp(`<\\s*(?:${pascal}|${kebab})(?:\\s|/|>)`).test(source);
  });
}

function resolveImport(
  fileMap: Map<string, InspectorFile>,
  from: string,
  specifier: string,
): { path: string; file: InspectorFile } | null {
  if (!specifier.startsWith(".")) return null;
  for (const candidate of importCandidates(from, specifier)) {
    const file = fileMap.get(candidate);
    if (file) return { path: candidate, file };
  }
  return null;
}

function importCandidates(from: string, specifier: string): string[] {
  const base = normalizePath(`${parentPath(from)}/${specifier}`);
  const candidates = [base];

  if (!hasKnownExtension(base)) {
    for (const extension of [".vue", ".ts", ".tsx", ".js", ".jsx"]) {
      candidates.push(`${base}${extension}`);
    }
    for (const extension of ["/index.vue", "/index.ts", "/index.js"]) {
      candidates.push(`${base}${extension}`);
    }
  }

  return candidates;
}

function normalizePath(path: string): string {
  const parts: string[] = [];
  for (const part of path.replace(/\\/g, "/").split("/")) {
    if (!part || part === ".") continue;
    if (part === "..") {
      parts.pop();
      continue;
    }
    parts.push(part);
  }
  return parts.join("/");
}

function parentPath(path: string): string {
  return path.includes("/") ? path.slice(0, path.lastIndexOf("/")) : "";
}

function fileKind(path: string): InspectorGraphNodeKind {
  if (path.endsWith(".vue")) return "vue";
  if (path.endsWith(".ts") || path.endsWith(".tsx")) return "typescript";
  if (path.endsWith(".js") || path.endsWith(".jsx")) return "javascript";
  return "other";
}

function isEntryPath(path: string): boolean {
  const basename = path.split("/").pop();
  return basename === "App.vue" || basename === "app.vue" || basename === "index.vue";
}

function hasKnownExtension(path: string): boolean {
  return /\.(vue|ts|tsx|js|jsx)$/.test(path);
}

function lineCount(source: string): number {
  return source ? source.split("\n").length : 0;
}

function toKebabCase(value: string): string {
  return value
    .replace(/([a-z0-9])([A-Z])/g, "$1-$2")
    .replace(/_/g, "-")
    .toLowerCase();
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
