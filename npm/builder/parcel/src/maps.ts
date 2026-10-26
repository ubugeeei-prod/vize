import ParcelSourceMap from "@parcel/source-map";
import MagicString from "magic-string";
import { diffChars } from "diff";

// Parcel's CJS package exposes a transpiled default. Handle both Node's CJS
// namespace and the bundler's default interop without depending on one emitter.
const SourceMap =
  (ParcelSourceMap as unknown as { default?: typeof ParcelSourceMap }).default ?? ParcelSourceMap;
type SourceMap = InstanceType<typeof ParcelSourceMap>;

/** Native parser geometry is UTF-8; JavaScript edits and source maps use UTF-16. */
export function byteOffset(source: string, offset: number): number {
  const bytes = Buffer.from(source);
  if (
    !Number.isSafeInteger(offset) ||
    offset < 0 ||
    offset > bytes.length ||
    (offset < bytes.length && (bytes[offset] & 0xc0) === 0x80)
  ) {
    throw new Error(`Invalid native UTF-8 source boundary: ${offset}`);
  }
  return bytes.toString("utf8", 0, offset).length;
}

export function nativeMap(json: string | undefined, projectRoot: string): SourceMap | undefined {
  if (!json) return undefined;
  const map = new SourceMap(projectRoot);
  map.addVLQMap(JSON.parse(json));
  return map;
}

/** Preserve every unchanged native mapping; inserted code remains unmapped. */
export function editMap(
  before: string,
  after: string,
  map: SourceMap | undefined,
  filename: string,
  projectRoot: string,
): SourceMap | undefined {
  if (!map || before === after) return map;
  const changes = diffChars(before, after, { maxEditLength: 10_000 });
  if (!changes) return undefined;
  const edited = new MagicString(before);
  let offset = 0;
  for (const change of changes) {
    if (change.added) edited.appendLeft(offset, change.value);
    else if (change.removed) {
      edited.remove(offset, offset + change.value.length);
      offset += change.value.length;
    } else offset += change.value.length;
  }
  if (edited.toString() !== after) throw new Error("Source-map edit reconstruction failed.");
  const result = new SourceMap(projectRoot);
  result.addVLQMap(edited.generateMap({ source: filename, includeContent: true, hires: true }));
  result.extends(map.toBuffer());
  return result;
}

export function blockMap(
  source: string,
  start: number,
  end: number,
  filename: string,
  projectRoot: string,
): SourceMap {
  const snippet = new MagicString(source).snip(byteOffset(source, start), byteOffset(source, end));
  const result = new SourceMap(projectRoot);
  result.addVLQMap(snippet.generateMap({ source: filename, includeContent: true, hires: true }));
  return result;
}
