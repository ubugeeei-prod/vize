import {
  ident,
  maskRange,
  maskRustSource,
  tokensOf,
  type RustToken,
} from "./davinci-storage-rust-syntax.ts";

export type StorageKind = "allocVec" | "allocString" | "s0String" | "arenaVec" | "smallVec";
export type StorageMeasurement = { directPaths: number; boundUses: number };
export type FileStorage = Record<StorageKind, StorageMeasurement>;

type UseImport = { path: string[]; alias: string; start: number; end: number; glob: boolean };

export const storageKinds: StorageKind[] = [
  "allocVec",
  "allocString",
  "s0String",
  "arenaVec",
  "smallVec",
];

export function emptyFileStorage(): FileStorage {
  return Object.fromEntries(
    storageKinds.map((kind) => [kind, { directPaths: 0, boundUses: 0 }]),
  ) as FileStorage;
}

export function hasStorage(measurement: FileStorage): boolean {
  return storageKinds.some((kind) => {
    const value = measurement[kind];
    return value.directPaths > 0 || value.boundUses > 0;
  });
}

function flattenUse(tokens: RustToken[], start: number, end: number): UseImport[] {
  let cursor = start;
  const imports: UseImport[] = [];

  function tree(prefix: string[], statementStart: number): void {
    if (tokens[cursor]?.value === "::") cursor += 1;
    const segments: string[] = [];
    while (cursor < end) {
      const segment = ident(tokens[cursor]);
      if (segment === undefined) break;
      segments.push(segment);
      cursor += 1;
      if (tokens[cursor]?.value !== "::") break;
      if (tokens[cursor + 1]?.value === "{") {
        cursor += 1;
        break;
      }
      cursor += 1;
    }
    const bareSelf = segments.length === 1 && segments[0] === "self";
    const path = [...prefix, ...segments.filter((segment) => segment !== "self")];
    if (tokens[cursor]?.value === "{") {
      cursor += 1;
      while (cursor < end && tokens[cursor]?.value !== "}") {
        tree(path, statementStart);
        if (tokens[cursor]?.value === ",") cursor += 1;
      }
      if (tokens[cursor]?.value === "}") cursor += 1;
      return;
    }
    let glob = false;
    if (tokens[cursor]?.value === "*") {
      glob = true;
      cursor += 1;
    }
    let alias = bareSelf ? (prefix.at(-1) ?? "") : (segments.at(-1) ?? "");
    if (tokens[cursor]?.value === "as") {
      cursor += 1;
      alias = ident(tokens[cursor]) ?? alias;
      cursor += 1;
    }
    imports.push({
      path,
      alias,
      start: statementStart,
      end: tokens[end]?.end ?? tokens[end - 1].end,
      glob,
    });
  }

  tree([], tokens[start - 1]?.start ?? tokens[start].start);
  return imports;
}

function importsOf(tokens: RustToken[]): UseImport[] {
  const imports: UseImport[] = [];
  for (let index = 0; index < tokens.length; index += 1) {
    if (tokens[index].value !== "use") continue;
    let end = index + 1;
    let depth = 0;
    while (end < tokens.length) {
      if (tokens[end].value === "{") depth += 1;
      else if (tokens[end].value === "}") depth -= 1;
      else if (tokens[end].value === ";" && depth === 0) break;
      end += 1;
    }
    imports.push(...flattenUse(tokens, index + 1, end));
    index = end;
  }
  return imports;
}

function targetKind(path: string[]): StorageKind | undefined {
  const joined = path.join("::");
  if (joined === "alloc::vec::Vec" || joined.startsWith("alloc::vec::Vec::")) return "allocVec";
  if (joined === "alloc::string::String" || joined.startsWith("alloc::string::String::")) {
    return "allocString";
  }
  if (joined === "vize_s0::String" || joined.startsWith("vize_s0::String::")) return "s0String";
  if (joined === "vize_s0::Vec" || joined.startsWith("vize_s0::Vec::")) return "arenaVec";
  if (joined === "vize_s0::SmallVec" || joined.startsWith("vize_s0::SmallVec::")) {
    return "smallVec";
  }
  return undefined;
}

function literalRoot(path: string[]): "alloc" | "vize_s0" | undefined {
  const first = path[0] === "crate" || path[0] === "self" ? path[1] : path[0];
  return first === "alloc" || first === "vize_s0" ? first : undefined;
}

