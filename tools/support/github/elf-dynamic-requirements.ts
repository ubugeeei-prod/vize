/**
 * Reads what a 64-bit little-endian ELF image (x86_64 or aarch64) asks of
 * the dynamic loader: its program interpreter, its `DT_NEEDED` libraries,
 * every `SHT_GNU_verneed` version it requires per library, which undefined
 * dynamic symbols bind to which of those versions, and whether it carries
 * packed relative relocations (`DT_RELR`, loadable only by glibc 2.36+).
 *
 * This is what `readelf --dynamic --version-info --dyn-syms` prints, read in
 * process so the release gate needs no binutils and runs on any host.
 */
import fs from "node:fs";

const SHT_DYNAMIC = 6;
const SHT_DYNSYM = 11;
const SHT_GNU_VERNEED = 0x6ffffffe;
const SHT_GNU_VERSYM = 0x6fffffff;
const PT_INTERP = 3;
const DT_NULL = 0n;
const DT_NEEDED = 1n;
const DT_RELRSZ = 35n;
const DT_RELR = 36n;
const EM_X86_64 = 62;
const EM_AARCH64 = 183;

export interface VersionRequirement {
  /** The library the version is resolved against, e.g. `libc.so.6`. */
  readonly library: string;
  /** The version name, e.g. `GLIBC_2.17` or `GLIBC_ABI_DT_RELR`. */
  readonly version: string;
  /** Undefined dynamic symbols bound to this version, sorted. */
  readonly symbols: readonly string[];
}

export interface ElfDynamicRequirements {
  readonly machine: "x86_64" | "aarch64";
  readonly interpreter: string | undefined;
  readonly needed: readonly string[];
  readonly requirements: readonly VersionRequirement[];
  readonly packedRelativeRelocations: boolean;
}

interface Section {
  readonly type: number;
  readonly offset: number;
  readonly size: number;
  readonly link: number;
  readonly entrySize: number;
}

class ElfFormatError extends Error {}

function checkedRange(bytes: Uint8Array, offset: number, size: number, what: string): void {
  if (offset < 0 || size < 0 || offset + size > bytes.length) {
    throw new ElfFormatError(`${what} lies outside the file (${offset}+${size} > ${bytes.length})`);
  }
}

function number(value: bigint, what: string): number {
  if (value > BigInt(Number.MAX_SAFE_INTEGER)) throw new ElfFormatError(`${what} is too large`);
  return Number(value);
}

function cString(bytes: Uint8Array, offset: number): string {
  let end = offset;
  while (end < bytes.length && bytes[end] !== 0) end += 1;
  if (end >= bytes.length) throw new ElfFormatError(`unterminated string at ${offset}`);
  return Buffer.from(bytes.buffer, bytes.byteOffset + offset, end - offset).toString("latin1");
}

function readSections(view: DataView, bytes: Uint8Array): Section[] {
  const tableOffset = number(view.getBigUint64(0x28, true), "section table offset");
  const entrySize = view.getUint16(0x3a, true);
  const count = view.getUint16(0x3c, true);
  if (count === 0) return [];
  if (entrySize < 64) throw new ElfFormatError(`section header size ${entrySize} is too small`);
  checkedRange(bytes, tableOffset, entrySize * count, "section header table");
  const sections: Section[] = [];
  for (let index = 0; index < count; index += 1) {
    const base = tableOffset + index * entrySize;
    sections.push({
      type: view.getUint32(base + 4, true),
      offset: number(view.getBigUint64(base + 0x18, true), "section offset"),
      size: number(view.getBigUint64(base + 0x20, true), "section size"),
      link: view.getUint32(base + 0x28, true),
      entrySize: number(view.getBigUint64(base + 0x38, true), "section entry size"),
    });
  }
  return sections;
}

function readInterpreter(view: DataView, bytes: Uint8Array): string | undefined {
  const tableOffset = number(view.getBigUint64(0x20, true), "program header offset");
  const entrySize = view.getUint16(0x36, true);
  const count = view.getUint16(0x38, true);
  checkedRange(bytes, tableOffset, entrySize * count, "program header table");
  for (let index = 0; index < count; index += 1) {
    const base = tableOffset + index * entrySize;
    if (view.getUint32(base, true) !== PT_INTERP) continue;
    const offset = number(view.getBigUint64(base + 8, true), "interpreter offset");
    return cString(bytes, offset);
  }
  return undefined;
}

function stringTable(sections: readonly Section[], section: Section, bytes: Uint8Array) {
  const strings = sections[section.link];
  if (strings === undefined) throw new ElfFormatError("section links to a missing string table");
  checkedRange(bytes, strings.offset, strings.size, "string table");
  return (offset: number) => cString(bytes, strings.offset + offset);
}

