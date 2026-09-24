import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import { createDataCodewords, encodeQrCode, qrCodeCapacity } from "./qr-code-encoder.ts";
import { qrCodeKanjiEncoder } from "./qr-code-kanji.ts";
import {
  createQrCodeAlphanumericSegment,
  createQrCodeByteSegment,
  createQrCodeEciSegment,
  createQrCodeKanjiSegment,
  createQrCodeNumericSegment,
  createQrCodeSegments,
  qrCodeSegmentBitLength,
} from "./qr-code-segments.ts";
import { characterCountBits, dataCodewordCount } from "./qr-code-tables.ts";
import type {
  QrCodeErrorCorrection,
  QrCodeKanjiEncoder,
  QrCodeMode,
  QrCodeSegment,
  QrCodeVersion,
} from "./qr-code-types.ts";

const ALPHANUMERIC = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ $%*+-./:";
const RANGE_VERSIONS: readonly QrCodeVersion[] = [1, 10, 27];

/** Deterministic pseudo-random generator so failures are reproducible. */
function random(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (Math.imul(state, 1_103_515_245) + 12_345) >>> 0;
    return state / 2 ** 32;
  };
}

function randomText(next: () => number, alphabet: readonly string[], length: number): string {
  let text = "";
  for (let index = 0; index < length; index++) {
    text += alphabet[Math.floor(next() * alphabet.length)] ?? "";
  }
  return text;
}

function modesFor(character: string, kanji: QrCodeKanjiEncoder): QrCodeMode[] {
  const modes: QrCodeMode[] = ["byte"];
  if (ALPHANUMERIC.includes(character)) modes.push("alphanumeric");
  if (character >= "0" && character <= "9") modes.push("numeric");
  if (kanji.toShiftJis(character.codePointAt(0) ?? 0) !== undefined) modes.push("kanji");
  return modes;
}

function segmentFor(mode: QrCodeMode, text: string): QrCodeSegment {
  switch (mode) {
    case "numeric":
      return createQrCodeNumericSegment(text);
    case "alphanumeric":
      return createQrCodeAlphanumericSegment(text);
    case "byte":
      return createQrCodeByteSegment(text);
    case "kanji":
      return createQrCodeKanjiSegment(text, qrCodeKanjiEncoder);
  }
}

/** Exhaustive minimum over every split of the text and every legal mode per piece. */
function bruteForceBits(text: string, version: QrCodeVersion): number {
  const characters = Array.from(text);
  let best = Number.POSITIVE_INFINITY;
  const visit = (start: number, bits: number): void => {
    if (bits >= best) return;
    if (start === characters.length) {
      best = bits;
      return;
    }
    for (let end = start + 1; end <= characters.length; end++) {
      const piece = characters.slice(start, end);
      const shared = piece
        .map((character) => modesFor(character, qrCodeKanjiEncoder))
        .reduce((left, right) => left.filter((mode) => right.includes(mode)));
      for (const mode of shared) {
        const segment = segmentFor(mode, piece.join(""));
        visit(end, bits + qrCodeSegmentBitLength([segment], version));
      }
    }
  };
  visit(0, 0);
  return best;
}

/** Test-only decoder for the data bit stream (mode, count, payload) of a symbol. */
function decodeDataCodewords(codewords: readonly number[], version: QrCodeVersion): string {
  const bits = codewords.flatMap((byte) =>
    Array.from({ length: 8 }, (_, index) => (byte >>> (7 - index)) & 1),
  );
  let position = 0;
  const read = (length: number): number => {
    let value = 0;
    for (let index = 0; index < length; index++) value = value * 2 + (bits[position++] ?? 0);
    return value;
  };
  let text = "";
  let eci: number | null = null;
  const bytes: number[] = [];
  const flushBytes = (): void => {
    if (bytes.length === 0) return;
    text += new TextDecoder(eci === 20 ? "shift_jis" : "utf-8").decode(new Uint8Array(bytes));
    bytes.length = 0;
  };
  while (position + 4 <= bits.length) {
    const indicator = read(4);
    if (indicator === 0) break;
    if (indicator === 0x7) {
      flushBytes();
      const first = read(8);
      if (first < 0x80) eci = first;
      else if (first < 0xc0) eci = ((first & 0x3f) << 8) | read(8);
      else eci = ((first & 0x1f) << 16) | read(16);
      continue;
    }
    const mode: QrCodeMode =
      indicator === 0x1
        ? "numeric"
        : indicator === 0x2
          ? "alphanumeric"
          : indicator === 0x4
            ? "byte"
            : "kanji";
    const count = read(characterCountBits(mode, version));
    if (mode !== "byte") flushBytes();
    if (mode === "numeric") {
      for (let remaining = count; remaining > 0; remaining -= 3) {
        const digits = Math.min(3, remaining);
        text += String(read(digits * 3 + 1)).padStart(digits, "0");
      }
    } else if (mode === "alphanumeric") {
      for (let remaining = count; remaining > 0; remaining -= 2) {
        if (remaining >= 2) {
          const pair = read(11);
          text += (ALPHANUMERIC[Math.floor(pair / 45)] ?? "") + (ALPHANUMERIC[pair % 45] ?? "");
        } else {
          text += ALPHANUMERIC[read(6)] ?? "";
        }
      }
    } else if (mode === "byte") {
      for (let index = 0; index < count; index++) bytes.push(read(8));
    } else {
      const shiftJis: number[] = [];
      for (let index = 0; index < count; index++) {
        const value = read(13);
        const offset = (Math.floor(value / 0xc0) << 8) | (value % 0xc0);
        const code = offset + (offset >= 0x1f00 ? 0xc140 : 0x8140);
        shiftJis.push(code >>> 8, code & 0xff);
      }
      text += new TextDecoder("shift_jis").decode(new Uint8Array(shiftJis));
    }
  }
  flushBytes();
  return text;
}

