// Edits that read the native compiler's `moduleShape` instead of parsing the
// emitted module again (#11583). No dependencies: every builder loads this.

/** The native `moduleShape` default-export span: UTF-8 byte offsets into the native code. */
export type NativeDefaultExportBytes = {
  defaultExportStart?: number | null;
  defaultExportKeywordEnd?: number | null;
};

/** The emitted module's text, edited at UTF-16 indices. */
export interface ModuleText {
  readonly code: string;
  replace(start: number, end: number, text: string): void;
}

function utf16Offset(bytes: Buffer, offset: number | null | undefined): number {
  if (offset == null || offset < 0 || !Number.isInteger(offset)) {
    throw new Error("[vize] Missing native component export geometry.");
  }
  if (offset > bytes.length || (offset < bytes.length && (bytes[offset] & 0xc0) === 0x80)) {
    throw new Error("[vize] Invalid native component export geometry.");
  }
  return bytes.subarray(0, offset).toString("utf8").length;
}

/** Locate the keyword boundary through ECMAScript whitespace and comments. */
export function findExportDefaultKeywordEnd(code: string, start: number): number | null {
  if (!code.startsWith("export", start)) return null;
  const gapStart = start + "export".length;
  let cursor = gapStart;
  while (cursor < code.length) {
    if (/\s/u.test(code[cursor])) {
      cursor++;
    } else if (code.startsWith("/*", cursor)) {
      const end = code.indexOf("*/", cursor + 2);
      if (end === -1) return null;
      cursor = end + 2;
    } else if (code.startsWith("//", cursor)) {
      cursor += 2;
      while (cursor < code.length && !/[\r\n\u2028\u2029]/u.test(code[cursor])) cursor++;
    } else {
      break;
    }
  }
  return cursor > gapStart && code.startsWith("default", cursor) ? cursor + "default".length : null;
}

/**
 * Rewrite `export default X` to `const name = X` at the span the native
 * compiler reported, converting its UTF-8 byte offsets against `module.code`.
 * The span must still contain the two keywords and only trivia between them,
 * so a UTF-16 index or a span made stale by an earlier edit throws instead of
 * corrupting the module.
 */
export function bindDefaultExport(
  module: ModuleText,
  shape: NativeDefaultExportBytes,
  name: string,
): void {
  const bytes = Buffer.from(module.code);
  const start = utf16Offset(bytes, shape.defaultExportStart);
  const end = utf16Offset(bytes, shape.defaultExportKeywordEnd);
  if (findExportDefaultKeywordEnd(module.code, start) !== end) {
    throw new Error("[vize] Native component export geometry does not describe this module.");
  }
  module.replace(start, end, `const ${name} =`);
}