function readDynamic(
  view: DataView,
  bytes: Uint8Array,
  sections: readonly Section[],
): { needed: string[]; packedRelativeRelocations: boolean } {
  const needed: string[] = [];
  let packedRelativeRelocations = false;
  for (const section of sections) {
    if (section.type !== SHT_DYNAMIC) continue;
    checkedRange(bytes, section.offset, section.size, "dynamic section");
    const name = stringTable(sections, section, bytes);
    for (let offset = section.offset; offset + 16 <= section.offset + section.size; offset += 16) {
      const tag = view.getBigInt64(offset, true);
      if (tag === DT_NULL) break;
      if (tag === DT_NEEDED) needed.push(name(number(view.getBigUint64(offset + 8, true), "name")));
      if (tag === DT_RELR || tag === DT_RELRSZ) packedRelativeRelocations = true;
    }
  }
  return { needed, packedRelativeRelocations };
}

/** Version index (`vna_other`) to the library and version name it requires. */
function readVersionNeeds(
  view: DataView,
  bytes: Uint8Array,
  sections: readonly Section[],
): Map<number, { library: string; version: string }> {
  const needs = new Map<number, { library: string; version: string }>();
  for (const section of sections) {
    if (section.type !== SHT_GNU_VERNEED) continue;
    checkedRange(bytes, section.offset, section.size, "version requirement section");
    const name = stringTable(sections, section, bytes);
    let entry = section.offset;
    for (;;) {
      checkedRange(bytes, entry, 16, "version requirement");
      const count = view.getUint16(entry + 2, true);
      const library = name(view.getUint32(entry + 4, true));
      let auxiliary = entry + view.getUint32(entry + 8, true);
      for (let index = 0; index < count; index += 1) {
        checkedRange(bytes, auxiliary, 16, "version requirement entry");
        const versionIndex = view.getUint16(auxiliary + 6, true);
        needs.set(versionIndex, { library, version: name(view.getUint32(auxiliary + 8, true)) });
        const next = view.getUint32(auxiliary + 12, true);
        if (next === 0) break;
        auxiliary += next;
      }
      const next = view.getUint32(entry + 12, true);
      if (next === 0) break;
      entry += next;
    }
  }
  return needs;
}

function readVersionedImports(
  view: DataView,
  bytes: Uint8Array,
  sections: readonly Section[],
): Map<number, string[]> {
  const imports = new Map<number, string[]>();
  const symbols = sections.find((section) => section.type === SHT_DYNSYM);
  const versions = sections.find((section) => section.type === SHT_GNU_VERSYM);
  if (symbols === undefined || versions === undefined) return imports;
  const entrySize = symbols.entrySize || 24;
  checkedRange(bytes, symbols.offset, symbols.size, "dynamic symbol table");
  checkedRange(bytes, versions.offset, versions.size, "symbol version table");
  const name = stringTable(sections, symbols, bytes);
  const count = Math.min(Math.floor(symbols.size / entrySize), Math.floor(versions.size / 2));
  for (let index = 1; index < count; index += 1) {
    const symbol = symbols.offset + index * entrySize;
    if (view.getUint16(symbol + 6, true) !== 0) continue; // defined here, not imported
    const versionIndex = view.getUint16(versions.offset + index * 2, true) & 0x7fff;
    if (versionIndex < 2) continue; // local or global: no version requirement
    const list = imports.get(versionIndex) ?? [];
    list.push(name(view.getUint32(symbol, true)));
    imports.set(versionIndex, list);
  }
  return imports;
}

export function readElfDynamicRequirements(bytes: Uint8Array): ElfDynamicRequirements {
  const magic = [0x7f, 0x45, 0x4c, 0x46]; // "\x7fELF"
  if (bytes.length < 64 || magic.some((byte, index) => bytes[index] !== byte)) {
    throw new ElfFormatError("not an ELF file");
  }
  if (bytes[4] !== 2 || bytes[5] !== 1) {
    throw new ElfFormatError("only 64-bit little-endian ELF images are supported");
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const machineCode = view.getUint16(0x12, true);
  const machine =
    machineCode === EM_X86_64 ? "x86_64" : machineCode === EM_AARCH64 ? "aarch64" : undefined;
  if (machine === undefined) throw new ElfFormatError(`unsupported ELF machine ${machineCode}`);
  const sections = readSections(view, bytes);
  if (sections.length === 0) throw new ElfFormatError("the image has no section headers");
  const { needed, packedRelativeRelocations } = readDynamic(view, bytes, sections);
  const needs = readVersionNeeds(view, bytes, sections);
  const imports = readVersionedImports(view, bytes, sections);
  const requirements = [...needs.entries()]
    .map(([index, need]) => ({ ...need, symbols: [...new Set(imports.get(index) ?? [])].sort() }))
    .sort((left, right) =>
      left.library === right.library
        ? left.version.localeCompare(right.version, "en", { numeric: true })
        : left.library.localeCompare(right.library),
    );
  return {
    machine,
    interpreter: readInterpreter(view, bytes),
    needed,
    requirements,
    packedRelativeRelocations,
  };
}

export function readElfFileDynamicRequirements(file: string): ElfDynamicRequirements {
  try {
    return readElfDynamicRequirements(fs.readFileSync(file));
  } catch (error) {
    if (error instanceof ElfFormatError) throw new Error(`${file}: ${error.message}`);
    throw error;
  }
}
