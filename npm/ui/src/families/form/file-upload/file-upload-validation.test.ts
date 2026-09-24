import assert from "node:assert/strict";

import { test } from "vite-plus/test";

import {
  effectiveMaxFiles,
  formatFileSize,
  matchAcceptType,
  matchesAccept,
  parseAccept,
  validateFiles,
} from "./file-upload-validation.ts";

function file(name: string, type = "", size = 1): File {
  return new File([new Uint8Array(size)], name, { type });
}

test("parses native accept lists into normalized tokens and drops invalid tokens", () => {
  assert.deepEqual(parseAccept(undefined), []);
  assert.deepEqual(parseAccept(null), []);
  assert.deepEqual(parseAccept(" , "), []);
  assert.deepEqual(parseAccept("IMAGE/*, .PNG ,application/pdf;charset=x, */*, *"), [
    { kind: "mime", type: "image", subtype: "*" },
    { kind: "extension", extension: ".png" },
    { kind: "mime", type: "application", subtype: "pdf" },
    { kind: "any" },
    { kind: "any" },
  ]);
  assert.deepEqual(parseAccept(".,image,a/b/c,*/png,te xt/plain"), []);
});

test("matches files by mime wildcard, exact mime, and case-insensitive extension", () => {
  assert.equal(matchesAccept(file("a.bin"), undefined), true);
  assert.equal(matchesAccept(file("a.bin"), ""), true);
  assert.equal(matchesAccept(file("photo.JPG", "image/jpeg"), "image/*"), true);
  assert.equal(matchesAccept(file("photo.jpg", "IMAGE/JPEG"), "image/jpeg"), true);
  assert.equal(matchesAccept(file("doc.txt", "text/plain; charset=utf-8"), "text/plain"), true);
  assert.equal(matchesAccept(file("REPORT.PDF"), ".pdf"), true);
  assert.equal(matchesAccept(file("archive.tar.gz"), ".tar.gz"), true);
  assert.equal(matchesAccept(file("notes.md", "text/markdown"), "image/*,.pdf"), false);
  assert.equal(matchesAccept(file("image.png", "image/png"), "video/*"), false);
  assert.equal(matchesAccept(file("any"), "*/*"), true);
  assert.equal(matchesAccept(file("x.png", "image/png"), parseAccept("image/png")), true);
});

test("decides dragged item types as accept, reject, or unknown", () => {
  assert.equal(matchAcceptType("image/png", undefined), "accept");
  assert.equal(matchAcceptType("image/png", "image/*"), "accept");
  assert.equal(matchAcceptType("text/plain", "image/*"), "reject");
  assert.equal(matchAcceptType("", "image/*"), "unknown");
  assert.equal(matchAcceptType("text/plain", "image/*,.txt"), "unknown");
  assert.equal(matchAcceptType("text/plain", "*"), "accept");
});

test("formats sizes with SI locale units and IEC suffixes", () => {
  assert.equal(formatFileSize(0), "0 bytes");
  assert.equal(formatFileSize(1), "1 byte");
  assert.equal(formatFileSize(999), "999 bytes");
  assert.equal(formatFileSize(1500), "1.5 kB");
  assert.equal(formatFileSize(1_250_000), "1.3 MB");
  assert.equal(formatFileSize(3e15 * 1000), "3,000 PB");
  assert.equal(formatFileSize(-5), "0 bytes");
  assert.equal(formatFileSize(Number.NaN), "0 bytes");
  assert.equal(formatFileSize(1536, { standard: "iec" }), "1.5\u00a0KiB");
  assert.equal(formatFileSize(1024 * 1024 * 3, { standard: "iec" }), "3\u00a0MiB");
  assert.equal(formatFileSize(512, { standard: "iec" }), "512\u00a0B");
  assert.equal(formatFileSize(1_234_567, { maximumFractionDigits: 2 }), "1.23 MB");
  assert.equal(formatFileSize(1500, { locale: "de-DE" }), "1,5 kB");
});

