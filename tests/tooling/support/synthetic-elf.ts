/**
 * Builds a minimal 64-bit little-endian ELF shared object with just the
 * dynamic-linking sections the glibc floor gate reads: `.dynstr`, `.dynamic`
 * (`DT_NEEDED`, optional `DT_RELR`), `.gnu.version_r`, `.dynsym` and
 * `.gnu.version`. Version indexes start at 2, in declaration order.
 */
export interface SyntheticElf {
  readonly machine?: number;
  readonly needed: readonly string[];
  readonly requirements: readonly { library: string; versions: readonly string[] }[];
  /** Undefined symbols and the version name each binds to. */
  readonly imports: readonly { symbol: string; version: string }[];
  readonly relr?: boolean;
}

export function syntheticElf(spec: SyntheticElf): Uint8Array {
  const strings: number[] = [0];
  const stringOffsets = new Map<string, number>();
  const intern = (value: string) => {
    const existing = stringOffsets.get(value);
    if (existing !== undefined) return existing;
    const offset = strings.length;
    strings.push(...Buffer.from(value, "latin1"), 0);
    stringOffsets.set(value, offset);
    return offset;
  };

  const dynamic: [bigint, bigint][] = spec.needed.map((library) => [1n, BigInt(intern(library))]);
  if (spec.relr) dynamic.push([36n, 0n], [35n, 0n]);
  dynamic.push([0n, 0n]);

  const versionIndex = new Map<string, number>();
  const verneed: number[][] = [];
  let nextIndex = 2;
  spec.requirements.forEach(({ library, versions }, entry) => {
    const record = new DataView(new ArrayBuffer(16 + versions.length * 16));
    record.setUint16(0, 1, true);
    record.setUint16(2, versions.length, true);
    record.setUint32(4, intern(library), true);
    record.setUint32(8, 16, true);
    record.setUint32(
      12,
      entry === spec.requirements.length - 1 ? 0 : 16 + versions.length * 16,
      true,
    );
    versions.forEach((version, index) => {
      const base = 16 + index * 16;
      versionIndex.set(version, nextIndex);
      record.setUint16(base + 6, nextIndex, true);
      record.setUint32(base + 8, intern(version), true);
      record.setUint32(base + 12, index === versions.length - 1 ? 0 : 16, true);
      nextIndex += 1;
    });
    verneed.push([...new Uint8Array(record.buffer)]);
  });

  const symbols = new DataView(new ArrayBuffer((spec.imports.length + 1) * 24));
  const versym = new DataView(new ArrayBuffer((spec.imports.length + 1) * 2));
  spec.imports.forEach(({ symbol, version }, index) => {
    symbols.setUint32((index + 1) * 24, intern(symbol), true);
    symbols.setUint8((index + 1) * 24 + 4, 0x12); // global function, undefined (shndx 0)
    versym.setUint16((index + 1) * 2, versionIndex.get(version) ?? 1, true);
  });

  const dynamicBytes = new DataView(new ArrayBuffer(dynamic.length * 16));
  dynamic.forEach(([tag, value], index) => {
    dynamicBytes.setBigInt64(index * 16, tag, true);
    dynamicBytes.setBigUint64(index * 16 + 8, value, true);
  });

  const bodies: { type: number; link: number; entrySize: number; bytes: Uint8Array }[] = [
    { type: 3, link: 0, entrySize: 0, bytes: Uint8Array.from(strings) },
    { type: 6, link: 1, entrySize: 16, bytes: new Uint8Array(dynamicBytes.buffer) },
    { type: 0x6ffffffe, link: 1, entrySize: 0, bytes: Uint8Array.from(verneed.flat()) },
    { type: 11, link: 1, entrySize: 24, bytes: new Uint8Array(symbols.buffer) },
    { type: 0x6fffffff, link: 4, entrySize: 2, bytes: new Uint8Array(versym.buffer) },
  ];
  let offset = 64;
  const placed = bodies.map((body) => {
    const at = offset;
    offset += body.bytes.length + ((8 - (body.bytes.length % 8)) % 8);
    return { ...body, at };
  });
  const sectionTable = offset;
  const sectionCount = placed.length + 1;
  const file = new Uint8Array(sectionTable + sectionCount * 64);
  const view = new DataView(file.buffer);
  file.set([0x7f, 0x45, 0x4c, 0x46, 2, 1, 1]);
  view.setUint16(0x10, 3, true);
  view.setUint16(0x12, spec.machine ?? 62, true);
  view.setUint32(0x14, 1, true);
  view.setBigUint64(0x28, BigInt(sectionTable), true);
  view.setUint16(0x34, 64, true);
  view.setUint16(0x3a, 64, true);
  view.setUint16(0x3c, sectionCount, true);
  for (const [index, section] of placed.entries()) {
    file.set(section.bytes, section.at);
    const header = sectionTable + (index + 1) * 64;
    view.setUint32(header + 4, section.type, true);
    view.setBigUint64(header + 0x18, BigInt(section.at), true);
    view.setBigUint64(header + 0x20, BigInt(section.bytes.length), true);
    view.setUint32(header + 0x28, section.link, true);
    view.setBigUint64(header + 0x38, BigInt(section.entrySize), true);
  }
  return file;
}
