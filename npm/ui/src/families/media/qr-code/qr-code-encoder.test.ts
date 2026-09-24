import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  QrCodeEncodeError,
  addErrorCorrectionAndInterleave,
  createDataCodewords,
  createQrCodeSegments,
  encodeQrCode,
  formatInformationBits,
  gf256Multiply,
  isQrCodeModuleDark,
  qrCodeCapacity,
  qrCodePenaltyScore,
  reedSolomonGenerator,
  reedSolomonRemainder,
  versionInformationBits,
} from "./qr-code-encoder.ts";
import {
  alignmentPatternPositions,
  dataCodewordCount,
  qrCodeSizeOf,
  rawDataModuleCount,
} from "./qr-code-tables.ts";
import type {
  QrCodeErrorCorrection,
  QrCodeMask,
  QrCodeMatrix,
  QrCodeVersion,
} from "./qr-code-types.ts";

const LEVELS: readonly QrCodeErrorCorrection[] = ["L", "M", "Q", "H"];
const MASKS: readonly QrCodeMask[] = [0, 1, 2, 3, 4, 5, 6, 7];
const VERSIONS = Array.from({ length: 40 }, (_, index) => index + 1).filter(
  (value): value is QrCodeVersion => value >= 1 && value <= 40,
);

function art(matrix: QrCodeMatrix): readonly string[] {
  const rows: string[] = [];
  for (let y = 0; y < matrix.size; y++) {
    let row = "";
    for (let x = 0; x < matrix.size; x++) row += isQrCodeModuleDark(matrix, x, y) ? "#" : ".";
    rows.push(row);
  }
  return rows;
}

function hexRows(matrix: QrCodeMatrix): readonly string[] {
  return art(matrix).map((row) =>
    BigInt(`0b1${row.replaceAll("#", "1").replaceAll(".", "0")}`).toString(16),
  );
}

function toHex(values: readonly number[]): string {
  return values.map((value) => value.toString(16).padStart(2, "0").toUpperCase()).join(" ");
}

// Reference symbols produced by Project Nayuki's QR Code generator algorithm
// (identical version, mask choice, and module grid; quiet zone excluded).
const HELLO_WORLD_LOW = [
  "#######...###.#######",
  "#.....#.###.#.#.....#",
  "#.###.#...###.#.###.#",
  "#.###.#.##..#.#.###.#",
  "#.###.#..#..#.#.###.#",
  "#.....#.#..#..#.....#",
  "#######.#.#.#.#######",
  ".........#...........",
  "#####.###..#.#.#.#.#.",
  "#...##..#.#.##..###.#",
  "##.##.#.#.#.###..###.",
  ".##..#.##..###.#.##..",
  "..#####.#...#.##....#",
  "........##...#####...",
  "#######.#...####..##.",
  "#.....#..#..##.#.###.",
  "#.###.#.#.#####.#..##",
  "#.###.#.###....###...",
  "#.###.#.#..##.##..#..",
  "#.....#.###.##..###..",
  "#######.##.#..#.#..#.",
];

const NUMERIC_MEDIUM = [
  "#######...###.#######",
  "#.....#.###...#.....#",
  "#.###.#..##...#.###.#",
  "#.###.#..#.##.#.###.#",
  "#.###.#.##.##.#.###.#",
  "#.....#....#..#.....#",
  "#######.#.#.#.#######",
  ".....................",
  "#.#.#.#...#.#...#..#.",
  "##.#....#.##.#.#...#.",
  "...##.###.##.###.###.",
  "##..##.#.#.###.##..#.",
  "..#..###.###.###....#",
  "........#.#...#....#.",
  "#######.....#...#...#",
  "#.....#...#...#..#.##",
  "#.###.#.###.#.#.###.#",
  "#.###.#..#.#.#.#.###.",
  "#.###.#.##.#.###..#.#",
  "#.....#....###.###...",
  "#######.#..#.###..#.#",
];

