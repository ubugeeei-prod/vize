import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { createHash } from "node:crypto";
import { createChildWidthStockObserver } from "./formatter-child-width-stock.mjs";

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const pins = {
  vue26: [
    "vue-template-compiler",
    "2.6.14",
    "4239b6a3f2c693d405fe33499c7761ba5f4fa4686eac24a1599e8aa6100e0342",
    "c3513f4d8564444f8b4b5ce4039ccf08c3026bae302fa1a7441b61020e4c4f5e",
  ],
  vue27: [
    "@vue/compiler-sfc",
    "2.7.16",
    "de5e66c8657714dedc11396c1d7c127a8bb6ad7bef1343fb6475c35839032b56",
    "ee04b984ef31bc6591d7f5709ee81ec6bc1c7561dd99dd45e8761086699f057a",
  ],
  vue: [
    "vue",
    "3.5.35",
    "bfcb34fd1c45a973dff086362c30d477f664781edc82447acb7cd34ff97cee59",
    "a5287f6056bc5c2c569d318063ee043f55a0d323fa7262f64456d58fd574f928",
  ],
  compilerDom: [
    "@vue/compiler-dom",
    "3.5.35",
    "9ee91e0b32adffc7b30d1f5db1abd5b26189f89625359685e98c3e9925f6d4b7",
    "21407fc5cebe255208627dd039f2284156be88abbd9123281a79581d349b48c1",
  ],
  compilerSfc: [
    "@vue/compiler-sfc",
    "3.5.35",
    "6651b21ce07f6ea473c6991ae2286019e888349ceb6b17d01fd781f27838c773",
    "a2226c3eedb827ce90fabd18413c8b6d4008a99269f403f86f160b733c546970",
  ],
  compilerSsr: [
    "@vue/compiler-ssr",
    "3.5.35",
    "75fdca354a69fbae01bf20df88744edbe8f67484df2a33fbd8ee942b2df0917a",
    "b43751e2c5171bdeda85fed83d4d236849dccbfe4f13de4e62adf6b895d70baa",
  ],
  renderer: [
    "@vue/server-renderer",
    "3.5.35",
    "99fd05b8ddb123f39d0e107eab997effa07672bb43a89b05ee4f27c9015b57c3",
    "0e7735be4e65399d20d9ea65840a0cd0eef7f935fd1b224e6c45b576be12ac13",
  ],
  happyDom: [
    "happy-dom",
    "20.11.2",
    "1969a15a9c418201351abb24ed8f8d3ce4ad0d70ea521b68bee3924f075cce8b",
    "d38c1614c3184c41b04199fe84d0e1406633414ff07f1003e675faae5628475a",
  ],
};

// Resolve only existing declared dependencies: tests' legacy aliases and UI's
// coherent Vue family. No store-path guesses, ancestor search, or installs.
export async function createVueHistoryProviders(repositoryRoot) {
  const testsRequire = createRequire(path.join(repositoryRoot, "tests/package.json"));
  const uiRequire = createRequire(path.join(repositoryRoot, "npm/ui/package.json"));
  const vueRequire = createRequire(uiRequire.resolve("vue"));
  const rendererRequire = createRequire(uiRequire.resolve("vue/server-renderer"));
  const providers = {
    vue26: [testsRequire, "vue-sfc-compiler-2-6/build.js", "vue-sfc-compiler-2-6"],
    vue27: [testsRequire, "vue-sfc-compiler-2-7/dist/compiler-sfc.js", "vue-sfc-compiler-2-7"],
    vue: [vueRequire, "vue", "vue"],
    compilerDom: [vueRequire, "@vue/compiler-dom", "@vue/compiler-dom"],
    compilerSfc: [vueRequire, "vue/compiler-sfc", "@vue/compiler-sfc"],
    compilerSsr: [rendererRequire, "@vue/compiler-ssr", "@vue/compiler-ssr"],
    renderer: [vueRequire, "vue/server-renderer", "@vue/server-renderer"],
    happyDom: [uiRequire, "happy-dom", "happy-dom"],
  };
  const identities = {};
  for (const [id, [require, specifier, packageName]] of Object.entries(providers)) {
    const entry = require.resolve(specifier),
      manifest = require.resolve(`${packageName}/package.json`);
    const manifestBytes = fs.readFileSync(manifest),
      descriptor = JSON.parse(manifestBytes);
    identities[id] = {
      entry,
      realEntry: fs.realpathSync(entry),
      manifest,
      name: descriptor.name,
      version: descriptor.version,
      manifestSha256: sha256(manifestBytes),
      entrySha256: sha256(fs.readFileSync(entry)),
    };
    assert.deepEqual(
      [
        descriptor.name,
        descriptor.version,
        identities[id].manifestSha256,
        identities[id].entrySha256,
      ],
      pins[id],
      `${id}: pinned manifest and compiler/runtime entry`,
    );
  }
  const c26 = testsRequire(identities.vue26.entry),
    c27 = testsRequire(identities.vue27.entry);
  // Sets DOM globals before runtime-dom captures its document, and compiles
  // complete descriptor.template.content without its author-recipe path.
  const stock3 = await createChildWidthStockObserver(repositoryRoot);
  for (const [id, identity] of Object.entries(stock3.identities)) {
    assert.equal(identity.realEntry, identities[id].realEntry, `${id}: coherent stock family`);
    assert.equal(identity.entrySha256, identities[id].entrySha256);
  }
  const Vue = uiRequire("vue"),
    server = uiRequire("vue/server-renderer");

  return { identities, c26, c27, Vue, server, stock3 };
}
