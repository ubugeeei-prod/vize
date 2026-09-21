// Davinci P4-12c — the pinned `pug` package as the independent oracle for
// the pug S1 dialect. The committed `.html` of every matrix fixture must be
// byte-identical to what `pug@3.0.4` renders for the fixture's
// `<template lang="pug">` content exactly as `@vue/compiler-sfc` hands it
// over (dedented, `doctype: "html"`, `pretty: false`); the Rust suites then
// hold vize's derived template and every compile lane to those bytes. Every
// refused fixture must be refused for its declared reason: pug itself
// fails, or the template needs a JavaScript engine (the static-pug guard
// throws), or it uses a structural construct a static Vue template cannot
// express. `VIZE_PUG_ORACLE_WRITE=1` regenerates the `.html` files.
import assert from "node:assert/strict";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";

import { assertStaticPug, parseSfc, pug, pugOptions } from "./support/pug/oracle-runtime.ts";

const fixtures = new URL("../_fixtures/davinci-pug/", import.meta.url);
const write = process.env.VIZE_PUG_ORACLE_WRITE === "1";
const lexPug = createRequire(createRequire(import.meta.url).resolve("pug"))("pug-lexer") as (
  source: string,
  options: { filename: string },
) => Array<{ type: string }>;

function names(dir: string): string[] {
  return fs
    .readdirSync(new URL(`${dir}/`, fixtures))
    .filter((file) => file.endsWith(".pug"))
    .map((file) => file.slice(0, -".pug".length))
    .sort();
}

function source(dir: string, name: string): string {
  return fs.readFileSync(new URL(`${dir}/${name}.pug`, fixtures), "utf8");
}

/** The content Vue gives pug: the SFC parser's dedented template body. */
function vueContent(pugSource: string, filename: string): string {
  const sfc = `<template lang="pug">${pugSource}</template>\n`;
  const template = parseSfc(sfc, { filename, sourceMap: false }).descriptor.template;
  assert.ok(template, `${filename} has a template`);
  return template.content;
}

function render(content: string, filename: string): string {
  return pug.render(content, { ...pugOptions, filename });
}

test("pug oracle: every matrix fixture's HTML is the pinned pug rendering", () => {
  const matrix = names("matrix");
  assert.equal(matrix.length, 26, "the matrix is pinned; grow it deliberately");
  for (const name of matrix) {
    const filename = `${name}.pug`;
    const content = vueContent(source("matrix", name), filename);
    const html = render(content, filename);
    const target = new URL(`matrix/${name}.html`, fixtures);
    if (write) fs.writeFileSync(target, html);
    assert.equal(fs.readFileSync(target, "utf8"), html, `${name}.html drifted from pug`);
  }
});

type Reason = "pug-error" | "executable" | "structural";

const refusals: Record<string, Reason> = {
  "and-attributes": "executable",
  "case-when": "executable",
  "code-interpolation": "executable",
  conditional: "executable",
  doctype: "structural",
  each: "executable",
  "executable-attribute": "executable",
  "executable-buffered-code": "executable",
  "extends-and-block": "pug-error",
  filter: "executable",
  include: "pug-error",
  "mixin-and-call": "executable",
  "unbuffered-code": "executable",
  while: "executable",
};

function observedReason(content: string, filename: string): Reason | "static" {
  try {
    assertStaticPug(content, filename, {});
  } catch {
    return "executable";
  }
  try {
    render(content, filename);
  } catch {
    return "pug-error";
  }
  const structural = new Set(["doctype", "block", "extends", "include", "yield"]);
  return lexPug(content, { filename }).some((token) => structural.has(token.type))
    ? "structural"
    : "static";
}

test("pug oracle: every refused fixture is refused for its declared reason", () => {
  const refused = names("refused");
  assert.deepEqual(refused, Object.keys(refusals).sort(), "refusal reasons are a bijection");
  for (const name of refused) {
    const filename = `${name}.pug`;
    const content = vueContent(source("refused", name), filename);
    assert.equal(observedReason(content, filename), refusals[name], name);
  }
});