const ALPHANUMERIC_QUARTILE = [
  "#######.##....#######",
  "#.....#.#..#..#.....#",
  "#.###.#.#..##.#.###.#",
  "#.###.#.#.....#.###.#",
  "#.###.#.#.#...#.###.#",
  "#.....#...#...#.....#",
  "#######.#.#.#.#######",
  "........#............",
  ".##.#.##....#.#.#####",
  ".#......####....#...#",
  "..##.###.##...#.##...",
  ".##.##.#..##.#.#.###.",
  "#...#.#.#.###.###.#.#",
  "........##.#..#...#.#",
  "#######.#.#....#.##..",
  "#.....#..#.##.##.#...",
  "#.###.#.#.#...#######",
  "#.###.#..#.#.#.#...#.",
  "#.###.#.#..#.###.#..#",
  "#.....#.#.####...#.##",
  "#######....#.###....#",
];

const URL =
  "https://github.com/ubugeeei-prod/vize/tree/main/npm/ui/src/families/media/qr-code?ref=vize";
const URL_HIGH_VERSION_9 = [
  "3fcb7cc3caf47f",
  "30408b6877ae41",
  "375b569f3b625d",
  "375ac68e05fd5d",
  "3745f4ffe5c45d",
  "30458c91837c41",
  "3fd5555555557f",
  "2007c5b13ed400",
  "2369b17fa1550c",
  "3d1deeb234ed40",
  "36663dfe6592f0",
  "22b767161ef42b",
  "3c72116ba97799",
  "302d1a8fd76cb3",
  "234c0ea97cf385",
  "2db27b0d98aecd",
  "2fc8a68ca5a493",
  "39193a8620b6f0",
  "2772bd0b4c1835",
  "2210663d791fe5",
  "2df3e4a376dfbe",
  "39b1210578afea",
  "24e518601053c8",
  "23a9d30d9ea12b",
  "33fa393f1111f0",
  "3d143f718f7d15",
  "275a51f5f3c15f",
  "3714e8d14c451f",
  "31f7e2df7d07f2",
  "3136f68213afd4",
  "37c0908afd05dd",
  "391b3fc2dc1075",
  "22ff33b2f55245",
  "2f385dfb96e75c",
  "2a6f4c2697982e",
  "2689a1e18e23b8",
  "23587ff8295608",
  "35879f7d7b76b7",
  "38d229e4ccc0d1",
  "2a97a41d2c463f",
  "31614de0ed2843",
  "2db9954b62a658",
  "3bd4d4723d8a55",
  "2c22230bbc4c4f",
  "22583dffd30ffd",
  "201750119bc518",
  "3fd54975ed1950",
  "30440131240919",
  "37592fbf4d97f1",
  "3754ceeb2747e2",
  "374685af4cd271",
  "3042b87f7f47fd",
  "3fccb09ea7e3f0",
];

test("matches the reference byte-mode symbol for Hello, world! at level L", () => {
  const matrix = encodeQrCode("Hello, world!", { errorCorrection: "L" });
  assert.equal(matrix.version, 1);
  assert.equal(matrix.size, 21);
  assert.equal(matrix.mask, 2);
  assert.equal(matrix.mode, "byte");
  assert.equal(matrix.errorCorrection, "L");
  assert.deepEqual(art(matrix), HELLO_WORLD_LOW);
});

test("matches reference numeric and alphanumeric symbols", () => {
  const numeric = encodeQrCode("01234567", { errorCorrection: "M" });
  assert.equal(numeric.mode, "numeric");
  assert.equal(numeric.mask, 0);
  assert.deepEqual(art(numeric), NUMERIC_MEDIUM);

  const alphanumeric = encodeQrCode("HELLO WORLD", { errorCorrection: "Q" });
  assert.equal(alphanumeric.mode, "alphanumeric");
  assert.equal(alphanumeric.mask, 0);
  assert.deepEqual(art(alphanumeric), ALPHANUMERIC_QUARTILE);
});

