import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFile, mkdir, readdir, writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

const [bundle, out] = process.argv.slice(2);
assert(bundle && out);
const bundleSha = "4bd5cbcf9bdae4e4264be4ddb14e416074ad96909c56ba9b604e93d5b76b06ec";
const hash = (text) => createHash("sha256").update(text).digest("hex");
assert.equal(hash(await readFile(bundle)), bundleSha);
const vue = await import(pathToFileURL(bundle));
assert.equal(vue.version, "3.5.26");
await mkdir(out, { recursive: true });
assert.deepEqual(await readdir(out), [], "capture output must be empty");
const cases = [
  [
    "named_slot_key_only",
    '<template><Select><template v-if="prefix" #prefix>header</template><template v-for="row in rows"><span :key="row.id">{{ row.label }}</span></template></Select></template>',
  ],
  [
    "pure_key_only",
    '<template><template v-for="row in rows"><span :key="row.id">{{ row.label }}</span></template></template>',
  ],
  [
    "pure_empty",
    '<template><template v-for="row in rows"><span>{{ row.label }}</span></template></template>',
  ],
  [
    "surviving_bound_prop",
    '<template><template v-for="row in rows"><span :key="row.id" :title="row.title">{{ row.label }}</span></template></template>',
  ],
  [
    "surviving_static_prop",
    '<template><template v-for="row in rows"><span :key="row.id" class="row">{{ row.label }}</span></template></template>',
  ],
  [
    "template_injected_key",
    '<template><template v-for="row in rows" :key="row.id"><span :key="row.childId">{{ row.label }}</span></template></template>',
  ],
  [
    "spread_with_key",
    '<template><template v-for="row in rows"><span :key="row.id" v-bind="row.props">{{ row.label }}</span></template></template>',
  ],
  [
    "multiple_children_keep_key",
    '<template><template v-for="row in rows"><span :key="row.id">{{ row.label }}</span><i /></template></template>',
  ],
  [
    "grandchild_keeps_key",
    '<template><template v-for="row in rows"><div :key="row.id"><span :key="row.childId">{{ row.label }}</span></div></template></template>',
  ],
  [
    "element_loop_keeps_key",
    '<template><span v-for="row in rows" :key="row.id">{{ row.label }}</span></template>',
  ],
  [
    "static_key_only",
    '<template><template v-for="row in rows"><span key="fixed">{{ row.label }}</span></template></template>',
  ],
];
const manifest = {
  compiler: {
    version: vue.version,
    entrypoint: "@vue/compiler-sfc/dist/compiler-sfc.esm-browser.js",
    url: "https://unpkg.com/@vue/compiler-sfc@3.5.26/dist/compiler-sfc.esm-browser.js",
    sha256: bundleSha,
  },
  sourceHead: "6a37d1d27006f75a6b414367513633d5c48cc2df",
  cases: [],
};
for (const [name, source] of cases) {
  const filename = `${name}.vue`;
  const parsed = vue.parse(source, { filename });
  assert.deepEqual(parsed.errors, []);
  const options = [
    { mode: "function", prefixIdentifiers: false, cacheHandlers: false, sourceMap: false },
    { mode: "module", prefixIdentifiers: true, cacheHandlers: false, sourceMap: false },
  ];
  const outputs = options.map((option) => {
    const result = vue.compileTemplate({
      source: parsed.descriptor.template.content,
      filename,
      id: name,
      compilerOptions: option,
      sourceMap: false,
    });
    const errors = result.errors.map((error) =>
      typeof error === "string"
        ? error
        : { name: error.name, message: error.message, code: error.code, loc: error.loc },
    );
    return {
      options: {
        source: parsed.descriptor.template.content,
        filename,
        id: name,
        compilerOptions: option,
        sourceMap: false,
      },
      code: result.code,
      codeSha256: hash(result.code),
      errors,
      tips: result.tips,
      map: result.map ?? null,
    };
  });
  const packet = {
    compiler: manifest.compiler,
    name,
    filename,
    source,
    sourceSha256: hash(source),
    template: parsed.descriptor.template.content,
    parseErrors: parsed.errors,
    outputs,
  };
  const bytes = JSON.stringify(packet, null, 2) + "\n";
  await writeFile(`${out}/${name}.json`, bytes);
  manifest.cases.push({
    name,
    path: `${name}.json`,
    sourceSha256: packet.sourceSha256,
    sha256: hash(bytes),
  });
  console.log(
    `${name}: ${outputs.map(({ errors, codeSha256 }) => `errors=${errors.length},codeSha256=${codeSha256}`).join("; ")}`,
  );
}
await writeFile(`${out}/cases.json`, JSON.stringify(manifest, null, 2) + "\n");
