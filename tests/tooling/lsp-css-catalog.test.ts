import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { generate } from "../../tools/commands/fixtures/generate-css-catalog.ts";

const ROOT = fileURLToPath(new URL("../../", import.meta.url));
const SOURCE = new URL("../../tools/data/css/fixtures/web-custom-data.json", import.meta.url);
const COLORS = new URL("../../tools/data/css/named-colors.json", import.meta.url);
const PIN = new URL("../../tools/data/css/source.json", import.meta.url);
const FAMILIES = [
  ["properties", 888, 888],
  ["atDirectives", 25, 25],
  ["pseudoClasses", 111, 110],
  ["pseudoElements", 90, 88],
] as const;

function record(value: unknown): asserts value is Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value));
}

function array(value: unknown): readonly unknown[] {
  assert.ok(Array.isArray(value));
  return value;
}

function text(value: unknown): string {
  assert.ok(typeof value === "string");
  return value;
}

/** Parse the actual Rust literals, rather than checking generated source spelling. */
class Reader {
  private offset = 0;
  private readonly bytes: Buffer;
  constructor(bytes: Buffer) {
    this.bytes = bytes;
  }

  take(size: number): Buffer {
    assert.ok(Number.isSafeInteger(size) && size >= 0);
    assert.ok(this.offset + size <= this.bytes.length, "Truncated Rust semantic packet");
    const value = this.bytes.subarray(this.offset, this.offset + size);
    this.offset += size;
    return value;
  }

  count(): number {
    const value = this.take(8).readBigUInt64LE();
    assert.ok(value <= BigInt(Number.MAX_SAFE_INTEGER));
    return Number(value);
  }

  string(expected: unknown): void {
    assert.deepEqual(this.take(this.count()), Buffer.from(text(expected), "utf8"));
  }

  present(): boolean {
    const flag = this.take(1)[0];
    assert.ok(flag === 0 || flag === 1);
    return flag === 1;
  }

  optional(expected: unknown): void {
    assert.equal(this.present(), expected !== undefined);
    if (expected !== undefined) this.string(expected);
  }

  strings(expected: unknown): void {
    const values = expected === undefined ? [] : array(expected);
    assert.equal(this.count(), values.length);
    for (const value of values) this.string(value);
  }

  baseline(expected: unknown): void {
    assert.equal(this.present(), expected !== undefined);
    if (expected === undefined) return;
    record(expected);
    this.string(expected.status);
    this.optional(expected.baseline_low_date);
    this.optional(expected.baseline_high_date);
  }

  value(expected: unknown): void {
    record(expected);
    this.string(expected.name);
    this.optional(expected.description);
    this.strings(expected.browsers);
    this.baseline(expected.baseline);
  }

  entry(expected: unknown): void {
    record(expected);
    this.string(expected.name);
    this.optional(expected.description);
    this.optional(expected.syntax);
    const references = expected.references === undefined ? [] : array(expected.references);
    assert.equal(this.count(), references.length);
    for (const reference of references) {
      record(reference);
      this.string(reference.name);
      this.string(reference.url);
    }
    const values = expected.values === undefined ? [] : array(expected.values);
    assert.equal(this.count(), values.length);
    for (const value of values) this.value(value);
    this.optional(expected.atRule);
    this.optional(expected.status);
    this.strings(expected.restrictions);
    this.strings(expected.browsers);
    this.baseline(expected.baseline);
    assert.equal(this.present(), expected.relevance !== undefined);
    if (expected.relevance !== undefined) {
      assert.equal(this.take(2).readUInt16LE(), expected.relevance);
    }
    const descriptors = expected.descriptors === undefined ? [] : array(expected.descriptors);
    assert.equal(this.count(), descriptors.length);
    for (const descriptor of descriptors) this.entry(descriptor);
    this.optional(expected.type);
  }

  done(): void {
    assert.equal(this.offset, this.bytes.length, "Unexpected Rust semantic packet remainder");
  }
}

function snapshot(url: URL, expectedSha: string): unknown {
  const bytes = readFileSync(url);
  assert.equal(createHash("sha256").update(bytes).digest("hex"), expectedSha);
  const value: unknown = JSON.parse(bytes.toString("utf8"));
  return value;
}

test("complete pinned CSS metadata and colors match compiled Rust and reproduce offline", () => {
  const source = snapshot(
    SOURCE,
    "7f228ab474664fe2a565fd88a2835fb6dda57a31da03aead96b14b965a5e205f",
  );
  const colors = snapshot(
    COLORS,
    "af6ce64a9eab7e00dbd09a104ee3e4d79477e3a05f5db88e75ac1e2dc74389e0",
  );
  const pin: unknown = JSON.parse(readFileSync(PIN, "utf8"));
  record(source);
  record(colors);
  record(pin);
  assert.equal(pin.upstream_commit, "2a253759802c9aba0d71b30e369fb65f1f062b5c");
  assert.equal(pin.upstream_blob, "6363c511b09a58747a5f31829d74dd772f840f0d");
  assert.equal(
    pin.snapshot_sha256,
    createHash("sha256").update(readFileSync(SOURCE)).digest("hex"),
  );
  record(pin.named_colors);
  assert.equal(pin.named_colors.upstream_blob, "62e633c658d580577a304d734e526cf3a6fe83d1");
  assert.equal(
    pin.named_colors.snapshot_sha256,
    createHash("sha256").update(readFileSync(COLORS)).digest("hex"),
  );
  assert.equal(source.version, 1.1);
  generate(true);
  const temporary = mkdtempSync(join(tmpdir(), "vize-css-catalog-"));
  try {
    const executable = join(temporary, "custody");
    execFileSync(
      process.env.RUSTC ?? "rustc",
      [
        "--edition=2024",
        "-Dwarnings",
        "tools/commands/fixtures/css-catalog-custody.rs",
        "-o",
        executable,
      ],
      { cwd: ROOT, stdio: "pipe" },
    );
    const reader = new Reader(execFileSync(executable, [], { maxBuffer: 4 * 1024 * 1024 }));
    assert.deepEqual(reader.take(8), Buffer.from("VIZECSS1"));
    let ownedValues = 0;
    for (const [family, rawCount, effectiveCount] of FAMILIES) {
      const original = array(source[family]);
      assert.equal(original.length, rawCount);
      const first = new Map<string, unknown>();
      for (const row of original) {
        record(row);
        const name = text(row.name);
        if (!first.has(name)) first.set(name, row);
        if (family === "properties") {
          ownedValues += row.values === undefined ? 0 : array(row.values).length;
        }
      }
      reader.string(family);
      assert.equal(first.size, effectiveCount);
      assert.equal(reader.count(), effectiveCount);
      for (const name of [...first.keys()].sort()) reader.entry(first.get(name));
    }
    assert.equal(ownedValues, 2148);
    record(colors.colors);
    record(colors.colorKeywords);
    assert.equal(Object.keys(colors.colors).length, 148);
    assert.equal(Object.keys(colors.colorKeywords).length, 2);
    const values = [...Object.entries(colors.colors), ...Object.entries(colors.colorKeywords)]
      .map(([name, description]) => ({ name, description: text(description) }))
      .sort((a, b) => (a.name.toLowerCase() < b.name.toLowerCase() ? -1 : 1));
    reader.string("colors");
    assert.equal(reader.count(), 150);
    for (const value of values) reader.value(value);
    reader.done();
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
});
