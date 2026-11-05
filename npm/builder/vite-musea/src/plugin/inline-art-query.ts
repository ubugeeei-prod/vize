import type { ArtFileInfo } from "../types/art.js";
import { isTrustedSourcePath } from "../trusted-path.js";

interface InlineArtQueryState {
  inlineArt: boolean;
  artFiles: ReadonlyMap<string, ArtFileInfo>;
  getConfigRoot: () => string;
  getScanRoots: () => string[];
}

/** Consume only the custom block already registered as this instance's inline Art. */
export function loadInlineArtCustomBlock(state: InlineArtQueryState, id: string): string | null {
  if (!state.inlineArt) return null;
  const queryStart = id.indexOf("?");
  if (queryStart === -1) return null;
  const requestPath = id.slice(0, queryStart);
  const file = requestPath.startsWith("/@fs/") ? requestPath.slice("/@fs".length) : requestPath;
  if (!file.endsWith(".vue") || file.endsWith(".art.vue")) return null;

  const query = new URLSearchParams(id.slice(queryStart + 1));
  if (
    query.getAll("vue").length !== 1 ||
    query.get("vue") !== "" ||
    query.getAll("type").length !== 1 ||
    query.get("type") !== "art" ||
    query.getAll("index").length !== 1
  ) {
    return null;
  }
  const indexText = query.get("index")!;
  if (!/^(?:0|[1-9]\d*)$/.test(indexText)) return null;
  const index = Number(indexText);
  if (!Number.isSafeInteger(index)) return null;

  const art = state.artFiles.get(file);
  if (
    !art?.isInline ||
    art.path !== file ||
    art.componentPath !== file ||
    art.inlineArtBlockIndex !== index ||
    !isTrustedSourcePath(state.getConfigRoot(), state.getScanRoots(), file)
  ) {
    return null;
  }

  // Musea has registered the metadata and variants separately. The host SFC
  // imports a default export for every custom block, with no Art runtime hook.
  return "export default {};\n";
}
