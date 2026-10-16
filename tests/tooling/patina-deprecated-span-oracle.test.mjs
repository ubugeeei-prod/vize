import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";

const root = path.resolve(import.meta.dirname, "../..");
const directory = path.join(root, "crates/vize_patina/tests/fixtures/deprecated-diagnostic-spans");
const manifest = JSON.parse(fs.readFileSync(path.join(directory, "source.json"), "utf8"));
const fixturePath = path.join(directory, "oracle.json");
const requireProvider = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
const hash = (value, algorithm = "sha256") => createHash(algorithm).update(value).digest("hex");
const minimal = [
  ["interpolation", "<template><div>{{ message | capitalize }}</div></template>"],
  [
    "multiline-unicode",
    "<!-- 雪😀 -->\r\n<template>\r\n  <div>{{\r\n    message | capitalize\r\n  }}</div>\r\n</template>",
  ],
  ["chained", "<template>{{ a | b | c }}</template>"],
  ["entity-grouping-0", "<template>{{ &#40;message&#41; | capitalize }}</template>"],
  ["entity-grouping-1", "<template>{{ &lpar;message&rpar; | capitalize }}</template>"],
  ["entity-grouping-2", '<template><div :id="&#40;message&#41; | capitalize" /></template>'],
  ["entity-grouping-3", '<template><div :id="&lpar;message&rpar; | capitalize" /></template>'],
  ["entity-grouping-4", '<template><div :id="&#32;&#40;message&#41; | capitalize" /></template>'],
  ["parenthesized-0", "<template>{{ (a) | f }}</template>"],
  ["parenthesized-1", "<template>{{ ((a)) | f }}</template>"],
  ["parenthesized-2", "<template>{{ ((a)+b) | f }}</template>"],
  ["parenthesized-3", "<template>{{ (a) ? b : c | f }}</template>"],
  ["parenthesized-4", "<template>{{ (a)() | f }}</template>"],
  ["parenthesized-5", "<template>{{ (a).b | f }}</template>"],
  ["bound-filter", '<template><div :id="rawId | toId" /></template>'],
  ["static-tag", '<template><router-link tag="button" /></template>'],
  ["bound-tag", '<template><RouterLink :tag="element" /></template>'],
  ["long-bound-tag", '<template><router-link v-bind:tag.prop="element" /></template>'],
  [
    "multiline-tag",
    '<!-- 雪😀 -->\r\n<template>\r\n<RouterLink\r\n  tag="button"\r\n/>\r\n</template>',
  ],
  ["clean", '<template><div>{{ a || b }}</div><router-link to="/" /></template>'],
  [
    "typescript-assertion",
    "<template><CodeEditor :type=\"activeScript.script_type as 'JavaScript' | 'CSS'\" /></template>",
  ],
];

