/**
 * Function tables for stripped x86-64 ELF executables, so perf samples in
 * them get function identities. perf reads only `.symtab`/`.dynsym`, and
 * both hosts ship without a symbol table:
 *
 * - tsgo (Go, `-s -w`): `.gopclntab` still names every function, as
 *   `go tool addr2line` reads it.
 * - tsrs (Rust, BOLT, stripped): `.eh_frame_hdr` lists every function start,
 *   so a frame becomes `tsrs!fn@<start>`: stable for one binary, not a name.
 *
 * `vaddrOf` turns a runtime address into the ELF virtual address through the
 * PT_LOAD segment holding the mapped file offset (PIE or not).
 */

import { readFileSync } from "node:fs";

type Section = { name: string; type: number; addr: number; offset: number; size: number };
type Load = { offset: number; vaddr: number; filesz: number };

export type FunctionTable = {
  kind: "symtab" | "gopclntab" | "eh_frame_hdr" | "none";
  /** Sorted function start addresses (ELF virtual addresses). */
  starts: Float64Array;
  names: string[];
  loads: Load[];
};

const SHT_SYMTAB = 2;
const PT_LOAD = 1;

function u64(buf: Buffer, at: number): number {
  return Number(buf.readBigUInt64LE(at));
}

function sections(buf: Buffer): Section[] {
  const shoff = u64(buf, 0x28);
  const shentsize = buf.readUInt16LE(0x3a);
  const shnum = buf.readUInt16LE(0x3c);
  const shstrndx = buf.readUInt16LE(0x3e);
  const raw = Array.from({ length: shnum }, (_, index) => {
    const at = shoff + index * shentsize;
    return {
      nameOffset: buf.readUInt32LE(at),
      type: buf.readUInt32LE(at + 4),
      addr: u64(buf, at + 0x10),
      offset: u64(buf, at + 0x18),
      size: u64(buf, at + 0x20),
    };
  });
  const strtab = raw[shstrndx];
  return raw.map(({ nameOffset, ...section }) => ({
    ...section,
    name: strtab ? cString(buf, strtab.offset + nameOffset) : "",
  }));
}

function loads(buf: Buffer): Load[] {
  const phoff = u64(buf, 0x20);
  const phentsize = buf.readUInt16LE(0x36);
  const phnum = buf.readUInt16LE(0x38);
  const out: Load[] = [];
  for (let index = 0; index < phnum; index++) {
    const at = phoff + index * phentsize;
    if (buf.readUInt32LE(at) !== PT_LOAD) continue;
    out.push({ offset: u64(buf, at + 8), vaddr: u64(buf, at + 0x10), filesz: u64(buf, at + 0x20) });
  }
  return out;
}

function cString(buf: Buffer, at: number): string {
  const end = buf.indexOf(0, at);
  return buf.toString("utf8", at, end < 0 ? buf.length : end);
}

/** Go 1.18+ `.gopclntab`: functab of (entryoff, funcoff), names in funcnametab. */
export function goFunctions(
  pcln: Buffer,
  textStart: number,
): { starts: number[]; names: string[] } {
  const magic = pcln.readUInt32LE(0);
  if (magic !== 0xfffffff0 && magic !== 0xfffffff1) {
    throw new Error(`unsupported .gopclntab magic 0x${magic.toString(16)}`);
  }
  const ptrSize = pcln.readUInt8(7);
  if (ptrSize !== 8) throw new Error(`.gopclntab pointer size ${ptrSize}`);
  const word = (index: number) => u64(pcln, 8 + index * ptrSize);
  const nfunc = word(0);
  const funcnameOffset = word(3);
  const pcOffset = word(7);
  const starts: number[] = [];
  const names: string[] = [];
  for (let index = 0; index < nfunc; index++) {
    const at = pcOffset + index * 8;
    const entryOff = pcln.readUInt32LE(at);
    const funcOff = pcln.readUInt32LE(at + 4);
    const nameOff = pcln.readInt32LE(pcOffset + funcOff + 4);
    starts.push(textStart + entryOff);
    names.push(cString(pcln, funcnameOffset + nameOff));
  }
  return { starts, names };
}