test("matches a multi-block reference symbol with version information", () => {
  const matrix = encodeQrCode(URL, { errorCorrection: "H" });
  assert.equal(matrix.version, 9);
  assert.equal(matrix.size, 53);
  assert.equal(matrix.mask, 6);
  assert.deepEqual(hexRows(matrix), URL_HIGH_VERSION_9);
});

test("reproduces the ISO/IEC 18004 worked example codewords for 01234567 at 1-M", () => {
  const segments = createQrCodeSegments("01234567", "auto");
  const data = createDataCodewords(segments, 1, "M");
  assert.equal(toHex(data), "10 20 0C 56 61 80 EC 11 EC 11 EC 11 EC 11 EC 11");
  const generator = reedSolomonGenerator(10);
  assert.equal(toHex(reedSolomonRemainder(data, generator)), "A5 24 D4 C1 ED 36 C7 87 2C 55");
  assert.equal(
    toHex(addErrorCorrectionAndInterleave(data, 1, "M")),
    "10 20 0C 56 61 80 EC 11 EC 11 EC 11 EC 11 EC 11 A5 24 D4 C1 ED 36 C7 87 2C 55",
  );
});

test("encodes alphanumeric pairs from the standard AC-42 example", () => {
  const [segment] = createQrCodeSegments("AC-42", "auto");
  assert.equal(segment?.mode, "alphanumeric");
  assert.equal(segment?.count, 5);
  assert.equal(segment?.bits.join(""), "00111001110" + "11100111001" + "000010");
});

test("implements GF(256) arithmetic and Reed-Solomon generators", () => {
  assert.equal(gf256Multiply(0, 0x53), 0);
  assert.equal(gf256Multiply(1, 0x53), 0x53);
  assert.equal(gf256Multiply(2, 0x80), 0x1d);
  assert.equal(gf256Multiply(0x53, 0xca), gf256Multiply(0xca, 0x53));
  // Generator for 7 ECC codewords: alpha exponents 87 229 146 149 238 102 21.
  assert.deepEqual(reedSolomonGenerator(7), [127, 122, 154, 164, 11, 68, 117]);
  assert.throws(() => reedSolomonGenerator(0), /VIZE_UI_QR_INVALID_OPTION/);
});

test("computes format information for every level and mask", () => {
  // ISO/IEC 18004 table C.1 masked format information strings.
  const expected: Readonly<Record<QrCodeErrorCorrection, readonly string[]>> = {
    L: [
      "111011111000100",
      "111001011110011",
      "111110110101010",
      "111100010011101",
      "110011000101111",
      "110001100011000",
      "110110001000001",
      "110100101110110",
    ],
    M: [
      "101010000010010",
      "101000100100101",
      "101111001111100",
      "101101101001011",
      "100010111111001",
      "100000011001110",
      "100111110010111",
      "100101010100000",
    ],
    Q: [
      "011010101011111",
      "011000001101000",
      "011111100110001",
      "011101000000110",
      "010010010110100",
      "010000110000011",
      "010111011011010",
      "010101111101101",
    ],
    H: [
      "001011010001001",
      "001001110111110",
      "001110011100111",
      "001100111010000",
      "000011101100010",
      "000001001010101",
      "000110100001100",
      "000100000111011",
    ],
  };
  for (const level of LEVELS) {
    for (const mask of MASKS) {
      assert.equal(
        formatInformationBits(level, mask).toString(2).padStart(15, "0"),
        expected[level][mask],
        `${level}/${mask}`,
      );
    }
  }
});

test("computes version information for versions 7 through 40", () => {
  assert.equal(versionInformationBits(7).toString(2).padStart(18, "0"), "000111110010010100");
  assert.equal(versionInformationBits(21).toString(2).padStart(18, "0"), "010101011010000011");
  assert.equal(versionInformationBits(40).toString(2).padStart(18, "0"), "101000110001101001");
});

