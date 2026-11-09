import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { buildEsbuild } from "../build/esbuild.ts";
import { buildWebpack } from "../build/webpack.ts";

type Builder = {
  name: string;
  build(outputRoot: string): Promise<string>;
};

const builders: Builder[] = [
  { name: "esbuild", build: buildEsbuild },
  { name: "webpack", build: buildWebpack },
];

function assertBuilderOutput(builder: string, outputDirectory: string): void {
  const html = fs.readFileSync(path.join(outputDirectory, "index.html"), "utf8");
  const javascript = fs.readFileSync(path.join(outputDirectory, "app.js"), "utf8");
  const cssFile = fs.readdirSync(outputDirectory).find((file) => file.endsWith(".css"));

  assert.match(html, new RegExp(`name="vize-builder" content="${builder}"`, "u"));
  assert.match(javascript, /Vize builder matrix/u);
  assert.ok(cssFile, `${builder} must emit a CSS asset`);
  const css = fs.readFileSync(path.join(outputDirectory, cssFile), "utf8");
  assert.match(css, /\.workspace\[data-v-[0-9a-f]{8}\]/u);
}

void test("the Node builder examples emit runnable JavaScript and scoped CSS", async () => {
  const outputRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-builder-matrix-"));
  try {
    const outputs = await Promise.all(builders.map((builder) => builder.build(outputRoot)));
    for (const [index, outputDirectory] of outputs.entries()) {
      assertBuilderOutput(builders[index].name, outputDirectory);
    }
  } finally {
    fs.rmSync(outputRoot, { recursive: true, force: true });
  }
});

const bunVersion = spawnSync("bun", ["--version"], { encoding: "utf8" });
const hasBun = bunVersion.status === 0;

void test(
  "the Bun example emits the same runnable JavaScript and scoped CSS contract",
  { skip: hasBun || process.env.CI ? false : "Bun is not installed in this host" },
  () => {
    const outputRoot = fs.mkdtempSync(path.join(os.tmpdir(), "vize-builder-bun-"));
    try {
      const script = fileURLToPath(new URL("../build/bun.ts", import.meta.url));
      const result = spawnSync("bun", ["run", script, outputRoot], {
        encoding: "utf8",
        maxBuffer: 16 * 1024 * 1024,
      });
      assert.equal(result.status, 0, result.stderr || result.stdout);
      assertBuilderOutput("bun", path.join(outputRoot, "bun"));
    } finally {
      fs.rmSync(outputRoot, { recursive: true, force: true });
    }
  },
);
