import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { registerNuxtServerPlugin } from "./server-plugins.ts";

const variants = { nitro2: "/module/legacy.mjs", nitro3: "/module/current.mjs" };
function host(source: string, exported = true) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "nuxt-server-api-"));
  const pkg = path.join(root, "node_modules/nuxt");
  fs.mkdirSync(pkg, { recursive: true });
  fs.writeFileSync(path.join(root, "package.json"), '{"private":true}');
  fs.writeFileSync(
    path.join(pkg, "package.json"),
    JSON.stringify({
      name: "nuxt",
      type: "module",
      exports: exported ? { "./kit": "./kit.mjs" } : { ".": "./kit.mjs" },
    }),
  );
  fs.writeFileSync(path.join(pkg, "kit.mjs"), source);
  return root;
}

void test("the host Kit owns selection between complete Nitro plugin variants", async () => {
  const output = fs.mkdtempSync(path.join(os.tmpdir(), "nuxt-server-observation-"));
  const receipt = path.join(output, "registration.json");
  const root = host(
    `import fs from 'node:fs'; export function addNitroPlugin(value) { fs.writeFileSync(${JSON.stringify(receipt)}, JSON.stringify(value)); }`,
  );
  try {
    await registerNuxtServerPlugin(root, variants, () =>
      assert.fail("modern host must select its own variant"),
    );
    assert.deepEqual(JSON.parse(fs.readFileSync(receipt, "utf8")), variants);
  } finally {
    fs.rmSync(root, { recursive: true });
    fs.rmSync(output, { recursive: true });
  }
});

void test("older host Kit retains the existing legacy plugin registration", async () => {
  const root = host("export const olderKit = true;");
  const calls: string[] = [];
  try {
    await registerNuxtServerPlugin(root, variants, (file) => calls.push(file));
    assert.deepEqual(calls, [variants.nitro2]);
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

void test("hosts without the public Kit subpath retain Nuxt and Bridge legacy registration", async () => {
  const root = host("export const olderKit = true;", false);
  const calls: string[] = [];
  try {
    await registerNuxtServerPlugin(root, variants, (file) => calls.push(file));
    assert.deepEqual(calls, [variants.nitro2]);
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

void test("an actual host import failure cannot become successful legacy support", async () => {
  const root = host('throw new Error("broken host Kit");');
  try {
    await assert.rejects(
      registerNuxtServerPlugin(root, variants, () => assert.fail("do not hide import errors")),
      /broken host Kit/,
    );
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

void test("an invalid exported registration API cannot silently use a legacy plugin", async () => {
  const root = host("export const addNitroPlugin = 42;");
  const legacy: string[] = [];
  try {
    await assert.rejects(
      registerNuxtServerPlugin(root, variants, (file) => legacy.push(file)),
      {
        name: "TypeError",
        message: "The host nuxt/kit addNitroPlugin export must be a function.",
      },
    );
    assert.deepEqual(legacy, []);
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});

void test("the selected public registration failure propagates without a second legacy plugin", async () => {
  const root = host(
    'export function addNitroPlugin() { throw new Error("unsupported host server"); }',
  );
  try {
    await assert.rejects(
      registerNuxtServerPlugin(root, variants, () =>
        assert.fail("do not hide registration errors"),
      ),
      /unsupported host server/,
    );
  } finally {
    fs.rmSync(root, { recursive: true });
  }
});