test("places alignment patterns at the standard centers", () => {
  assert.deepEqual(alignmentPatternPositions(1), []);
  assert.deepEqual(alignmentPatternPositions(2), [6, 18]);
  assert.deepEqual(alignmentPatternPositions(7), [6, 22, 38]);
  assert.deepEqual(alignmentPatternPositions(15), [6, 26, 48, 70]);
  assert.deepEqual(alignmentPatternPositions(22), [6, 26, 50, 74, 98]);
  assert.deepEqual(alignmentPatternPositions(32), [6, 34, 60, 86, 112, 138]);
  assert.deepEqual(alignmentPatternPositions(36), [6, 24, 50, 76, 102, 128, 154]);
  assert.deepEqual(alignmentPatternPositions(40), [6, 30, 58, 86, 114, 142, 170]);
});

test("matches the standard codeword capacity tables", () => {
  assert.equal(qrCodeSizeOf(1), 21);
  assert.equal(qrCodeSizeOf(40), 177);
  assert.equal(rawDataModuleCount(1), 208);
  assert.equal(rawDataModuleCount(40), 29648);
  assert.deepEqual(
    LEVELS.map((level) => dataCodewordCount(1, level)),
    [19, 16, 13, 9],
  );
  assert.deepEqual(
    LEVELS.map((level) => dataCodewordCount(40, level)),
    [2956, 2334, 1666, 1276],
  );
  // ISO/IEC 18004 table 7 character capacities.
  assert.equal(qrCodeCapacity(1, "L", "numeric"), 41);
  assert.equal(qrCodeCapacity(1, "L", "alphanumeric"), 25);
  assert.equal(qrCodeCapacity(1, "L", "byte"), 17);
  assert.equal(qrCodeCapacity(1, "H", "byte"), 7);
  assert.equal(qrCodeCapacity(10, "M", "alphanumeric"), 311);
  assert.equal(qrCodeCapacity(40, "L", "numeric"), 7089);
  assert.equal(qrCodeCapacity(40, "L", "alphanumeric"), 4296);
  assert.equal(qrCodeCapacity(40, "L", "byte"), 2953);
  assert.equal(qrCodeCapacity(40, "H", "byte"), 1273);
});

test("selects the smallest version at the exact capacity boundary", () => {
  for (const version of [1, 9, 10, 26, 27, 40] satisfies QrCodeVersion[]) {
    for (const level of LEVELS) {
      const bytes = qrCodeCapacity(version, level, "byte");
      const exact = encodeQrCode(new Uint8Array(bytes), { errorCorrection: level });
      assert.ok(exact.version <= version, `${version}-${level} holds ${bytes} bytes`);
      if (version < 40) {
        const over = encodeQrCode(new Uint8Array(bytes + 1), {
          errorCorrection: level,
          minVersion: version,
        });
        assert.ok(over.version > version, `${version}-${level} overflows at ${bytes + 1} bytes`);
      }
    }
  }
  for (const version of VERSIONS) {
    const digits = "7".repeat(qrCodeCapacity(version, "Q", "numeric"));
    assert.equal(
      encodeQrCode(digits, { errorCorrection: "Q", minVersion: version }).version,
      version,
    );
  }
  // Dozens of version-40 encodes: allow for loaded CI workers.
}, 30_000);

