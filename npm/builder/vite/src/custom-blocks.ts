import type { CustomBlockInfo, NativeCustomBlockInfo } from "./sfc-types.ts";

const RESERVED_BLOCK_TYPES = new Set(["style", "script", "template"]);

/** Map native `blockType` blocks onto the shape Vite emits and reloads. */
export function normalizeCustomBlocks(
  blocks: ReadonlyArray<NativeCustomBlockInfo> | null | undefined,
): CustomBlockInfo[] {
  if (!blocks) return [];
  return blocks.map((block) => {
    const attrs: Record<string, string | true> = {};
    for (const attr of block.attrs ?? []) {
      const value = attr.value;
      attrs[attr.name] = value == null || value === "" ? true : value;
    }
    return {
      type: block.blockType,
      content: block.content,
      src: block.src || null,
      attrs,
      index: block.index,
    };
  });
}

/**
 * Query for one custom block, without the leading `?`.
 *
 * `vue` is a bare flag (`vue&type=`), which is what user plugins match.
 * Vite may re-serialize that empty param as `vue=&type=`; load accepts both.
 * `index` is the block's position in the custom-block list. A `src` block
 * carries `src=true` rather than the path. Other attributes follow, except `src`.
 */
export function customBlockQuery(block: CustomBlockInfo, index: number): string {
  const parts = ["vue", `type=${block.type}`, `index=${index}`];
  if (block.src) parts.push("src=true");
  for (const [key, value] of Object.entries(block.attrs)) {
    if (key === "src") continue;
    parts.push(value === true ? key : `${key}=${value}`);
  }
  return parts.join("&");
}

export function customBlockRequest(
  filePath: string,
  block: CustomBlockInfo,
  index: number,
): string {
  return `${filePath}?${customBlockQuery(block, index)}`;
}

/** Import plus call, matching the rspack custom-block wiring. */
export function customBlockModuleCode(
  filePath: string,
  blocks: readonly CustomBlockInfo[],
): string {
  return blocks
    .map((block, index) => {
      const request = JSON.stringify(customBlockRequest(filePath, block, index));
      return (
        `import block${index} from ${request};\n` +
        `if (typeof block${index} === 'function') block${index}(_sfc_main);`
      );
    })
    .join("\n");
}

/**
 * Insert `insertion` immediately before `export default _sfc_main`, so the
 * call runs after `_sfc_main` exists and before importers read the export.
 * Appends when that export is not a statement in this module.
 */
export function insertBeforeSfcMainDefaultExport(code: string, insertion: string): string {
  const block = insertion.endsWith("\n") ? insertion : `${insertion}\n`;
  const lineStart = sfcMainDefaultExportLineStart(code);
  if (lineStart < 0) return `${code}${code.endsWith("\n") ? "" : "\n"}${block}`;
  return `${code.slice(0, lineStart)}${block}${code.slice(lineStart)}`;
}

function sfcMainDefaultExportLineStart(code: string): number {
  const marker = "export default _sfc_main";
  let at = code.lastIndexOf(marker);
  while (at >= 0) {
    const lineStart = code.lastIndexOf("\n", at - 1) + 1;
    if (code.slice(lineStart, at).trim() === "") return lineStart;
    at = code.lastIndexOf(marker, at - 1);
  }
  return -1;
}

export interface CustomBlockQuery {
  type: string;
  index: number;
}

/** A Vue custom-block query: `type` is present and is not a built-in block. */
export function parseCustomBlockQuery(id: string): CustomBlockQuery | null {
  const start = id.indexOf("?");
  if (start < 0) return null;
  const params = new URLSearchParams(id.slice(start + 1));
  if (!params.has("vue")) return null;
  const type = params.get("type");
  if (!type || RESERVED_BLOCK_TYPES.has(type)) return null;
  const index = Number.parseInt(params.get("index") ?? "0", 10);
  if (!Number.isInteger(index) || index < 0) return null;
  return { type, index };
}