function canonical(path: string[], bindings: ReadonlyMap<string, string[]>): string[] | undefined {
  let normalized = path;
  if ((normalized[0] === "crate" || normalized[0] === "self") && normalized.length > 1) {
    normalized = normalized.slice(1);
  }
  if (normalized[0] === "alloc" || normalized[0] === "std" || normalized[0] === "vize_s0") {
    return normalized;
  }
  const prefix = bindings.get(normalized[0]);
  return prefix ? [...prefix, ...normalized.slice(1)] : undefined;
}

export type ScanResult = { storage: FileStorage; issues: string[] };

function isForbiddenStd(path: string[]): boolean {
  const joined = path.join("::");
  return (
    joined === "std" ||
    joined === "std::vec" ||
    joined.startsWith("std::vec::") ||
    joined === "std::string" ||
    joined.startsWith("std::string::") ||
    joined === "std::collections" ||
    joined.startsWith("std::collections::") ||
    /^std::(?:prelude::(?:v1|rust_\d+)::)?(?:String|Vec)(?:::|$)/u.test(joined)
  );
}

export function scanStorage(source: string): ScanResult {
  const code = maskRustSource(source);
  const tokens = tokensOf(code);
  const imports = importsOf(tokens);
  const bindings = new Map<string, string[]>([
    ["alloc", ["alloc"]],
    ["std", ["std"]],
    ["vize_s0", ["vize_s0"]],
  ]);
  const externPattern =
    /\bextern\s+crate\s+(?:r#)?(alloc|std)(?:\s+as\s+(?:r#)?([A-Za-z_]\w*))?\s*;/gu;
  for (const match of code.matchAll(externPattern)) {
    bindings.set(match[2] ?? match[1], [match[1]]);
  }
  for (let pass = 0; pass <= imports.length; pass += 1) {
    let changed = false;
    for (const imported of imports) {
      const resolved = canonical(imported.path, bindings);
      if (
        resolved &&
        imported.alias &&
        bindings.get(imported.alias)?.join("::") !== resolved.join("::")
      ) {
        bindings.set(imported.alias, resolved);
        changed = true;
      }
    }
    if (!changed) break;
  }

  const storage = emptyFileStorage();
  const issues: string[] = [];
  for (const imported of imports) {
    const resolved = canonical(imported.path, bindings);
    const mentionsStorageRoot =
      imported.path.some((part) => part === "alloc" || part === "std" || part === "vize_s0") ||
      bindings.has(imported.path[0]);
    if (mentionsStorageRoot && (!resolved || imported.glob)) {
      issues.push(`unresolved or glob storage import: ${imported.path.join("::")}`);
      continue;
    }
    if (resolved && isForbiddenStd(resolved)) {
      issues.push(`forbidden std storage import: ${imported.path.join("::")}`);
    }
    const kind = resolved ? targetKind(resolved) : undefined;
    if (kind && literalRoot(imported.path)) storage[kind].directPaths += 1;
  }

  const withoutImports = code.split("");
  for (const imported of imports) maskRange(withoutImports, imported.start, imported.end);
  // `extern crate alloc;` / `extern crate std;` are linkage declarations, not
  // storage paths — the loop above already read them to register bindings, so
  // the body scan must not re-read their crate name as a bare `std` use.
  for (const match of code.matchAll(externPattern)) {
    if (match.index === undefined) continue;
    maskRange(withoutImports, match.index, match.index + match[0].length);
  }
  const bodyTokens = tokensOf(withoutImports.join(""));
  const consumed = new Set<number>();
  for (let index = 0; index < bodyTokens.length; index += 1) {
    const first = ident(bodyTokens[index]);
    if (first === undefined) continue;
    const path = [first];
    let cursor = index;
    while (bodyTokens[cursor + 1]?.value === "::") {
      const next = ident(bodyTokens[cursor + 2]);
      if (next === undefined) break;
      path.push(next);
      cursor += 2;
    }
    const resolved = canonical(path, bindings);
    const kind = resolved ? targetKind(resolved) : undefined;
    if (!kind) {
      if (resolved && isForbiddenStd(resolved)) {
        issues.push(`forbidden std storage path: ${path.join("::")}`);
      }
      continue;
    }
    if (literalRoot(path)) storage[kind].directPaths += 1;
    else storage[kind].boundUses += 1;
    for (let used = index; used <= cursor; used += 1) consumed.add(used);
    index = cursor;
  }
  for (let index = 0; index < bodyTokens.length; index += 1) {
    if (consumed.has(index)) continue;
    const name = ident(bodyTokens[index]);
    const resolved = name ? bindings.get(name) : undefined;
    const kind = resolved ? targetKind(resolved) : undefined;
    if (kind) storage[kind].boundUses += 1;
  }
  return { storage, issues };
}
