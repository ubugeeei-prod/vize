// Value helpers for LSP fix-history sessions (#6883): host-path
// normalization, marker positions, completion projection and edit mirroring.
import assert from "node:assert/strict";
import { fileURLToPath, pathToFileURL } from "node:url";

const repositoryRoot = fileURLToPath(new URL("../../", import.meta.url));

/** Replace every location that depends on the temporary workspace or host. */
export function normalize(value: unknown, workspace: string): unknown {
  const uri = pathToFileURL(workspace).href;
  const encoded = uri.replaceAll("%", "%25");
  const root = repositoryRoot.replace(/\/$/, "");
  const rootUri = pathToFileURL(root).href;
  const text = (input: string) =>
    input
      .replaceAll(uri, "${workspace}")
      .replaceAll(encoded, "${workspace}")
      .replaceAll(workspace, "${workspacePath}")
      .replaceAll(rootUri, "${repo}")
      .replaceAll(root, "${repoPath}")
      .replace(/typescript-(?:darwin|linux|win32)-(?:arm64|x64)/g, "typescript-${platform}")
      // Opaque editor-session mirrors of the native checker live in a host temp dir.
      .replace(
        /file:\/\/[^"\s]*?\/vize-canon\/editor\/sessions\/session-[^/]+\/projects\/[0-9a-f]+/g,
        "${checkerSession}",
      );
  const walk = (item: unknown): unknown => {
    if (typeof item === "string") return text(item);
    if (Array.isArray(item)) return item.map(walk);
    if (item && typeof item === "object")
      return Object.fromEntries(
        Object.entries(item)
          // LSP `data` objects are opaque round-trip payloads for the server
          // (generated-code coordinates, hashes); semantic token arrays stay.
          .filter(([key, inner]) => key !== "data" || Array.isArray(inner))
          .map(([key, inner]) => [text(key), walk(inner)]),
      );
    return item;
  };
  return walk(value);
}

export function materialize(value: unknown, workspace: string): unknown {
  const uri = pathToFileURL(workspace).href;
  const walk = (item: unknown): unknown => {
    if (typeof item === "string")
      return item.replaceAll("${workspace}", uri).replaceAll("${workspacePath}", workspace);
    if (Array.isArray(item)) return item.map(walk);
    if (item && typeof item === "object")
      return Object.fromEntries(Object.entries(item).map(([key, inner]) => [walk(key), walk(inner)]));
    return item;
  };
  return walk(value);
}

/** Zero-based LSP position (UTF-16 columns) of an authored marker. */
export function positionOf(text: string, at: { find: string; delta?: number; nth?: number }) {
  let offset = -1;
  for (let n = 0; n <= (at.nth ?? 0); n++) {
    offset = text.indexOf(at.find, offset + 1);
    assert(offset >= 0, `marker not found: ${at.find}`);
  }
  offset += at.delta ?? 0;
  const before = text.slice(0, offset);
  const line = before.split("\n").length - 1;
  return { line, character: offset - (before.lastIndexOf("\n") + 1) };
}

/** Keep the complete items with the selected labels and the list size. */
export function pickLabels(result: unknown, labels: string[]) {
  if (result === null || result === undefined) return null;
  const list = result as { items?: unknown[]; isIncomplete?: boolean };
  const items = (Array.isArray(result) ? result : (list.items ?? [])) as { label: string }[];
  return {
    ...(Array.isArray(result) ? {} : { isIncomplete: list.isIncomplete }),
    count: items.length,
    items: items.filter((item) => labels.includes(item.label)),
  };
}

type LspPosition = { line: number; character: number };
type LspRange = { start: LspPosition; end: LspPosition };

/** UTF-16 offset of an LSP position (clamped like the server's line table). */
function offsetAt(text: string, position: LspPosition) {
  let offset = 0;
  for (let line = 0; line < position.line; line++) {
    const next = text.indexOf("\n", offset);
    if (next < 0) return text.length;
    offset = next + 1;
  }
  const end = text.indexOf("\n", offset);
  return Math.min(offset + position.character, end < 0 ? text.length : end);
}

/** Mirror a didChange content change so later markers see the edited text. */
export function applyChange(text: string, change: { range?: LspRange; text: string }) {
  if (change.range === undefined) return change.text;
  const start = offsetAt(text, change.range.start);
  const end = Math.max(start, offsetAt(text, change.range.end));
  return text.slice(0, start) + change.text + text.slice(end);
}

export function languageId(file: string) {
  if (file.endsWith(".vue")) return "vue";
  if (file.endsWith(".tsx")) return "typescriptreact";
  if (file.endsWith(".jsx")) return "javascriptreact";
  if (/\.[cm]?ts$/.test(file)) return "typescript";
  if (/\.[cm]?js$/.test(file)) return "javascript";
  if (file.endsWith(".html")) return "html";
  return "plaintext";
}
