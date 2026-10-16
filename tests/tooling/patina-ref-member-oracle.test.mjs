import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { gunzipSync } from "node:zlib";

const root = path.resolve(import.meta.dirname, "../..");
const directory = path.join(root, "crates/vize_patina/tests/fixtures/ref-member-access");
const manifest = JSON.parse(fs.readFileSync(path.join(directory, "source.json"), "utf8"));
const fixturePath = path.join(directory, "oracle.json");
const fixture = JSON.parse(fs.readFileSync(fixturePath, "utf8"));
const requireProvider = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
const hash = (bytes, algorithm = "sha256") => createHash(algorithm).update(bytes).digest("hex");
const bodies = [
  ["direct", "const items=ref([]);items.map(x=>x);"],
  ["optional", "const area=computed(()=>({bottom:1}));const y=area?.bottom.value;"],
  ["string-key", "const items=ref([]);const n=items['length'];"],
  ["numeric-key", "const items=ref([]);const first=items[0];"],
  [
    "value-and-effect",
    "const items=ref([]);items.value.map(x=>x);items?.value;items['value'];items.effect;items['effect'];",
  ],
  ["unknown-key", "const items=ref([]);items[key];items?.[key];"],
  ["ordinary", "const items=[];items.map(x=>x);const state=reactive({x:1});state.x;"],
  ["shadowed-parameter", "const items=ref([]);function render(items){return items.map(x=>x)}"],
  ["shadowed-local", "const items=ref([]);{const items=[];items.map(x=>x)}"],
  ["passed-ref", "const items=ref([]);watch(items,()=>{});useRef(items);"],
  ["ref-replacement", "let count=ref(0);count=ref(1);"],
  ["alias", "const make=ref;const items=make([]);items.map(x=>x);"],
];

async function capture() {
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
  const rules = { "vue/no-ref-as-operand": "error" };
  const eslint = new (requireProvider("eslint").ESLint)({
    cwd: root,
    overrideConfigFile: true,
    overrideConfig: [
      {
        files: ["**/*.vue"],
        languageOptions: {
          parser: requireProvider("vue-eslint-parser"),
          parserOptions: {
            parser: requireProvider("@typescript-eslint/parser"),
            ecmaVersion: "latest",
            sourceType: "module",
          },
        },
        plugins: { vue: requireProvider("eslint-plugin-vue") },
        rules,
      },
    ],
  });
  const examples = bodies.map(([id, body]) => ({
    id,
    source: `<script setup>import {ref,computed,reactive,watch} from 'vue';${body}</script>`,
  }));
  examples.push({
    id: "import-alias",
    source:
      "<script setup>import {ref as make} from 'vue';const items=make([]);items.map(x=>x);</script>",
  });
  examples.push({
    id: "namespace",
    source: "<script setup>import * as Vue from 'vue';const items=Vue.ref([]);items[0];</script>",
  });
  for (const identity of manifest.originals) {
    const example = fixture.originals.find((entry) => entry.id === identity.path);
    const bytes = Buffer.from(example.source);
    assert.equal(bytes.length, identity.bytes);
    assert.equal(hash(bytes), identity.sha256);
    const blob = Buffer.concat([Buffer.from(`blob ${bytes.length}\0`), bytes]);
    assert.equal(hash(blob, "sha1"), identity.gitBlob);
    examples.push(example);
  }
  const captures = [];
  for (const example of examples) {
    const filePath = path.join(
      root,
      "owned-ref-members",
      example.id.endsWith(".vue") ? example.id : `${example.id}.vue`,
    );
    const packets = await eslint.lintText(example.source, { filePath });
    captures.push({
      ...example,
      packets: packets.map((packet) => ({ ...packet, filePath: path.relative(root, filePath) })),
    });
  }
  return { providers, rules, originals: fixture.originals, captures };
}

if (process.argv.includes("--record")) {
  fs.writeFileSync(fixturePath, JSON.stringify(await capture(), null, 2) + "\n");
} else {
  await test("ref member controls and pinned full originals retain complete official packets and fixes", async () => {
    assert.equal(manifest.originals.length, 2);
    assert.equal(manifest.originals[0].pin, manifest.originals[1].pin);
    const original = gunzipSync(
      fs.readFileSync(path.join(directory, "original-matrix-linter.json.gz")),
    );
    assert.equal(original.length, manifest.originalMatrix.bytes);
    assert.equal(hash(original), manifest.originalMatrix.sha256);
    const report = JSON.parse(original);
    assert.deepEqual(JSON.parse(report.stdout), report.parsed);
    assert.equal(report.parsed.length, manifest.originalMatrix.files);
    const actual = await capture();
    assert.deepEqual(actual, fixture);
    assert.equal(actual.captures.length, 16);
    for (const entry of actual.captures) {
      assert.equal(entry.packets.length, 1);
      assert.equal(entry.packets[0].fatalErrorCount, 0, entry.id);
    }
  });
}
