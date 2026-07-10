import type { ArtFileInfo } from "./types/index.js";

export function sortedArts(arts: Iterable<ArtFileInfo>): ArtFileInfo[] {
  return [...arts].sort(compareArtFiles);
}

export function compareArtFiles(a: ArtFileInfo, b: ArtFileInfo): number {
  const order = compareOrder(a.metadata.order, b.metadata.order);
  if (order !== 0) return order;

  const title = a.metadata.title.localeCompare(b.metadata.title, undefined, { numeric: true });
  if (title !== 0) return title;

  return a.path.localeCompare(b.path, undefined, { numeric: true });
}

function compareOrder(a: number | undefined, b: number | undefined): number {
  if (a === undefined && b === undefined) return 0;
  if (a === undefined) return 1;
  if (b === undefined) return -1;
  return a - b;
}
