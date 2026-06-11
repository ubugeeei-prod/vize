import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { generateCorpus, SFC_TEMPLATES } from "../../bench/generate.mjs";

function withTempDir(fn) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "vize-bench-generator-"));
  try {
    return fn(dir);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

function readVueCorpus(dir) {
  const files = fs
    .readdirSync(dir)
    .filter((file) => file.endsWith(".vue"))
    .sort();
  return {
    files,
    bodies: files.map((file) => fs.readFileSync(path.join(dir, file), "utf8")),
  };
}

test("benchmark generator writes a diversified unique SFC corpus", () => {
  withTempDir((dir) => {
    const fileCount = SFC_TEMPLATES.length * 2;
    fs.writeFileSync(path.join(dir, "Component9999.vue"), "<template><p>stale</p></template>\n");

    const result = generateCorpus({ fileCount, benchDir: dir, log: () => {} });
    const { files, bodies } = readVueCorpus(dir);

    assert.equal(result.fileCount, fileCount);
    assert.equal(files.length, fileCount);
    assert.equal(files[0], "Component0000.vue");
    assert.equal(files.at(-1), `Component${String(fileCount - 1).padStart(4, "0")}.vue`);
    assert.ok(!files.includes("Component9999.vue"), "stale generated components must be removed");
    assert.equal(new Set(bodies).size, fileCount, "every generated SFC body must be unique");
    assert.ok(bodies.every((body) => !body.includes("__BENCH_ID__")));

    assert.ok(bodies.some((body) => body.includes("<script setup>")));
    assert.ok(
      bodies.some(
        (body) => body.includes("export default {") && body.includes("Options API board"),
      ),
      "corpus must include an Options API SFC",
    );
    assert.ok(
      bodies.some(
        (body) => body.includes('<script setup lang="ts">') && body.includes("type ResourceState"),
      ),
      "corpus must include a TS-heavy SFC",
    );
    assert.ok(
      bodies.some(
        (body) =>
          body.includes("large-template-grid") &&
          (body.match(/class="template-row/g) ?? []).length >= 18,
      ),
      "corpus must include a large-template SFC",
    );
    assert.ok(
      bodies.some(
        (body) =>
          body.includes('<script lang="ts">') &&
          body.includes("export default class ClassBenchComponent"),
      ),
      "corpus must include a class-style SFC",
    );

    for (const requiredFile of [
      "tsconfig.json",
      "eslint.config.mjs",
      "vize.config.json",
      "main.ts",
      "index.html",
    ]) {
      assert.ok(fs.existsSync(path.join(dir, requiredFile)), `${requiredFile} must be generated`);
    }

    const tsconfig = JSON.parse(fs.readFileSync(path.join(dir, "tsconfig.json"), "utf8"));
    assert.deepEqual(tsconfig.include, ["./*.vue"]);
    assert.deepEqual(tsconfig.compilerOptions.paths.vue, ["../node_modules/vue"]);

    const eslintConfig = fs.readFileSync(path.join(dir, "eslint.config.mjs"), "utf8");
    assert.match(eslintConfig, /eslint-plugin-vue/);
    assert.match(eslintConfig, /vue\/multi-word-component-names/);

    const vizeConfig = JSON.parse(fs.readFileSync(path.join(dir, "vize.config.json"), "utf8"));
    assert.equal(vizeConfig.typeChecker.checkTemplateBindings, true);

    const viteEntry = fs.readFileSync(path.join(dir, "main.ts"), "utf8");
    assert.match(viteEntry, /import Component0000 from '\.\/Component0000\.vue'/);
    const lastComponent = `Component${String(fileCount - 1).padStart(4, "0")}`;
    assert.ok(viteEntry.includes(`import ${lastComponent} from './${lastComponent}.vue'`));
    assert.match(viteEntry, /h\(Component0000\)/);

    const indexHtml = fs.readFileSync(path.join(dir, "index.html"), "utf8");
    assert.match(indexHtml, /src="\.\/main\.ts"/);
  });
});

test("benchmark generator output is deterministic for the same count", () => {
  withTempDir((firstDir) => {
    withTempDir((secondDir) => {
      const fileCount = SFC_TEMPLATES.length + 3;
      generateCorpus({ fileCount, benchDir: firstDir, log: () => {} });
      generateCorpus({ fileCount, benchDir: secondDir, log: () => {} });

      const firstEntries = fs.readdirSync(firstDir).sort();
      const secondEntries = fs.readdirSync(secondDir).sort();
      assert.deepEqual(firstEntries, secondEntries);

      for (const entry of firstEntries) {
        assert.equal(
          fs.readFileSync(path.join(firstDir, entry), "utf8"),
          fs.readFileSync(path.join(secondDir, entry), "utf8"),
          `${entry} should be deterministic`,
        );
      }
    });
  });
});