async function capture() {
  const vue = requireProvider("eslint-plugin-vue");
  const providers = Object.fromEntries(
    ["eslint", "eslint-plugin-vue", "vue-eslint-parser", "@typescript-eslint/parser"].map(
      (name) => [name, requireProvider(`${name}/package.json`).version],
    ),
  );
  assert.deepEqual(providers, {
    eslint: "10.4.1",
    "eslint-plugin-vue": "10.9.2",
    "vue-eslint-parser": "10.4.1",
    "@typescript-eslint/parser": "8.65.0",
  });
  const configuration = {
    files: ["**/*.vue"],
    languageOptions: {
      parser: requireProvider("vue-eslint-parser"),
      parserOptions: {
        parser: requireProvider("@typescript-eslint/parser"),
        ecmaVersion: "latest",
        sourceType: "module",
      },
    },
    plugins: { vue },
    rules: Object.fromEntries(manifest.rules.map((name) => [name, "error"])),
  };
  const eslint = new (requireProvider("eslint").ESLint)({
    cwd: root,
    overrideConfigFile: true,
    overrideConfig: [configuration],
  });
  const cases = minimal.map(([id, source]) => ({ id, filename: `minimal/${id}.vue`, source }));
  for (const project of [{ ...manifest, id: "" }, ...manifest.additionalProjects]) {
    for (const original of project.originals) {
      const bytes = fs.readFileSync(path.join(directory, "originals", project.id, original.path));
      assert.equal(bytes.length, original.bytes);
      assert.equal(hash(bytes), original.sha256);
      assert.equal(
        hash(Buffer.concat([Buffer.from(`blob ${bytes.length}\0`), bytes]), "sha1"),
        original.gitBlob,
      );
      cases.push({
        id: project.id ? `${project.id}/${original.path}` : original.path,
        filename: original.path,
        source: bytes.toString(),
      });
    }
  }
  const captures = [];
  for (const example of cases) {
    const filePath = path.join(root, "owned", example.filename);
    const packets = await eslint.lintText(example.source, { filePath });
    // Only the ephemeral checkout prefix is recorded as a stable path. All
    // messages, source, suggestions, counts and other provider fields survive.
    captures.push({
      ...example,
      packets: packets.map((packet) => ({ ...packet, filePath: `owned/${example.filename}` })),
    });
  }
  // The Vue filter splitter precedes the typed expression parser. Preserve a
  // full no-TS-provider witness of the same assertion, rather than treating
  // its type-union pipe as a native runtime filter.
  const untyped = new (requireProvider("eslint").ESLint)({
    cwd: root,
    overrideConfigFile: true,
    overrideConfig: [
      {
        ...configuration,
        languageOptions: {
          parser: requireProvider("vue-eslint-parser"),
          parserOptions: { ecmaVersion: "latest", sourceType: "module" },
        },
      },
    ],
  });
  const assertion = cases.find((entry) => entry.id === "typescript-assertion");
  const untypedPackets = await untyped.lintText(assertion.source, {
    filePath: path.join(root, "owned", assertion.filename),
  });
  return {
    providers,
    rules: configuration.rules,
    captures,
    assertionWithoutTypedParser: untypedPackets.map((packet) => ({
      ...packet,
      filePath: `owned/${assertion.filename}`,
    })),
  };
}

if (process.argv.includes("--record")) {
  fs.writeFileSync(fixturePath, JSON.stringify(await capture(), null, 2) + "\n");
} else {
  await test("deprecated filter and router tag spans retain complete pinned official packets and original source identities", async () => {
    assert.equal(manifest.pin, "6858a9ad67483025f6a9432a926beb9327037be3");
    assert.equal(manifest.originals.length, 11);
    assert.equal(manifest.originalDivergence.falseNegatives.length, 14);
    assert.equal(manifest.originalDivergence.falsePositives.length, 14);
    assert.deepEqual(
      manifest.additionalProjects.map((project) => [project.id, project.originals.length]),
      [
        ["zy-player", 2],
        ["habitica", 14],
        ["frappe-builder", 1],
      ],
    );
    let originalFiles = 0;
    for (const receipt of manifest.originalMatrixCaptures) {
      const raw = gunzipSync(fs.readFileSync(path.join(directory, receipt.path)));
      assert.equal(raw.length, receipt.rawBytes);
      assert.equal(hash(raw), receipt.rawSha256);
      const report = JSON.parse(raw);
      assert.equal(report.project, receipt.project);
      assert.equal(report.parsed.length, receipt.files);
      // Keep every original producer field and ordered packet, including all
      // unrelated rule messages. This audit never rewrites a current golden.
      assert.deepEqual(JSON.parse(report.stdout), report.parsed);
      originalFiles += report.parsed.length;
    }
    assert.equal(originalFiles, 655);
    const actual = await capture();
    assert.deepEqual(actual, JSON.parse(fs.readFileSync(fixturePath, "utf8")));
    for (const example of actual.captures) {
      assert.equal(example.packets.length, 1);
      assert.equal(example.packets[0].fatalErrorCount, 0, example.id);
    }
  });
}
