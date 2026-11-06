import { createHash } from "node:crypto";
import type { ModuleImportNapi, ModuleShapeNapi } from "@vizejs/native";

/** Private per-component state, with no global registry or public component properties. */
export const hmrRuntime = `
const records = new WeakMap();
const namespace = (value, source) => Object.keys(value).sort().map(key => [source + ':' + key, value[key]]);
export function remember(component, signature, collect) {
  if ((typeof component !== 'object' || component === null) && typeof component !== 'function') return;
  let imports = null;
  try { imports = collect(namespace); } catch {}
  records.set(component, { signature, imports });
}
export function read(component) { return records.get(component); }
export function same(before, after) {
  return before && after && before.signature === after.signature &&
    before.imports && after.imports && before.imports.length === after.imports.length &&
    after.imports.every((entry, i) => entry[0] === before.imports[i][0] && Object.is(entry[1], before.imports[i][1]));
}
`;

function stringIndexAtByteOffset(code: string, offset: number | null | undefined): number | null {
  if (offset == null || offset < 0) return null;
  const bytes = Buffer.from(code);
  if (offset > bytes.length) return null;
  return bytes.subarray(0, offset).toString("utf8").length;
}

/**
 * Capture actual imported values before hot replacement. Equal module text alone
 * is insufficient: a script dependency can change while the SFC text stays equal.
 * Missing native facts, side effects, dynamic imports and cyclic TDZs disable reuse.
 */
export async function instrumentHmr(
  code: string,
  shape: ModuleShapeNapi | undefined,
  runtime: string,
  identity: string,
  hostImports: readonly ModuleImportNapi[] = [],
): Promise<string> {
  if (
    !shape?.imports ||
    shape.hasSideEffectImport ||
    shape.hasReexport ||
    shape.hasDynamicImport ||
    // An escaped identifier can collide after decoding even when its spelling
    // does not occur in `code`; keep that rare authored case conservative.
    /\\u(?:\{[\da-f]+\}|[\da-f]{4})/i.test(code)
  )
    return code;

  const used = new Set<string>();
  const unique = (base: string) => {
    let name = base;
    while (used.has(name) || code.includes(name)) name += "_";
    used.add(name);
    return name;
  };
  const remember = unique("__vize_parcel_remember");
  const namespace = unique("__vize_parcel_namespace");
  const imports = [...shape.imports, ...hostImports].map(({ source, local, namespace: all }) => {
    const key = JSON.stringify(`${source}:${local}`);
    return all ? `...${namespace}(${local}, ${key})` : `[${key}, ${local}]`;
  });

  let target: string;
  let emitted = code;
  if (shape.hasDefaultExport) {
    if (shape.defaultExportLocal) target = shape.defaultExportLocal;
    else if (shape.defaultExportIsSfcMain) target = "_sfc_main";
    else {
      target = unique("__vize_parcel_component");
      const start = stringIndexAtByteOffset(code, shape.defaultExportStart);
      const keywordEnd = stringIndexAtByteOffset(code, shape.defaultExportKeywordEnd);
      const end = stringIndexAtByteOffset(code, shape.defaultExportEnd);
      if (
        start == null ||
        keywordEnd == null ||
        end == null ||
        start > keywordEnd ||
        keywordEnd > end
      )
        return code;
      const keyword = code.slice(start, keywordEnd);
      if (!keyword.startsWith("export") || !keyword.endsWith("default")) return code;
      emitted = `${code.slice(0, start)}const ${target} =${code.slice(keywordEnd, end)};${code.slice(end)}\nexport default ${target};`;
    }
  } else if (shape.hasNamedRenderExport) target = "render";
  else return code;
  const signature = createHash("sha256").update(code).update(identity).digest("hex");
  return `import { remember as ${remember} } from ${JSON.stringify(runtime)};\n${emitted}\n${remember}(${target}, ${JSON.stringify(signature)}, (${namespace}) => [${imports.join(",")}]);\n`;
}
