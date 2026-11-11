/**
 * tsgo's own CPU profile (`--pprofDir`): the gzipped profile.proto Go writes,
 * decoded without `go tool pprof`. Go attributes inlined frames, which perf
 * cannot see in a stripped binary, so this is the Go-level cross-check of
 * the perf profile's tsgo role.
 */

import { readFileSync } from "node:fs";
import { gunzipSync } from "node:zlib";

import { cleanSymbol } from "./typecheck-profile-rules.ts";

type Field = { field: number; wire: number; value: bigint | Buffer };

function* fields(buf: Buffer): Generator<Field> {
  let at = 0;
  const varint = (): bigint => {
    let result = 0n;
    let shift = 0n;
    for (;;) {
      const byte = buf[at++];
      if (byte === undefined) throw new Error("truncated varint");
      result |= BigInt(byte & 0x7f) << shift;
      if ((byte & 0x80) === 0) return result;
      shift += 7n;
    }
  };
  while (at < buf.length) {
    const key = Number(varint());
    const wire = key & 7;
    const field = key >>> 3;
    if (wire === 0) yield { field, wire, value: varint() };
    else if (wire === 2) {
      const length = Number(varint());
      yield { field, wire, value: buf.subarray(at, at + length) };
      at += length;
    } else if (wire === 1) {
      yield { field, wire, value: buf.readBigUInt64LE(at) };
      at += 8;
    } else if (wire === 5) {
      yield { field, wire, value: BigInt(buf.readUInt32LE(at)) };
      at += 4;
    } else throw new Error(`unsupported protobuf wire type ${wire}`);
  }
}

/** A repeated integer field, packed or not. */
function integers(value: bigint | Buffer): number[] {
  if (typeof value === "bigint") return [Number(value)];
  const out: number[] = [];
  let at = 0;
  while (at < value.length) {
    let result = 0n;
    let shift = 0n;
    for (;;) {
      const byte = value[at++]!;
      result |= BigInt(byte & 0x7f) << shift;
      if ((byte & 0x80) === 0) break;
      shift += 7n;
    }
    out.push(Number(BigInt.asIntN(64, result)));
  }
  return out;
}

export type PprofTop = {
  sampleType: string;
  unit: string;
  total: number;
  functions: Array<{ name: string; flat: number; cum: number }>;
};

/** Flat and cumulative value per function of the profile's `sampleType` (default: the last, cpu nanoseconds). */
export function pprofTop(gz: Buffer, sampleType = "cpu"): PprofTop {
  const raw = gz[0] === 0x1f && gz[1] === 0x8b ? gunzipSync(gz) : gz;
  const strings: string[] = [];
  const types: Array<[number, number]> = [];
  const samples: Array<{ locations: number[]; values: number[] }> = [];
  const locations = new Map<number, number[]>();
  const functions = new Map<number, number>();
  for (const { field, value } of fields(raw)) {
    if (field === 6) strings.push((value as Buffer).toString("utf8"));
    else if (field === 1) {
      let type = 0;
      let unit = 0;
      for (const sub of fields(value as Buffer)) {
        if (sub.field === 1) type = Number(sub.value);
        if (sub.field === 2) unit = Number(sub.value);
      }
      types.push([type, unit]);
    } else if (field === 2) {
      const sample = { locations: [] as number[], values: [] as number[] };
      for (const sub of fields(value as Buffer)) {
        if (sub.field === 1) sample.locations.push(...integers(sub.value));
        if (sub.field === 2) sample.values.push(...integers(sub.value));
      }
      samples.push(sample);
    } else if (field === 4) {
      let id = 0;
      const lines: number[] = [];
      for (const sub of fields(value as Buffer)) {
        if (sub.field === 1) id = Number(sub.value);
        if (sub.field === 4) {
          for (const line of fields(sub.value as Buffer)) {
            if (line.field === 1) lines.push(Number(line.value));
          }
        }
      }
      locations.set(id, lines);
    } else if (field === 5) {
      let id = 0;
      let name = 0;
      for (const sub of fields(value as Buffer)) {
        if (sub.field === 1) id = Number(sub.value);
        if (sub.field === 2) name = Number(sub.value);
      }
      functions.set(id, name);
    }
  }
  const typeNames = types.map(([type, unit]) => [strings[type] ?? "", strings[unit] ?? ""]);
  let index = typeNames.findIndex(([type]) => type === sampleType);
  if (index < 0) index = typeNames.length - 1;
  const [type, unit] = typeNames[index] ?? ["", ""];
  const flat = new Map<string, number>();
  const cum = new Map<string, number>();
  let total = 0;
  for (const sample of samples) {
    const value = sample.values[index] ?? 0;
    total += value;
    // Leaf location first; within a location the first line is the innermost inlined call.
    const names = sample.locations.flatMap((id) =>
      (locations.get(id) ?? []).map((fn) => cleanSymbol(strings[functions.get(fn) ?? 0] ?? "?")),
    );
    if (names[0]) flat.set(names[0], (flat.get(names[0]) ?? 0) + value);
    for (const name of new Set(names)) cum.set(name, (cum.get(name) ?? 0) + value);
  }
  return {
    sampleType: type!,
    unit: unit!,
    total,
    functions: [...cum].map(([name, value]) => ({ name, flat: flat.get(name) ?? 0, cum: value })),
  };
}

export const readPprofTop = (path: string): PprofTop => pprofTop(readFileSync(path));