test("optimal segmentation matches an exhaustive search for every version range", () => {
  const alphabet = [...Array.from("0123456789ABCZ $:abé"), "点", "茗", "日", "😀"];
  const next = random(0x5eed);
  let mixed = 0;
  for (let sample = 0; sample < 400; sample++) {
    const text = randomText(next, alphabet, 1 + Math.floor(next() * 7));
    for (const version of RANGE_VERSIONS) {
      const segments = createQrCodeSegments(text, { version, kanji: qrCodeKanjiEncoder });
      assert.equal(
        qrCodeSegmentBitLength(segments, version),
        bruteForceBits(text, version),
        `${JSON.stringify(text)} at version ${version}`,
      );
      assert.equal(segments.map((segment) => segment.count).reduce((a, b) => a + b, 0) > 0, true);
      if (new Set(segments.map((segment) => segment.mode)).size > 1) mixed++;
    }
  }
  assert.ok(mixed > 100, "the sample exercises mixed segments");
});

test("mixes modes to beat single-mode encoding and re-optimizes per version range", () => {
  const url = "HTTPS://EXAMPLE.COM/ORDER/31415926535897932384626";
  const optimal = createQrCodeSegments(url);
  assert.deepEqual(
    optimal.map((segment) => [segment.mode, segment.count]),
    [
      ["alphanumeric", 26],
      ["numeric", 23],
    ],
  );
  const single = createQrCodeSegments(url, { segmentation: "single" });
  assert.deepEqual(
    single.map((segment) => segment.mode),
    ["alphanumeric"],
  );
  assert.ok(qrCodeSegmentBitLength(optimal, 1) < qrCodeSegmentBitLength(single, 1));

  // Count-indicator widths change at versions 10 and 27, and so does the optimum.
  const text = "177aA958506755C";
  const summaries = RANGE_VERSIONS.map((version) =>
    createQrCodeSegments(text, { version })
      .map((segment) => `${segment.mode}:${segment.count}`)
      .join(","),
  );
  assert.deepEqual(summaries, [
    "byte:5,numeric:9,alphanumeric:1",
    "byte:5,numeric:9,alphanumeric:1",
    "byte:4,alphanumeric:11",
  ]);

  const matrix = encodeQrCode(url, { errorCorrection: "L" });
  assert.equal(matrix.mode, "mixed");
  assert.deepEqual(matrix.segments, [
    { mode: "alphanumeric", count: 26 },
    { mode: "numeric", count: 23 },
  ]);
  assert.equal(matrix.eci, null);
  assert.equal(
    encodeQrCode(url, { errorCorrection: "L", segmentation: "single" }).mode,
    "alphanumeric",
  );
});

test("encodes segments whose data stream decodes back to the original text", () => {
  const alphabet = [
    ...Array.from("0123456789ABCDEFXYZ $%*+-./:abcxyzé€"),
    "点",
    "茗",
    "漢",
    "字",
    "ア",
    "😀",
  ];
  const next = random(0xc0ffee);
  const levels: readonly QrCodeErrorCorrection[] = ["L", "M", "Q", "H"];
  for (let sample = 0; sample < 200; sample++) {
    const text = randomText(next, alphabet, 1 + Math.floor(next() * 120));
    const kanji = sample % 2 === 0 ? qrCodeKanjiEncoder : undefined;
    const ecc = levels[sample % 4] ?? "M";
    const matrix = encodeQrCode(text, { errorCorrection: ecc, kanji, utf8Eci: sample % 3 === 0 });
    const segments = [
      ...(matrix.eci === null ? [] : [createQrCodeEciSegment(matrix.eci)]),
      ...createQrCodeSegments(text, { kanji, version: matrix.version }),
    ];
    assert.deepEqual(
      segments.map((segment) =>
        segment.mode === "eci"
          ? { mode: "eci", count: matrix.eci }
          : { mode: segment.mode, count: segment.count },
      ),
      matrix.segments,
    );
    const codewords = createDataCodewords(segments, matrix.version, matrix.errorCorrection);
    assert.equal(decodeDataCodewords(codewords, matrix.version), text, JSON.stringify(text));
  }
});