/** `.eh_frame_hdr` binary search table: one entry per FDE, datarel|sdata4 encoded. */
export function ehFrameStarts(hdr: Buffer, hdrAddr: number): number[] {
  const version = hdr.readUInt8(0);
  const ptrEnc = hdr.readUInt8(1);
  const countEnc = hdr.readUInt8(2);
  const tableEnc = hdr.readUInt8(3);
  // pcrel|sdata4 frame pointer, udata4 count, datarel|sdata4 table: what every linker writes.
  if (version !== 1 || ptrEnc !== 0x1b || countEnc !== 0x03 || tableEnc !== 0x3b) {
    throw new Error(
      `unsupported .eh_frame_hdr encodings ${[version, ptrEnc, countEnc, tableEnc].join(",")}`,
    );
  }
  const count = hdr.readUInt32LE(8);
  const starts: number[] = [];
  for (let index = 0; index < count; index++) {
    starts.push(hdrAddr + hdr.readInt32LE(12 + index * 8));
  }
  return starts;
}

/** The best function table a stripped binary offers; `symtab` means perf already names it. */
export function functionTable(path: string): FunctionTable {
  const buf = readFileSync(path);
  if (buf.readUInt32BE(0) !== 0x7f454c46 || buf.readUInt8(4) !== 2 || buf.readUInt8(5) !== 1) {
    throw new Error(`${path}: not a 64-bit little-endian ELF`);
  }
  const all = sections(buf);
  const segments = loads(buf);
  const empty = (kind: FunctionTable["kind"]): FunctionTable => ({
    kind,
    starts: new Float64Array(),
    names: [],
    loads: segments,
  });
  if (all.some((section) => section.type === SHT_SYMTAB)) return empty("symtab");
  const pcln = all.find((section) => section.name === ".gopclntab");
  const text = all.find((section) => section.name === ".text");
  if (pcln && text) {
    const go = goFunctions(buf.subarray(pcln.offset, pcln.offset + pcln.size), text.addr);
    return sorted("gopclntab", go.starts, go.names, segments);
  }
  const hdr = all.find((section) => section.name === ".eh_frame_hdr");
  if (hdr) {
    const starts = ehFrameStarts(buf.subarray(hdr.offset, hdr.offset + hdr.size), hdr.addr);
    return sorted(
      "eh_frame_hdr",
      starts,
      starts.map((start) => `fn@${start.toString(16)}`),
      segments,
    );
  }
  return empty("none");
}

function sorted(
  kind: FunctionTable["kind"],
  starts: number[],
  names: string[],
  segments: Load[],
): FunctionTable {
  const order = starts.map((_, index) => index).sort((a, b) => starts[a]! - starts[b]!);
  return {
    kind,
    starts: Float64Array.from(order, (index) => starts[index]!),
    names: order.map((index) => names[index]!),
    loads: segments,
  };
}

/** ELF virtual address of a mapped file offset; null outside every PT_LOAD. */
export function vaddrOf(table: FunctionTable, fileOffset: number): number | null {
  for (const load of table.loads) {
    if (fileOffset >= load.offset && fileOffset < load.offset + load.filesz) {
      return fileOffset - load.offset + load.vaddr;
    }
  }
  return null;
}

/** The function whose start is the closest at or below `vaddr`. */
export function functionAt(table: FunctionTable, vaddr: number): string | null {
  const { starts } = table;
  let low = 0;
  let high = starts.length - 1;
  if (high < 0 || vaddr < starts[0]!) return null;
  while (low < high) {
    const mid = (low + high + 1) >> 1;
    if (starts[mid]! <= vaddr) low = mid;
    else high = mid - 1;
  }
  return table.names[low] ?? null;
}
