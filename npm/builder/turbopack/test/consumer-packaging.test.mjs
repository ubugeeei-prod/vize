import assert from "node:assert/strict";
import { test } from "node:test";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { assertBundledHelpers } from "./prepare-consumer.mjs";
import { sourceNativeResolutions } from "./consumer-lock.mjs";
import { normalHostShutdown } from "./host-process.mjs";
import { spawn } from "node:child_process";
import { once } from "node:events";
import { readPublishedMap } from "./output-source-map.mjs";

void test("host map custody follows live JS/CSS footers and refuses missing maps", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "vize-turbopack-maps-"));
  try {
    const javascript = path.join(root, "组件 space.js");
    const stylesheet = path.join(root, "组件 space.css");
    const live = Buffer.from('{"version":3,"sources":["authored.vue"]}');
    await fs.writeFile(path.join(root, "live map.js.map"), live);
    await fs.writeFile(path.join(root, "live map.css.map"), live);
    await fs.writeFile(`${javascript}.map`, "an orphan from the previous build");
    await fs.writeFile(javascript, "globalThis.value=1;\n//# sourceMappingURL=live%20map.js.map\n");
    await fs.writeFile(stylesheet, ".a{}\n/*# sourceMappingURL=live%20map.css.map */\n");
    for (const filename of [javascript, stylesheet]) {
      const result = await readPublishedMap(filename);
      assert.deepEqual(result.bytes, live);
      assert.notEqual(result.mapFile, `${filename}.map`);
    }
    await fs.writeFile(javascript, "globalThis.value=1;");
    assert.equal(await readPublishedMap(javascript), null);
    await fs.writeFile(javascript, "//# sourceMappingURL=missing.js.map\n");
    await assert.rejects(readPublishedMap(javascript), { code: "ENOENT" });
    await fs.writeFile(javascript, "//# sourceMappingURL=https://example.invalid/map\n");
    await assert.rejects(readPublishedMap(javascript), /must publish a local source map/);
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

void test("public bundle inspection visits nested loader entries and rejects private imports", async () => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "vize-turbopack-packaging-"));
  try {
    await fs.mkdir(path.join(root, "loader"));
    await fs.writeFile(
      path.join(root, "index.mjs"),
      'export { default } from "./loader/index.mjs";',
    );
    const entry = path.join(root, "loader/index.mjs");
    await fs.writeFile(entry, "export default function loader() {};");
    await assertBundledHelpers(root);
    for (const statement of [
      'import { MappedModule } from "@vizejs/builder-shared/mapped-module";',
      'const helper = require("@vizejs/builder-shared/mapped-module");',
      'const helper = import("@vizejs/builder-shared/mapped-module");',
    ]) {
      await fs.writeFile(entry, statement);
      await assert.rejects(assertBundledHelpers(root));
    }
  } finally {
    await fs.rm(root, { recursive: true, force: true });
  }
});

void test("controlled SIGTERM handles Next's numeric exit without accepting earlier crashes", async (t) => {
  if (process.platform === "win32")
    return t.skip("the actual host job uses Linux signal semantics");
  async function terminated(script, request) {
    const child = spawn(process.execPath, ["-e", script], { stdio: ["ignore", "pipe", "pipe"] });
    t.after(() => {
      if (child.exitCode === null && child.signalCode === null) child.kill("SIGKILL");
    });
    const closed = once(child, "close");
    if (request) await once(child.stdout, "data");
    else await closed;
    const shutdown = {
      statusBefore: child.exitCode,
      signalBefore: child.signalCode,
      requested: child.kill("SIGTERM"),
    };
    const [status, signal] = await closed;
    return normalHostShutdown({ status, signal }, shutdown);
  }
  const ready = 'process.stdout.write("ready");setInterval(()=>{},1000);';
  assert.equal(await terminated(ready, true), true);
  assert.equal(
    await terminated('process.on("SIGTERM",()=>process.exit(143));' + ready, true),
    true,
  );
  assert.equal(await terminated('process.on("SIGTERM",()=>process.exit(1));' + ready, true), false);
  assert.equal(await terminated("process.exit(143)", false), false);
  assert.equal(await terminated("process.exit(1)", false), false);
});

void test("source custody reads pnpm project documents and rejects alternate native resolutions", () => {
  const project = {
    importers: {
      ".": {
        dependencies: { "@vizejs/turbopack-plugin": { version: "file:../packs/turbopack.tgz" } },
      },
    },
    overrides: { "@vizejs/native": "file:../packs/native.tgz" },
    packages: {
      "@vizejs/native@file:../packs/native.tgz": {
        resolution: { tarball: "file:../packs/native.tgz", integrity: "sha512-selected" },
      },
    },
  };
  const manager = "---\nlockfileVersion: '9.0'\npackages: {}\n";
  const serialize = (value) => "---\n" + JSON.stringify(value) + "\n";
  const check = (text) =>
    sourceNativeResolutions(
      text,
      "/receipt/consumer",
      "/receipt/packs/native.tgz",
      "sha512-selected",
    );
  assert.equal(check(serialize(project)).length, 1);
  assert.equal(check(manager + serialize(project)).length, 1);
  const missingOverride = structuredClone(project);
  delete missingOverride.overrides;
  const registry = structuredClone(project);
  registry.packages["@vizejs/native@0.474.0"] = Object.values(registry.packages)[0];
  const wrongIntegrity = structuredClone(project);
  Object.values(wrongIntegrity.packages)[0].resolution.integrity = "sha512-other";
  const otherFile = structuredClone(project);
  Object.values(otherFile.packages)[0].resolution.tarball = "file:../packs/other.tgz";
  for (const invalid of [missingOverride, registry, wrongIntegrity, otherFile]) {
    assert.throws(() => check(manager + serialize(invalid)));
  }
  assert.throws(() => check(serialize({ packages: registry.packages }) + serialize(project)));
  assert.throws(() => check(serialize(project) + serialize(project)));
  assert.throws(() => check("{ invalid YAML\n"));
});