test("chooses the smallest version using the segments of each version range", () => {
  const next = random(42);
  for (let sample = 0; sample < 60; sample++) {
    const text = randomText(next, Array.from("0123456789ABC:ab"), 20 + Math.floor(next() * 1500));
    const matrix = encodeQrCode(text, { errorCorrection: "M" });
    const bits = qrCodeSegmentBitLength(
      createQrCodeSegments(text, { version: matrix.version }),
      matrix.version,
    );
    assert.ok(bits <= dataCodewordCount(matrix.version, "M") * 8);
    if (matrix.version > 1) {
      const smaller = (matrix.version - 1) as QrCodeVersion;
      const smallerBits = qrCodeSegmentBitLength(
        createQrCodeSegments(text, { version: smaller }),
        smaller,
      );
      assert.ok(smallerBits > dataCodewordCount(smaller, "M") * 8, `${text.length} chars`);
    }
  }
});

test("maps JIS X 0208 characters to Shift_JIS and packs 13-bit Kanji values", () => {
  // ISO/IEC 18004 Kanji examples: 点 (0x935F) -> 0x0D9F, 茗 (0xE4AA) -> 0x1AAA.
  assert.equal(qrCodeKanjiEncoder.toShiftJis("点".codePointAt(0) ?? 0), 0x935f);
  assert.equal(qrCodeKanjiEncoder.toShiftJis("茗".codePointAt(0) ?? 0), 0xe4aa);
  const segment = createQrCodeKanjiSegment("点茗", qrCodeKanjiEncoder);
  assert.equal(segment.mode, "kanji");
  assert.equal(segment.count, 2);
  assert.equal(segment.bits.join(""), "0110110011111" + "1101010101010");

  const decoder = new TextDecoder("shift_jis", { fatal: true });
  let checked = 0;
  for (let lead = 0x81; lead <= 0xea; lead++) {
    if ((lead > 0x9f && lead < 0xe0) || lead === 0x87) continue;
    for (let trail = 0x40; trail <= 0xfc; trail++) {
      if (trail === 0x7f) continue;
      let character: string;
      try {
        character = decoder.decode(new Uint8Array([lead, trail]));
      } catch {
        continue;
      }
      const code = qrCodeKanjiEncoder.toShiftJis(character.codePointAt(0) ?? 0);
      assert.ok(code !== undefined, character);
      assert.equal(decoder.decode(new Uint8Array([code >>> 8, code & 0xff])), character);
      checked++;
    }
  }
  assert.ok(checked > 6800, `${checked} characters round-trip`);

  // JIS spellings of characters that code page 932 maps elsewhere also encode.
  assert.equal(qrCodeKanjiEncoder.toShiftJis(0x301c), 0x8160);
  assert.equal(qrCodeKanjiEncoder.toShiftJis(0x2212), 0x817c);
  assert.equal(qrCodeKanjiEncoder.toShiftJis("A".codePointAt(0) ?? 0), undefined);
  assert.equal(qrCodeKanjiEncoder.toShiftJis("①".codePointAt(0) ?? 0), undefined);
  assert.equal(qrCodeKanjiEncoder.toShiftJis(0x1f600), undefined);
  assert.ok(Object.isFrozen(qrCodeKanjiEncoder));
});

test("uses Kanji mode only with an encoder and validates Kanji input", () => {
  const text = "日本語の漢字";
  const withKanji = encodeQrCode(text, { kanji: qrCodeKanjiEncoder });
  assert.equal(withKanji.mode, "kanji");
  assert.deepEqual(withKanji.segments, [{ mode: "kanji", count: 6 }]);
  assert.equal(encodeQrCode(text).mode, "byte");
  assert.equal(
    encodeQrCode(text, { kanji: qrCodeKanjiEncoder, segmentation: "single" }).mode,
    "kanji",
  );
  assert.equal(
    qrCodeSegmentBitLength(createQrCodeSegments(text, { kanji: qrCodeKanjiEncoder }), 1),
    4 + 8 + 6 * 13,
  );
  assert.throws(() => encodeQrCode(text, { mode: "kanji" }), /VIZE_UI_QR_INVALID_MODE/);
  assert.throws(
    () => encodeQrCode("漢A", { mode: "kanji", kanji: qrCodeKanjiEncoder }),
    /VIZE_UI_QR_INVALID_MODE/,
  );
  const outOfRange: QrCodeKanjiEncoder = { toShiftJis: () => 0xa0a0 };
  assert.throws(() => createQrCodeKanjiSegment("x", outOfRange), /VIZE_UI_QR_INVALID_MODE/);
  assert.equal(qrCodeCapacity(1, "L", "kanji"), 10);
  assert.equal(qrCodeCapacity(40, "L", "kanji"), 1817);
  assert.equal(qrCodeCapacity(40, "H", "kanji"), 784);
});