test("throws typed diagnostics for data overflow and invalid options", () => {
  const overflow = (): QrCodeMatrix => encodeQrCode("9".repeat(7090), { errorCorrection: "L" });
  assert.throws(overflow, (error: unknown) => {
    assert.ok(error instanceof QrCodeEncodeError);
    assert.equal(error.code, "VIZE_UI_QR_DATA_TOO_LONG");
    assert.match(error.message, /^VIZE_UI_QR_DATA_TOO_LONG: /);
    return true;
  });
  assert.throws(() => encodeQrCode("x".repeat(20), { version: 1 }), /VIZE_UI_QR_DATA_TOO_LONG/);
  assert.throws(() => encodeQrCode("abc", { mode: "numeric" }), /VIZE_UI_QR_INVALID_MODE/);
  assert.throws(() => encodeQrCode("abc", { mode: "alphanumeric" }), /VIZE_UI_QR_INVALID_MODE/);
  assert.throws(
    () => encodeQrCode(new Uint8Array([1]), { mode: "numeric" }),
    /VIZE_UI_QR_INVALID_MODE/,
  );
  assert.throws(
    () => encodeQrCode("a", { minVersion: 5, maxVersion: 2 }),
    /VIZE_UI_QR_INVALID_OPTION/,
  );
  // @ts-expect-error runtime guard for untyped callers
  assert.throws(() => encodeQrCode("a", { version: 41 }), /VIZE_UI_QR_INVALID_OPTION/);
  // @ts-expect-error runtime guard for untyped callers
  assert.throws(() => encodeQrCode("a", { mask: 8 }), /VIZE_UI_QR_INVALID_OPTION/);
  // @ts-expect-error runtime guard for untyped callers
  assert.throws(() => encodeQrCode("a", { errorCorrection: "X" }), /VIZE_UI_QR_INVALID_OPTION/);
});

test("honors fixed version, fixed mask, forced mode, and error correction boost", () => {
  const fixed = encodeQrCode("HELLO", { version: 5, mask: 3, errorCorrection: "L" });
  assert.equal(fixed.version, 5);
  assert.equal(fixed.size, 37);
  assert.equal(fixed.mask, 3);
  assert.equal(fixed.errorCorrection, "L");

  const forced = encodeQrCode("12345", { mode: "byte" });
  assert.equal(forced.mode, "byte");
  assert.equal(encodeQrCode("12345").mode, "numeric");
  assert.equal(encodeQrCode("hello").mode, "byte");
  assert.equal(encodeQrCode(new Uint8Array([0, 255])).mode, "byte");

  const boosted = encodeQrCode("HELLO", { errorCorrection: "L", boostErrorCorrection: true });
  assert.equal(boosted.version, 1);
  assert.equal(boosted.errorCorrection, "H");
  assert.equal(encodeQrCode("HELLO", { errorCorrection: "L" }).errorCorrection, "L");
});

test("encodes UTF-8 text and empty values", () => {
  const unicode = encodeQrCode("日本語", { errorCorrection: "L" });
  const bytes = encodeQrCode(new TextEncoder().encode("日本語"), { errorCorrection: "L" });
  assert.deepEqual(unicode.modules, bytes.modules);
  const empty = encodeQrCode("");
  assert.equal(empty.mode, null);
  assert.equal(empty.version, 1);
});

test("picks the mask with the lowest penalty and returns frozen output", () => {
  const auto = encodeQrCode("Hello, world!", { errorCorrection: "L" });
  const penalties = MASKS.map((mask) => {
    const matrix = encodeQrCode("Hello, world!", { errorCorrection: "L", mask });
    return qrCodePenaltyScore(matrix.size, matrix.modules);
  });
  assert.equal(auto.mask, penalties.indexOf(Math.min(...penalties)));
  assert.ok(Object.isFrozen(auto));
  assert.ok(Object.isFrozen(auto.modules));
  assert.equal(isQrCodeModuleDark(auto, -1, 0), false);
  assert.equal(isQrCodeModuleDark(auto, 0, 0), true);
  assert.deepEqual(encodeQrCode("Hello, world!", { errorCorrection: "L" }), auto);
});

test("scores the four standard penalty rules", () => {
  // 5x5 all light: rows/columns N1 = 5 * (3 + 0) * 2, 2x2 blocks N2 = 16 * 3, balance N4 = 9 * 10.
  const light = Array.from<boolean>({ length: 25 }).fill(false);
  assert.equal(qrCodePenaltyScore(5, light), 30 + 48 + 90);
  const checker = Array.from(
    { length: 25 },
    (_, index) => (index % 5) % 2 === Math.floor(index / 5) % 2,
  );
  assert.ok(qrCodePenaltyScore(5, checker) < qrCodePenaltyScore(5, light));
});
