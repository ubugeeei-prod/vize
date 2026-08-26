import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
// Paths are resolved from the package cwd: the runner virtualizes import.meta.url.
import path from "node:path";

import { test } from "vite-plus/test";

import config, { cssBrowserFloor } from "../vite.config.ts";

test("the package build declares an explicit browser floor for stylesheet lowering", () => {
  assert.ok(cssBrowserFloor.length >= 4, "the floor must name every evergreen engine");
  for (const target of cssBrowserFloor) {
    assert.match(target, /^[a-z]+\d+(?:\.\d+)?$/, `${target} must pin an explicit version`);
  }

  const css = (
    config as {
      readonly pack?: {
        readonly css?: {
          readonly inject?: boolean;
          readonly minify?: boolean;
          readonly target?: readonly string[];
        };
      };
    }
  ).pack?.css;
  assert.deepEqual(css?.target, cssBrowserFloor, "pack css lowering must use the declared floor");
  assert.equal(css?.minify, true, "the packaged stylesheet ships minified");
  assert.equal(css?.inject, true, "component entries keep their static stylesheet imports");
});

test("authored nesting, layers, logical properties, and color functions compile to the floor", async () => {
  // source-contract: the authored style language is a source fact; its lowered
  // form is proven on the packaged stylesheet below.
  const source = await readFile(path.resolve("src/visually-hidden.vue"), "utf8");
  // source-contract: nesting, layers, logical properties, and native color.
  assert.match(source, /@layer vize\.ui/);
  // source-contract: the focus guard is authored as a nested :where() rule.
  assert.match(source, /&:where\(:focus-within\)/);
  // source-contract: box dimensions are authored as logical properties.
  assert.match(source, /inline-size|block-size/);
  // source-contract: the inert paint color is authored as a native color function.
  assert.match(source, /oklch\(/);

  const stylesheet = await readFile(path.resolve("dist/style.css"), "utf8");

  // Cascade layers, logical properties, and oklch() are native at the floor
  // and ship verbatim.
  assert.match(stylesheet, /@layer vize\.ui\{/);
  assert.match(stylesheet, /inline-size:1px/);
  assert.match(stylesheet, /block-size:1px/);
  assert.match(stylesheet, /oklch\(/);
  assert.match(stylesheet, /--vize-ui-visually-hidden-background/);

  // CSS Nesting is newer than the floor, so authored nesting always flattens.
  assert.doesNotMatch(stylesheet, /&/, "nested selectors must be lowered for the floor");
  assert.match(
    stylesheet,
    /\[data-vize-ui=visually-hidden\]\[data-v-[0-9a-f]{8}\]:where\(:focus-within\)\{clip-path:inset\(50%\)\}/,
    "the nested focus guard must flatten into a scoped standalone rule",
  );
});

test("scoped style semantics survive the down-compile", async () => {
  const stylesheet = await readFile(path.resolve("dist/style.css"), "utf8");

  assert.match(stylesheet, /\[data-v-[0-9a-f]{8}\]/, "shipped rules keep their scope attribute");
  assert.doesNotMatch(
    stylesheet,
    /\[data-vize-ui=visually-hidden\](?!\[data-v-[0-9a-f]{8}\])/,
    "every component selector must keep its scope attribute after lowering",
  );
});

test("styles never arrive through runtime CSS-in-JS", async () => {
  const distFiles = await readdir(path.resolve("dist"));
  const stylesheets = distFiles.filter((file) => file.endsWith(".css"));
  assert.deepEqual(stylesheets, ["style.css"], "styles ship only as the opt-in stylesheet");

  for (const file of distFiles.filter((name) => name.endsWith(".mjs"))) {
    const output = await readFile(path.resolve("dist", file), "utf8");
    assert.doesNotMatch(
      output,
      /createElement\(\s*["']style["']\s*\)/,
      `${file} must not create style elements at runtime`,
    );
    assert.doesNotMatch(output, /\.insertRule\(/, `${file} must not insert rules at runtime`);
    assert.doesNotMatch(
      output,
      /new\s+CSSStyleSheet|adoptedStyleSheets\s*=/,
      `${file} must not construct or replace stylesheets at runtime`,
    );
  }
});