test("writes ECI headers with 8, 16, and 24 bit designators", () => {
  const cases: readonly [number, string][] = [
    [3, "00000011"],
    [127, "01111111"],
    [128, "10" + "00000010000000"],
    [16_383, "10" + "11111111111111"],
    [16_384, "110" + "000000100000000000000"],
    [999_999, "110" + (999_999).toString(2).padStart(21, "0")],
  ];
  for (const [designator, bits] of cases) {
    const segment = createQrCodeEciSegment(designator);
    assert.equal(segment.mode, "eci");
    assert.equal(segment.bits.join(""), bits);
    assert.equal(qrCodeSegmentBitLength([segment], 40), 4 + bits.length);
  }
  for (const invalid of [-1, 1_000_000, 1.5, Number.NaN]) {
    assert.throws(() => createQrCodeEciSegment(invalid), /VIZE_UI_QR_INVALID_OPTION/);
  }

  const data = createDataCodewords(
    [createQrCodeEciSegment(26), createQrCodeByteSegment("é")],
    1,
    "L",
  );
  assert.equal(
    data
      .slice(0, 4)
      .map((byte) => byte.toString(16).padStart(2, "0"))
      .join(" "),
    "71 a4 02 c3",
  );
});

test("utf8Eci and eci prefix the data and count toward capacity", () => {
  const seventeen = "abcdefghijklmnopq";
  assert.equal(encodeQrCode(seventeen, { errorCorrection: "L" }).version, 1);
  const declared = encodeQrCode(seventeen, { errorCorrection: "L", utf8Eci: true });
  assert.equal(declared.version, 2, "the 12-bit ECI header overflows version 1-L");
  assert.equal(declared.eci, 26);
  assert.deepEqual(declared.segments, [
    { mode: "eci", count: 26 },
    { mode: "byte", count: 17 },
  ]);
  assert.equal(declared.mode, "byte");
  assert.equal(encodeQrCode("x", { eci: 26, utf8Eci: true }).eci, 26);
  assert.equal(encodeQrCode("x", { eci: 20000 }).segments[0]?.count, 20000);
  assert.throws(() => encodeQrCode("x", { eci: 3, utf8Eci: true }), /VIZE_UI_QR_INVALID_OPTION/);
  assert.throws(() => encodeQrCode("x", { eci: -2 }), /VIZE_UI_QR_INVALID_OPTION/);
  // @ts-expect-error runtime guard for untyped callers
  assert.throws(() => encodeQrCode("x", { segmentation: "greedy" }), /VIZE_UI_QR_INVALID_OPTION/);
});

test("encodes explicit segment arrays as given", () => {
  const segments = [
    createQrCodeAlphanumericSegment("ID:"),
    createQrCodeNumericSegment("0042"),
    createQrCodeByteSegment(new Uint8Array([0xff])),
  ];
  const matrix = encodeQrCode(segments, { mode: "numeric", segmentation: "single" });
  assert.equal(matrix.mode, "mixed");
  assert.deepEqual(matrix.segments, [
    { mode: "alphanumeric", count: 3 },
    { mode: "numeric", count: 4 },
    { mode: "byte", count: 1 },
  ]);
  assert.equal(encodeQrCode([]).mode, null);
  assert.ok(Object.isFrozen(segments[0]));
  assert.ok(Object.isFrozen(segments[0]?.bits));
  assert.throws(() => createQrCodeNumericSegment("12a"), /VIZE_UI_QR_INVALID_MODE/);
  assert.throws(() => createQrCodeAlphanumericSegment("abc"), /VIZE_UI_QR_INVALID_MODE/);
  assert.deepEqual(
    createQrCodeSegments(new Uint8Array([1, 2])).map((segment) => segment.mode),
    ["byte"],
  );
  assert.throws(
    () => createQrCodeSegments(new Uint8Array([1]), { mode: "kanji" }),
    /VIZE_UI_QR_INVALID_MODE/,
  );
  // Counts that overflow the indicator make a segment list unencodable at that version.
  assert.equal(
    qrCodeSegmentBitLength([createQrCodeByteSegment(new Uint8Array(256))], 9),
    Number.POSITIVE_INFINITY,
  );
});