test("resolves the effective file cap", () => {
  assert.equal(effectiveMaxFiles(false, 10), 1);
  assert.equal(effectiveMaxFiles(true, undefined), Number.POSITIVE_INFINITY);
  assert.equal(effectiveMaxFiles(true, Number.NaN), Number.POSITIVE_INFINITY);
  assert.equal(effectiveMaxFiles(true, 2.9), 2);
  assert.equal(effectiveMaxFiles(true, -1), 0);
});

test("reports every per-file failure in check order with default messages", () => {
  const big = file("big.txt", "text/plain", 20);
  const result = validateFiles({
    accept: "image/*",
    candidates: [big],
    current: [],
    maxSize: 10,
  });
  assert.deepEqual(result.accepted, []);
  assert.equal(result.files.length, 0);
  assert.deepEqual(result.rejected, [
    {
      file: big,
      errors: [
        { code: "file-invalid-type", message: "File type must be one of: image/*" },
        { code: "file-too-large", message: "File is larger than 10 bytes" },
      ],
    },
  ]);
  const small = file("small.png", "image/png", 1);
  assert.deepEqual(validateFiles({ candidates: [small], current: [], minSize: 5 }).rejected, [
    { file: small, errors: [{ code: "file-too-small", message: "File is smaller than 5 bytes" }] },
  ]);
});

test("single uploads replace the current file and reject simultaneous candidates", () => {
  const current = file("old.png");
  const next = file("new.png");
  assert.deepEqual(validateFiles({ candidates: [next], current: [current] }).files, [next]);
  const other = file("other.png");
  const result = validateFiles({ candidates: [next, other], current: [current] });
  assert.deepEqual(result.files, [current]);
  assert.deepEqual(
    result.rejected.map((rejection) => rejection.errors.map((error) => error.code)),
    [["too-many-files"], ["too-many-files"]],
  );
  assert.equal(result.rejected[0]?.errors[0]?.message, "Only one file can be added");
});

test("multiple uploads append up to maxFiles and reject the overflow", () => {
  const current = file("a.png");
  const b = file("b.png");
  const c = file("c.png");
  const invalid = file("d.exe");
  const result = validateFiles({
    accept: ".png",
    candidates: [invalid, b, c],
    current: [current],
    maxFiles: 2,
    multiple: true,
  });
  assert.deepEqual(result.files, [current, b]);
  assert.deepEqual(result.accepted, [b]);
  assert.deepEqual(
    result.rejected.map(({ file: rejected, errors }) => [
      rejected.name,
      errors.map((error) => error.code),
    ]),
    [
      ["d.exe", ["file-invalid-type"]],
      ["c.png", ["too-many-files"]],
    ],
  );
  assert.equal(result.rejected[1]?.errors[0]?.message, "No more than 2 files can be added");
});

test("custom validators map codes, strings, and error objects", () => {
  const seen: number[] = [];
  const files = [file("a"), file("b"), file("c"), file("d"), file("e")];
  const result = validateFiles({
    candidates: files,
    current: [],
    multiple: true,
    validate(candidate, accepted) {
      seen.push(accepted.length);
      if (candidate.name === "a") return null;
      if (candidate.name === "b") return "file-too-large";
      if (candidate.name === "c") return "Name is reserved";
      if (candidate.name === "d") return { code: "custom", message: "Object error" };
      return "custom";
    },
  });
  assert.deepEqual(seen, [0, 1, 1, 1, 1]);
  assert.deepEqual(
    result.rejected.map(({ errors }) => errors[0]),
    [
      { code: "file-too-large", message: "File is larger than 0 bytes" },
      { code: "custom", message: "Name is reserved" },
      { code: "custom", message: "Object error" },
      { code: "custom", message: "File is invalid" },
    ],
  );
});

test("message factories and size formatters replace default copy", () => {
  const huge = file("huge.bin", "", 50);
  const result = validateFiles({
    candidates: [huge],
    current: [],
    formatSize: (bytes) => `${bytes}B`,
    maxSize: 10,
    messages: {
      "file-too-large": ({ file: rejected, formatSize, maxSize }) =>
        `${rejected.name} > ${formatSize(maxSize ?? 0)}`,
    },
  });
  assert.equal(result.rejected[0]?.errors[0]?.message, "huge.bin > 10B");
});
