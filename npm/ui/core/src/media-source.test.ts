import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";
import { pathToFileURL } from "node:url";
import { test } from "vite-plus/test";
import { createPDFSource } from "./pdf-source.ts";
import { normalizeMediaSource } from "./media-source.ts";
import type { MediaSourceKind } from "./media-source.ts";

test("accepts relative, encrypted, and object media sources", () => {
  for (const [source, kind] of [
    ["/audio/intro.mp3", "audio"],
    ["../video/intro.mp4", "video"],
    ["//media.example.test/poster.webp", "image"],
    ["https://media.example.test/subtitles.vtt", "track"],
    ["blob:session-id", "stream"],
  ] as const) {
    assert.equal(normalizeMediaSource(` ${source} `, { kind }), source);
  }
});

test("requires an explicit opt-in for unencrypted remote sources", () => {
  const source = "http://localhost:4173/video.mp4";
  assert.throws(
    () => normalizeMediaSource(source, { kind: "video" }),
    /\[VIZE_UI_MEDIA_DISALLOWED_SOURCE\]/,
  );
  assert.equal(normalizeMediaSource(source, { kind: "video", allowInsecure: true }), source);
});

test("accepts only category-matched inline data", () => {
  const sources = [
    ["data:audio/ogg;base64,T2dnUw==", "audio"],
    ["data:image/png;base64,iVBORw0KGgo=", "image"],
    ["data:video/mp4;base64,AAAA", "video"],
    ["data:application/pdf;base64,JVBERi0xLjQ=", "pdf"],
    ["data:text/vtt;charset=utf-8,WEBVTT%0A%0A", "track"],
    ["data:text/vtt;base64,V0VCVlRU", "track"],
  ] as const;

  for (const [source, kind] of sources) {
    assert.equal(normalizeMediaSource(source, { kind }), source);
  }

  assert.throws(
    () => normalizeMediaSource(sources[1][0], { kind: "video" }),
    /\[VIZE_UI_MEDIA_DISALLOWED_SOURCE\]/,
  );
  assert.throws(
    () => normalizeMediaSource("data:video/mp4;base64,AAAA", { kind: "stream" }),
    /\[VIZE_UI_MEDIA_DISALLOWED_SOURCE\]/,
  );
});

test("rejects malformed and script-capable sources", () => {
  for (const source of [
    "",
    "javascript:alert(1)",
    "file:///private/report.pdf",
    "https://media.example.test/video.mp4\nscript",
    "data:video/mp4;base64,not-base64",
  ]) {
    assert.throws(
      () => normalizeMediaSource(source, { kind: "video" }),
      /\[VIZE_UI_MEDIA_(?:INVALID|DISALLOWED)_SOURCE\]/,
    );
  }

  assert.throws(
    () => normalizeMediaSource("data:text/vtt,WEBVTT%QQ", { kind: "track" }),
    /\[VIZE_UI_MEDIA_DISALLOWED_SOURCE\]/,
  );
  assert.throws(
    () => normalizeMediaSource(42 as unknown as string, { kind: "video" }),
    /\[VIZE_UI_MEDIA_INVALID_SOURCE\]/,
  );
  assert.throws(
    () => normalizeMediaSource("/media", { kind: "unknown" as MediaSourceKind }),
    /\[VIZE_UI_MEDIA_INVALID_KIND\]/,
  );
});

test("sets and replaces PDF page fragments", () => {
  assert.equal(createPDFSource("/report.pdf", { page: 3 }), "/report.pdf#page=3");
  assert.equal(
    createPDFSource("/report.pdf#zoom=page-width&page=2&view=Fit", { page: 7 }),
    "/report.pdf#zoom=page-width&view=Fit&page=7",
  );
  assert.equal(createPDFSource("/report.pdf#page=2"), "/report.pdf#page=2");

  for (const page of [0, -1, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
    assert.throws(
      () => createPDFSource("/report.pdf", { page }),
      /\[VIZE_UI_MEDIA_INVALID_PDF_PAGE\]/,
    );
  }
});

test("publishes independent ESM entries and declarations", async () => {
  const packageJson = JSON.parse(await readFile(path.resolve("package.json"), "utf8")) as {
    readonly exports: Readonly<Record<string, { readonly import: string; readonly types: string }>>;
  };

  for (const exportName of ["./media", "./media/pdf", "./media/source"]) {
    const entry = packageJson.exports[exportName];
    if (entry === undefined) assert.fail(`Missing package export: ${exportName}`);
    await stat(path.resolve(entry.import));
    await stat(path.resolve(entry.types));
  }

  const distributionUrl = pathToFileURL(path.resolve("dist/media-source.mjs"));
  const sourceEntry = (await import(distributionUrl.href)) as {
    readonly normalizeMediaSource: unknown;
  };
  assert.equal(typeof sourceEntry.normalizeMediaSource, "function");
});
