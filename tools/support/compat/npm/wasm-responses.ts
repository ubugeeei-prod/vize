// Exercise both installed browser packages with the same named inputs and options.
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { pathToFileURL } from "node:url";
import { serialize } from "node:v8";

type Browser = typeof import("../../../../npm/wasm/index.js");

async function load(directory: string): Promise<Browser> {
  const entry: Browser = await import(pathToFileURL(path.join(directory, "index.js")).href);
  await entry.init(fs.readFileSync(path.join(directory, "vize_vitrine_bg.wasm")));
  return entry;
}

/** Preserve undefined, typed arrays, map entries, property order and the public error value. */
export function capture(operation: () => unknown): Buffer {
  let response;
  try {
    response = { returned: operation() };
  } catch (error) {
    response =
      error instanceof Error
        ? { error: { name: error.name, message: error.message, cause: error.cause } }
        : { threw: error };
  }
  return serialize(response);
}

export async function compareWasmResponses(baseDir: string, headDir: string, corpus: string) {
  const base = await load(baseDir);
  const head = await load(headDir);
  const calls: Record<string, number> = {};
  const differences: { input: string; method: string }[] = [];
  let bytes = 0;
  function check(input: string, method: string, oldCall: () => unknown, newCall: () => unknown) {
    const before = capture(oldCall);
    const after = capture(newCall);
    calls[method] = (calls[method] ?? 0) + 1;
    bytes += before.byteLength;
    if (!before.equals(after)) differences.push({ input, method });
  }

  const template = "<section><i v-for='row in rows' :key='row.id'>雪{{row.label}}</i></section>";
  const source =
    "<script setup lang='ts'>const rows: Row[]=[]</script>\r\n<template>" +
    template +
    "</template>";
  const options = { filename: "src/Rows.vue", sourceMap: true };
  const css = ".label { color: red; }";
  for (const method of ["compile", "compileVapor", "parseTemplate"] as const) {
    for (const target of [
      options,
      { ...options, ssr: true },
      { ...options, customRenderer: true },
    ]) {
      check(
        "original template",
        method,
        () => base[method](template, target),
        () => head[method](template, target),
      );
    }
  }
  const jsx = "export const View = (p: { id: string }) => <b id={p.id}>{p.id}</b>";
  check(
    "TSX",
    "compileJsx",
    () => base.compileJsx(jsx, { filename: "View.tsx" }),
    () => head.compileJsx(jsx, { filename: "View.tsx" }),
  );
  check(
    "CSS",
    "compileCss",
    () => base.compileCss(css),
    () => head.compileCss(css),
  );
  check(
    "strict parser",
    "parseTemplate",
    () => base.parseTemplate("<i />", { templateSyntax: "strict" }),
    () => head.parseTemplate("<i />", { templateSyntax: "strict" }),
  );

  const previous = new base.Compiler();
  const candidate = new head.Compiler();
  try {
    for (const method of ["compile", "compileVapor", "parse", "parseSfc", "compileSfc"] as const) {
      const input = method.endsWith("Sfc") ? source : template;
      check(
        "resident compiler",
        "Compiler." + method,
        () => previous[method](input, options),
        () => candidate[method](input, options),
      );
    }
    check(
      "resident compiler",
      "Compiler.compileCss",
      () => previous.compileCss(css),
      () => candidate.compileCss(css),
    );
  } finally {
    previous.free();
    candidate.free();
  }

  const names = execFileSync(
    "git",
    [
      "ls-files",
      "-z",
      "--",
      "tests/_fixtures/differential/*.vue",
      "tests/_fixtures/differential/*.vue.txt",
    ],
    { cwd: corpus, encoding: "utf8" },
  )
    .split("\0")
    .filter(Boolean);
  if (!names.length) throw new Error("no original Vue fixtures in the selected revision");
  const decoder = new TextDecoder("utf-8", { fatal: true, ignoreBOM: true });
  let invalidUtf8 = 0;
  for (const name of names) {
    const buffer = fs.readFileSync(path.join(corpus, name));
    let doc;
    try {
      doc = decoder.decode(buffer);
    } catch (error) {
      if (!(error instanceof TypeError)) throw error;
      invalidUtf8++;
      continue;
    }
    const filename = name.replace(/\.vue\.txt$/u, ".vue");
    check(
      name,
      "parseSfc",
      () => base.parseSfc(doc, { filename }),
      () => head.parseSfc(doc, { filename }),
    );
    const compiler = { filename, sourceMap: true };
    check(
      name,
      "compileSfc",
      () => base.compileSfc(doc, compiler),
      () => head.compileSfc(doc, compiler),
    );
    check(
      name,
      "lintSfc",
      () => base.lintSfc(doc, { filename }),
      () => head.lintSfc(doc, { filename }),
    );
    check(
      name,
      "formatSfc",
      () => base.formatSfc(doc),
      () => head.formatSfc(doc),
    );
  }
  return { fixtures: names.length, invalidUtf8, calls, bytes, differences };
}
