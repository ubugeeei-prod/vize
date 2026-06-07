import assert from "node:assert/strict";
import test from "node:test";

import { appendOriginalVueSourceForUnoCss, MAX_UNOCSS_ORIGINAL_SOURCE_BYTES } from "./unocss.ts";

void test("UnoCSS bridge strips multi-param query and reads the bare path", () => {
  let seenPath: string | undefined;
  const result = appendOriginalVueSourceForUnoCss("compiled", "/path/App.vue?foo=1&bar=2", {
    maxBytes: 1024,
    readSize: (filePath) => {
      seenPath = filePath;
      return 8;
    },
    readFile: (filePath) => {
      // readFile must also receive the path stripped of the query.
      assert.equal(filePath, "/path/App.vue");
      return "<template/>";
    },
  });

  assert.equal(seenPath, "/path/App.vue", "path is taken from before the '?'");
  assert.equal(result, "compiled\n<template/>");
});

void test("UnoCSS bridge returns code unchanged for an empty id without reading", () => {
  let touched = false;
  const result = appendOriginalVueSourceForUnoCss("compiled", "", {
    readSize: () => {
      touched = true;
      return 1;
    },
    readFile: () => {
      touched = true;
      return "x";
    },
  });

  assert.equal(result, "compiled", "empty id yields a falsy filePath and short-circuits");
  assert.equal(touched, false, "neither readSize nor readFile is invoked");
});

void test("UnoCSS bridge returns code unchanged for a pure '?query' id", () => {
  // "?vue".split("?")[0] === "" -> falsy filePath -> short circuit.
  let touched = false;
  const result = appendOriginalVueSourceForUnoCss("compiled", "?vue", {
    readSize: () => {
      touched = true;
      return 1;
    },
  });

  assert.equal(result, "compiled");
  assert.equal(touched, false);
});

void test("UnoCSS bridge appends when size is exactly at the maxBytes boundary", () => {
  const result = appendOriginalVueSourceForUnoCss("compiled", "/src/Edge.vue", {
    maxBytes: 10,
    readSize: () => 10,
    readFile: () => "SRC",
  });

  assert.equal(result, "compiled\nSRC", "size == maxBytes is appended (strict '>' comparison)");
});

void test("UnoCSS bridge skips when size is one byte over maxBytes", () => {
  let didRead = false;
  const result = appendOriginalVueSourceForUnoCss("compiled", "/src/Edge.vue", {
    maxBytes: 10,
    readSize: () => 11,
    readFile: () => {
      didRead = true;
      return "SRC";
    },
  });

  assert.equal(result, "compiled", "size == maxBytes + 1 is skipped");
  assert.equal(didRead, false, "oversized source is never read");
});

void test("UnoCSS bridge default maxBytes equals MAX_UNOCSS_ORIGINAL_SOURCE_BYTES", () => {
  assert.equal(MAX_UNOCSS_ORIGINAL_SOURCE_BYTES, 2 * 1024 * 1024);

  // Exactly at the default cap -> appended.
  const atCap = appendOriginalVueSourceForUnoCss("compiled", "/src/Big.vue", {
    readSize: () => MAX_UNOCSS_ORIGINAL_SOURCE_BYTES,
    readFile: () => "SRC",
  });
  assert.equal(atCap, "compiled\nSRC", "default cap is the exported constant and is inclusive");

  // One over the default cap -> skipped.
  const overCap = appendOriginalVueSourceForUnoCss("compiled", "/src/Big.vue", {
    readSize: () => MAX_UNOCSS_ORIGINAL_SOURCE_BYTES + 1,
    readFile: () => "SRC",
  });
  assert.equal(overCap, "compiled", "default cap rejects sources one byte larger");
});

void test("UnoCSS bridge with Infinity maxBytes never skips on size", () => {
  const result = appendOriginalVueSourceForUnoCss("compiled", "/src/Huge.vue", {
    maxBytes: Infinity,
    readSize: () => Number.MAX_SAFE_INTEGER,
    readFile: () => "SRC",
  });

  assert.equal(result, "compiled\nSRC", "Infinity cap means no size value can exceed it");
});

void test("UnoCSS bridge swallows a throwing readSize and leaves code intact", () => {
  let didRead = false;
  const result = appendOriginalVueSourceForUnoCss("compiled", "/src/Fail.vue", {
    maxBytes: 1024,
    readSize: () => {
      throw new Error("stat failed");
    },
    readFile: () => {
      didRead = true;
      return "SRC";
    },
  });

  assert.equal(result, "compiled", "a thrown size error is caught and code is returned");
  assert.equal(didRead, false, "readFile is not reached when readSize throws");
});

void test("UnoCSS bridge swallows a throwing readFile and leaves code intact", () => {
  const result = appendOriginalVueSourceForUnoCss("compiled", "/src/Fail.vue", {
    maxBytes: 1024,
    readSize: () => 5,
    readFile: () => {
      throw new Error("read failed");
    },
  });

  assert.equal(result, "compiled", "a thrown read error is caught and code is returned");
});

void test("UnoCSS bridge appends the exact stub source after a newline", () => {
  const original = '<template><div text-red @click="x" /></template>';
  const result = appendOriginalVueSourceForUnoCss(
    "compiled-output",
    "/src/App.vue?vue&type=style",
    {
      maxBytes: 4096,
      readSize: () => original.length,
      readFile: () => original,
    },
  );

  assert.equal(result, `compiled-output\n${original}`);
  assert.ok(result.includes(original), "the original source is present in the output");
  assert.ok(result.startsWith("compiled-output\n"), "code precedes the appended source");
});

void test("UnoCSS bridge with empty code yields just the newline-prefixed source", () => {
  const result = appendOriginalVueSourceForUnoCss("", "/src/Empty.vue", {
    maxBytes: 1024,
    readSize: () => 3,
    readFile: () => "SRC",
  });

  // Empty code still gets the leading "\n" separator from the template literal.
  assert.equal(result, "\nSRC", "empty code produces a leading newline before the source");
});
